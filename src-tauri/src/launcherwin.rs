//! Finestra del Launcher (fase L2): una finestra Tauri senza bordi, trasparente e sempre in
//! primo piano, creata nascosta all'avvio e mostrata/nascosta dalla scorciatoia globale.
//!
//! A differenza del Dock **deve poter ricevere la tastiera**: niente `WS_EX_NOACTIVATE`. Compare
//! sul monitor dove sta il cursore, centrata in orizzontale e a circa un quinto dall'alto, e si
//! nasconde con Esc (lo fa la pagina), quando perde il focus o con la stessa scorciatoia.
//!
//! La scorciatoia è `RegisterHotKey` su un thread con il proprio ciclo di messaggi. Se la
//! combinazione è già presa da un altro programma lo si dice (una finestra di avviso, una sola
//! volta) e si riprova ogni 30 secondi, invece di non aprirsi in silenzio.

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::core::{w, HSTRING};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_NOREPEAT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetMessageW, GetWindowLongPtrW, GetWindowRect, MessageBoxW, PostThreadMessageW, SetTimer,
    SetWindowDisplayAffinity, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, MB_ICONINFORMATION, MB_OK, MSG,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
    WM_APP, WM_HOTKEY, WM_TIMER, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

use crate::dockgeom::{capture_margin, capture_rect, Rect};

pub const LABEL: &str = "launcher";
/// Larghezza del pannello, in px CSS.
pub const WIDTH: f64 = 680.0;
/// Altezza minima (solo il campo) e massima (campo + risultati), in px CSS.
pub const MIN_HEIGHT: f64 = 72.0;
pub const MAX_HEIGHT: f64 = 640.0;
/// Quanto dopo l'apertura si ignora la perdita di focus: la finestra può riceverla e perderla
/// per un attimo mentre Windows decide chi è in primo piano.
const FOCUS_GRACE: Duration = Duration::from_millis(350);
/// Dopo quanto si controlla che il primo piano sia arrivato davvero.
const FOCUS_CHECK: Duration = Duration::from_millis(500);
const RETRY_EVERY: Duration = Duration::from_secs(30);
const HOTKEY_ID: i32 = 0x4C41; // "LA"

/// Istante dell'ultima apertura, in ms da `START`; 0 = mai.
static SHOWN_AT_MS: AtomicU64 = AtomicU64::new(0);
static VISIBLE: AtomicBool = AtomicBool::new(false);
/// Scala del monitor su cui è stata posizionata la finestra (f64 in bit): serve a convertire
/// l'altezza che la pagina comunica in px CSS.
static SCALE_BITS: AtomicU64 = AtomicU64::new(0);
static START: Mutex<Option<Instant>> = Mutex::new(None);
/// HWND della finestra, per la cattura dello sfondo.
static HWND_RAW: AtomicIsize = AtomicIsize::new(0);
/// La finestra è esclusa dalla cattura (serve alla sfocatura: altrimenti si sfocherebbe da sola).
static EXCLUDED: AtomicBool = AtomicBool::new(false);

fn since_start() -> Duration {
    let mut g = START.lock().unwrap_or_else(|e| e.into_inner());
    g.get_or_insert_with(Instant::now).elapsed()
}

// ------------------------------------------------------------------ pure

/// Porta l'altezza comunicata dalla pagina dentro i limiti (NaN e infiniti compresi). Pura, testata.
pub fn clamp_height(h: f64) -> f64 {
    if h.is_finite() { h.clamp(MIN_HEIGHT, MAX_HEIGHT) } else { MIN_HEIGHT }
}

/// Posizione e dimensione fisiche del pannello dentro l'area di lavoro `area` (px fisici) di un
/// monitor con scala `scale`: centrato in orizzontale, a un quinto dall'alto, mai fuori dall'area.
/// Pura, testata.
pub fn place(area: Rect, scale: f64, height_css: f64) -> Rect {
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let w = ((WIDTH * scale).round() as i32).min(area.w).max(1);
    let h = ((clamp_height(height_css) * scale).round() as i32).min(area.h).max(1);
    let x = area.x + (area.w - w) / 2;
    let mut y = area.y + area.h / 5;
    if y + h > area.y + area.h {
        y = (area.y + area.h - h).max(area.y);
    }
    Rect { x, y, w, h }
}

