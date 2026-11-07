mod taskbar;
mod watchdog;

#[cfg(test)]
mod tests;

use windows::core::BOOL;
use windows::Win32::System::Console::SetConsoleCtrlHandler;

/// Il nascondimento della taskbar è disattivato di default: serve
/// esplicitamente `EOSOS_HIDE_TASKBAR=1`.
fn hide_requested() -> bool {
    std::env::var("EOSOS_HIDE_TASKBAR").is_ok_and(|v| v == "1")
}

unsafe extern "system" fn on_ctrl(_event: u32) -> BOOL {
    taskbar::restore_if_hidden();
    false.into() // lascia proseguire la gestione di default
}

/// Installa i ripristini "in-process": panic hook e handler Ctrl+C/chiusura.
fn install_local_safety_nets() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        taskbar::restore_if_hidden();
        default_hook(info);
    }));
    unsafe {
        let _ = SetConsoleCtrlHandler(Some(on_ctrl), true);
    }
}

/// Nasconde la taskbar solo se il watchdog è partito: senza rete di sicurezza
/// non si nasconde nulla.
fn enable_taskbar_hiding() {
    match watchdog::spawn() {
        Ok(mut child) => {
            let hidden = taskbar::hide_real();
            eprintln!("EosOS: taskbar nascosta ({hidden} finestre), watchdog attivo");
            // Se il watchdog muore la rete di sicurezza non c'è più: ripristina ed esci.
            std::thread::spawn(move || {
                let _ = child.wait();
                taskbar::restore_real();
                std::process::exit(1);
            });
        }
        Err(e) => eprintln!("EosOS: watchdog non avviabile ({e}); la taskbar NON viene nascosta"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Modalità watchdog: niente Tauri, solo sorveglianza.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--watchdog") {
        let pid = args.get(2).and_then(|s| s.parse().ok()).expect("pid mancante");
        watchdog::run(pid);
    }

    install_local_safety_nets();
    if hide_requested() {
        enable_taskbar_hiding();
    }

    tauri::Builder::default()
        .build(tauri::generate_context!())
        .expect("errore durante l'avvio di EosOS")
        .run(|_app, event| {
            if let tauri::RunEvent::Exit = event {
                taskbar::restore_if_hidden();
            }
        });
}
