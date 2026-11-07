//! Watchdog: processo separato (lo stesso exe lanciato con
//! `--watchdog <pid> <nome-evento>`) che ripristina la taskbar se EosOS muore in
//! qualunque modo, e gestisce la scorciatoia globale di emergenza
//! Ctrl+Alt+Shift+Esc.
//!
//! Sicurezza:
//! - il watchdog apre UN handle al genitore all'avvio e lo tiene: finché è
//!   aperto il PID non può essere riciclato, quindi il kill colpisce sempre il
//!   processo giusto;
//! - rifiuta di sorvegliare un processo che non sia questo stesso eseguibile;
//! - segnala "pronto" al genitore solo dopo aver registrato la scorciatoia: il
//!   genitore nasconde la taskbar solo dopo quel segnale.

use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{
    CreateEventW, OpenEventW, OpenProcess, SetEvent, TerminateProcess, WaitForSingleObject,
    EVENT_MODIFY_STATE, INFINITE, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

use crate::util::image_path;
use crate::{appbar, taskbar};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
const HOTKEY_ID: i32 = 0x0E05;
const READY_TIMEOUT: Duration = Duration::from_secs(5);
const LOG_MAX_BYTES: u64 = 1_000_000;

/// Percorso del log: %LOCALAPPDATA%\EosOS\logs\watchdog.log (con rotazione).
pub fn log_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("EosOS").join("logs").join("watchdog.log")
}

fn log(msg: &str) {
    let path = log_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > LOG_MAX_BYTES) {
        let _ = std::fs::rename(&path, path.with_extension("log.old"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "[pid {}] {msg}", std::process::id());
    }
}

fn ready_event_name(parent_pid: u32) -> String {
    format!("Local\\EosOS-watchdog-ready-{parent_pid}")
}

/// Avvia il watchdog e aspetta che sia davvero operativo (scorciatoia
/// registrata). Errore se non parte, muore o non diventa pronto entro 5 s.
pub fn spawn_ready() -> std::io::Result<Child> {
    let pid = std::process::id();
    let name = HSTRING::from(ready_event_name(pid));
    let event = unsafe { CreateEventW(None, true, false, &name) }
        .map_err(|e| std::io::Error::other(format!("CreateEventW: {e}")))?;

    let exe = std::env::current_exe()?;
    let mut child = Command::new(exe)
        .arg("--watchdog")
        .arg(pid.to_string())
        .creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP)
        .spawn()?;

    let start = Instant::now();
    let result = loop {
        if unsafe { WaitForSingleObject(event, 0) } == WAIT_OBJECT_0 {
            break Ok(());
        }
        if let Ok(Some(status)) = child.try_wait() {
            break Err(std::io::Error::other(format!("watchdog terminato subito ({status})")));
        }
        if start.elapsed() > READY_TIMEOUT {
            break Err(std::io::Error::other("watchdog non pronto entro 5 s"));
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    unsafe {
        let _ = CloseHandle(event);
    }
    if result.is_err() {
        let _ = child.kill();
    }
    result.map(|()| child)
}

/// Blocca finché il processo `pid` termina; poi ripristina le finestre
/// indicate. Ritorna false se il processo non si poteva osservare (già morto
/// o accesso negato): in quel caso ripristina comunque.
#[cfg(test)]
pub fn wait_and_restore(pid: u32, classes: &[&str]) -> bool {
    let handle: Option<HANDLE> = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid).ok() };
    let observed = match handle {
        Some(h) => {
            let r = unsafe { WaitForSingleObject(h, INFINITE) };
            unsafe {
                let _ = CloseHandle(h);
            }
            r == WAIT_OBJECT_0
        }
        None => false,
    };
    let n = taskbar::restore(classes);
    log(&format!("processo {pid} terminato (osservato={observed}); finestre ripristinate: {n}"));
    observed
}

fn same_exe(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
}

fn restore_everything() {
    let n = taskbar::restore_real();
    appbar::remove_if_registered();
    log(&format!("ripristino eseguito (taskbar nascoste: {n})"));
}

/// Entry point del processo watchdog. Non ritorna.
pub fn run(parent_pid: u32) -> ! {
    log(&format!("watchdog avviato per il processo {parent_pid}"));

    // Un solo handle, tenuto aperto: impedisce il riciclo del PID.
    let access = PROCESS_SYNCHRONIZE | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION;
    let parent = match unsafe { OpenProcess(access, false, parent_pid) } {
        Ok(h) => h,
        Err(e) => {
            log(&format!("genitore non apribile ({e}): ripristino e uscita"));
            restore_everything();
            std::process::exit(2);
        }
    };

    // Rifiuta PID che non sono questo stesso eseguibile.
    let me = std::env::current_exe().ok();
    match (image_path(parent), me) {
        (Some(p), Some(m)) if same_exe(&p, &m) => {}
        other => {
            log(&format!("il processo {parent_pid} non è EosOS ({other:?}): rifiuto"));
            std::process::exit(3);
        }
    }

    // Thread che aspetta la morte del genitore e ripristina tutto.
    let parent_raw = parent.0 as isize;
    std::thread::spawn(move || {
        let h = HANDLE(parent_raw as *mut _);
        unsafe { WaitForSingleObject(h, INFINITE) };
        log(&format!("processo {parent_pid} terminato"));
        restore_everything();
        std::process::exit(0);
    });

    // Scorciatoia globale: se non si registra, il watchdog non è utile come
    // rete di sicurezza completa -> esce con errore e il genitore non nasconde nulla.
    let mods = MOD_CONTROL | MOD_ALT | MOD_SHIFT | MOD_NOREPEAT;
    if let Err(e) = unsafe { RegisterHotKey(None, HOTKEY_ID, mods, VK_ESCAPE.0 as u32) } {
        log(&format!("registrazione scorciatoia fallita: {e}"));
        std::process::exit(4);
    }
    log("scorciatoia Ctrl+Alt+Shift+Esc registrata");

    // Segnala "pronto" al genitore.
    let name = HSTRING::from(ready_event_name(parent_pid));
    match unsafe { OpenEventW(EVENT_MODIFY_STATE, false, &name) } {
        Ok(ev) => unsafe {
            let _ = SetEvent(ev);
            let _ = CloseHandle(ev);
        },
        Err(e) => log(&format!("evento 'pronto' non apribile: {e}")),
    }

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
            log("scorciatoia di emergenza premuta");
            restore_everything();
            // Uccide il genitore solo se è ancora vivo (handle ancora nostro).
            if unsafe { WaitForSingleObject(parent, 0) } == WAIT_TIMEOUT {
                unsafe {
                    let _ = TerminateProcess(parent, 1);
                }
                log("EosOS terminato");
            }
            std::process::exit(0);
        }
    }
    std::process::exit(0);
}
