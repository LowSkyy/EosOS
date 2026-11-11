//! Mission Control (fasi M2 e M3): le finestre panoramica (una per monitor), le miniature vive, l'animazione
//! e i modi di aprirla.
//!
//! Ogni monitor ha una finestra Tauri senza bordi, trasparente, sempre in primo piano, creata nascosta
//! all'avvio (come il Launcher). All'apertura:
//! 1. si **scatta uno screenshot di ogni monitor** (prima di mostrare nulla) per lo sfondo sfocato: le
//!    finestre sotto non si muovono mentre la panoramica è aperta, quindi lo sfondo può essere statico;
//! 2. si elencano le finestre e si dispongono a griglia (`mc::snapshot`);
//! 3. si registrano le **miniature vive** con `DwmRegisterThumbnail` (Windows disegna la finestra
//!    sorgente dentro la nostra, in tempo reale; le compone **sopra** la pagina) e si manda la vista alla
//!    pagina;
//! 4. **animazione (M3)**: ogni miniatura nasce esattamente sopra la finestra vera e scivola alla sua
//!    cella in 220 ms (`mctrigger::thumb_at`, curva ease-out); alla chiusura fa il percorso inverso in
//!    180 ms prima di nascondere tutto. Mentre la panoramica è aperta lo sfondo e le etichette della pagina
//!    compaiono con una dissolvenza.
//!
//! Modi di aprirla: la voce del Launcher, la **scorciatoia registrata** (predefinita Ctrl+Alt+Su, con
//! avviso e nuovi tentativi se è già presa) e l'**angolo attivo** (spento di default: il cursore fermo
//! 500 ms nell'angolo in alto a sinistra di un monitor; non scatta con un pulsante del mouse premuto né
//! con un'app a schermo intero). Win+Tab e Alt+Tab richiedono un hook di tastiera e arrivano in M6.
//!
//! Sicurezza: la pagina conosce le finestre solo per `id`; attivare o chiudere accetta soltanto id che la
//! panoramica ha appena elencato e che esistono ancora. Chiudere una finestra manda `WM_CLOSE`
//! (mai si termina un processo). Alla chiusura tutto si libera e la pagina dimentica titoli e icone.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::core::{w, HSTRING};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmQueryThumbnailSourceSize, DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
    DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_VISIBLE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, RegisterHotKey, HOT_KEY_MODIFIERS, MOD_NOREPEAT};
use windows::Win32::UI::Shell::SHQueryUserNotificationState;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetCursorPos, GetForegroundWindow, GetMessageW, GetTopWindow, GetWindow, GWL_EXSTYLE, GW_HWNDNEXT, WS_EX_TOPMOST, GetWindowLongPtrW, IsIconic, IsWindow,
    IsWindowVisible, MessageBoxW, PostMessageW, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    TranslateMessage, GWL_STYLE, HWND_TOPMOST, MB_ICONINFORMATION, MB_OK, MSG, SWP_FRAMECHANGED, SWP_NOACTIVATE,
    SW_RESTORE, WM_CLOSE, WM_HOTKEY, WM_TIMER, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
};

use crate::dockgeom::Rect;
use crate::launcherwin::{display_hotkey, parse_hotkey, Hotkey};
use crate::mc::{monitors, snapshot, thumb_rect, view_of, McMonitor, View};
use crate::mctrigger::{
    blocks_corner, corner_step, in_corner, thumb_at, CornerState, CLOSE_MS, CORNER_PX, DWELL_MS, FRAME_MS, OPEN_MS,
};

/// Titolo comune delle finestre panoramica (una per monitor).
pub const TITLE: &str = "EosOS Mission Control";
const PREFIX: &str = "mc-";
/// Quanto si aspetta dopo "chiudi finestra" prima di rileggere l'elenco.
const REFRESH_AFTER: Duration = Duration::from_millis(400);
/// Scorciatoia di apertura predefinita.
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+Up";
const HOTKEY_ID: i32 = 0x4D43; // "MC"
const RETRY_EVERY: Duration = Duration::from_secs(30);

static APP: OnceLock<AppHandle> = OnceLock::new();
/// L'angolo attivo è acceso (copia in memoria dell'impostazione, letta dal thread dell'angolo).
static CORNER: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Closed,
    Opening,
    Open,
    Closing,
}

/// Una miniatura da animare: dalla posizione della finestra vera alla sua cella (pixel dello schermo).
#[derive(Clone)]
struct Anim {
    /// Id della finestra sorgente (per nascondere la sua miniatura durante un trascinamento).
    id: u64,
    handle: isize,
    /// Origine del monitor: il rettangolo di destinazione di DWM è in coordinate della finestra panoramica.
    origin: (i32, i32),
    from: Rect,
    to: Rect,
}

struct State {
    phase: Phase,
    /// Id elencati: gli unici su cui si può agire.
    ids: Vec<u64>,
    /// Miniature registrate (handle DWM).
    thumbs: Vec<isize>,
    anims: Vec<Anim>,
    /// Scatti dello sfondo, uno per monitor (fotogramma serializzato come in `backdrop::encode`).
    backdrops: Vec<Vec<u8>>,
    /// Generazione: sale a ogni apertura o chiusura; un'animazione si ferma se non è più la sua.
    epoch: u64,
    /// Fotogrammi e durata (ms) dell'ultima animazione (per la diagnostica e le prove).
    frames: u32,
    anim_ms: u64,
    /// Alt+Tab: finestre in ordine di uso recente (la prima è quella che era in primo piano) e quella evidenziata.
    order: Vec<u64>,
    sel: Option<usize>,
}