/// La perdita di focus è davvero da seguire (non un'oscillazione dell'apertura)? Pura, testata.
pub fn focus_loss_counts(since_shown: Duration) -> bool {
    since_shown >= FOCUS_GRACE
}

// ------------------------------------------------------------------ finestra

fn win_of(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window(LABEL)
}

/// Il Launcher non compare in Alt+Tab né nella taskbar, ma si attiva (riceve la tastiera).
unsafe fn apply_styles(hwnd: HWND) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let want = (ex | WS_EX_TOOLWINDOW.0 as isize) & !(WS_EX_APPWINDOW.0 as isize | WS_EX_NOACTIVATE.0 as isize);
        if want != ex {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, want);
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
    }
}

fn create(app: &AppHandle) -> tauri::Result<()> {
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=launcher".into()))
        .title("EosOS Launcher")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .inner_size(WIDTH, MIN_HEIGHT)
        .build()?;
    if let Ok(hwnd) = win.hwnd() {
        HWND_RAW.store(hwnd.0 as isize, Ordering::Relaxed);
        unsafe { apply_styles(hwnd) };
    }
    Ok(())
}

/// Monitor dove sta il cursore: (area di lavoro in px fisici, scala). Ripiego: il primario.
fn target_monitor(app: &AppHandle, win: &tauri::WebviewWindow) -> Option<(Rect, f64)> {
    let cur = win.cursor_position().ok();
    let monitors = app.available_monitors().ok()?;
    let to_rect = |m: &tauri::Monitor| {
        let a = m.work_area();
        (Rect { x: a.position.x, y: a.position.y, w: a.size.width as i32, h: a.size.height as i32 }, m.scale_factor())
    };
    if let Some(c) = cur
        && let Some(m) = monitors.iter().find(|m| {
            let (p, s) = (m.position(), m.size());
            c.x >= f64::from(p.x) && c.x < f64::from(p.x) + f64::from(s.width) && c.y >= f64::from(p.y) && c.y < f64::from(p.y) + f64::from(s.height)
        })
    {
        return Some(to_rect(m));
    }
    app.primary_monitor().ok().flatten().as_ref().map(to_rect)
}

/// Con la sfocatura attiva la finestra è esclusa dalla cattura dello schermo (come il Dock):
/// effetto collaterale, screenshot e registrazioni non includono il Launcher. Se il sistema non
/// lo supporta (prima della versione 2004) non c'è sfocatura e il vetro resta semplice.
fn update_affinity() {
    let raw = HWND_RAW.load(Ordering::Relaxed);
    if raw == 0 {
        return;
    }
    let want = crate::dockwin::blur_enabled();
    let ok = unsafe {
        SetWindowDisplayAffinity(HWND(raw as *mut _), if want { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE })
    }
    .is_ok();
    EXCLUDED.store(want && ok, Ordering::Relaxed);
}

/// Cosa catturare per la sfocatura: (finestra, rettangolo da catturare), in pixel fisici. `None` se
/// il Launcher è nascosto, la sfocatura è spenta o la finestra non si è potuta escludere.
pub fn capture_info() -> Option<(Rect, Rect)> {
    if !is_visible() || !EXCLUDED.load(Ordering::Relaxed) {
        return None;
    }
    let hwnd = HWND(HWND_RAW.load(Ordering::Relaxed) as *mut _);
    let mut r = RECT::default();
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    let win = unsafe {
        GetWindowRect(hwnd, &mut r).ok()?;
        if !GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut mi).as_bool() {
            return None;
        }
        Rect { x: r.left, y: r.top, w: r.right - r.left, h: r.bottom - r.top }
    };
    let m = mi.rcMonitor;
    let monitor = Rect { x: m.left, y: m.top, w: m.right - m.left, h: m.bottom - m.top };
    let scale = f64::from_bits(SCALE_BITS.load(Ordering::Relaxed));
    capture_rect(win, monitor, capture_margin(if scale > 0.0 { scale } else { 1.0 })).map(|c| (win, c))
}

