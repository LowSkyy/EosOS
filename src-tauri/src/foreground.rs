//! Nome dell'app in primo piano, tramite SetWinEventHook(EVENT_SYSTEM_FOREGROUND).
//!
//! Un thread dedicato installa l'hook (fuori contesto) e gira il message loop
//! necessario a ricevere gli eventi. Ad ogni cambio emette l'evento Tauri
//! `foreground-changed` verso il frontend.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::core::{w, HSTRING};
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Accessibility::{
    SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetClassNameW, GetForegroundWindow, GetMessageW, GetWindow, GetWindowTextW,
    IsWindowVisible, GW_HWNDNEXT,
    GetWindowThreadProcessId, TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS,
};

use crate::util::image_path;

pub const EVENT_NAME: &str = "foreground-changed";

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActiveApp {
    pub name: String,
    pub pid: u32,
}

static APP: OnceLock<AppHandle> = OnceLock::new();
static LAST: Mutex<Option<ActiveApp>> = Mutex::new(None);
static DESCRIPTIONS: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

/// Classi delle finestre della shell: quando una di queste è in primo piano
/// l'utente sta guardando il desktop.
const DESKTOP_CLASSES: [&str; 4] =
    ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

/// Nome da mostrare a partire da descrizione del file, nome dell'exe e classe
/// della finestra. Funzione pura, testata.
pub fn display_name(
    description: Option<&str>,
    exe_stem: &str,
    class: &str,
    title: &str,
) -> String {
    if DESKTOP_CLASSES.contains(&class) {
        return "Desktop".to_string();
    }
    // Le app UWP girano dentro ApplicationFrameHost: il nome utile è il titolo.
    if exe_stem.eq_ignore_ascii_case("ApplicationFrameHost") && !title.trim().is_empty() {
        return title.trim().to_string();
    }
    if let Some(d) = description.map(str::trim).filter(|d| !d.is_empty()) {
        return d.to_string();
    }
    let mut chars = exe_stem.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => "Sconosciuta".to_string(),
    }
}

fn file_description(path: &Path) -> Option<String> {
    let wide = HSTRING::from(path.to_string_lossy().as_ref());
    unsafe {
        let size = GetFileVersionInfoSizeW(&wide, None);
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        GetFileVersionInfoW(&wide, None, size, buf.as_mut_ptr() as *mut _).ok()?;

        let mut p: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut len = 0u32;
        let (lang, cp) = if VerQueryValueW(
            buf.as_ptr() as *const _,
            w!("\\VarFileInfo\\Translation"),
            &mut p,
            &mut len,
        )
        .as_bool()
            && len >= 4
            && !p.is_null()
        {
            let t = *(p as *const [u16; 2]);
            (t[0], t[1])
        } else {
            (0x0409, 0x04B0)
        };

        let key = HSTRING::from(format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\FileDescription"));
        if !VerQueryValueW(buf.as_ptr() as *const _, &key, &mut p, &mut len).as_bool()
            || p.is_null()
            || len == 0
        {
            return None;
        }
        let s = std::slice::from_raw_parts(p as *const u16, len as usize);
        Some(String::from_utf16_lossy(s).trim_end_matches('\0').to_string())
    }
}

fn cached_description(path: &Path) -> Option<String> {
    let cache = DESCRIPTIONS.get_or_init(Default::default);
    let key = path.to_string_lossy().to_lowercase();
    if let Some(v) = cache.lock().ok()?.get(&key) {
        return v.clone();
    }
    let d = file_description(path);
    cache.lock().ok()?.insert(key, d.clone());
    d
}

fn window_string(hwnd: HWND, f: unsafe fn(HWND, &mut [u16]) -> i32) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { f(hwnd, &mut buf) };
    if n > 0 { String::from_utf16_lossy(&buf[..n as usize]) } else { String::new() }
}

