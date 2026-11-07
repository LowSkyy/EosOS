//! Test del ripristino taskbar. Usano finestre finte con classi uniche: la vera
//! taskbar non viene mai toccata.

use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, IsWindowVisible,
    PostMessageW, RegisterClassW, TranslateMessage, MSG, WM_CLOSE, WNDCLASSW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
};

use crate::{taskbar, watchdog};

static COUNTER: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}

/// Finestra finta visibile, con il suo message loop in un thread dedicato.
struct FakeWindow {
    class: String,
    hwnd: isize,
    thread: Option<JoinHandle<()>>,
}

impl FakeWindow {
    fn new() -> Self {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let class = format!("EosOSTestTray_{}_{}", std::process::id(), n);
        let (tx, rx) = mpsc::channel();
        let class_thread = class.clone();
        let thread = thread::spawn(move || unsafe {
            let wide: Vec<u16> = class_thread.encode_utf16().chain(Some(0)).collect();
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                lpszClassName: PCWSTR(wide.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                PCWSTR(wide.as_ptr()),
                w!("test"),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                1,
                1,
                None,
                None,
                None,
                None,
            )
            .expect("CreateWindowExW");
            tx.send(hwnd.0 as isize).unwrap();
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let _ = DestroyWindow(hwnd);
        });
        let hwnd = rx.recv_timeout(Duration::from_secs(5)).expect("finestra non creata");
        FakeWindow { class, hwnd, thread: Some(thread) }
    }

    fn visible(&self) -> bool {
        unsafe { IsWindowVisible(HWND(self.hwnd as *mut _)).as_bool() }
    }
}

impl Drop for FakeWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = PostMessageW(Some(HWND(self.hwnd as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0));
            // WM_CLOSE con DefWindowProc distrugge la finestra ma non esce dal loop:
            // WM_QUIT arriva con PostThreadMessage; qui basta lasciare il thread.
        }
        let _ = self.thread.take();
    }
}

#[test]
fn hide_then_restore_roundtrip() {
    let win = FakeWindow::new();
    let classes = [win.class.as_str()];
    assert!(win.visible());

    assert_eq!(taskbar::hide(&classes), 1);
    assert!(!win.visible(), "deve essere nascosta");

    assert_eq!(taskbar::restore(&classes), 1);
    assert!(win.visible(), "deve tornare visibile");
}

#[test]
fn restore_is_idempotent_and_safe_when_nothing_hidden() {
    let win = FakeWindow::new();
    let classes = [win.class.as_str()];
    assert_eq!(taskbar::restore(&classes), 0);
    assert_eq!(taskbar::restore(&classes), 0);
    assert!(win.visible());
}

#[test]
fn hide_does_not_touch_other_classes() {
    let a = FakeWindow::new();
    let b = FakeWindow::new();
    taskbar::hide(&[a.class.as_str()]);
    assert!(!a.visible());
    assert!(b.visible(), "una classe diversa non deve essere toccata");
    taskbar::restore(&[a.class.as_str()]);
}

#[test]
fn watchdog_restores_when_watched_process_is_killed() {
    let win = FakeWindow::new();
    let class = win.class.clone();
    taskbar::hide(&[class.as_str()]);
    assert!(!win.visible());

    // Processo finto da sorvegliare, poi ucciso brutalmente.
    let mut child = Command::new("ping")
        .args(["-n", "60", "127.0.0.1"])
        .stdout(Stdio::null())
        .spawn()
        .expect("ping");
    let pid = child.id();

    let watcher = thread::spawn(move || watchdog::wait_and_restore(pid, &[class.as_str()]));
    thread::sleep(Duration::from_millis(300));
    assert!(!win.visible(), "finché il processo vive non si ripristina");

    child.kill().unwrap();
    let observed = watcher.join().unwrap();
    assert!(observed, "la morte del processo deve essere osservata");
    assert!(win.visible(), "alla morte del processo la finestra torna visibile");
}

#[test]
fn watchdog_restores_immediately_if_parent_already_gone() {
    let win = FakeWindow::new();
    let class = win.class.clone();
    taskbar::hide(&[class.as_str()]);

    let observed = watchdog::wait_and_restore(0x7FFF_FFF0, &[class.as_str()]);
    assert!(!observed);
    assert!(win.visible());
}