pub fn is_visible() -> bool {
    VISIBLE.load(Ordering::Relaxed)
}

/// Mostra il Launcher sul monitor del cursore e gli dà la tastiera.
pub fn show(app: &AppHandle) {
    let Some(win) = win_of(app) else { return };
    let Some((area, scale)) = target_monitor(app, &win) else { return };
    let r = place(area, scale, MIN_HEIGHT);
    SCALE_BITS.store(scale.to_bits(), Ordering::Relaxed);
    let _ = win.set_position(PhysicalPosition::new(r.x, r.y));
    let _ = win.set_size(PhysicalSize::new(r.w as u32, r.h as u32));
    SHOWN_AT_MS.store(since_start().as_millis() as u64 + 1, Ordering::Relaxed);
    VISIBLE.store(true, Ordering::Relaxed);
    update_affinity();
    let _ = win.show();
    if let Ok(hwnd) = win.hwnd() {
        unsafe { apply_styles(hwnd) };
        crate::dockact::focus(hwnd);
    }
    let _ = win.set_focus();
    // La pagina azzera il campo e lo mette a fuoco.
    let _ = app.emit_to(LABEL, "launcher-open", ());
    // Se il primo piano non arriva (Windows lo nega), la finestra non servirebbe a nulla.
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FOCUS_CHECK);
        if !is_visible() {
            return;
        }
        let Some(win) = win_of(&app) else { return };
        if let Ok(hwnd) = win.hwnd()
            && unsafe { GetForegroundWindow() } != hwnd
        {
            hide(&app);
        }
    });
}

pub fn hide(app: &AppHandle) {
    VISIBLE.store(false, Ordering::Relaxed);
    if let Some(win) = win_of(app) {
        let _ = win.hide();
    }
}

pub fn toggle(app: &AppHandle) {
    if is_visible() {
        hide(app);
    } else {
        show(app);
    }
}

/// La pagina comunica l'altezza che le serve (campo + risultati), in px CSS. La cima resta ferma.
pub fn fit(app: &AppHandle, height_css: f64) {
    let Some(win) = win_of(app) else { return };
    let scale = f64::from_bits(SCALE_BITS.load(Ordering::Relaxed));
    let scale = if scale > 0.0 { scale } else { win.scale_factor().unwrap_or(1.0) };
    let h = (clamp_height(height_css) * scale).round() as u32;
    let w = (WIDTH * scale).round() as u32;
    let _ = win.set_size(PhysicalSize::new(w, h));
}

/// Eventi della finestra del Launcher: perdita di focus -> nasconde; chiusura (Alt+F4) -> nasconde
/// senza chiudere l'app. Restituisce `true` se l'evento era suo.
pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) -> bool {
    if window.label() != LABEL {
        return false;
    }
    match event {
        tauri::WindowEvent::Focused(false) => {
            let shown = SHOWN_AT_MS.load(Ordering::Relaxed);
            let elapsed = since_start().saturating_sub(Duration::from_millis(shown.saturating_sub(1)));
            if is_visible() && focus_loss_counts(elapsed) {
                hide(window.app_handle());
            }
        }
        tauri::WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide(window.app_handle());
        }
        _ => {}
    }
    true
}

// ------------------------------------------------------------------ scorciatoia

/// Una scorciatoia globale: modificatori (`MOD_*`) e codice di tasto virtuale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hotkey {
    pub mods: u32,
    pub vk: u32,
}

const MOD_ALT_BIT: u32 = 0x1;
const MOD_CTRL_BIT: u32 = 0x2;
const MOD_SHIFT_BIT: u32 = 0x4;
const MOD_WIN_BIT: u32 = 0x8;
const VK_SPACE_CODE: u32 = 0x20;

