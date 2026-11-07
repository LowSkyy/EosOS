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
    DispatchMessageW, GetCursorPos, GetForegroundWindow, GetMessageW, IsIconic, IsWindow, IsWindowVisible,
    MessageBoxW, PostMessageW, SetTimer, ShowWindow, TranslateMessage, MB_ICONINFORMATION, MB_OK, MSG, SW_RESTORE,
    WM_CLOSE, WM_HOTKEY, WM_TIMER,
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
const DEFAULT_HOTKEY: &str = "Ctrl+Alt+Up";
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
                s.anims.push(Anim { handle: th, origin: (m.x, m.y), from, to });
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
    snap.iter().enumerate().map(|(i, m)| view_of(i, m, &thumbs[i], &icons)).collect()
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

/// Apre la panoramica su tutti i monitor. Se non è chiusa non fa nulla.
pub fn open(app: &AppHandle) {
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
    for (i, m) in mons.iter().enumerate() {
        let Some(win) = win_of(app, i) else { continue };
        let _ = win.set_position(PhysicalPosition::new(m.rect.x, m.rect.y));
        let _ = win.set_size(PhysicalSize::new(m.rect.w as u32, m.rect.h as u32));
        let _ = win.show();
        if let Some(h) = hwnd_of(&win) {
            unsafe { crate::launcherwin::apply_styles(h) };
            if i == focus {
                crate::dockact::focus(h);
            }
        }
    }
    if let Some(win) = win_of(app, focus) {
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
        s.backdrops.clear();
    }
    unregister_all();
    for l in all_labels(app) {
        if let Some(w) = app.get_webview_window(&l) {
            let _ = w.hide();
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
    let on = !CORNER.load(Ordering::Relaxed);
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
    }
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

fn try_register(h: Hotkey) -> bool {
    unsafe { RegisterHotKey(None, HOTKEY_ID, HOT_KEY_MODIFIERS(h.mods) | MOD_NOREPEAT, h.vk) }.is_ok()
}

/// Thread della scorciatoia: la registra, apre/chiude la panoramica a ogni pressione e, se la
/// combinazione è occupata da un altro programma, avvisa una volta e riprova ogni 30 secondi.
fn hotkey_thread(app: AppHandle) {
    let hk = configured_hotkey();
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
            // Il focus può passare da una finestra panoramica all'altra (due monitor): si controlla dopo un attimo.
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(250));
                if st().phase == Phase::Closed {
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
