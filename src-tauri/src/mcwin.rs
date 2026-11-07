//! Mission Control (fase M2): le finestre panoramica (una per monitor) e le miniature vive.
//!
//! Ogni monitor ha una finestra Tauri senza bordi, trasparente, sempre in primo piano, creata nascosta
//! all'avvio (come il Launcher). All'apertura:
//! 1. si **scatta uno screenshot di ogni monitor** (prima di mostrare nulla) per lo sfondo sfocato: le
//!    finestre sotto non si muovono mentre la panoramica è aperta, quindi lo sfondo può essere statico;
//! 2. si elencano le finestre e si dispongono a griglia (`mc::snapshot`);
//! 3. si registrano le **miniature vive** con `DwmRegisterThumbnail` (Windows disegna la finestra
//!    sorgente dentro la nostra, in tempo reale; le compone **sopra** la pagina, per questo la pagina non
//!    disegna nulla sopra le miniature) e si manda la vista alla pagina.
//!
//! Sicurezza: la pagina conosce le finestre solo per `id`; attivare o chiudere accetta soltanto id che la
//! panoramica ha appena elencato e che esistono ancora. Chiudere una finestra manda `WM_CLOSE`
//! (mai si termina un processo). Alla chiusura tutto si libera e la pagina dimentica titoli e icone.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmQueryThumbnailSourceSize, DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
    DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_VISIBLE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, IsIconic, IsWindow, IsWindowVisible, PostMessageW, ShowWindow, SW_RESTORE, WM_CLOSE,
};

use crate::dockgeom::Rect;
use crate::mc::{monitors, snapshot, thumb_rect, view_of, McMonitor, View};

/// Titolo comune delle finestre panoramica (una per monitor).
pub const TITLE: &str = "EosOS Mission Control";
const PREFIX: &str = "mc-";
/// Quanto si aspetta dopo "chiudi finestra" prima di rileggere l'elenco.
const REFRESH_AFTER: Duration = Duration::from_millis(400);

static APP: OnceLock<AppHandle> = OnceLock::new();

#[derive(Default)]
struct State {
    open: bool,
    /// Id elencati: gli unici su cui si può agire.
    ids: Vec<u64>,
    /// Miniature registrate (handle DWM).
    thumbs: Vec<isize>,
    /// Scatti dello sfondo, uno per monitor (fotogramma serializzato come in `backdrop::encode`).
    backdrops: Vec<Vec<u8>>,
}

static ST: Mutex<State> = Mutex::new(State { open: false, ids: Vec::new(), thumbs: Vec::new(), backdrops: Vec::new() });

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

/// Crea le finestre dei monitor presenti (nascoste). Le mancanti si creano anche all'apertura.
pub fn start(app: &AppHandle) {
    let _ = APP.set(app.clone());
    for i in 0..monitors(app).len() {
        if let Err(e) = create_one(app, i) {
            eprintln!("mission control: finestra {i} non creata: {e}");
        }
    }
}

fn hwnd_of(win: &tauri::WebviewWindow) -> Option<HWND> {
    win.hwnd().ok()
}

fn unregister_all() {
    let handles: Vec<isize> = std::mem::take(&mut st().thumbs);
    for h in handles {
        unsafe {
            let _ = DwmUnregisterThumbnail(h);
        }
    }
}

