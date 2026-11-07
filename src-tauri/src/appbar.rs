//! Barre superiori registrate come appbar (SHAppBarMessage), una per monitor:
//! le finestre massimizzate si fermano sotto di loro invece di finirci sotto.
//!
//! La registrazione va SEMPRE annullata (ABM_REMOVE) in ogni percorso di
//! uscita, altrimenti lo spazio di lavoro resta ridotto.
//!
//! Multi-monitor: `reconcile` confronta i monitor presenti con le finestre
//! esistenti, crea/rimuove/riposiziona le barre e corregge qualsiasi deriva
//! (cambio di scala DPI, monitor scollegato, risoluzione cambiata). Viene
//! chiamata periodicamente da un thread.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Shell::{
    ABE_BOTTOM, ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA, SHAppBarMessage,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WM_APP, WS_EX_NOACTIVATE,
};

/// Altezza della barra in pixel logici (a 100% di scala).
pub const BAR_HEIGHT_LOGICAL: f64 = 28.0;
/// Etichetta della finestra della barra sul monitor primario (da tauri.conf.json).
pub const MAIN_LABEL: &str = "main";
const POLL_INTERVAL: Duration = Duration::from_millis(1000);

// ---------------------------------------------------------------------------
// Geometria: funzioni pure, testate senza toccare il sistema.
// ---------------------------------------------------------------------------

/// Geometria di un monitor in pixel fisici.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorInfo {
    /// Nome stabile del monitor (es. `\\.\DISPLAY2`).
    pub key: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale: f64,
    pub primary: bool,
}

impl MonitorInfo {
    pub fn rect(&self) -> RECT {
        RECT { left: self.x, top: self.y, right: self.x + self.width, bottom: self.y + self.height }
    }
}

/// Altezza della barra in pixel fisici per una data scala.
pub fn bar_height_px(scale: f64) -> i32 {
    (BAR_HEIGHT_LOGICAL * scale).round().max(1.0) as i32
}

/// Rettangolo (pixel fisici) che la barra occupa in cima al monitor.
pub fn bar_rect(m: &MonitorInfo) -> RECT {
    RECT { left: m.x, top: m.y, right: m.x + m.width, bottom: m.y + bar_height_px(m.scale) }
}

/// Etichetta della finestra per un monitor: "main" per il primario,
/// "bar-<nome ripulito>" per gli altri (le etichette Tauri non ammettono `\` e `.`).
pub fn label_for(m: &MonitorInfo) -> String {
    if m.primary {
        return MAIN_LABEL.to_string();
    }
    let clean: String = m
        .key
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("bar-{}", clean.trim_matches('_'))
}

/// Mappa etichetta -> monitor desiderata.
pub fn desired(monitors: &[MonitorInfo]) -> BTreeMap<String, MonitorInfo> {
    monitors.iter().map(|m| (label_for(m), m.clone())).collect()
}

/// Cosa creare e cosa rimuovere dato lo stato desiderato e le finestre barra esistenti.
pub fn diff(
    desired: &BTreeMap<String, MonitorInfo>,
    existing: &[String],
) -> (Vec<String>, Vec<String>) {
    let existing: HashSet<&String> = existing.iter().collect();
    let create = desired.keys().filter(|l| !existing.contains(l)).cloned().collect();
    let remove = existing
        .iter()
        .filter(|l| !desired.contains_key(**l))
        .map(|l| (*l).clone())
        .collect();
    (create, remove)
}

fn is_bar_label(label: &str) -> bool {
    label == MAIN_LABEL || label.starts_with("bar-")
}

// ---------------------------------------------------------------------------
// Registrazione appbar.
// ---------------------------------------------------------------------------

/// HWND delle barre registrate.
static REGISTERED: Mutex<Vec<isize>> = Mutex::new(Vec::new());

fn registered() -> std::sync::MutexGuard<'static, Vec<isize>> {
    REGISTERED.lock().unwrap_or_else(|e| e.into_inner())
}

fn new_data_edge(hwnd: HWND, edge: u32) -> APPBARDATA {
    APPBARDATA {
        cbSize: size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        uCallbackMessage: WM_APP + 0x10,
        uEdge: edge,
        ..Default::default()
    }
}

fn new_data(hwnd: HWND) -> APPBARDATA {
    new_data_edge(hwnd, ABE_TOP)
}

/// Ultima fascia in basso applicata (finestra, rettangolo): evita di rimandare al
/// sistema, ogni secondo, una modifica dell'area di lavoro che non è cambiata.
type BottomStrip = (isize, (i32, i32, i32, i32));
static LAST_BOTTOM: Mutex<Option<BottomStrip>> = Mutex::new(None);