static ST: Mutex<State> = Mutex::new(State {
    phase: Phase::Closed,
    ids: Vec::new(),
    thumbs: Vec::new(),
    anims: Vec::new(),
    backdrops: Vec::new(),
    epoch: 0,
    frames: 0,
    anim_ms: 0,
    order: Vec::new(),
    sel: None,
});

fn st() -> std::sync::MutexGuard<'static, State> {
    ST.lock().unwrap_or_else(|e| e.into_inner())
}

fn label(i: usize) -> String {
    format!("{PREFIX}{i}")
}

fn win_of(app: &AppHandle, i: usize) -> Option<tauri::WebviewWindow> {
    app.get_webview_window(&label(i))
}

/// Etichette di tutte le finestre panoramica esistenti.
fn all_labels(app: &AppHandle) -> Vec<String> {
    let mut v: Vec<String> = app.webview_windows().keys().filter(|k| k.starts_with(PREFIX)).cloned().collect();
    v.sort();
    v
}

fn create_one(app: &AppHandle, i: usize) -> tauri::Result<()> {
    let win = WebviewWindowBuilder::new(app, label(i), WebviewUrl::App(format!("index.html?view=mc&m={i}").into()))
        .title(TITLE)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .inner_size(800.0, 600.0)
        .build()?;
    crate::chromeguard::protect(&win);
    crate::webmem::hidden(&win);
    if let Ok(hwnd) = win.hwnd() {
        unsafe { crate::launcherwin::apply_styles(hwnd) };
    }
    Ok(())
}

/// Crea le finestre dei monitor presenti (nascoste) e avvia i modi di aprire la panoramica. Le finestre
/// mancanti si creano anche all'apertura.
pub fn start(app: &AppHandle) {
    let _ = APP.set(app.clone());
    for i in 0..monitors(app).len() {
        if let Err(e) = create_one(app, i) {
            eprintln!("mission control: finestra {i} non creata: {e}");
        }
    }
    CORNER.store(crate::mcstore::load().hot_corner, Ordering::Relaxed);
    let a = app.clone();
    std::thread::spawn(move || hotkey_thread(a));
    let a = app.clone();
    std::thread::spawn(move || corner_thread(a));
    let c = crate::mcstore::load();
    crate::mchook::start(on_key_cmd, keys_config(&c));
}

/// Stili di cornice che la panoramica non deve avere: con una cornice la finestra esce dal monitor e le
/// miniature (posizionate rispetto all'area client) risultano sfalsate.
const FRAME_STYLES: isize = (WS_CAPTION.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0 | WS_THICKFRAME.0) as isize;

/// Toglie gli stili di cornice e porta la finestra ESATTAMENTE sul rettangolo `r` (pixel fisici) con Win32,
/// senza passare dalle conversioni di Tauri. Dopo un cambio di scala dal vivo Tauri riapplica la cornice (lo
/// stesso difetto che ebbe il Dock), quindi lo si rifà a ogni apertura. Pura nel senso che dipende solo da `r`.
unsafe fn fit_exact(hwnd: HWND, r: Rect) {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        if style & FRAME_STYLES != 0 {
            let fresh = GetWindowLongPtrW(hwnd, GWL_STYLE);
            SetWindowLongPtrW(hwnd, GWL_STYLE, fresh & !FRAME_STYLES);
        }
        let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), r.x, r.y, r.w, r.h, SWP_NOACTIVATE | SWP_FRAMECHANGED);
    }
}

fn hwnd_of(win: &tauri::WebviewWindow) -> Option<HWND> {
    win.hwnd().ok()
}

fn unregister_all() {
    let handles: Vec<isize> = {
        let mut s = st();
        s.anims.clear();
        std::mem::take(&mut s.thumbs)
    };
    for h in handles {
        unsafe {
            let _ = DwmUnregisterThumbnail(h);
        }
    }
}

/// Imposta il rettangolo di destinazione di una miniatura (pixel dello schermo, convertito nelle coordinate
/// della finestra panoramica del suo monitor).
fn set_dest(handle: isize, origin: (i32, i32), r: Rect) -> bool {
    let props = DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_RECTDESTINATION,
        rcDestination: RECT { left: r.x - origin.0, top: r.y - origin.1, right: r.x - origin.0 + r.w, bottom: r.y - origin.1 + r.h },
        ..Default::default()
    };
    unsafe { DwmUpdateThumbnailProperties(handle, &props) }.is_ok()
}

