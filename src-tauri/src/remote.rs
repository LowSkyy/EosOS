//! Controllo remoto rilevato: la sfocatura dello sfondo si spegne da sola, altrimenti Dock, barra e pannelli sarebbero invisibili.
//!
//! Perche': per sfocare lo sfondo le finestre di EosOS sono escluse dalle catture dello schermo (`WDA_EXCLUDEFROMCAPTURE`), e un
//! programma di controllo remoto (AnyDesk, TeamViewer, Desktop remoto...) cattura lo schermo come tutti gli altri: chi guarda da
//! lontano non vede il Dock, la barra e i pannelli. Qui si guarda ogni due secondi se c'e' una sessione remota e, finche' c'e',
//! `dockwin::blur_enabled()` risponde «no» (senza cambiare l'impostazione salvata); finita la sessione la sfocatura torna.
//!
//! Come si riconosce una sessione (nessuna di queste regole e' infallibile; sono dichiarate in `docs/CONTROLLO-REMOTO.md`):
//! - Desktop remoto di Windows: `SM_REMOTESESSION`.
//! - Processi che esistono solo durante una sessione (TeamViewer_Desktop, Chrome Remote Desktop, UltraViewer, VNC).
//! - Programmi sempre in esecuzione come servizio (AnyDesk, RustDesk, Assistenza rapida): conta solo se hanno una finestra visibile
//!   (AnyDesk mostra la sua finestra quando c'e' una connessione). Aprire la finestra di AnyDesk in locale conta come sessione:
//!   si perde la sfocatura finche' e' aperta, nient'altro.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetSystemMetrics, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible, SM_REMOTESESSION};

const EVERY: Duration = Duration::from_secs(2);

/// (nome del file, serve una finestra visibile per dire che c'e' una sessione).
pub const TOOLS: &[(&str, bool)] = &[
    ("anydesk.exe", true),
    ("rustdesk.exe", true),
    ("quickassist.exe", true),
    ("teamviewer_desktop.exe", false),
    ("remoting_desktop.exe", false),
    ("ultraviewer_desktop.exe", false),
    ("winvnc.exe", false),
    ("tvnserver.exe", false),
];

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// C'e' una sessione di controllo remoto in corso (ultimo controllo).
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// La sfocatura e' sospesa per via del controllo remoto (c'e' una sessione e l'opzione e' accesa, di base lo e').
pub fn blur_suspended() -> bool {
    active() && !crate::mcstore::load().remote_keep_blur
}

/// Decide dai processi in esecuzione: `(nome minuscolo, pid)` e i pid che hanno una finestra visibile. Pura, testata.
pub fn decide(procs: &[(String, u32)], with_window: &HashSet<u32>, remote_session: bool) -> bool {
    remote_session
        || procs.iter().any(|(name, pid)| TOOLS.iter().any(|(tool, needs_window)| name == tool && (!needs_window || with_window.contains(pid))))
}

fn processes() -> Vec<(String, u32)> {
    let mut out = Vec::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok {
            let len = e.szExeFile.iter().position(|c| *c == 0).unwrap_or(e.szExeFile.len());
            out.push((String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase(), e.th32ProcessID));
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    out
}

unsafe extern "system" fn collect(h: HWND, data: LPARAM) -> windows::core::BOOL {
    // SAFETY: `data` e' il puntatore all'insieme di `visible_window_pids`, valido per tutta l'enumerazione.
    let set = unsafe { &mut *(data.0 as *mut HashSet<u32>) };
    unsafe {
        if IsWindowVisible(h).as_bool() && GetWindowTextLengthW(h) > 0 {
            let mut pid = 0u32;
            GetWindowThreadProcessId(h, Some(&mut pid));
            set.insert(pid);
        }
    }
    windows::core::BOOL(1)
}

fn visible_window_pids() -> HashSet<u32> {
    let mut set = HashSet::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut set as *mut _ as isize));
    }
    set
}

fn detect() -> bool {
    let rdp = unsafe { GetSystemMetrics(SM_REMOTESESSION) } != 0;
    let procs = processes();
    let wanted: Vec<(String, u32)> = procs.into_iter().filter(|(n, _)| TOOLS.iter().any(|(t, _)| t == n)).collect();
    // Le finestre si elencano solo se serve: la maggior parte del tempo non c'e' nessuno di questi programmi.
    let windows = if wanted.is_empty() { HashSet::new() } else { visible_window_pids() };
    decide(&wanted, &windows, rdp)
}

/// Avvia il controllo periodico. Quando lo stato cambia avvisa le pagine (il Dock rifa' lo sfondo con o senza sfocatura).
pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        let now = detect();
        if ACTIVE.swap(now, Ordering::SeqCst) != now {
            eprintln!("controllo remoto: {}", if now { "rilevato, sfocatura sospesa" } else { "finito, sfocatura ripristinata" });
            let _ = app.emit("dock-settings-changed", crate::dockstore::ui());
        }
        std::thread::sleep(EVERY);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(name: &str, pid: u32) -> (String, u32) {
        (name.to_string(), pid)
    }

    #[test]
    fn nothing_running_means_no_remote_session() {
        assert!(!decide(&[], &HashSet::new(), false));
        assert!(!decide(&[p("chrome.exe", 1)], &HashSet::new(), false));
    }

    #[test]
    fn remote_desktop_session_counts() {
        assert!(decide(&[], &HashSet::new(), true));
    }

    #[test]
    fn service_like_tools_need_a_visible_window() {
        let procs = [p("anydesk.exe", 10), p("anydesk.exe", 11)];
        assert!(!decide(&procs, &HashSet::new(), false), "AnyDesk solo in sottofondo: niente sessione");
        assert!(!decide(&procs, &HashSet::from([99]), false));
        assert!(decide(&procs, &HashSet::from([11]), false), "AnyDesk con la finestra della sessione aperta");
    }

    #[test]
    fn session_only_processes_count_without_a_window() {
        assert!(decide(&[p("teamviewer_desktop.exe", 5)], &HashSet::new(), false));
        assert!(decide(&[p("remoting_desktop.exe", 6)], &HashSet::new(), false));
        assert!(!decide(&[p("teamviewer.exe", 7)], &HashSet::new(), false), "TeamViewer fermo in sottofondo non e' una sessione");
    }
}
