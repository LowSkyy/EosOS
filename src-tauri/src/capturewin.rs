//! La finestra della **selezione** per la cattura dello schermo: un velo trasparente sul monitor dove sta il cursore, dove si trascina un
//! rettangolo (la pagina è `Capture.tsx`, vista `?view=capture`). Esc o un clic senza trascinare annullano.
//!
//! La finestra si crea la prima volta e poi resta nascosta (come le altre); prima di scattare si nasconde, così non finisce nell'immagine.
//! La pagina manda a Rust solo un rettangolo in px CSS e la scala: la conversione in pixel fisici e i controlli stanno in `capture.rs`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

use crate::dockgeom::Rect;

pub const LABEL: &str = "capture";

static VISIBLE: AtomicBool = AtomicBool::new(false);
/// Il monitor (px fisici) su cui sta il velo adesso: serve per convertire il rettangolo scelto.
static MONITOR: Mutex<Option<Rect>> = Mutex::new(None);

fn win_of(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window(LABEL)
}

fn ensure(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    if let Some(w) = win_of(app) {
        return Some(w);
    }
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=capture".into()))
        .title("EosOS Cattura")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .build()
        .ok()?;
    crate::chromeguard::protect(&win);
    crate::webmem::hidden(&win);
    Some(win)
}

/// Apre il velo sul monitor del cursore.
pub fn begin(app: &AppHandle) {
    if crate::lockscreen::is_locked() {
        return;
    }
    let Some(win) = ensure(app) else { return };
    let monitor = crate::capture::cursor_monitor();
    *MONITOR.lock().unwrap_or_else(|e| e.into_inner()) = Some(monitor);
    let _ = win.set_size(PhysicalSize::new(monitor.w.max(1) as u32, monitor.h.max(1) as u32));
    let _ = win.set_position(PhysicalPosition::new(monitor.x, monitor.y));
    VISIBLE.store(true, Ordering::Relaxed);
    crate::webmem::shown(&win);
    let _ = win.show();
    if let Ok(hwnd) = win.hwnd() {
        crate::dockact::focus(hwnd);
    }
    let _ = win.set_focus();
    let _ = app.emit_to(LABEL, "capture-open", ());
}

/// Nasconde il velo.
pub fn end(app: &AppHandle) {
    if let Some(win) = win_of(app) {
        let _ = win.hide();
        crate::webmem::hidden(&win);
    }
    VISIBLE.store(false, Ordering::Relaxed);
}

/// Il velo è aperto (per le prove).
#[allow(dead_code)]
pub fn is_open() -> bool {
    VISIBLE.load(Ordering::Relaxed)
}

/// La pagina ha scelto un rettangolo: si nasconde il velo e si scatta. `None` se la selezione è troppo piccola o assurda (si annulla).
pub fn finish(app: &AppHandle, x: f64, y: f64, w: f64, h: f64, dpr: f64) -> Option<crate::capture::Shot> {
    let monitor = *MONITOR.lock().unwrap_or_else(|e| e.into_inner());
    let rect = monitor.and_then(|m| crate::capture::region_to_rect(x, y, w, h, dpr, m));
    end(app);
    let rect = rect?;
    std::thread::sleep(Duration::from_millis(120)); // il velo sparisce dallo schermo
    crate::capture::shoot(rect)
}