/// Registra le miniature vive dei monitor nelle rispettive finestre e restituisce, per monitor, il
/// rettangolo finale (pixel dello schermo) di quelle riuscite. Con `animate` ogni miniatura parte
/// esattamente sopra la finestra vera; senza, nasce già alla sua cella. Una finestra che Windows non
/// lascia riprodurre (per esempio di un processo con privilegi più alti) resta senza miniatura.
fn register_thumbs(app: &AppHandle, snap: &[McMonitor], animate: bool) -> Vec<HashMap<u64, Rect>> {
    let mut all = Vec::new();
    for (i, m) in snap.iter().enumerate() {
        let mut map = HashMap::new();
        let Some(dest) = win_of(app, i).as_ref().and_then(hwnd_of) else {
            all.push(map);
            continue;
        };
        for w in m.windows.iter().filter(|w| !w.minimized) {
            let src = HWND(w.id as usize as *mut _);
            unsafe {
                let Ok(th) = DwmRegisterThumbnail(dest, src) else { continue };
                let fit = DwmQueryThumbnailSourceSize(th)
                    .ok()
                    .and_then(|size: SIZE| thumb_rect(Rect { x: w.cell[0], y: w.cell[1], w: w.cell[2], h: w.cell[3] }, (size.cx, size.cy), m.scale));
                let Some(to) = fit else {
                    let _ = DwmUnregisterThumbnail(th);
                    continue;
                };
                let from = if animate { Rect { x: w.rect[0], y: w.rect[1], w: w.rect[2].max(1), h: w.rect[3].max(1) } } else { to };
                let r = from;
                let props = DWM_THUMBNAIL_PROPERTIES {
                    dwFlags: DWM_TNP_RECTDESTINATION | DWM_TNP_VISIBLE | DWM_TNP_OPACITY,
                    rcDestination: RECT { left: r.x - m.x, top: r.y - m.y, right: r.x - m.x + r.w, bottom: r.y - m.y + r.h },
                    opacity: 255,
                    fVisible: true.into(),
                    ..Default::default()
                };
                if DwmUpdateThumbnailProperties(th, &props).is_err() {
                    let _ = DwmUnregisterThumbnail(th);
                    continue;
                }
                let mut s = st();
                s.thumbs.push(th);
                s.anims.push(Anim { id: w.id, handle: th, origin: (m.x, m.y), from, to });
                map.insert(w.id, to);
            }
        }
        all.push(map);
    }
    all
}

/// Icone (data URL) delle app presenti, una per identità.
fn icons_for(snap: &[McMonitor]) -> HashMap<PathBuf, String> {
    let mut out = HashMap::new();
    for w in snap.iter().flat_map(|m| &m.windows) {
        if !out.contains_key(&w.identity)
            && let Some(icon) = crate::dock::cached_icon(&w.identity)
        {
            out.insert(w.identity.clone(), icon);
        }
    }
    out
}

fn views(app: &AppHandle, snap: &[McMonitor], animate: bool) -> Vec<View> {
    let thumbs = register_thumbs(app, snap, animate);
    let icons = icons_for(snap);
    let desktops = crate::vdesk::list();
    let rep = crate::vdint::report(false);
    let vd = crate::mc::VdInfo {
        internal: rep.caps.internal,
        move_window: rep.caps.move_window,
        remove_any: rep.caps.remove_any,
        rename: rep.caps.rename,
        reason: rep.reason.clone(),
    };
    snap.iter()
        .enumerate()
        .map(|(i, m)| {
            let mut v = view_of(i, m, &thumbs[i], &icons);
            v.desktops = desktops.clone();
            v.vd = vd.clone();
            v
        })
        .collect()
}

fn remember_ids(snap: &[McMonitor]) {
    st().ids = snap.iter().flat_map(|m| m.windows.iter().map(|w| w.id)).collect();
}

/// Il monitor dove sta il cursore (indice); 0 se non si sa.
fn cursor_monitor(mons: &[crate::mc::Mon]) -> usize {
    let mut p = POINT::default();
    if unsafe { GetCursorPos(&mut p) }.is_err() {
        return 0;
    }
    mons.iter().position(|m| m.rect.contains((p.x, p.y))).unwrap_or(0)
}

/// Fa scorrere le miniature per `total` ms (avanti: dalla finestra alla cella; indietro: il contrario).
/// Si ferma se la generazione cambia (nuova apertura o chiusura immediata). Restituisce i fotogrammi fatti.
fn run_anim(epoch: u64, reverse: bool, total: u64) -> u32 {
    let anims: Vec<Anim> = st().anims.clone();
    let start = Instant::now();
    let mut frames = 0u32;
    loop {
        if st().epoch != epoch {
            return frames;
        }
        let elapsed = start.elapsed().as_millis() as u64;
        for a in &anims {
            let _ = set_dest(a.handle, a.origin, thumb_at(a.from, a.to, elapsed, total, reverse));
        }
        frames += 1;
        if elapsed >= total {
            return frames;
        }
        std::thread::sleep(Duration::from_millis(FRAME_MS));
    }
}