/// App proprietaria della finestra `hwnd`, se determinabile.
fn app_from_hwnd(hwnd: HWND) -> Option<ActiveApp> {
    if hwnd.0.is_null() {
        return None;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }
    let class = window_string(hwnd, GetClassNameW);
    let title = window_string(hwnd, GetWindowTextW);

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok();
    let path = handle.and_then(image_path);
    if let Some(h) = handle {
        unsafe {
            let _ = CloseHandle(h);
        }
    }

    let stem = path
        .as_deref()
        .and_then(|p| p.file_stem())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let description = path.as_deref().and_then(cached_description);
    let name = display_name(description.as_deref(), &stem, &class, &title);
    Some(ActiveApp { name, pid })
}

fn update(hwnd: HWND) {
    let Some(app) = app_from_hwnd(hwnd) else { return };
    // Mai mostrare EosOS stessa come app attiva (l'hook salta già i propri eventi,
    // la lettura iniziale no).
    if app.pid == std::process::id() {
        return;
    }
    {
        let Ok(mut last) = LAST.lock() else { return };
        if last.as_ref() == Some(&app) {
            return;
        }
        *last = Some(app.clone());
    }
    if std::env::var("EOSOS_DEBUG_FOREGROUND").is_ok() {
        eprintln!("foreground: {} (pid {})", app.name, app.pid);
    }
    if let Some(handle) = APP.get() {
        let _ = handle.emit(EVENT_NAME, app);
    }
}

unsafe extern "system" fn on_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    // OBJID_WINDOW = 0: ci interessa la finestra, non i suoi oggetti figli.
    if id_object == 0 {
        update(hwnd);
    }
}

/// Ultima app nota (per il frontend che parte dopo l'evento).
pub fn current() -> Option<ActiveApp> {
    LAST.lock().ok().and_then(|l| l.clone())
}

/// Finestra da usare per lo stato iniziale. All'avvio il primo piano è spesso
/// EosOS stessa: in quel caso si scende lungo l'ordine Z fino alla prima
/// finestra visibile, con titolo e non nostra.
fn initial_window() -> HWND {
    let fg = unsafe { GetForegroundWindow() };
    let own = std::process::id();
    let mut hwnd = fg;
    for _ in 0..64 {
        if hwnd.0.is_null() {
            break;
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        let usable = pid != own
            && unsafe { IsWindowVisible(hwnd) }.as_bool()
            && !window_string(hwnd, GetWindowTextW).is_empty();
        if usable {
            return hwnd;
        }
        hwnd = unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.unwrap_or_default();
    }
    fg
}

/// Avvia il thread con l'hook.
pub fn start(app: AppHandle) {
    let _ = APP.set(app);
    std::thread::spawn(|| unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(on_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if hook.0.is_null() {
            eprintln!("EosOS: SetWinEventHook fallito");
            return;
        }
        // Stato iniziale: l'app già in primo piano all'avvio.
        update(initial_window());

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let _ = UnhookWinEvent(hook);
    });
}

#[cfg(test)]
mod tests {
    use super::display_name;

    #[test]
    fn uses_file_description_when_present() {
        assert_eq!(display_name(Some("Blocco note"), "notepad", "Notepad", "x"), "Blocco note");
    }

    #[test]
    fn falls_back_to_capitalized_exe_name() {
        assert_eq!(display_name(None, "notepad", "Notepad", ""), "Notepad");
        assert_eq!(display_name(Some("  "), "code", "Chrome_WidgetWin_1", ""), "Code");
    }

    #[test]
    fn desktop_classes_show_desktop() {
        for c in ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"] {
            assert_eq!(display_name(Some("Windows Explorer"), "explorer", c, ""), "Desktop");
        }
    }

    #[test]
    fn explorer_file_window_is_not_desktop() {
        assert_eq!(
            display_name(Some("Esplora file"), "explorer", "CabinetWClass", "Documenti"),
            "Esplora file"
        );
    }

    #[test]
    fn uwp_uses_window_title() {
        assert_eq!(
            display_name(Some("Application Frame Host"), "ApplicationFrameHost", "ApplicationFrameWindow", "Calcolatrice"),
            "Calcolatrice"
        );
    }

    #[test]
    fn empty_everything_is_unknown() {
        assert_eq!(display_name(None, "", "X", ""), "Sconosciuta");
    }
}