/// Alt+Spazio.
pub const DEFAULT_HOTKEY: Hotkey = Hotkey { mods: MOD_ALT_BIT, vk: VK_SPACE_CODE };

/// Legge una scorciatoia scritta come "Ctrl+Alt+Space" (maiuscole libere; accetta anche i nomi
/// italiani Maiusc e Spazio). Serve almeno un modificatore oltre a Maiusc: il solo Maiusc o un
/// tasto nudo dirotterebbero la digitazione normale. Un solo tasto, senza ripetizioni; tasti
/// ammessi: Spazio, lettere, cifre, F1-F12; Alt+F4 è rifiutata. Pura, testata.
pub fn parse_hotkey(text: &str) -> Option<Hotkey> {
    let (mut mods, mut key): (u32, Option<u32>) = (0, None);
    for raw in text.split('+') {
        let t = raw.trim().to_lowercase();
        let bit = match t.as_str() {
            "ctrl" | "control" | "ctl" => MOD_CTRL_BIT,
            "alt" => MOD_ALT_BIT,
            "shift" | "maiusc" => MOD_SHIFT_BIT,
            "win" | "windows" | "super" | "meta" | "cmd" => MOD_WIN_BIT,
            _ => 0,
        };
        if bit != 0 {
            if mods & bit != 0 {
                return None;
            }
            mods |= bit;
            continue;
        }
        let vk = match t.as_str() {
            "space" | "spazio" => VK_SPACE_CODE,
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_alphabetic() => u32::from(s.as_bytes()[0].to_ascii_uppercase()),
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_digit() => u32::from(s.as_bytes()[0]),
            s if s.starts_with('f') && s[1..].parse::<u32>().is_ok_and(|n| (1..=12).contains(&n)) => {
                0x70 + s[1..].parse::<u32>().unwrap_or(1) - 1
            }
            _ => return None,
        };
        if key.replace(vk).is_some() {
            return None;
        }
    }
    let vk = key?;
    if mods & !MOD_SHIFT_BIT == 0 || (mods == MOD_ALT_BIT && vk == 0x73) {
        return None;
    }
    Some(Hotkey { mods, vk })
}

/// La scorciatoia come testo canonico ("Ctrl+Alt+Space"), che `parse_hotkey` rilegge uguale.
/// Pura, testata.
pub fn format_hotkey(h: Hotkey) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (bit, name) in [(MOD_CTRL_BIT, "Ctrl"), (MOD_ALT_BIT, "Alt"), (MOD_SHIFT_BIT, "Shift"), (MOD_WIN_BIT, "Win")] {
        if h.mods & bit != 0 {
            parts.push(name.into());
        }
    }
    parts.push(match h.vk {
        VK_SPACE_CODE => "Space".into(),
        0x70..=0x7B => format!("F{}", h.vk - 0x70 + 1),
        v => char::from_u32(v).map(|c| c.to_string()).unwrap_or_default(),
    });
    parts.join("+")
}

/// Come si mostra all'utente ("Ctrl+Alt+Spazio").
pub fn display_hotkey(h: Hotkey) -> String {
    format_hotkey(h).replace("Space", "Spazio").replace("Shift", "Maiusc")
}

/// La scorciatoia salvata, o quella predefinita se manca o non si legge.
fn configured_hotkey() -> Hotkey {
    crate::launcherstore::settings().hotkey.as_deref().and_then(parse_hotkey).unwrap_or(DEFAULT_HOTKEY)
}

/// Messaggio privato per il thread della scorciatoia: "cambia combinazione".
const WM_CHANGE_HOTKEY: u32 = WM_APP + 1;
static HOTKEY_THREAD: AtomicU32 = AtomicU32::new(0);
type Change = (Hotkey, std::sync::mpsc::Sender<bool>);
static PENDING_CHANGE: Mutex<Option<Change>> = Mutex::new(None);

fn try_register(h: Hotkey) -> bool {
    unsafe { RegisterHotKey(None, HOTKEY_ID, HOT_KEY_MODIFIERS(h.mods) | MOD_NOREPEAT, h.vk) }.is_ok()
}