/// Apre la panoramica su tutti i monitor. Se è ancora in chiusura (l'animazione dura 180 ms) aspetta che
/// finisca, al massimo 600 ms, invece di ignorare la richiesta; se è già aperta o in apertura non fa nulla.
pub fn open(app: &AppHandle) {
    if crate::lockscreen::is_locked() {
        return;
    }
    for _ in 0..40 {
        if st().phase != Phase::Closing {
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    let epoch = {
        let mut s = st();
        if s.phase != Phase::Closed {
            return;
        }
        s.phase = Phase::Opening;
        s.epoch += 1;
        s.epoch
    };
    let mons = monitors(app);
    if mons.is_empty() {
        st().phase = Phase::Closed;
        return;
    }
    for i in 0..mons.len() {
        if win_of(app, i).is_none() {
            let _ = create_one(app, i);
        }
    }
    // 1. Lo sfondo: PRIMA di mostrare qualunque finestra panoramica.
    let backdrops: Vec<Vec<u8>> = mons.iter().map(|m| crate::backdrop::capture_bytes(m.rect).unwrap_or_default()).collect();
    // 2. Le finestre e la loro disposizione.
    let snap = snapshot(app);
    remember_ids(&snap);
    st().backdrops = backdrops;
    // 3. Le finestre panoramica, a tutto monitor.
    let focus = cursor_monitor(&mons);
    // Con Alt o Windows premuti (Alt+Tab, Win+Tab) il primo piano si prende dopo: il trucco dell'Alt di
    // `focus` e di `set_focus` azzererebbe il tasto dell'utente nello stato di Windows.
    let defer_focus = crate::dockact::alt_or_win_down();
    for (i, m) in mons.iter().enumerate() {
        let Some(win) = win_of(app, i) else { continue };
        let _ = win.set_position(PhysicalPosition::new(m.rect.x, m.rect.y));
        let _ = win.set_size(PhysicalSize::new(m.rect.w as u32, m.rect.h as u32));
        if let Some(h) = hwnd_of(&win) {
            unsafe { fit_exact(h, m.rect) };
        }
        crate::webmem::shown(&win);
        let _ = win.show();
        if let Some(h) = hwnd_of(&win) {
            unsafe {
                crate::launcherwin::apply_styles(h);
                fit_exact(h, m.rect); // dopo show: Tauri puo' aver riapplicato qualcosa
            }
            if i == focus && !defer_focus {
                crate::dockact::focus(h);
            }
        }
    }
    if defer_focus {
        let app = app.clone();
        std::thread::spawn(move || {
            // Si aspetta che l'utente lasci Alt/Windows (al massimo 3 s). Durante una sessione Alt+Tab il
            // primo piano non serve (guida l'hook) e al rilascio di Alt la panoramica si chiude.
            for _ in 0..60 {
                if !crate::dockact::alt_or_win_down() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            std::thread::sleep(Duration::from_millis(60));
            if matches!(st().phase, Phase::Open | Phase::Opening) && !crate::mchook::diag().session {
                take_focus(&app, focus);
            }
        });
    } else if let Some(win) = win_of(app, focus) {
        let _ = win.set_focus();
    }
    // 4. Le miniature (sopra le finestre vere) e la vista per ogni pagina; poi l'animazione.
    for v in views(app, &snap, true) {
        let _ = app.emit_to(label(v.index), "mc-open", &v);
    }
    std::thread::spawn(move || {
        let t = Instant::now();
        let frames = run_anim(epoch, false, OPEN_MS);
        let mut s = st();
        if s.epoch == epoch && s.phase == Phase::Opening {
            s.phase = Phase::Open;
            s.frames = frames;
            s.anim_ms = t.elapsed().as_millis() as u64;
        }
    });
}

/// Porta in primo piano la finestra panoramica del monitor `focus`.
fn take_focus(app: &AppHandle, focus: usize) {
    if let Some(win) = win_of(app, focus) {
        if let Some(h) = hwnd_of(&win) {
            crate::dockact::focus(h);
        }
        let _ = win.set_focus();
    }
}

/// Chiude subito, senza animazione: libera le miniature, nasconde le finestre e fa dimenticare tutto alle pagine.
pub fn close(app: &AppHandle) {
    {
        let mut s = st();
        if s.phase == Phase::Closed {
            return;
        }
        s.phase = Phase::Closed;
        s.epoch += 1;
        s.ids.clear();
        s.order.clear();
        s.sel = None;
        s.backdrops.clear();
    }
    unregister_all();
    for l in all_labels(app) {
        if let Some(w) = app.get_webview_window(&l) {
            let _ = w.hide();
            crate::webmem::hidden(&w);
        }
        let _ = app.emit_to(l, "mc-close", ());
    }
}

/// Chiude con l'animazione inversa (le miniature tornano sopra le finestre vere) e poi esegue `after`. Se
/// la panoramica è ancora in apertura o già in chiusura chiude subito.
pub fn close_animated(app: &AppHandle, after: Option<Box<dyn FnOnce() + Send>>) {
    let epoch = {
        let mut s = st();
        if s.phase != Phase::Open {
            drop(s);
            close(app);
            if let Some(f) = after {
                f();
            }
            return;
        }
        s.phase = Phase::Closing;
        s.epoch += 1;
        s.epoch
    };
    for l in all_labels(app) {
        let _ = app.emit_to(l, "mc-closing", ());
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let t = Instant::now();
        let frames = run_anim(epoch, true, CLOSE_MS);
        let still = {
            let mut s = st();
            let same = s.epoch == epoch;
            if same {
                s.frames = frames;
                s.anim_ms = t.elapsed().as_millis() as u64;
            }
            same
        };
        if still {
            close(&app);
            if let Some(f) = after {
                f();
            }
        }
    });
}

fn is_known(id: u64) -> bool {
    st().ids.contains(&id)
}

fn hwnd_from(id: u64) -> HWND {
    HWND(id as usize as *mut _)
}

/// Porta in primo piano la finestra `id` (ripristinandola se è minimizzata) e chiude la panoramica con
/// l'animazione. `false` se l'id non è tra quelli elencati o la finestra non esiste più.
pub fn activate(app: &AppHandle, id: u64) -> bool {
    if !is_known(id) {
        return false;
    }
    if !unsafe { IsWindow(Some(hwnd_from(id))) }.as_bool() {
        return false;
    }
    // L'HWND non attraversa i thread: dentro la chiusura si ricrea dall'id.
    close_animated(
        app,
        Some(Box::new(move || unsafe {
            let hwnd = hwnd_from(id);
            if IsWindow(Some(hwnd)).as_bool() {
                if IsIconic(hwnd).as_bool() {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
                crate::dockact::focus(hwnd);
            }
        })),
    );
    true
}

/// Chiede alla finestra `id` di chiudersi (`WM_CLOSE`, come il pulsante X) e dopo un attimo rilegge
/// l'elenco. Mai si termina un processo. `false` se l'id non è tra quelli elencati.
pub fn close_window(app: &AppHandle, id: u64) -> bool {
    if !is_known(id) || !unsafe { IsWindow(Some(hwnd_from(id))) }.as_bool() {
        return false;
    }
    let ok = unsafe { PostMessageW(Some(hwnd_from(id)), WM_CLOSE, WPARAM(0), LPARAM(0)) }.is_ok();
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(REFRESH_AFTER);
        refresh(&app);
    });
    ok
}

/// Rilegge le finestre e ridisegna la panoramica (dopo averne chiusa una), senza animazione.
pub fn refresh(app: &AppHandle) {
    if st().phase != Phase::Open {
        return;
    }
    let snap = snapshot(app);
    remember_ids(&snap);
    unregister_all();
    for v in views(app, &snap, false) {
        let _ = app.emit_to(label(v.index), "mc-update", &v);
    }
}

/// Passa al desktop virtuale `index` (indice nell'elenco). La panoramica si chiude subito (le finestre
/// cambiano) e i tasti di Windows partono dopo un attimo, a panoramica nascosta. `false` se l'indice non
/// è valido o è già il desktop corrente.
pub fn switch_desktop(app: &AppHandle, index: usize) -> bool {
    let list = crate::vdesk::list();
    if crate::vdesk::plan_switch(crate::vdesk::current_index(&list), index, list.len()).is_none() {
        return false;
    }
    close(app);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        let _ = crate::vdesk::switch_to(index);
    });
    true
}

