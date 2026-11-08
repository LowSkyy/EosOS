//! Pannello degli indicatori (fase I3): una piccola finestra di vetro che si apre sotto
//! l'indicatore cliccato nella barra (volume con cursore e silenzio, dettagli di rete e batteria).
//!
//! Come il Launcher: una finestra Tauri creata nascosta all'avvio, senza bordi, trasparente, sempre
//! in primo piano, che riceve il mouse e la tastiera; si nasconde con Esc (lo fa la pagina), cliccando
//! fuori (perdita del focus) o ricliccando lo stesso indicatore. Sfondo sfocato con lo stesso canale
//! di cattura del Dock e del Launcher (finestra esclusa dalla cattura).
//!
//! Ricliccare l'indicatore mentre il pannello è aperto: se il clic ha già tolto il focus al pannello
//! (che si è nascosto un attimo prima) non lo si riapre subito: `decide` lo riconosce.

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowRect, SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
};

use crate::dockgeom::{capture_margin, capture_rect, Rect};

pub const LABEL: &str = "panel";
/// Larghezza del pannello, in px CSS.
pub const WIDTH: f64 = 300.0;
pub const MIN_HEIGHT: f64 = 60.0;
pub const MAX_HEIGHT: f64 = 400.0;
/// Altezza provvisoria all'apertura: la pagina comunica poi quella vera (`fit`).
const START_HEIGHT: f64 = 150.0;
/// Distanza dall'indicatore e dai bordi dello schermo, in px CSS.
const GAP: f64 = 6.0;
const EDGE: f64 = 8.0;
const FOCUS_GRACE: Duration = Duration::from_millis(350);
const FOCUS_CHECK: Duration = Duration::from_millis(500);
/// Un clic sull'indicatore subito dopo che il pannello si è chiuso perdendo il focus lo ha
/// chiuso lui: non si riapre.
const REOPEN_BLOCK: Duration = Duration::from_millis(400);

/// Fino a quando (ms dall'avvio) la perdita del focus non chiude il pannello: mentre si legge un menu
/// "pigro" l'app apre il suo menu e si prende il focus per un istante.
static BUSY_UNTIL_MS: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    START.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Instant::now).elapsed().as_millis() as u64
}

/// Per i prossimi `ms` millisecondi non si chiude il pannello se perde il focus.
pub fn busy_for(ms: u64) {
    BUSY_UNTIL_MS.store(now_ms() + ms, Ordering::Relaxed);
}

fn is_busy() -> bool {
    now_ms() < BUSY_UNTIL_MS.load(Ordering::Relaxed)
}

/// Riporta il focus sul pannello (dopo che un'app glielo ha tolto per un istante).
pub fn refocus(app: &AppHandle) {
    if !is_visible() {
        return;
    }
    if let Some(win) = win_of(app) {
        if let Ok(hwnd) = win.hwnd() {
            crate::dockact::focus(hwnd);
        }
        let _ = win.set_focus();
    }
}

static VISIBLE: AtomicBool = AtomicBool::new(false);
static SCALE_BITS: AtomicU64 = AtomicU64::new(0);
static HWND_RAW: AtomicIsize = AtomicIsize::new(0);
static EXCLUDED: AtomicBool = AtomicBool::new(false);
static START: Mutex<Option<Instant>> = Mutex::new(None);

#[derive(Default)]
struct State {
    /// Indicatore mostrato ora.
    key: Option<String>,
    shown_at: Option<Instant>,
    /// Ultima chiusura per perdita di focus: (indicatore, istante).
    lost_focus: Option<(String, Instant)>,
}
static STATE: Mutex<State> = Mutex::new(State { key: None, shown_at: None, lost_focus: None });

fn lock() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

// ------------------------------------------------------------------ pure

