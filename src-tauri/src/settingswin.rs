//! Impostazioni di EosOS, fase S2: la finestra. Una sola finestra Tauri creata nascosta all'avvio (come il
//! Launcher e il pannello), senza bordi, trasparente e con gli angoli arrotondati disegnati dalla pagina. Si
//! apre al centro del monitor dove sta il cursore, riceve la tastiera e si chiude con Esc, con il pulsante o con
//! Alt+F4 (che la nasconde soltanto: EosOS non si chiude). Se è già aperta la porta davanti e cambia sezione.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::dockgeom::Rect;

pub const LABEL: &str = "settings";
/// Id della voce «Impostazioni di EosOS…» nei menu nativi (Dock, barra).
pub const MENU_ID: &str = "eos:settings";
/// Testo della voce, uguale ovunque compaia.
pub const MENU_LABEL: &str = "Impostazioni di EosOS…";

/// L'id di una voce di menu nativo è quello delle impostazioni? Pura, testata.
pub fn is_menu_id(id: &str) -> bool {
    id == MENU_ID
}
/// Dimensioni della finestra, in px CSS.
pub const WIDTH: f64 = 780.0;
pub const HEIGHT: f64 = 560.0;
/// Distanza minima dai bordi del monitor, in px CSS (su monitor piccoli la finestra si restringe).
const MARGIN: f64 = 24.0;

static VISIBLE: AtomicBool = AtomicBool::new(false);

/// Posizione e dimensione fisiche della finestra: centrata nel monitor, ridotta se il monitor è piccolo e mai
/// fuori dai suoi bordi. Pura, testata.
pub fn place(monitor: Rect, scale: f64) -> Rect {
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let margin = (MARGIN * scale).round() as i32;
    let w = ((WIDTH * scale).round() as i32).min((monitor.w - 2 * margin).max(1)).max(1);
    let h = ((HEIGHT * scale).round() as i32).min((monitor.h - 2 * margin).max(1)).max(1);
    Rect { x: monitor.x + (monitor.w - w) / 2, y: monitor.y + (monitor.h - h) / 2, w, h }
}

fn win_of(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window(LABEL)
}

fn create(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=settings".into()))
        .title("EosOS Impostazioni")
        .decorations(false)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .inner_size(WIDTH, HEIGHT)
        .build()?;
    Ok(())
}

pub fn start(app: &AppHandle) {
    if let Err(e) = create(app) {
        eprintln!("impostazioni: finestra non creata: {e}");
    }
}

/// Il monitor (px fisici, scala) che contiene il cursore, o il primo.
fn cursor_monitor(app: &AppHandle) -> Option<(Rect, f64)> {
    let mut p = POINT::default();
    let have = unsafe { GetCursorPos(&mut p) }.is_ok();
    let monitors = app.available_monitors().ok()?;
    let to = |m: &tauri::Monitor| (Rect { x: m.position().x, y: m.position().y, w: m.size().width as i32, h: m.size().height as i32 }, m.scale_factor());
    monitors
        .iter()
        .find(|m| {
            let (pos, s) = (m.position(), m.size());
            have && p.x >= pos.x && p.x < pos.x + s.width as i32 && p.y >= pos.y && p.y < pos.y + s.height as i32
        })
        .or_else(|| monitors.first())
        .map(to)
}

/// Apre la finestra (o la porta davanti) e, se indicata, passa alla sezione `section`.
pub fn open(app: &AppHandle, section: Option<&str>) {
    let Some(win) = win_of(app) else { return };
    if !VISIBLE.load(Ordering::Relaxed)
        && let Some((monitor, scale)) = cursor_monitor(app)
    {
        let r = place(monitor, scale);
        let _ = win.set_size(PhysicalSize::new(r.w as u32, r.h as u32));
        let _ = win.set_position(PhysicalPosition::new(r.x, r.y));
    }
    VISIBLE.store(true, Ordering::Relaxed);
    let _ = win.show();
    if let Ok(hwnd) = win.hwnd() {
        crate::dockact::focus(hwnd);
    }
    let _ = win.set_focus();
    let _ = app.emit_to(LABEL, "settings-open", section);
}

/// Come `open`, ma dopo un attimo: il Launcher da cui parte l'azione deve prima finire di chiudersi.
pub fn open_soon(section: Option<String>) {
    if let Some(app) = APP.get().cloned() {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            open(&app, section.as_deref());
        });
    }
}

static APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

pub fn remember(app: &AppHandle) {
    let _ = APP.set(app.clone());
}

pub fn hide(app: &AppHandle) {
    VISIBLE.store(false, Ordering::Relaxed);
    if let Some(win) = win_of(app) {
        let _ = win.hide();
    }
}

/// Eventi della finestra: la chiusura la nasconde soltanto. `true` se l'evento era suo.
pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) -> bool {
    if window.label() != LABEL {
        return false;
    }
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        hide(window.app_handle());
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn only_the_exact_menu_id_opens_the_settings() {
        assert!(is_menu_id("eos:settings"));
        for other in ["", "eos:settings ", "Eos:settings", "ind:battery", "dock:open:x", "eos:setting"] {
            assert!(!is_menu_id(other), "{other:?}");
        }
    }

    #[test]
    fn the_window_is_centered_in_the_monitor() {
        let r = place(mon(0, 0, 1920, 1080), 1.0);
        assert_eq!((r.w, r.h), (780, 560));
        assert_eq!((r.x, r.y), ((1920 - 780) / 2, (1080 - 560) / 2));
        // Un monitor a sinistra (coordinate negative) e a scala 1,5.
        let r = place(mon(-1920, 0, 1920, 1080), 1.5);
        assert_eq!((r.w, r.h), (1170, 840));
        assert_eq!(r.x, -1920 + (1920 - 1170) / 2);
    }

    #[test]
    fn a_small_monitor_shrinks_the_window_but_never_exceeds_it() {
        let m = mon(0, 0, 800, 600);
        let r = place(m, 1.0);
        assert!(r.w <= m.w - 48 && r.h <= m.h - 48, "{r:?}");
        assert!(r.x >= m.x && r.y >= m.y && r.x + r.w <= m.x + m.w && r.y + r.h <= m.y + m.h);
        // Monitor minuscolo o scala assurda: mai misure nulle o negative, mai un panico.
        for (mm, sc) in [(mon(0, 0, 10, 10), 1.0), (mon(0, 0, 1920, 1080), 0.0), (mon(0, 0, 1920, 1080), f64::NAN), (mon(0, 0, 1920, 1080), -3.0)] {
            let r = place(mm, sc);
            assert!(r.w >= 1 && r.h >= 1, "{r:?}");
        }
    }
}