/// Crea un nuovo desktop virtuale (e Windows ci passa) dopo aver chiuso la panoramica.
pub fn new_desktop(app: &AppHandle) -> bool {
    if st().phase == Phase::Closed {
        return false;
    }
    close(app);
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(200));
        let _ = crate::vdesk::create();
    });
    true
}

/// Nasconde o mostra la miniatura viva di una finestra (mentre la pagina la trascina disegna al suo posto
/// un'etichetta che segue il cursore). Solo per id elencati.
pub fn set_thumb_visible(id: u64, visible: bool) -> bool {
    if !is_known(id) {
        return false;
    }
    let handle = st().anims.iter().find(|a| a.id == id).map(|a| a.handle);
    let Some(h) = handle else { return false };
    let props = DWM_THUMBNAIL_PROPERTIES { dwFlags: DWM_TNP_VISIBLE, fVisible: visible.into(), ..Default::default() };
    unsafe { DwmUpdateThumbnailProperties(h, &props) }.is_ok()
}

/// Sposta la finestra `id` sul desktop `index` (backend interno). Solo con la panoramica aperta, un id elencato,
/// un indice valido e un profilo che ha superato la verifica. Dopo lo spostamento la panoramica si ridisegna: la
/// finestra non è più sul desktop corrente e sparisce dalla griglia.
pub fn move_window_to(app: &AppHandle, id: u64, index: usize) -> bool {
    if st().phase != Phase::Open || !is_known(id) || !unsafe { IsWindow(Some(hwnd_from(id))) }.as_bool() {
        return false;
    }
    let list = crate::vdesk::list();
    if index >= list.len() || list[index].current || !crate::vdint::caps().move_window {
        return false;
    }
    let ok = crate::vdint::move_window(id as usize, index).is_ok();
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(350));
        refresh(&app);
    });
    ok
}

/// Rimuove il desktop `index`. Con il backend interno: qualunque desktop, le sue finestre vanno al vicino. Senza:
/// solo il desktop corrente, con Win+Ctrl+F4 a panoramica chiusa. Con un solo desktop non si fa nulla.
pub fn remove_desktop(app: &AppHandle, index: usize) -> bool {
    if st().phase != Phase::Open {
        return false;
    }
    let list = crate::vdesk::list();
    if list.len() < 2 || index >= list.len() {
        return false;
    }
    if crate::vdint::caps().remove_any {
        let ok = crate::vdint::remove_desktop(index).is_ok();
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(350));
            refresh(&app);
        });
        return ok;
    }
    if !list[index].current {
        return false; // senza il backend interno non si puo' rimuovere un altro desktop
    }
    close(app);
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(200));
        let _ = crate::vdesk::remove_current();
    });
    true
}