/// Cosa fare quando si clicca l'indicatore `key`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    /// Aprire (o cambiare contenuto e posizione se è aperto su un altro indicatore).
    Show,
    /// Chiudere: è aperto proprio su questo indicatore.
    Hide,
    /// Non fare nulla: il clic ha appena chiuso il pannello perdendogli il focus.
    Ignore,
}

/// Decide l'azione di un clic su un indicatore. `open`: indicatore mostrato ora (se visibile);
/// `lost`: ultimo indicatore chiuso per perdita di focus e da quanto tempo. Pura, testata.
pub fn decide(open: Option<&str>, key: &str, lost: Option<(&str, Duration)>) -> Toggle {
    match open {
        Some(k) if k == key => Toggle::Hide,
        Some(_) => Toggle::Show,
        None => match lost {
            Some((k, ago)) if k == key && ago < REOPEN_BLOCK => Toggle::Ignore,
            _ => Toggle::Show,
        },
    }
}

/// Posizione e dimensione fisiche del pannello: centrato sotto l'indicatore (`anchor`, px fisici),
/// a `GAP` di distanza, e mai fuori dal monitor (`monitor`, px fisici) di almeno `EDGE`. Se non c'è
/// spazio sotto l'indicatore sale sopra. Pura, testata.
pub fn place(anchor: Rect, monitor: Rect, scale: f64, height_css: f64) -> Rect {
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let edge = (EDGE * scale).round() as i32;
    let w = ((WIDTH * scale).round() as i32).min((monitor.w - 2 * edge).max(1)).max(1);
    let h = ((clamp_height(height_css) * scale).round() as i32).min((monitor.h - 2 * edge).max(1)).max(1);
    let center = anchor.x + anchor.w / 2;
    let x = (center - w / 2).clamp(monitor.x + edge, (monitor.x + monitor.w - edge - w).max(monitor.x + edge));
    let gap = (GAP * scale).round() as i32;
    let below = anchor.y + anchor.h + gap;
    let y = if below + h + edge <= monitor.y + monitor.h {
        below
    } else {
        (anchor.y - gap - h).max(monitor.y + edge)
    };
    Rect { x, y, w, h }
}

/// Porta l'altezza comunicata dalla pagina dentro i limiti (NaN e infiniti compresi). Pura, testata.
pub fn clamp_height(h: f64) -> f64 {
    if h.is_finite() { h.clamp(MIN_HEIGHT, MAX_HEIGHT) } else { MIN_HEIGHT }
}

/// Gli indicatori che hanno un pannello. Pura, testata.
pub fn is_panel_key(key: &str) -> bool {
    matches!(key, "battery" | "network" | "volume" | "calendar") || menu_index(key).is_some()
}

/// `"menu:3"` → 3: la tendina del menu globale usa questa stessa finestra. Pura, testata.
pub fn menu_index(key: &str) -> Option<usize> {
    key.strip_prefix("menu:")?.parse().ok().filter(|i| *i < 64)
}

/// Il pannello è aperto su un menu della barra e quale (per il passaggio con il mouse da un titolo all'altro).
pub fn open_menu() -> Option<usize> {
    if !is_visible() {
        return None;
    }
    lock().key.as_deref().and_then(menu_index)
}

/// Il rettangolo che la pagina descrive (px CSS, relativo alla finestra della barra) → px fisici
/// dello schermo, data l'origine della finestra e la sua scala. `None` se i numeri non sono sensati.
/// Pura, testata.
pub fn anchor_to_physical(origin: (i32, i32), scale: f64, left: f64, top: f64, w: f64, h: f64) -> Option<Rect> {
    let ok = |v: f64| v.is_finite() && v.abs() < 20_000.0;
    if !(scale.is_finite() && scale > 0.0) || ![left, top, w, h].iter().all(|&v| ok(v)) || w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(Rect {
        x: origin.0 + (left * scale).round() as i32,
        y: origin.1 + (top * scale).round() as i32,
        w: (w * scale).round() as i32,
        h: (h * scale).round() as i32,
    })
}

// ------------------------------------------------------------------ finestra