/// Cambia la scorciatoia subito, senza riavviare. `true` se la nuova combinazione è stata
/// registrata (e salvata); `false` se è occupata da un altro programma o il thread non risponde:
/// in quel caso resta quella di prima.
pub fn change_hotkey(h: Hotkey) -> bool {
    let tid = HOTKEY_THREAD.load(Ordering::Relaxed);
    if tid == 0 {
        return false;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    *PENDING_CHANGE.lock().unwrap_or_else(|e| e.into_inner()) = Some((h, tx));
    if unsafe { PostThreadMessageW(tid, WM_CHANGE_HOTKEY, WPARAM(0), LPARAM(0)) }.is_err() {
        return false;
    }
    rx.recv_timeout(Duration::from_secs(2)).unwrap_or(false)
}

/// Thread della scorciatoia: registra la combinazione, ascolta WM_HOTKEY e accetta cambi a caldo.
/// Se la combinazione è occupata avvisa una volta e riprova ogni 30 secondi (timer del thread).
fn hotkey_thread(app: AppHandle) {
    HOTKEY_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
    let mut current = configured_hotkey();
    let mut registered = try_register(current);
    let mut warned = false;
    let warn = |h: Hotkey| {
        eprintln!("launcher: {} è già usata da un altro programma", display_hotkey(h));
        let text = HSTRING::from(format!(
            "La scorciatoia {} del Launcher è già usata da un altro programma (per esempio PowerToys Run). Chiudi quel programma: EosOS riproverà da solo ogni 30 secondi.",
            display_hotkey(h)
        ));
        std::thread::spawn(move || unsafe {
            MessageBoxW(None, &text, w!("EosOS"), MB_OK | MB_ICONINFORMATION);
        });
    };
    if !registered {
        warned = true;
        warn(current);
    }
    unsafe { SetTimer(None, 1, RETRY_EVERY.as_millis() as u32, None) };
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            match msg.message {
                WM_HOTKEY if msg.wParam == WPARAM(HOTKEY_ID as usize) => toggle(&app),
                WM_TIMER if !registered => {
                    registered = try_register(current);
                    if registered && warned {
                        eprintln!("launcher: {} registrata dopo un conflitto", display_hotkey(current));
                    }
                }
                WM_CHANGE_HOTKEY => {
                    let change = PENDING_CHANGE.lock().unwrap_or_else(|e| e.into_inner()).take();
                    if let Some((new, tx)) = change {
                        if registered {
                            let _ = UnregisterHotKey(None, HOTKEY_ID);
                        }
                        if try_register(new) {
                            current = new;
                            registered = true;
                            warned = false;
                            let _ = tx.send(true);
                        } else {
                            registered = try_register(current); // si torna a quella di prima
                            let _ = tx.send(false);
                        }
                    }
                }
                _ => {}
            }
        }
        let _ = UnregisterHotKey(None, HOTKEY_ID);
    }
}

pub fn start(app: &AppHandle) {
    if let Err(e) = create(app) {
        eprintln!("launcher: finestra non creata: {e}");
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || hotkey_thread(app));
}

#[cfg(test)]
mod tests {
    use super::*;

    const MON: Rect = Rect { x: 0, y: 0, w: 1920, h: 1040 };

    #[test]
    fn clamp_height_limits_and_survives_garbage() {
        assert_eq!(clamp_height(10.0), MIN_HEIGHT);
        assert_eq!(clamp_height(300.0), 300.0);
        assert_eq!(clamp_height(9999.0), MAX_HEIGHT);
        assert_eq!(clamp_height(f64::NAN), MIN_HEIGHT);
        assert_eq!(clamp_height(f64::INFINITY), MIN_HEIGHT);
        assert_eq!(clamp_height(-5.0), MIN_HEIGHT);
    }

