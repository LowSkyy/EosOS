//! Barra superiore registrata come appbar (SHAppBarMessage): le finestre
//! massimizzate si fermano sotto di lei invece di finirci sotto.
//!
//! La registrazione va SEMPRE annullata (ABM_REMOVE) in ogni percorso di
//! uscita, altrimenti lo spazio di lavoro resta ridotto.

use std::sync::atomic::{AtomicIsize, Ordering};

use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Shell::{
    ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA, SHAppBarMessage,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WM_APP, WS_EX_NOACTIVATE,
};

/// Altezza della barra in pixel logici (a 100% di scala).
pub const BAR_HEIGHT_LOGICAL: f64 = 28.0;

/// HWND della barra registrata (0 = nessuna).
static REGISTERED: AtomicIsize = AtomicIsize::new(0);

fn new_data(hwnd: HWND) -> APPBARDATA {
    APPBARDATA {
        cbSize: size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        uCallbackMessage: WM_APP + 0x10,
        uEdge: ABE_TOP,
        ..Default::default()
    }
}

/// Registra `hwnd` come appbar in alto sul rettangolo del monitor `mon` e
/// ritorna il rettangolo (in pixel fisici) che la barra deve occupare.
pub fn register(hwnd: HWND, mon: RECT, height_px: i32) -> RECT {
    let mut abd = new_data(hwnd);
    unsafe {
        SHAppBarMessage(ABM_NEW, &mut abd);
        REGISTERED.store(hwnd.0 as isize, Ordering::SeqCst);

        abd.rc = RECT { left: mon.left, top: mon.top, right: mon.right, bottom: mon.top + height_px };
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        // Il sistema può aver spostato il lato; l'altezza resta quella voluta.
        abd.rc.bottom = abd.rc.top + height_px;
        SHAppBarMessage(ABM_SETPOS, &mut abd);
    }
    abd.rc
}

/// Annulla la registrazione se presente. Idempotente.
pub fn remove_if_registered() {
    let h = REGISTERED.swap(0, Ordering::SeqCst);
    if h != 0 {
        let mut abd = new_data(HWND(h as *mut _));
        unsafe {
            SHAppBarMessage(ABM_REMOVE, &mut abd);
        }
    }
}

/// Configura la finestra principale come barra superiore sul monitor che la
/// contiene: senza focus, posizionata e dimensionata, poi mostrata.
pub fn setup(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    let hwnd = window.hwnd()?;
    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or("nessun monitor disponibile")?;

    // La barra non deve mai diventare la finestra in primo piano.
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | WS_EX_NOACTIVATE.0 as isize);
    }

    let pos = monitor.position();
    let size = monitor.size();
    let height = (BAR_HEIGHT_LOGICAL * monitor.scale_factor()).round() as i32;
    let mon = RECT {
        left: pos.x,
        top: pos.y,
        right: pos.x + size.width as i32,
        bottom: pos.y + size.height as i32,
    };

    let rc = register(hwnd, mon, height);
    if std::env::var("EOSOS_DEBUG_APPBAR").is_ok() {
        eprintln!(
            "appbar: monitor={:?} altezza={} rc=({},{},{},{})",
            (mon.left, mon.top, mon.right, mon.bottom),
            height,
            rc.left,
            rc.top,
            rc.right,
            rc.bottom
        );
    }

    window.set_position(PhysicalPosition::new(rc.left, rc.top))?;
    window.set_size(PhysicalSize::new((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32))?;
    window.show()?;
    window.set_position(PhysicalPosition::new(rc.left, rc.top))?;
    if std::env::var("EOSOS_DEBUG_APPBAR").is_ok() {
        eprintln!(
            "appbar: finestra pos={:?} size={:?}",
            window.outer_position()?,
            window.outer_size()?
        );
    }
    Ok(())
}

/// Ricava la finestra principale dall'app.
pub fn main_window(app: &tauri::App) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}
