//! Finestra del Dock (fase D2): una finestra Tauri trasparente, sempre in primo
//! piano, centrata in basso sul monitor primario.
//!
//! Le misure le decide il CSS: il frontend misura la pillola e le comunica con
//! il comando `dock_fit`; qui la finestra nativa si adatta e viene ritagliata
//! (SetWindowRgn) alla pillola arrotondata. Il ritaglio serve due volte: dà gli
//! angoli arrotondati al vetro e fa passare i clic nelle zone fuori dalla pillola.
//!
//! Niente sfocatura dello sfondo: tutti gli effetti di Windows (acrilico, blur,
//! sfondo di sistema) riempiono l'intero rettangolo della finestra e ignorano il
//! ritaglio, lasciando angoli quadrati. Il vetro è quindi una tinta traslucida
//! con bordo luminoso (CSS) dentro una capsula perfetta. Una vera sfocatura
//! richiederebbe la Composition API (fase D5).
//!
//! Il Dock non è un appbar: non riserva spazio e non tocca la taskbar.

use std::sync::Mutex;
use std::time::Duration;

use crate::glass;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub const LABEL: &str = "dock";
/// Distanza (px logici) tra il Dock e il bordo inferiore dell'area di lavoro.
const BOTTOM_MARGIN: f64 = 8.0;
const POLL_INTERVAL: Duration = Duration::from_millis(1000);

/// Misure della pillola in pixel CSS (logici), come le vede il frontend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub width: f64,
    pub height: f64,
    pub radius: f64,
}

/// Rettangolo in pixel fisici.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

static FIT: Mutex<Option<Fit>> = Mutex::new(None);
/// Ultimo stato applicato (rettangolo, raggio fisico): evita lavoro inutile a ogni sondaggio.
static APPLIED: Mutex<Option<(Rect, i32)>> = Mutex::new(None);

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Posizione e dimensione fisiche del Dock: centrato in orizzontale nell'area di
/// lavoro, appoggiato in basso con un piccolo margine. Funzione pura, testata.
pub fn dock_rect(work: Rect, scale: f64, fit: &Fit) -> Rect {
    let w = ((fit.width * scale).ceil() as i32).clamp(1, work.w.max(1));
    let h = ((fit.height * scale).ceil() as i32).clamp(1, work.h.max(1));
    let margin = (BOTTOM_MARGIN * scale).round() as i32;
    Rect { x: work.x + (work.w - w) / 2, y: work.y + work.h - h - margin, w, h }
}

/// I valori che arrivano dal frontend si controllano: niente NaN o misure assurde.
pub fn sanitize(width: f64, height: f64, radius: f64) -> Option<Fit> {
    let ok = |v: f64, max: f64| v.is_finite() && v > 0.0 && v <= max;
    (ok(width, 8000.0) && ok(height, 1000.0) && radius.is_finite() && (0.0..=500.0).contains(&radius))
        .then_some(Fit { width, height, radius })
}

fn create(app: &AppHandle) -> tauri::Result<()> {
    let (pos, scale) = match app.primary_monitor() {
        Ok(Some(m)) => {
            let p = m.work_area().position;
            (PhysicalPosition::new(p.x as f64, p.y as f64), m.scale_factor())
        }
        _ => (PhysicalPosition::new(0.0, 0.0), 1.0),
    };
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=dock".into()))
        .title("EosOS Dock")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .position(pos.x, pos.y)
        // Dimensione provvisoria: la finestra è nascosta finché il frontend non misura la pillola.
        .inner_size(900.0 / scale, 200.0 / scale)
        .build()?;
    // Il Dock non deve mai rubare il focus né comparire in Alt+Tab.
    if let Ok(hwnd) = win.hwnd() {
        unsafe {
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                ex | WS_EX_NOACTIVATE.0 as isize | WS_EX_TOOLWINDOW.0 as isize,
            );
        }
    }
    Ok(())
}