    #[test]
    fn place_centers_horizontally_and_sits_a_fifth_down() {
        let r = place(MON, 1.0, 72.0);
        assert_eq!((r.w, r.h), (680, 72));
        assert_eq!(r.x, (1920 - 680) / 2);
        assert_eq!(r.y, 1040 / 5);
    }

    #[test]
    fn place_scales_with_dpi_and_follows_the_monitor() {
        let second = Rect { x: -2560, y: 100, w: 2560, h: 1300 };
        let r = place(second, 1.5, 100.0);
        assert_eq!((r.w, r.h), (1020, 150));
        assert_eq!(r.x, -2560 + (2560 - 1020) / 2);
        assert_eq!(r.y, 100 + 1300 / 5);
        assert!(r.x >= second.x && r.x + r.w <= second.x + second.w);
    }

    #[test]
    fn place_never_leaves_the_work_area() {
        // Area bassa: il pannello alto sale per restare dentro.
        let low = Rect { x: 0, y: 0, w: 1366, h: 500 };
        let r = place(low, 1.0, MAX_HEIGHT);
        assert!(r.y >= 0 && r.y + r.h <= 500);
        // Area più stretta del pannello: si restringe.
        let narrow = Rect { x: 10, y: 0, w: 400, h: 800 };
        let r = place(narrow, 1.0, 72.0);
        assert_eq!((r.x, r.w), (10, 400));
        // Scala assurda: ripiego su 1.
        assert_eq!(place(MON, f64::NAN, 72.0).w, 680);
        assert_eq!(place(MON, 0.0, 72.0).w, 680);
    }

    #[test]
    fn hotkeys_parse_in_english_and_italian_and_roundtrip() {
        assert_eq!(parse_hotkey("Alt+Space"), Some(DEFAULT_HOTKEY));
        assert_eq!(parse_hotkey(" alt + spazio "), Some(DEFAULT_HOTKEY));
        let h = parse_hotkey("Ctrl+Alt+Space").unwrap();
        assert_eq!(h, Hotkey { mods: 0x3, vk: 0x20 });
        assert_eq!(parse_hotkey("Ctrl+Maiusc+K"), Some(Hotkey { mods: 0x6, vk: u32::from(b'K') }));
        assert_eq!(parse_hotkey("Win+F9"), Some(Hotkey { mods: 0x8, vk: 0x78 }));
        assert_eq!(parse_hotkey("ctrl+7"), Some(Hotkey { mods: 0x2, vk: u32::from(b'7') }));
        for text in ["Alt+Space", "Ctrl+Alt+Space", "Ctrl+Shift+K", "Win+F9", "Ctrl+Alt+Shift+Win+F12", "Ctrl+7"] {
            let h = parse_hotkey(text).unwrap();
            assert_eq!(format_hotkey(h), text);
            assert_eq!(parse_hotkey(&format_hotkey(h)), Some(h));
        }
        assert_eq!(display_hotkey(parse_hotkey("Ctrl+Shift+Space").unwrap()), "Ctrl+Maiusc+Spazio");
    }

    #[test]
    fn dangerous_or_malformed_hotkeys_are_rejected() {
        for bad in [
            "", "Space", "K", "Shift+K", "Shift+Space", "Alt", "Ctrl+Alt", "Alt+Alt+K", "Ctrl+K+L", "Ctrl+Esc",
            "Ctrl+F13", "Ctrl+F0", "Ctrl++K", "+", "Alt+F4", "Ctrl+ctrl+K", "Ctrl+é",
        ] {
            assert_eq!(parse_hotkey(bad), None, "{bad:?} non deve essere accettata");
        }
        assert!(parse_hotkey("Ctrl+F4").is_some(), "solo Alt+F4 e' vietata");
    }

    #[test]
    fn focus_loss_is_ignored_only_right_after_opening() {
        assert!(!focus_loss_counts(Duration::from_millis(0)));
        assert!(!focus_loss_counts(Duration::from_millis(349)));
        assert!(focus_loss_counts(Duration::from_millis(350)));
        assert!(focus_loss_counts(Duration::from_secs(5)));
    }
}
