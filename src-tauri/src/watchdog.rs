//! Watchdog: processo separato (lo stesso exe lanciato con `--watchdog <pid>`)
//! che ripristina la taskbar se EosOS muore in qualunque modo, e gestisce la
//! scorciatoia globale di emergenza Ctrl+Alt+Shift+Esc.

use std::io::Write;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use std::time::Duration;

use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, INFINITE, PROCESS_SYNCHRONIZE,
    PROCESS_TERMINATE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

use crate::taskbar;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
const HOTKEY_ID: i32 = 0x0E05;

fn log(msg: &str) {
    let path = std::env::temp_dir().join("eosos-watchdog.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "[pid {}] {msg}", std::process::id());
    }
}

/// Avvia il watchdog come processo figlio staccato.
pub fn spawn() -> std::io::Result<Child> {
    let exe = std::env::current_exe()?;
    Command::new(exe)
        .arg("--watchdog")
        .arg(std::process::id().to_string())
        .creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP)
        .spawn()
}

/// Blocca finché il processo `pid` termina; poi ripristina le finestre
/// indicate. Ritorna false se il processo non si poteva osservare (già morto
/// o accesso negato): in quel caso ripristina comunque.
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

fn kill_process(pid: u32) {
    unsafe {
        if let Ok(h) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(h, 1);
            let _ = CloseHandle(h);
        }
    }
}

/// Entry point del processo watchdog. Non ritorna.
pub fn run(parent_pid: u32) -> ! {
    log(&format!("watchdog avviato per il processo {parent_pid}"));

    // Thread che aspetta la morte del genitore e ripristina la taskbar.
    std::thread::spawn(move || {
        wait_and_restore(parent_pid, &taskbar::TASKBAR_CLASSES);
        std::process::exit(0);
    });

    // Thread principale: scorciatoia globale di emergenza.
    let mods = MOD_CONTROL | MOD_ALT | MOD_SHIFT | MOD_NOREPEAT;
    if let Err(e) = unsafe { RegisterHotKey(None, HOTKEY_ID, mods, VK_ESCAPE.0 as u32) } {
        log(&format!("registrazione scorciatoia fallita: {e}"));
        // Senza scorciatoia il watchdog resta comunque utile.
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }
    log("scorciatoia Ctrl+Alt+Shift+Esc registrata");

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
            log("scorciatoia di emergenza premuta");
            let n = taskbar::restore_real();
            log(&format!("taskbar ripristinata ({n}); chiudo EosOS"));
            kill_process(parent_pid);
            std::process::exit(0);
        }
    }
    std::process::exit(0);
}