/// Riserva una fascia alta `height_px` in basso sul monitor `mon` per `hwnd` (il
/// Dock), registrandolo come appbar se non lo è già, e ritorna il rettangolo
/// assegnato dal sistema. Se in basso c'è la taskbar, la fascia si ferma sopra di
/// lei: `rc.bottom` è il bordo inferiore *disponibile* al Dock. Il sistema modifica
/// l'area di lavoro solo quando il rettangolo cambia.
pub fn reserve_bottom(hwnd: HWND, mon: RECT, height_px: i32) -> RECT {
    let mut abd = new_data_edge(hwnd, ABE_BOTTOM);
    unsafe {
        let key = hwnd.0 as isize;
        let is_new = {
            let mut r = registered();
            if r.contains(&key) {
                false
            } else {
                r.push(key);
                true
            }
        };
        if is_new {
            SHAppBarMessage(ABM_NEW, &mut abd);
            // Una registrazione nuova non ha ancora nessuna posizione: anche se il
            // rettangolo è uguale all'ultimo applicato, va comunicato di nuovo. Succede
            // dopo il riavvio di Explorer, che perde tutte le appbar (vedi
            // `handle_explorer_restart`): senza questo la riserva andrebbe persa.
            *LAST_BOTTOM.lock().unwrap_or_else(|e| e.into_inner()) = None;
        }
        abd.rc = RECT { left: mon.left, top: mon.bottom - height_px, right: mon.right, bottom: mon.bottom };
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        // Il sistema può spostare il bordo (taskbar); l'altezza resta quella voluta.
        abd.rc.top = abd.rc.bottom - height_px;
        let want = (abd.rc.left, abd.rc.top, abd.rc.right, abd.rc.bottom);
        let mut last = LAST_BOTTOM.lock().unwrap_or_else(|e| e.into_inner());
        if *last != Some((key, want)) {
            SHAppBarMessage(ABM_SETPOS, &mut abd);
            *last = Some((key, want));
        }
    }
    abd.rc
}

/// Rilascia la fascia in basso del Dock. Idempotente.
pub fn release_bottom(hwnd: HWND) {
    *LAST_BOTTOM.lock().unwrap_or_else(|e| e.into_inner()) = None;
    remove(hwnd);
}

/// Registra `hwnd` come appbar in alto sul monitor `mon` (se non lo è già) e
/// ritorna il rettangolo (pixel fisici) che la barra deve occupare.
pub fn register(hwnd: HWND, mon: RECT, height_px: i32) -> RECT {
    let mut abd = new_data(hwnd);
    unsafe {
        let key = hwnd.0 as isize;
        let is_new = {
            let mut r = registered();
            if r.contains(&key) {
                false
            } else {
                r.push(key);
                true
            }
        };
        if is_new {
            SHAppBarMessage(ABM_NEW, &mut abd);
        }
        abd.rc = RECT { left: mon.left, top: mon.top, right: mon.right, bottom: mon.top + height_px };
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        // Il sistema può aver spostato il lato; l'altezza resta quella voluta.
        abd.rc.bottom = abd.rc.top + height_px;
        SHAppBarMessage(ABM_SETPOS, &mut abd);
    }
    abd.rc
}

/// Annulla la registrazione di una barra. Idempotente.
pub fn remove(hwnd: HWND) {
    let key = hwnd.0 as isize;
    let was = {
        let mut r = registered();
        let n = r.len();
        r.retain(|h| *h != key);
        r.len() != n
    };
    if was {
        let mut abd = new_data(hwnd);
        unsafe {
            SHAppBarMessage(ABM_REMOVE, &mut abd);
        }
    }
}

/// Annulla la registrazione di tutte le barre. Idempotente, per i percorsi di uscita.
pub fn remove_if_registered() {
    *LAST_BOTTOM.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let all: Vec<isize> = std::mem::take(&mut *registered());
    for h in all {
        let mut abd = new_data(HWND(h as *mut _));
        unsafe {
            SHAppBarMessage(ABM_REMOVE, &mut abd);
        }
    }
}

// ---------------------------------------------------------------------------
// Gestione delle finestre.
// ---------------------------------------------------------------------------

/// Ultimo rettangolo applicato a ciascuna barra (per saltare il lavoro inutile).
type AppliedMap = HashMap<String, (MonitorInfo, (i32, i32, i32, i32))>;
static APPLIED: Mutex<Option<AppliedMap>> = Mutex::new(None);

fn debug_enabled() -> bool {
    std::env::var("EOSOS_DEBUG_APPBAR").is_ok()
}