fn win_of(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window(LABEL)
}

fn since_start() -> Duration {
    START.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Instant::now).elapsed()
}

fn create(app: &AppHandle) -> tauri::Result<()> {
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=panel".into()))
        .title("EosOS Pannello")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .inner_size(WIDTH, START_HEIGHT)
        .build()?;
    if let Ok(hwnd) = win.hwnd() {
        HWND_RAW.store(hwnd.0 as isize, Ordering::Relaxed);
        unsafe { crate::launcherwin::apply_styles(hwnd) };
    }
    Ok(())
}

pub fn start(app: &AppHandle) {
    since_start();
    if let Err(e) = create(app) {
        eprintln!("pannello: finestra non creata: {e}");
    }
}

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

/// Cosa catturare per la sfocatura (come nel Launcher). `None` se nascosto o non escluso.
pub fn capture_info() -> Option<(Rect, Rect)> {
    if !VISIBLE.load(Ordering::Relaxed) || !EXCLUDED.load(Ordering::Relaxed) {
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

/// Monitor (px fisici, scala) che contiene il punto.
fn monitor_at(app: &AppHandle, x: i32, y: i32) -> Option<(Rect, f64)> {
    let monitors = app.available_monitors().ok()?;
    monitors
        .iter()
        .find(|m| {
            let (p, s) = (m.position(), m.size());
            x >= p.x && x < p.x + s.width as i32 && y >= p.y && y < p.y + s.height as i32
        })
        .map(|m| (Rect { x: m.position().x, y: m.position().y, w: m.size().width as i32, h: m.size().height as i32 }, m.scale_factor()))
}

/// Clic su un indicatore: apre, cambia contenuto o chiude il pannello (vedi `decide`). `anchor`: il
/// rettangolo dell'indicatore in px fisici dello schermo.
pub fn toggle(app: &AppHandle, key: &str, anchor: Rect) {
    if !is_panel_key(key) {
        return;
    }
    let (open, lost) = {
        let st = lock();
        let open = if is_visible() { st.key.clone() } else { None };
        let lost = st.lost_focus.as_ref().map(|(k, at)| (k.clone(), at.elapsed()));
        (open, lost)
    };
    match decide(open.as_deref(), key, lost.as_ref().map(|(k, d)| (k.as_str(), *d))) {
        Toggle::Hide => hide(app),
        Toggle::Ignore => {}
        Toggle::Show => show(app, key, anchor),
    }
}

fn show(app: &AppHandle, key: &str, anchor: Rect) {
    let Some(win) = win_of(app) else { return };
    let Some((monitor, scale)) = monitor_at(app, anchor.x + anchor.w / 2, anchor.y + anchor.h / 2) else { return };
    let r = place(anchor, monitor, scale, START_HEIGHT);
    SCALE_BITS.store(scale.to_bits(), Ordering::Relaxed);
    let _ = win.set_position(PhysicalPosition::new(r.x, r.y));
    let _ = win.set_size(PhysicalSize::new(r.w as u32, r.h as u32));
    {
        let mut st = lock();
        st.key = Some(key.to_string());
        st.shown_at = Some(Instant::now());
        st.lost_focus = None;
    }
    VISIBLE.store(true, Ordering::Relaxed);
    update_affinity();
    let _ = win.show();
    if let Ok(hwnd) = win.hwnd() {
        unsafe { crate::launcherwin::apply_styles(hwnd) };
        crate::dockact::focus(hwnd);
    }
    let _ = win.set_focus();
    let _ = app.emit_to(LABEL, "panel-open", key);
    // Se il primo piano non arriva, il pannello non serve a nulla (non si chiuderebbe mai).
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FOCUS_CHECK);
        if !is_visible() || is_busy() {
            return;
        }
        if let Some(win) = win_of(&app)
            && let Ok(hwnd) = win.hwnd()
            && unsafe { GetForegroundWindow() } != hwnd
        {
            hide(&app);
        }
    });
}

