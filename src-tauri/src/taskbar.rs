//! Nasconde e ripristina la taskbar di Windows.
//!
//! Le funzioni lavorano su un elenco di nomi di classe finestra, così i test
//! possono usare finestre finte senza toccare la vera taskbar.

use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, IsWindowVisible, ShowWindow, SW_HIDE, SW_SHOWNA,
};

/// Classi della taskbar principale e delle taskbar sui monitor secondari.
pub const TASKBAR_CLASSES: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

/// Vero se questo processo ha nascosto la taskbar e non l'ha ancora ripristinata.
static HIDDEN: AtomicBool = AtomicBool::new(false);

struct Ctx<'a> {
    classes: &'a [&'a str],
    found: Vec<HWND>,
}

unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = unsafe { &mut *(lparam.0 as *mut Ctx) };
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    if n > 0 {
        let name = String::from_utf16_lossy(&buf[..n as usize]);
        if ctx.classes.iter().any(|c| *c == name) {
            ctx.found.push(hwnd);
        }
    }
    true.into()
}

/// Trova tutte le finestre top-level con una delle classi indicate.
pub fn find(classes: &[&str]) -> Vec<HWND> {
    let mut ctx = Ctx { classes, found: Vec::new() };
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut ctx as *mut Ctx as isize));
    }
    ctx.found
}

/// Nasconde le finestre con le classi indicate. Ritorna quante ne ha nascoste.
pub fn hide(classes: &[&str]) -> usize {
    let mut n = 0;
    for hwnd in find(classes) {
        unsafe {
            if IsWindowVisible(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_HIDE);
                n += 1;
            }
        }
    }
    if n > 0 {
        HIDDEN.store(true, Ordering::SeqCst);
    }
    n
}

/// Mostra tutte le finestre con le classi indicate. Idempotente e sicura da
/// chiamare in qualsiasi momento (anche se non era stato nascosto nulla).
/// Ritorna quante erano effettivamente nascoste.
pub fn restore(classes: &[&str]) -> usize {
    let mut n = 0;
    for hwnd in find(classes) {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                n += 1;
            }
            let _ = ShowWindow(hwnd, SW_SHOWNA);
        }
    }
    HIDDEN.store(false, Ordering::SeqCst);
    n
}

/// Nasconde la vera taskbar.
pub fn hide_real() -> usize {
    hide(&TASKBAR_CLASSES)
}

/// Ripristina la vera taskbar, sempre.
pub fn restore_real() -> usize {
    restore(&TASKBAR_CLASSES)
}

/// Ripristina la vera taskbar solo se l'abbiamo nascosta noi (per panic hook
/// e uscita normale: non fa nulla quando il nascondimento non è attivo).
pub fn restore_if_hidden() {
    if HIDDEN.load(Ordering::SeqCst) {
        restore_real();
    }
}