fn monitors_of(app: &AppHandle) -> tauri::Result<Vec<MonitorInfo>> {
    let primary = app.primary_monitor()?;
    let primary_name = primary.as_ref().and_then(|m| m.name().cloned());
    let mut out = Vec::new();
    for m in app.available_monitors()? {
        let key = m.name().cloned().unwrap_or_else(|| format!("{}_{}", m.position().x, m.position().y));
        let is_primary = match (&primary_name, m.name()) {
            (Some(p), Some(n)) => p == n,
            _ => false,
        };
        out.push(MonitorInfo {
            key,
            x: m.position().x,
            y: m.position().y,
            width: m.size().width as i32,
            height: m.size().height as i32,
            scale: m.scale_factor(),
            primary: is_primary,
        });
    }
    // Solo build debug: simula lo scollegamento di un monitor (per i test dal vivo).
    #[cfg(debug_assertions)]
    if let Ok(name) = std::fs::read_to_string(std::env::temp_dir().join("eosos-hide-monitor")) {
        let name = name.trim();
        out.retain(|m| m.key != name);
    }
    // Se nessuno risulta primario (nome mancante), il primo lo diventa.
    if !out.is_empty() && !out.iter().any(|m| m.primary) {
        out[0].primary = true;
    }
    Ok(out)
}

fn create_window(app: &AppHandle, label: &str, info: &MonitorInfo) -> tauri::Result<WebviewWindow> {
    let h = bar_height_px(info.scale);
    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("EosOS")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .position(info.x as f64, info.y as f64)
        .inner_size(info.width as f64 / info.scale, h as f64 / info.scale)
        .build()
}

/// Porta la barra `window` nello stato desiderato per il monitor `info`.
fn apply(window: &WebviewWindow, info: &MonitorInfo) -> Result<(), Box<dyn std::error::Error>> {
    let label = window.label().to_string();
    let hwnd = window.hwnd()?;

    let target = bar_rect(info);
    let height = target.bottom - target.top;
    let want = (target.left, target.top, target.right - target.left, height);

    // Salta se già applicato e la finestra non è andata alla deriva.
    let actual = {
        let p = window.outer_position()?;
        let s = window.outer_size()?;
        (p.x, p.y, s.width as i32, s.height as i32)
    };
    let unchanged = APPLIED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|m| m.get(&label))
        .is_some_and(|(prev, _)| prev == info);
    if unchanged && actual == want && window.is_visible()? {
        return Ok(());
    }

    // La barra non deve mai diventare la finestra in primo piano.
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | WS_EX_NOACTIVATE.0 as isize);
    }

    let rc = register(hwnd, info.rect(), height);
    let size = PhysicalSize::new((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32);
    window.set_position(PhysicalPosition::new(rc.left, rc.top))?;
    window.set_size(size)?;
    if !window.is_visible()? {
        crate::glass::apply(window);
        window.show()?;
        // La prima visualizzazione può spostare la finestra: si riafferma la posizione.
        window.set_position(PhysicalPosition::new(rc.left, rc.top))?;
    }

    if debug_enabled() {
        eprintln!(
            "appbar[{label}]: monitor={} scala={} rc=({},{},{},{}) prima={:?}",
            info.key, info.scale, rc.left, rc.top, rc.right, rc.bottom, actual
        );
    }
    APPLIED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashMap::new)
        .insert(label, (info.clone(), (rc.left, rc.top, rc.right, rc.bottom)));
    Ok(())
}

/// Ultimo handle noto della taskbar, per accorgersi del riavvio di Explorer.
static LAST_TRAY: Mutex<Option<isize>> = Mutex::new(None);

/// Quando Explorer si riavvia perde tutte le appbar registrate: si dimenticano
/// le registrazioni (senza ABM_REMOVE, sono già sparite) così `apply` le rifà.
fn handle_explorer_restart() {
    let mut last = LAST_TRAY.lock().unwrap_or_else(|e| e.into_inner());
    let (next, restarted) = crate::taskbar::detect_restart(*last, crate::taskbar::tray_handle());
    *last = next;
    if restarted {
        registered().clear();
        *APPLIED.lock().unwrap_or_else(|e| e.into_inner()) = None;
        if debug_enabled() {
            eprintln!("appbar: Explorer riavviato, registrazioni da rifare");
        }
    }
}

/// Allinea le barre ai monitor presenti: crea, rimuove, riposiziona.
pub fn reconcile(app: &AppHandle) {
    handle_explorer_restart();
    let monitors = match monitors_of(app) {
        Ok(m) if !m.is_empty() => m,
        _ => return,
    };
    let want = desired(&monitors);
    let existing: Vec<String> = app
        .webview_windows()
        .keys()
        .filter(|l| is_bar_label(l))
        .cloned()
        .collect();
    let (create, remove_labels) = diff(&want, &existing);

    for label in remove_labels {
        if let Some(w) = app.get_webview_window(&label) {
            if let Ok(hwnd) = w.hwnd() {
                remove(hwnd);
            }
            let _ = w.destroy();
        }
        APPLIED.lock().unwrap_or_else(|e| e.into_inner()).as_mut().map(|m| m.remove(&label));
        if debug_enabled() {
            eprintln!("appbar[{label}]: rimossa");
        }
    }
    for label in create {
        if let Some(info) = want.get(&label)
            && let Err(e) = create_window(app, &label, info)
        {
            eprintln!("EosOS: creazione barra {label} fallita: {e}");
        }
    }
    for (label, info) in &want {
        if let Some(w) = app.get_webview_window(label)
            && let Err(e) = apply(&w, info)
        {
            eprintln!("EosOS: posizionamento barra {label} fallito: {e}");
        }
    }
}