pub fn hide(app: &AppHandle) {
    VISIBLE.store(false, Ordering::Relaxed);
    lock().key = None;
    if let Some(win) = win_of(app) {
        let _ = win.hide();
    }
    let _ = app.emit_to(LABEL, "panel-close", ());
}

/// La pagina comunica l'altezza che le serve (px CSS).
pub fn fit(app: &AppHandle, height_css: f64) {
    let Some(win) = win_of(app) else { return };
    let scale = f64::from_bits(SCALE_BITS.load(Ordering::Relaxed));
    let scale = if scale > 0.0 { scale } else { win.scale_factor().unwrap_or(1.0) };
    let _ = win.set_size(PhysicalSize::new((WIDTH * scale).round() as u32, (clamp_height(height_css) * scale).round() as u32));
}

/// Eventi della finestra: perdita di focus → nasconde (e ricorda da chi, per `decide`); Alt+F4 →
/// nasconde senza chiudere EosOS. `true` se l'evento era suo.
pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) -> bool {
    if window.label() != LABEL {
        return false;
    }
    match event {
        tauri::WindowEvent::Focused(false) => {
            let (key, shown) = {
                let st = lock();
                (st.key.clone(), st.shown_at)
            };
            let counts = shown.is_some_and(|t| t.elapsed() >= FOCUS_GRACE) && !is_busy();
            if is_visible() && counts {
                if let Some(k) = key {
                    lock().lost_focus = Some((k, Instant::now()));
                }
                hide(window.app_handle());
            }
        }
        tauri::WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide(window.app_handle());
        }
        _ => {}
    }
    let _ = since_start();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_keys_are_recognized_and_bounded() {
        assert_eq!(menu_index("menu:0"), Some(0));
        assert_eq!(menu_index("menu:11"), Some(11));
        for bad in ["menu:", "menu:x", "menu:-1", "menu:64", "menu:1:2", "battery", "Menu:1", ""] {
            assert_eq!(menu_index(bad), None, "{bad:?}");
        }
        assert!(is_panel_key("menu:3") && is_panel_key("volume") && !is_panel_key("menu:99"));
        assert!(is_panel_key("calendar") && !is_panel_key("Calendar") && !is_panel_key("calendar "));
    }

    const MON: Rect = Rect { x: 0, y: 0, w: 1920, h: 1080 };
    fn anchor(x: i32, w: i32) -> Rect {
        Rect { x, y: 3, w, h: 22 }
    }

    #[test]
    fn click_decisions() {
        assert_eq!(decide(None, "volume", None), Toggle::Show, "chiuso: apre");
        assert_eq!(decide(Some("volume"), "volume", None), Toggle::Hide, "aperto sullo stesso: chiude");
        assert_eq!(decide(Some("volume"), "network", None), Toggle::Show, "aperto su un altro: cambia");
        // Il clic ha appena chiuso il pannello togliendogli il focus: non si riapre.
        assert_eq!(decide(None, "volume", Some(("volume", Duration::from_millis(50)))), Toggle::Ignore);
        // Ma se era un altro indicatore, o e' passato del tempo, si apre.
        assert_eq!(decide(None, "volume", Some(("network", Duration::from_millis(50)))), Toggle::Show);
        assert_eq!(decide(None, "volume", Some(("volume", Duration::from_millis(900)))), Toggle::Show);
        assert_eq!(decide(None, "volume", Some(("volume", REOPEN_BLOCK))), Toggle::Show, "il limite e' escluso");
    }

    #[test]
    fn panel_sits_centered_under_the_indicator() {
        let r = place(anchor(1700, 20), MON, 1.0, 150.0);
        assert_eq!((r.w, r.h), (300, 150));
        assert_eq!(r.x, 1710 - 150);
        assert_eq!(r.y, 3 + 22 + 6);
    }

    #[test]
    fn panel_stays_inside_the_monitor() {
        // Indicatore tutto a destra: il pannello si ferma a EDGE dal bordo.
        let r = place(anchor(1900, 20), MON, 1.0, 150.0);
        assert_eq!(r.x + r.w, 1920 - 8);
        // Tutto a sinistra.
        let l = place(anchor(0, 10), MON, 1.0, 150.0);
        assert_eq!(l.x, 8);
        // Secondo monitor a sinistra dell'origine (coordinate negative).
        let second = Rect { x: -1920, y: 0, w: 1920, h: 1080 };
        let s = place(Rect { x: -30, y: 3, w: 20, h: 22 }, second, 1.0, 150.0);
        assert!(s.x >= second.x && s.x + s.w <= second.x + second.w);
        // Monitor piu' stretto del pannello: si restringe.
        let tiny = Rect { x: 0, y: 0, w: 200, h: 300 };
        let t = place(anchor(100, 10), tiny, 1.0, 150.0);
        assert!(t.w <= 200 - 16 && t.x >= 8);
    }

    #[test]
    fn panel_flips_above_when_there_is_no_room_below() {
        let bottom = Rect { x: 1000, y: 1050, w: 20, h: 22 };
        let r = place(bottom, MON, 1.0, 150.0);
        assert!(r.y + r.h <= 1050, "sopra l'indicatore: {r:?}");
        assert!(r.y >= 8);
    }

    #[test]
    fn panel_scales_with_dpi_and_survives_garbage() {
        let r = place(Rect { x: 2500, y: 4, w: 30, h: 33 }, Rect { x: 0, y: 0, w: 2880, h: 1620 }, 1.5, 150.0);
        assert_eq!((r.w, r.h), (450, 225));
        assert_eq!(r.y, 4 + 33 + 9);
        assert_eq!(place(anchor(100, 10), MON, f64::NAN, 150.0).w, 300);
        assert_eq!(place(anchor(100, 10), MON, -2.0, 150.0).w, 300);
        assert_eq!(place(anchor(100, 10), MON, 1.0, f64::NAN).h, MIN_HEIGHT as i32);
    }

    #[test]
    fn heights_are_clamped() {
        assert_eq!(clamp_height(10.0), MIN_HEIGHT);
        assert_eq!(clamp_height(200.0), 200.0);
        assert_eq!(clamp_height(9999.0), MAX_HEIGHT);
        assert_eq!(clamp_height(f64::INFINITY), MIN_HEIGHT);
    }

    #[test]
    fn only_the_three_indicators_have_a_panel() {
        for k in ["battery", "network", "volume"] {
            assert!(is_panel_key(k));
        }
        for k in ["", "sound", "Volume", "clock", "../x"] {
            assert!(!is_panel_key(k), "{k:?}");
        }
    }

    #[test]
    fn page_rectangle_converts_to_screen_pixels() {
        let r = anchor_to_physical((100, 50), 1.5, 10.0, 2.0, 20.0, 22.0).unwrap();
        assert_eq!((r.x, r.y, r.w, r.h), (115, 53, 30, 33));
        assert_eq!(anchor_to_physical((-1920, 0), 1.0, 5.0, 0.0, 10.0, 10.0).unwrap().x, -1915);
        for bad in [(f64::NAN, 0.0, 10.0, 10.0), (0.0, f64::INFINITY, 10.0, 10.0), (0.0, 0.0, 0.0, 10.0), (0.0, 0.0, 10.0, -1.0), (1e9, 0.0, 10.0, 10.0)] {
            assert!(anchor_to_physical((0, 0), 1.0, bad.0, bad.1, bad.2, bad.3).is_none(), "{bad:?}");
        }
        assert!(anchor_to_physical((0, 0), 0.0, 0.0, 0.0, 10.0, 10.0).is_none());
        assert!(anchor_to_physical((0, 0), f64::NAN, 0.0, 0.0, 10.0, 10.0).is_none());
    }
}
