//! «Modalità gioco»: un'app a tutto schermo (gioco, video) in primo piano.
//!
//! Con un gioco che occupa la GPU, tutto quello che EosOS fa di grafico gli ruba tempo e a sua volta va a scatti: la cattura dello sfondo
//! per la sfocatura (copia dello schermo dalla GPU decine di volte al secondo) e la sfocatura del Dock stesso. Finche' c'e' un'app a tutto
//! schermo in primo piano le catture del Dock e delle barre si fermano, la sfocatura si spegne (resta il vetro semplice) e le animazioni
//! del Dock diventano istantanee. Finito il gioco tutto torna come prima.

use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::System::Threading::{GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::Foundation::CloseHandle;
use windows::core::PWSTR;
use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, GWL_STYLE, WS_CAPTION};

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// C'e' un'app a tutto schermo in primo piano (ultimo controllo).
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// Una finestra e' «a tutto schermo»: copre il monitor intero e non ha la barra del titolo di sistema. Pura, testata.
pub fn is_fullscreen(win: RECT, monitor: RECT, style: u32, shell: bool) -> bool {
    !shell && win.left <= monitor.left && win.top <= monitor.top && win.right >= monitor.right && win.bottom >= monitor.bottom && style & WS_CAPTION.0 != WS_CAPTION.0
}

/// Giochi che contano anche in finestra (FiveM / GTA V): niente barra del titolo non e' il solo segno di un gioco. Pura, testata.
pub fn is_game_exe(name: &str) -> bool {
    let n = name.to_lowercase();
    n == "gta5.exe" || n == "gta5_enhanced.exe" || n.starts_with("fivem") || n.contains("gtaprocess")
}

fn exe_name(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 520];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(h);
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        ok.then(|| path.rsplit(std::path::MAIN_SEPARATOR).next().unwrap_or("").to_string())
    }
}

fn detect() -> bool {
    unsafe {
        let h: HWND = GetForegroundWindow();
        if h.0.is_null() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(h, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            return false;
        }
        if exe_name(pid).is_some_and(|n| is_game_exe(&n)) {
            return true;
        }
        let mut buf = [0u16; 64];
        let n = GetClassNameW(h, &mut buf);
        let class = String::from_utf16_lossy(&buf[..n.max(0) as usize]);
        let shell = ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"].contains(&class.as_str());
        let mut r = RECT::default();
        if GetWindowRect(h, &mut r).is_err() {
            return false;
        }
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if !GetMonitorInfoW(MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST), &mut mi).as_bool() {
            return false;
        }
        is_fullscreen(r, mi.rcMonitor, GetWindowLongPtrW(h, GWL_STYLE) as u32, shell)
    }
}

/// Rilegge lo stato. Vero se e' cambiato rispetto a prima.
pub fn refresh() -> bool {
    let now = detect();
    ACTIVE.swap(now, Ordering::SeqCst) != now
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(l: i32, t: i32, rr: i32, b: i32) -> RECT {
        RECT { left: l, top: t, right: rr, bottom: b }
    }

    #[test]
    fn a_borderless_window_covering_the_monitor_is_fullscreen() {
        assert!(is_fullscreen(r(0, 0, 1920, 1080), r(0, 0, 1920, 1080), 0x9000_0000, false));
        assert!(is_fullscreen(r(-8, -8, 1928, 1088), r(0, 0, 1920, 1080), 0, false));
    }

    #[test]
    fn windows_with_a_title_bar_or_smaller_are_not() {
        assert!(!is_fullscreen(r(0, 0, 1920, 1080), r(0, 0, 1920, 1080), WS_CAPTION.0, false), "massimizzata con barra del titolo");
        assert!(!is_fullscreen(r(100, 100, 900, 700), r(0, 0, 1920, 1080), 0, false));
    }

    #[test]
    fn known_game_executables() {
        assert!(is_game_exe("FiveM_b3258_GTAProcess.exe"));
        assert!(is_game_exe("FiveM.exe"));
        assert!(is_game_exe("GTA5.exe"));
        assert!(!is_game_exe("chrome.exe"));
    }

    #[test]
    fn the_desktop_is_never_fullscreen() {
        assert!(!is_fullscreen(r(0, 0, 1920, 1080), r(0, 0, 1920, 1080), 0, true));
    }
}