/// Primo allineamento e thread che segue i cambi di monitor/scala.
pub fn start(app: &AppHandle) {
    reconcile(app);
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(POLL_INTERVAL);
        reconcile(&app);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(key: &str, x: i32, y: i32, w: i32, h: i32, scale: f64, primary: bool) -> MonitorInfo {
        MonitorInfo { key: key.into(), x, y, width: w, height: h, scale, primary }
    }

    #[test]
    fn height_scales_with_dpi() {
        assert_eq!(bar_height_px(1.0), 28);
        assert_eq!(bar_height_px(1.25), 35);
        assert_eq!(bar_height_px(1.5), 42);
        assert_eq!(bar_height_px(1.75), 49);
        assert_eq!(bar_height_px(2.0), 56);
        assert_eq!(bar_height_px(3.0), 84);
    }

    #[test]
    fn bar_rect_follows_monitor_with_negative_origin() {
        // Secondario a sinistra del primario, scala 150%.
        let m = mon(r"\\.\DISPLAY2", -2560, -200, 2560, 1440, 1.5, false);
        let r = bar_rect(&m);
        assert_eq!((r.left, r.top, r.right, r.bottom), (-2560, -200, 0, -200 + 42));
    }

    #[test]
    fn each_monitor_uses_its_own_scale() {
        let a = bar_rect(&mon("A", 0, 0, 1920, 1080, 1.0, true));
        let b = bar_rect(&mon("B", 1920, 0, 3840, 2160, 2.0, false));
        assert_eq!(a.bottom - a.top, 28);
        assert_eq!(b.bottom - b.top, 56);
        assert_eq!(b.right - b.left, 3840);
    }

    #[test]
    fn primary_is_main_and_others_are_bar_labels() {
        let ms = [mon(r"\\.\DISPLAY1", 0, 0, 1920, 1080, 1.0, true), mon(r"\\.\DISPLAY2", -1920, 0, 1920, 1080, 1.0, false)];
        let d = desired(&ms);
        assert!(d.contains_key("main"));
        let other: Vec<&String> = d.keys().filter(|k| *k != "main").collect();
        assert_eq!(other.len(), 1);
        assert!(other[0].starts_with("bar-"), "{}", other[0]);
        assert!(other[0].chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'), "{}", other[0]);
    }

    #[test]
    fn diff_creates_missing_and_removes_stale() {
        let ms = [mon("D1", 0, 0, 1920, 1080, 1.0, true), mon("D2", -1920, 0, 1920, 1080, 1.0, false)];
        let d = desired(&ms);

        // Solo main esiste: va creata la barra del secondo monitor.
        let (create, remove) = diff(&d, &["main".to_string()]);
        assert_eq!(create.len(), 1);
        assert!(remove.is_empty());

        // Il secondo monitor sparisce: la sua barra va rimossa, main resta.
        let d1 = desired(&ms[..1]);
        let (create, remove) = diff(&d1, &["main".to_string(), create[0].clone()]);
        assert!(create.is_empty());
        assert_eq!(remove.len(), 1);
        assert_ne!(remove[0], "main");
    }

    #[test]
    fn stable_state_has_no_actions() {
        let ms = [mon("D1", 0, 0, 1920, 1080, 1.0, true)];
        let d = desired(&ms);
        let (c, r) = diff(&d, &["main".to_string()]);
        assert!(c.is_empty() && r.is_empty());
    }

    #[test]
    fn primary_change_moves_main_label() {
        // D2 diventa primario: main passa a D2 e D1 riceve una barra "bar-".
        let before = desired(&[mon("D1", 0, 0, 1920, 1080, 1.0, true), mon("D2", -1920, 0, 1920, 1080, 1.0, false)]);
        let after = desired(&[mon("D1", 0, 0, 1920, 1080, 1.0, false), mon("D2", -1920, 0, 1920, 1080, 1.0, true)]);
        assert_eq!(before["main"].key, "D1");
        assert_eq!(after["main"].key, "D2");
        let existing: Vec<String> = before.keys().cloned().collect();
        let (create, remove) = diff(&after, &existing);
        assert_eq!(create.len(), 1);
        assert_eq!(remove.len(), 1);
    }
}