/// Registra le miniature vive dei monitor nelle rispettive finestre e restituisce, per monitor, il
/// rettangolo (pixel dello schermo) di quelle riuscite. Una finestra che Windows non lascia
/// riprodurre (per esempio di un processo con privilegi più alti) resta senza miniatura.
fn register_thumbs(app: &AppHandle, snap: &[McMonitor]) -> Vec<HashMap<u64, Rect>> {
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
                let Some(r) = fit else {
                    let _ = DwmUnregisterThumbnail(th);
                    continue;
                };
                // Il rettangolo di destinazione è in coordinate della finestra panoramica (= del monitor).
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
                st().thumbs.push(th);
                map.insert(w.id, r);
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

fn views(app: &AppHandle, snap: &[McMonitor]) -> Vec<View> {
    let thumbs = register_thumbs(app, snap);
    let icons = icons_for(snap);
    snap.iter().enumerate().map(|(i, m)| view_of(i, m, &thumbs[i], &icons)).collect()
}

fn remember_ids(snap: &[McMonitor]) {
    st().ids = snap.iter().flat_map(|m| m.windows.iter().map(|w| w.id)).collect();
}

/// Il monitor dove sta il cursore (indice); 0 se non si sa.
fn cursor_monitor(mons: &[crate::mc::Mon]) -> usize {
    let mut p = windows::Win32::Foundation::POINT::default();
    if unsafe { GetCursorPos(&mut p) }.is_err() {
        return 0;
    }
    mons.iter().position(|m| m.rect.contains((p.x, p.y))).unwrap_or(0)
}

/// Apre la panoramica su tutti i monitor. Se è già aperta non fa nulla.
pub fn open(app: &AppHandle) {
    if st().open {
        return;
    }
    let mons = monitors(app);
    if mons.is_empty() {
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
    {
        let mut s = st();
        s.backdrops = backdrops;
        s.open = true;
    }
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
    // 4. Le miniature e la vista per ogni pagina.
    for v in views(app, &snap) {
        let _ = app.emit_to(label(v.index), "mc-open", &v);
    }
}

/// Chiude la panoramica: libera le miniature, nasconde le finestre e fa dimenticare tutto alle pagine.
pub fn close(app: &AppHandle) {
    {
        let mut s = st();
        if !s.open {
            return;
        }
        s.open = false;
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

fn is_known(id: u64) -> bool {
    st().ids.contains(&id)
}

fn hwnd_from(id: u64) -> HWND {
    HWND(id as usize as *mut _)
}

/// Porta in primo piano la finestra `id` (ripristinandola se è minimizzata) e chiude la panoramica.
/// `false` se l'id non è tra quelli elencati o la finestra non esiste più.
pub fn activate(app: &AppHandle, id: u64) -> bool {
    if !is_known(id) {
        return false;
    }
    let hwnd = hwnd_from(id);
    if !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        return false;
    }
    close(app);
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
    }
    crate::dockact::focus(hwnd);
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

/// Rilegge le finestre e ridisegna la panoramica (dopo averne chiusa una).
pub fn refresh(app: &AppHandle) {
    if !st().open {
        return;
    }
    let snap = snapshot(app);
    remember_ids(&snap);
    unregister_all();
    for v in views(app, &snap) {
        let _ = app.emit_to(label(v.index), "mc-update", &v);
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

/// Stato per la diagnostica e le prove: aperta?, quante miniature, quanti id.
#[derive(Serialize)]
pub struct Diag {
    pub open: bool,
    pub thumbs: usize,
    pub windows: usize,
}

pub fn diag() -> Diag {
    let s = st();
    Diag { open: s.open, thumbs: s.thumbs.len(), windows: s.ids.len() }
}

/// Eventi delle finestre panoramica: perdere il focus a favore di una finestra non panoramica chiude;
/// Alt+F4 chiude la panoramica senza chiudere EosOS. `true` se l'evento era loro.
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
                if !st().open {
                    return;
                }
                let fg = unsafe { GetForegroundWindow() };
                let on_mc = all_labels(&app)
                    .iter()
                    .filter_map(|l| app.get_webview_window(l))
                    .filter_map(|w| hwnd_of(&w))
                    .any(|h| h == fg);
                let visible = all_labels(&app)
                    .iter()
                    .filter_map(|l| app.get_webview_window(l))
                    .filter_map(|w| hwnd_of(&w))
                    .any(|h| unsafe { IsWindowVisible(h) }.as_bool());
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