/// Rinomina il desktop `index` (backend interno). Il nome è ripulito e limitato a 60 caratteri.
pub fn rename_desktop(app: &AppHandle, index: usize, name: &str) -> bool {
    if st().phase != Phase::Open || index >= crate::vdesk::list().len() || !crate::vdint::caps().rename {
        return false;
    }
    let ok = crate::vdint::rename_desktop(index, name).is_ok();
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        refresh(&app);
    });
    ok
}

/// Apre o chiude la panoramica (la scorciatoia).
pub fn toggle(app: &AppHandle) {
    let phase = st().phase;
    match phase {
        Phase::Closed => open(app),
        _ => close_animated(app, None),
    }
}

/// Lo scatto dello sfondo del monitor `index` (vuoto se non c'è).
pub fn backdrop(index: usize) -> Vec<u8> {
    st().backdrops.get(index).cloned().unwrap_or_default()
}

/// Apre la panoramica dall'esterno dell'app (per esempio dal Launcher), con un attimo di attesa
/// perché il Launcher abbia finito di chiudersi e non si riprenda il primo piano.
pub fn open_soon() {
    if let Some(app) = APP.get().cloned() {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            open(&app);
        });
    }
}

/// Accende o spegne l'angolo attivo e salva la scelta (azione del Launcher).
pub fn toggle_corner() {
    set_corner(!CORNER.load(Ordering::Relaxed));
}

/// Imposta l'angolo attivo e salva la scelta.
pub fn set_corner(on: bool) {
    crate::mcstore::set_hot_corner(on);
    CORNER.store(on, Ordering::Relaxed);
}

/// Stato per la diagnostica e le prove.
#[derive(Serialize)]
pub struct Diag {
    pub open: bool,
    pub phase: Phase,
    pub thumbs: usize,
    pub windows: usize,
    /// Fotogrammi dell'ultima animazione.
    pub frames: u32,
    pub anim_ms: u64,
    pub corner: bool,
    /// L'hook di Win+Tab e Alt+Tab.
    pub keys: crate::mchook::Diag,
    /// Alt+Tab: la finestra evidenziata.
    pub selected: Option<u64>,
    /// Alt+Tab: l'ordine di uso recente su cui si scorre (identificatori delle finestre).
    pub order: Vec<u64>,
}

pub fn diag() -> Diag {
    let s = st();
    Diag {
        open: s.phase != Phase::Closed,
        phase: s.phase,
        thumbs: s.thumbs.len(),
        windows: s.ids.len(),
        frames: s.frames,
        anim_ms: s.anim_ms,
        corner: CORNER.load(Ordering::Relaxed),
        keys: crate::mchook::diag(),
        selected: s.sel.and_then(|i| s.order.get(i).copied()),
        order: s.order.clone(),
    }
}

// ------------------------------------------------------------------ Win+Tab e Alt+Tab (hook di tastiera)

fn keys_config(s: &crate::mcstore::Settings) -> crate::mckeys::Config {
    crate::mckeys::Config { alt_tab: !s.windows_alt_tab, win_tab: !s.windows_win_tab, win_tap: !s.win_key_off }
}

/// Le finestre elencate in ordine di uso recente: l'ordine di sovrapposizione di Windows (la più in alto per prima).
///
/// Le finestre "sempre in primo piano" restano in fondo all'elenco (altrimenti passerebbero per prime senza che
/// l'utente le abbia usate) e `front`, la finestra che era davanti prima di aprire, va comunque per prima.
fn recent_order(ids: &[u64], front: Option<u64>) -> Vec<u64> {
    let mut out = Vec::new();
    let mut topmost = Vec::new();
    let mut h = unsafe { GetTopWindow(None) }.ok();
    let mut guard = 0;
    while let Some(cur) = h {
        let id = cur.0 as usize as u64;
        if ids.contains(&id) {
            let ex = unsafe { GetWindowLongPtrW(cur, GWL_EXSTYLE) } as u32;
            if ex & WS_EX_TOPMOST.0 != 0 { topmost.push(id) } else { out.push(id) }
        }
        guard += 1;
        if guard > 5000 {
            break;
        }
        h = unsafe { GetWindow(cur, GW_HWNDNEXT) }.ok();
    }
    out.extend(topmost);
    // Elencate ma non trovate nella catena (rarissimo): in coda, senza perderle.
    for id in ids {
        if !out.contains(id) {
            out.push(*id);
        }
    }
    if let Some(f) = front
        && let Some(p) = out.iter().position(|&x| x == f)
    {
        let id = out.remove(p);
        out.insert(0, id);
    }
    out
}

/// Esegue un comando dell'hook (nel suo thread, fuori dal callback).
fn on_key_cmd(cmd: crate::mckeys::Cmd) {
    use crate::mckeys::Cmd;
    let Some(app) = APP.get().cloned() else { return };
    match cmd {
        Cmd::Toggle => toggle(&app),
        Cmd::Next => alt_step(&app, 1),
        Cmd::Prev => alt_step(&app, -1),
        Cmd::Commit => alt_commit(&app),
        Cmd::Launcher => crate::launcherwin::toggle(&app),
        Cmd::Cancel => {
            reset_selection();
            close_animated(&app, None);
        }
    }
}

