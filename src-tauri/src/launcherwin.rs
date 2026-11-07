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

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::core::w;
use windows::Win32::Foundation::{HWND, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_NOREPEAT, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetMessageW, GetWindowLongPtrW, GetWindowRect, MessageBoxW, SetWindowDisplayAffinity,
    SetWindowLongPtrW, SetWindowPos, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, GWL_EXSTYLE,
    MB_ICONINFORMATION, MB_OK, MSG, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WM_HOTKEY, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
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

/// Thread della scorciatoia: registra Alt+Spazio, ascolta WM_HOTKEY e, se la combinazione è
/// occupata, avvisa una volta e riprova.
fn hotkey_thread(app: AppHandle) {
    let mut warned = false;
    loop {
        let ok = unsafe { RegisterHotKey(None, HOTKEY_ID, MOD_ALT | MOD_NOREPEAT, u32::from(VK_SPACE.0)) }.is_ok();
        if ok {
            if warned {
                eprintln!("launcher: Alt+Spazio registrata dopo un conflitto");
            }
            break;
        }
        if !warned {
            warned = true;
            eprintln!("launcher: Alt+Spazio è già usata da un altro programma");
            std::thread::spawn(|| unsafe {
                MessageBoxW(
                    None,
                    w!("La scorciatoia Alt+Spazio del Launcher è già usata da un altro programma (per esempio PowerToys Run). Chiudi quel programma: EosOS riproverà da solo ogni 30 secondi."),
                    w!("EosOS"),
                    MB_OK | MB_ICONINFORMATION,
                );
            });
        }
        std::thread::sleep(RETRY_EVERY);
    }
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY && msg.wParam == WPARAM(HOTKEY_ID as usize) {
                toggle(&app);
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
    fn focus_loss_is_ignored_only_right_after_opening() {
        assert!(!focus_loss_counts(Duration::from_millis(0)));
        assert!(!focus_loss_counts(Duration::from_millis(349)));
        assert!(focus_loss_counts(Duration::from_millis(350)));
        assert!(focus_loss_counts(Duration::from_secs(5)));
    }
}
