//! Nasconde e ripristina la taskbar di Windows.
//!
//! Le funzioni lavorano su un elenco di nomi di classe finestra, così i test
//! possono usare finestre finte senza toccare la vera taskbar.

use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::Shell::{ABM_GETSTATE, ABM_SETSTATE, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage};
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
    // Un panic non deve attraversare il confine FFI (sarebbe un abort): lo si assorbe.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut buf = [0u16; 256];
        let n = unsafe { GetClassNameW(hwnd, &mut buf) };
        if n > 0 {
            let name = String::from_utf16_lossy(&buf[..n as usize]);
            if ctx.classes.iter().any(|c| *c == name) {
                ctx.found.push(hwnd);
            }
        }
    }));
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

/// Vero se la finestra appartiene a un processo `explorer.exe`. Evita di
/// nascondere finestre di altri processi che usino per caso lo stesso nome di
/// classe. Vale solo per nascondere: il ripristino non filtra mai.
pub(crate) fn owned_by_explorer(hwnd: HWND) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return false;
    }
    let Ok(h) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return false;
    };
    let name = crate::util::image_path(h)
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase()));
    unsafe {
        let _ = CloseHandle(h);
    }
    name.as_deref() == Some("explorer.exe")
}

/// Nasconde le finestre con le classi indicate. Ritorna quante ne ha nascoste.
#[cfg(test)]
pub fn hide(classes: &[&str]) -> usize {
    hide_matching(classes, |_| true)
}

fn hide_matching(classes: &[&str], accept: impl Fn(HWND) -> bool) -> usize {
    let mut n = 0;
    for hwnd in find(classes).into_iter().filter(|h| accept(*h)) {
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

/// File che esiste finché abbiamo attivato noi l'auto-nascondi della taskbar.
/// Serve a ripristinare l'impostazione anche dopo un kill di tutti i processi:
/// al prossimo avvio di EosOS viene riletto e annullato.
fn autohide_marker() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(std::path::PathBuf::from(base).join("EosOS").join("autohide-pending"))
}

fn appbar_state(hwnd: HWND) -> (APPBARDATA, usize) {
    let mut abd = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        ..Default::default()
    };
    let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut abd) };
    (abd, state)
}

fn set_state(hwnd: HWND, state: usize) {
    let mut abd = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        lParam: LPARAM(state as isize),
        ..Default::default()
    };
    unsafe {
        SHAppBarMessage(ABM_SETSTATE, &mut abd);
    }
}

/// Attiva l'auto-nascondi della taskbar: così Windows non riserva più la sua
/// fascia nell'area di lavoro (altrimenti resta uno spazio vuoto in basso).
/// Se era già attivo dall'utente non tocca nulla.
fn enable_autohide() {
    let Some(tray) = find(&["Shell_TrayWnd"]).first().copied() else { return };
    let (_, state) = appbar_state(tray);
    if state & ABS_AUTOHIDE as usize != 0 {
        return;
    }
    if let Some(marker) = autohide_marker() {
        if let Some(dir) = marker.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        // Prima il marcatore, poi la modifica: in caso di kill nel mezzo il ripristino è comunque innocuo.
        if std::fs::write(&marker, b"1").is_err() {
            return; // senza marcatore non si cambia un'impostazione dell'utente
        }
    } else {
        return;
    }
    set_state(tray, state | ABS_AUTOHIDE as usize);
}

/// Annulla l'auto-nascondi se l'abbiamo attivato noi. Idempotente.
pub fn restore_autohide() {
    let Some(marker) = autohide_marker() else { return };
    if !marker.exists() {
        return;
    }
    if let Some(tray) = find(&["Shell_TrayWnd"]).first().copied() {
        let (_, state) = appbar_state(tray);
        set_state(tray, state & !(ABS_AUTOHIDE as usize));
    }
    let _ = std::fs::remove_file(marker);
}

/// Handle della taskbar principale (cambia quando Explorer si riavvia).
pub fn tray_handle() -> Option<isize> {
    find(&["Shell_TrayWnd"]).first().map(|h| h.0 as isize)
}

/// Dato l'ultimo handle noto e quello attuale, dice se Explorer è stato
/// riavviato e quale handle ricordare. Durante il riavvio (nessuna taskbar)
/// l'ultimo handle noto resta quello vecchio.
pub fn detect_restart(last: Option<isize>, now: Option<isize>) -> (Option<isize>, bool) {
    match (last, now) {
        (_, None) => (last, false),
        (None, Some(n)) => (Some(n), false),
        (Some(l), Some(n)) if l == n => (last, false),
        (Some(_), Some(n)) => (Some(n), true),
    }
}

/// Vero finché la taskbar va tenuta nascosta. Il lock serializza il thread di
/// mantenimento e il ripristino: dopo `restore_real` nessuno la rinasconde.
static KEEP_HIDDEN: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

/// Nasconde la vera taskbar e la tiene nascosta: Explorer, dopo il cambio di
/// auto-nascondi, tende a rimostrarla, quindi un thread la ri-nasconde.
pub fn hide_real() -> usize {
    enable_autohide();
    let n = {
        let mut keep = KEEP_HIDDEN.lock().unwrap_or_else(|e| e.into_inner());
        *keep = true;
        hide_matching(&TASKBAR_CLASSES, owned_by_explorer)
    };
    std::thread::spawn(|| {
        let mut last = tray_handle();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let keep = KEEP_HIDDEN.lock().unwrap_or_else(|e| e.into_inner());
            if !*keep {
                break;
            }
            // Explorer riavviato: la nuova taskbar ha un altro handle. Si riapplica
            // l'auto-nascondi (se non è rimasto) e la si nasconde.
            let now = tray_handle();
            if now.is_some() && now != last {
                enable_autohide();
            }
            if now.is_some() {
                last = now;
            }
            hide_matching(&TASKBAR_CLASSES, owned_by_explorer);
        }
    });
    n
}

/// Ripristina la vera taskbar, sempre.
pub fn restore_real() -> usize {
    let mut keep = KEEP_HIDDEN.lock().unwrap_or_else(|e| e.into_inner());
    *keep = false;
    restore_autohide();
    restore(&TASKBAR_CLASSES)
}

/// Ripristina la vera taskbar solo se l'abbiamo nascosta noi (per panic hook
/// e uscita normale: non fa nulla quando il nascondimento non è attivo).
pub fn restore_if_hidden() {
    if HIDDEN.load(Ordering::SeqCst) {
        restore_real();
    } else {
        restore_autohide();
    }
}