fn reset_selection() {
    let mut s = st();
    s.sel = None;
    s.order.clear();
}

/// Alt+Tab: apre la panoramica se serve, sposta l'evidenziazione di `delta` finestre e la comunica alle pagine.
fn alt_step(app: &AppHandle, delta: i32) {
    // La finestra davanti si legge PRIMA di aprire (poi il primo piano e' la panoramica).
    let front = {
        let h = unsafe { GetForegroundWindow() };
        if h.0.is_null() { None } else { Some(h.0 as usize as u64) }
    };
    if !matches!(st().phase, Phase::Open | Phase::Opening) {
        open(app);
    }
    let ids = {
        let s = st();
        if s.phase == Phase::Closed {
            return;
        }
        if s.order.is_empty() { Some(s.ids.clone()) } else { None }
    };
    // L'ordine delle finestre si legge fuori dal lock (interroga Windows).
    let fresh = ids.map(|ids| recent_order(&ids, front));
    let picked = {
        let mut s = st();
        if let Some(order) = fresh
            && s.order.is_empty()
        {
            s.order = order;
        }
        let next = crate::mckeys::cycle(s.order.len(), s.sel, delta);
        s.sel = next;
        next.and_then(|i| s.order.get(i).copied())
    };
    if let Some(id) = picked {
        for l in all_labels(app) {
            let _ = app.emit_to(l, "mc-select", id);
        }
    }
}

/// Alt rilasciato: attiva la finestra evidenziata (o chiude, se non ce n'è).
fn alt_commit(app: &AppHandle) {
    let target = {
        let s = st();
        s.sel.and_then(|i| s.order.get(i).copied())
    };
    reset_selection();
    match target {
        Some(id) if activate(app, id) => {}
        _ => close_animated(app, None),
    }
}

fn apply_keys() {
    crate::mchook::set_config(keys_config(&crate::mcstore::load()));
}

/// Azione del Launcher: "alt" o "win" passa fra la scorciatoia di Windows e quella di Mission Control.
pub fn toggle_key(which: &str) -> bool {
    let mc = crate::mcstore::load();
    match which {
        "alt" => set_key("alt", mc.windows_alt_tab), // `keep` vero = quello di Windows: lo si inverte
        "win" => set_key("win", mc.windows_win_tab),
        _ => return false,
    }
    true
}

/// Sceglie se Alt+Tab ("alt") o Win+Tab ("win") sono quelli di EosOS (`eos` vero) o di Windows. Vale subito.
pub fn set_key(which: &str, eos: bool) {
    match which {
        "alt" => crate::mcstore::set_windows_alt_tab(!eos),
        "win" => crate::mcstore::set_windows_win_tab(!eos),
        _ => return,
    };
    apply_keys();
}

/// Il tasto Windows da solo apre il Launcher (`on`) o lascia il menu Start. Vale subito.
pub fn set_win_key_launcher(on: bool) {
    crate::mcstore::set_win_key_launcher(on);
    apply_keys();
}

// ------------------------------------------------------------------ scorciatoia

fn configured_hotkey() -> Hotkey {
    crate::mcstore::load()
        .hotkey
        .as_deref()
        .and_then(parse_hotkey)
        .or_else(|| parse_hotkey(DEFAULT_HOTKEY))
        .unwrap_or(Hotkey { mods: 0x3, vk: 0x26 })
}

/// Messaggio privato per il thread della scorciatoia: "cambia combinazione".
const WM_CHANGE_HOTKEY: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 3;
static HOTKEY_THREAD: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
type Change = (Hotkey, std::sync::mpsc::Sender<bool>);
static PENDING_CHANGE: Mutex<Option<Change>> = Mutex::new(None);