/// Ritaglia la finestra a un rettangolo arrotondato (anche zona dei clic).
fn clip_to_pill(win: &tauri::WebviewWindow, w: i32, h: i32, radius_px: i32) {
    let Ok(hwnd) = win.hwnd() else { return };
    unsafe {
        let d = radius_px * 2;
        let rgn = CreateRoundRectRgn(0, 0, w + 1, h + 1, d, d);
        // Dopo la chiamata la regione appartiene al sistema.
        SetWindowRgn(hwnd, Some(rgn), true);
    }
}

/// Porta il Dock nella posizione e dimensione desiderate.
pub fn reconcile(app: &AppHandle) {
    let Some(fit) = *lock(&FIT) else { return };
    let Some(win) = app.get_webview_window(LABEL) else { return };
    let Ok(Some(mon)) = app.primary_monitor() else { return };
    let wa = mon.work_area();
    let work = Rect {
        x: wa.position.x,
        y: wa.position.y,
        w: wa.size.width as i32,
        h: wa.size.height as i32,
    };
    let scale = mon.scale_factor();
    let rect = dock_rect(work, scale, &fit);
    let radius_px = (fit.radius * scale).round() as i32;

    let visible = win.is_visible().unwrap_or(false);
    if visible && *lock(&APPLIED) == Some((rect, radius_px)) {
        return;
    }
    let pos = PhysicalPosition::new(rect.x, rect.y);
    let _ = win.set_size(PhysicalSize::new(rect.w as u32, rect.h as u32));
    let _ = win.set_position(pos);
    clip_to_pill(&win, rect.w, rect.h, radius_px);
    if !visible {
        glass::square_borderless(&win);
        let _ = win.show();
        // La prima visualizzazione può spostare la finestra: si riafferma la posizione.
        let _ = win.set_position(pos);
        // Mostrare la finestra e applicare il vetro può azzerare il ritaglio: lo si riafferma.
        clip_to_pill(&win, rect.w, rect.h, radius_px);
    }
    *lock(&APPLIED) = Some((rect, radius_px));
}

/// Il frontend ha misurato la pillola.
pub fn fit(app: &AppHandle, fit: Fit) {
    *lock(&FIT) = Some(fit);
    reconcile(app);
}

/// Crea la finestra e segue i cambi di monitor, scala e area di lavoro.
pub fn start(app: &AppHandle) {
    if let Err(e) = create(app) {
        eprintln!("EosOS: creazione del Dock fallita: {e}");
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(POLL_INTERVAL);
        reconcile(&app);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIT: Fit = Fit { width: 400.0, height: 74.0, radius: 26.0 };

    #[test]
    fn centered_at_the_bottom_of_the_work_area() {
        let work = Rect { x: 0, y: 28, w: 1920, h: 1020 };
        let r = dock_rect(work, 1.0, &FIT);
        assert_eq!((r.w, r.h), (400, 74));
        assert_eq!(r.x, 760);
        assert_eq!(r.y, 28 + 1020 - 74 - 8);
    }

    #[test]
    fn scale_and_monitor_origin_are_respected() {
        // Monitor primario con origine negativa, scala 150%.
        let work = Rect { x: -2560, y: 0, w: 2560, h: 1400 };
        let r = dock_rect(work, 1.5, &FIT);
        assert_eq!((r.w, r.h), (600, 111));
        assert_eq!(r.x, -2560 + (2560 - 600) / 2);
        assert_eq!(r.y, 1400 - 111 - 12);
    }

    #[test]
    fn never_wider_than_the_work_area() {
        let r = dock_rect(Rect { x: 0, y: 0, w: 300, h: 600 }, 1.0, &FIT);
        assert_eq!((r.x, r.w), (0, 300));
    }

    #[test]
    fn sanitize_rejects_nonsense() {
        assert!(sanitize(400.0, 74.0, 26.0).is_some());
        assert!(sanitize(f64::NAN, 74.0, 26.0).is_none());
        assert!(sanitize(0.0, 74.0, 26.0).is_none());
        assert!(sanitize(400.0, -1.0, 26.0).is_none());
        assert!(sanitize(400.0, 74.0, f64::INFINITY).is_none());
        assert!(sanitize(1e9, 74.0, 26.0).is_none());
    }
}
