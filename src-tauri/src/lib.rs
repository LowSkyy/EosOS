mod appbar;
mod foreground;
mod glass;
mod taskbar;
mod util;
mod watchdog;

#[cfg(test)]
mod tests;

use tauri::Manager;
use windows::core::BOOL;
use windows::Win32::System::Console::SetConsoleCtrlHandler;

#[tauri::command]
fn get_active_app() -> Option<foreground::ActiveApp> {
    foreground::current()
}

#[tauri::command]
fn glass_state() -> &'static str {
    glass::state().as_str()
}

/// Il nascondimento della taskbar è disattivato di default: serve
/// esplicitamente `EOSOS_HIDE_TASKBAR=1`.
fn hide_requested() -> bool {
    std::env::var("EOSOS_HIDE_TASKBAR").is_ok_and(|v| v == "1")
}

/// Annulla tutto ciò che EosOS ha cambiato nel sistema. Idempotente, chiamabile
/// da qualunque percorso di uscita (uscita normale, panic, Ctrl+C...).
fn cleanup() {
    appbar::remove_if_registered();
    taskbar::restore_if_hidden();
}

unsafe extern "system" fn on_ctrl(_event: u32) -> BOOL {
    cleanup();
    false.into() // lascia proseguire la gestione di default
}

/// Installa i ripristini "in-process": panic hook e handler Ctrl+C/chiusura.
fn install_local_safety_nets() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        cleanup();
        default_hook(info);
    }));
    unsafe {
        let _ = SetConsoleCtrlHandler(Some(on_ctrl), true);
    }
}

/// Nasconde la taskbar solo se il watchdog è PRONTO (scorciatoia registrata):
/// senza rete di sicurezza non si nasconde nulla.
fn enable_taskbar_hiding() {
    match watchdog::spawn_ready() {
        Ok(mut child) => {
            let hidden = taskbar::hide_real();
            eprintln!("EosOS: taskbar nascosta ({hidden} finestre), watchdog pronto");
            // Se il watchdog muore la rete di sicurezza non c'è più: ripristina ed esci.
            std::thread::spawn(move || {
                let _ = child.wait();
                cleanup();
                taskbar::restore_real();
                std::process::exit(1);
            });
        }
        Err(e) => eprintln!("EosOS: watchdog non pronto ({e}); la taskbar NON viene nascosta"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Modalità watchdog: niente Tauri, solo sorveglianza.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--watchdog") {
        let pid = args.get(2).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
            eprintln!("uso: --watchdog <pid>");
            std::process::exit(64)
        });
        watchdog::run(pid);
    }

    // Recupero: se un'esecuzione precedente è stata uccisa lasciando l'auto-nascondi attivo.
    taskbar::restore_autohide();
    install_local_safety_nets();
    if hide_requested() {
        enable_taskbar_hiding();
    }

    // Solo build debug o con la feature `test-hooks`: provoca un panic dopo 9 s per testare il panic hook.
    #[cfg(any(debug_assertions, feature = "test-hooks"))]
    if std::env::var("EOSOS_TEST_PANIC").is_ok_and(|v| v == "1") {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(9));
            panic!("panic di test (EOSOS_TEST_PANIC)");
        });
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_active_app, glass_state])
        // Chiudere una barra (es. Alt+F4) chiude l'intera app e fa scattare il
        // ripristino. Le barre rimosse da `reconcile` usano destroy(), che non
        // passa di qui.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window.app_handle().exit(0);
            }
        })
        .setup(|app| {
            appbar::start(app.handle());
            foreground::start(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("errore durante l'avvio di EosOS")
        .run(|_app, event| {
            if let tauri::RunEvent::Exit = event {
                cleanup();
            }
        });
}