/// Cambia la scorciatoia di Mission Control subito, senza riavviare. `true` se la nuova combinazione è stata
/// registrata; `false` se è occupata (da un altro programma o dal Launcher) o il thread non risponde: resta
/// quella di prima. Il salvataggio lo fa chi chiama.
pub fn change_hotkey(h: Hotkey) -> bool {
    let tid = HOTKEY_THREAD.load(Ordering::Relaxed);
    if tid == 0 {
        return false;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    *PENDING_CHANGE.lock().unwrap_or_else(|e| e.into_inner()) = Some((h, tx));
    if unsafe { windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(tid, WM_CHANGE_HOTKEY, WPARAM(0), LPARAM(0)) }.is_err() {
        return false;
    }
    rx.recv_timeout(Duration::from_secs(2)).unwrap_or(false)
}

fn try_register(h: Hotkey) -> bool {
    unsafe { RegisterHotKey(None, HOTKEY_ID, HOT_KEY_MODIFIERS(h.mods) | MOD_NOREPEAT, h.vk) }.is_ok()
}

/// Thread della scorciatoia: la registra, apre/chiude la panoramica a ogni pressione e, se la
/// combinazione è occupata da un altro programma, avvisa una volta e riprova ogni 30 secondi.
fn hotkey_thread(app: AppHandle) {
    HOTKEY_THREAD.store(unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }, Ordering::Relaxed);
    let mut hk = configured_hotkey();
    let mut registered = try_register(hk);
    if !registered {
        eprintln!("mission control: {} è già usata da un altro programma", display_hotkey(hk));
        let text = HSTRING::from(format!(
            "La scorciatoia {} di Mission Control è già usata da un altro programma. Chiudi quel programma: EosOS riproverà da solo ogni 30 secondi.",
            display_hotkey(hk)
        ));
        std::thread::spawn(move || unsafe {
            MessageBoxW(None, &text, w!("EosOS"), MB_OK | MB_ICONINFORMATION);
        });
    }
    unsafe { SetTimer(None, 1, RETRY_EVERY.as_millis() as u32, None) };
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            match msg.message {
                WM_HOTKEY if msg.wParam == WPARAM(HOTKEY_ID as usize) => {
                    // L'apertura è lenta (screenshot, miniature): fuori dal ciclo dei messaggi.
                    let app = app.clone();
                    std::thread::spawn(move || toggle(&app));
                }
                WM_TIMER if !registered => registered = try_register(hk),
                WM_CHANGE_HOTKEY => {
                    let change = PENDING_CHANGE.lock().unwrap_or_else(|e| e.into_inner()).take();
                    if let Some((new, tx)) = change {
                        if registered {
                            let _ = windows::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey(None, HOTKEY_ID);
                        }
                        if try_register(new) {
                            hk = new;
                            registered = true;
                            let _ = tx.send(true);
                        } else {
                            registered = try_register(hk); // si torna a quella di prima
                            let _ = tx.send(false);
                        }
                    }
                }
                _ => {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}

// ------------------------------------------------------------------ angolo attivo

fn button_down(vk: i32) -> bool {
    unsafe { GetAsyncKeyState(vk) < 0 }
}

/// App a schermo intero o modalità presentazione in corso (l'angolo non deve scattare).
fn fullscreen_busy() -> bool {
    unsafe { SHQueryUserNotificationState() }.is_ok_and(|s| blocks_corner(s.0))
}

/// Thread dell'angolo attivo: ogni 60 ms guarda il cursore; scatta dopo 500 ms nell'angolo, una volta.
fn corner_thread(app: AppHandle) {
    let t0 = Instant::now();
    let mut state = CornerState::default();
    let mut rects: Vec<Rect> = Vec::new();
    let mut rects_at: Option<Instant> = None;
    loop {
        std::thread::sleep(Duration::from_millis(60));
        if !CORNER.load(Ordering::Relaxed) {
            state = CornerState::default();
            continue;
        }
        if rects_at.is_none_or(|t| t.elapsed() > Duration::from_secs(2)) {
            rects = monitors(&app).iter().map(|m| m.rect).collect();
            rects_at = Some(Instant::now());
        }
        let mut p = POINT::default();
        if unsafe { GetCursorPos(&mut p) }.is_err() {
            continue;
        }
        let inside = in_corner((p.x, p.y), &rects, CORNER_PX);
        // Trascinando una finestra (pulsante premuto) o con la panoramica già aperta non scatta; la query
        // dello schermo intero si fa solo quando serve.
        let blocked = inside && (st().phase != Phase::Closed || button_down(0x01) || button_down(0x02) || fullscreen_busy());
        let (next, fire) = corner_step(state, inside, blocked, t0.elapsed().as_millis() as u64, DWELL_MS);
        state = next;
        if fire {
            open(&app);
        }
    }
}

/// Eventi delle finestre panoramica: perdere il focus a favore di una finestra non panoramica chiude (subito:
/// ormai l'utente è altrove); Alt+F4 chiude la panoramica senza chiudere EosOS. `true` se l'evento era loro.
pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) -> bool {
    if !window.label().starts_with(PREFIX) {
        return false;
    }
    match event {
        tauri::WindowEvent::Focused(false) => {
            let app = window.app_handle().clone();
            // La generazione si legge ADESSO: se fra l'evento e il controllo la panoramica viene chiusa e
            // riaperta (l'evento arriva dalla chiusura di prima), il controllo non riguarda la nuova.
            let epoch = st().epoch;
            // Il focus può passare da una finestra panoramica all'altra (due monitor): si controlla dopo un attimo.
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(250));
                // Durante Alt+Tab il primo piano puo' restare all'app di prima (Windows lo nega senza input
                // fresco, e il trucco dell'Alt non si puo' usare con l'Alt dell'utente premuto): la sessione,
                // guidata dall'hook, chiude da sola al rilascio di Alt o con Esc.
                if st().phase == Phase::Closed || st().epoch != epoch || crate::mchook::diag().session {
                    return;
                }
                let fg = unsafe { GetForegroundWindow() };
                let handles: Vec<HWND> =
                    all_labels(&app).iter().filter_map(|l| app.get_webview_window(l)).filter_map(|w| hwnd_of(&w)).collect();
                let on_mc = handles.contains(&fg);
                let visible = handles.iter().any(|&h| unsafe { IsWindowVisible(h) }.as_bool());
                if visible && !on_mc {
                    close(&app);
                }
            });
        }
        tauri::WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            close(window.app_handle());
        }
        _ => {}
    }
    true
}
