mod appbar;
mod backdrop;
mod dock;
mod dockact;
mod dockgeom;
mod dockstore;
mod dockwin;
mod foreground;
mod glass;
mod launcher;
mod launcherrank;
mod launcherstore;
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

/// Voci del Dock (fissate + aperte). Lavoro bloccante (COM, icone): fuori dal thread UI.
#[tauri::command(async)]
fn dock_items(have: Option<Vec<String>>) -> Vec<dock::DockItem> {
    // `have`: gli id di cui il frontend ha già l'icona, che non si rimanda.
    dock::items(&have.unwrap_or_default())
}

/// Clic su un'icona del Dock. Il frontend manda solo l'`id` di una voce che il
/// Dock sta mostrando: il percorso da eseguire lo ricava Rust.
#[tauri::command(async)]
fn dock_activate(id: String) {
    if let Some((item, windows)) = dock::find_with_windows(&id) {
        dockact::activate(&item, &windows);
    }
}

/// Clic destro su un'icona: menu nativo con Apri / Esci / Fissa o Rimuovi.
#[tauri::command(async)]
fn dock_menu(app: tauri::AppHandle, window: tauri::Window, id: String) -> tauri::Result<()> {
    use dockact::{menu_id, MenuAction};
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
    let Some(item) = dock::find(&id) else { return Ok(()) };
    let open = MenuItem::with_id(&app, menu_id(MenuAction::Open, &id), "Apri", true, None::<&str>)?;
    let quit = MenuItem::with_id(&app, menu_id(MenuAction::Quit, &id), "Esci", item.running, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(&app)?;
    let pin = if item.pinned {
        MenuItem::with_id(&app, menu_id(MenuAction::Unpin, &id), "Rimuovi dal Dock", true, None::<&str>)?
    } else {
        MenuItem::with_id(&app, menu_id(MenuAction::Pin, &id), "Fissa nel Dock", true, None::<&str>)?
    };
    let menu = Menu::with_items(&app, &[&open, &quit, &sep, &pin])?;
    dockwin::set_menu_open(true);
    let r = window.popup_menu(&menu);
    dockwin::set_menu_open(false);
    r
}

/// Il frontend del Dock comunica le misure della zona e della capsula (px CSS).
#[tauri::command(async)]
fn dock_fit(app: tauri::AppHandle, fit: dockgeom::FitArgs) {
    if let Some(fit) = dockgeom::sanitize(fit) {
        dockwin::fit(&app, fit);
    }
}

/// Ultimo fotogramma dello sfondo da sfocare, se più recente di `since` (vuoto
/// altrimenti). Risposta binaria: intestazione + pixel RGBA, vedi `backdrop::encode`.
#[tauri::command(async)]
fn dock_backdrop(since: u64) -> tauri::ipc::Response {
    tauri::ipc::Response::new(backdrop::latest(since))
}

/// Ricerca del Launcher tra le app. `have`: gli id di cui il frontend ha già l'icona. Lavoro
/// bloccante (shell, icone): fuori dal thread UI.
#[tauri::command(async)]
fn launcher_search(query: String, have: Option<Vec<String>>) -> Vec<launcher::LauncherItem> {
    launcher::search(&query, &have.unwrap_or_default())
}

/// Avvia un'app del Launcher. Il frontend manda solo l'`id` di un risultato che ha ricevuto;
/// ciò che si esegue lo ricava Rust dall'indice.
#[tauri::command(async)]
fn launcher_launch(id: String) -> bool {
    launcher::launch(&id)
}

/// Trascinamento di un'icona: la mette nella posizione `index` tra le fissate
/// (contata senza di essa). Un'app aperta non fissata viene fissata lì. Come
/// sempre, il frontend manda solo l'`id` di una voce che il Dock sta mostrando.
#[tauri::command(async)]
fn dock_reorder(id: String, index: usize) {
    if let Some(item) = dock::find(&id) {
        let path = std::path::PathBuf::from(&item.path);
        let fallback = dock::Pinned { name: item.name.clone(), path: path.clone() };
        dockstore::move_to(&path, Some(fallback), index);
    }
}

/// Clic destro sullo sfondo della capsula: impostazioni del Dock.
#[tauri::command(async)]
fn dock_settings_menu(app: tauri::AppHandle, window: tauri::Window) -> tauri::Result<()> {
    use dockact::{menu_id, MenuAction};
    use tauri::menu::{CheckMenuItem, Menu};
    let hide = CheckMenuItem::with_id(
        &app,
        menu_id(MenuAction::ToggleAutoHide, "-"),
        "Nascondi automaticamente",
        true,
        dockwin::autohide(),
        None::<&str>,
    )?;
    let current = dockstore::nearest_glass_id(dockstore::settings().glass);
    let sep = tauri::menu::PredefinedMenuItem::separator(&app)?;
    let levels = dockstore::GLASS_LEVELS
        .iter()
        .map(|(id, label, _)| {
            CheckMenuItem::with_id(&app, menu_id(MenuAction::SetGlass, id), *label, true, *id == current, None::<&str>)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let blur = CheckMenuItem::with_id(
        &app,
        menu_id(MenuAction::ToggleBlur, "-"),
        "Sfocatura dello sfondo",
        true,
        dockwin::blur_enabled(),
        None::<&str>,
    )?;
    let reserve = CheckMenuItem::with_id(
        &app,
        menu_id(MenuAction::ToggleReserve, "-"),
        "Riserva spazio alle finestre",
        true,
        dockwin::reserve_enabled(),
        None::<&str>,
    )?;
    let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&hide, &reserve, &blur, &sep];
    items.extend(levels.iter().map(|l| l as &dyn tauri::menu::IsMenuItem<tauri::Wry>));
    let menu = Menu::with_items(&app, &items)?;
    dockwin::set_menu_open(true);
    let r = window.popup_menu(&menu);
    dockwin::set_menu_open(false);
    r
}

/// Impostazioni del Dock (vetro e auto-nascondi) per il frontend.
#[tauri::command]
fn dock_settings() -> dockstore::Settings {
    dockstore::settings()
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
        .on_menu_event(|app, event| {
            if let Some((action, id)) = dockact::parse_menu_id(event.id().as_ref()) {
                let app = app.clone();
                // Fuori dal thread UI: le azioni enumerano finestre e avviano processi.
                std::thread::spawn(move || {
                    dockact::run_menu_action(action, &id);
                    // Il frontend del Dock aggiorna subito vetro e impostazioni.
                    let _ = tauri::Emitter::emit(&app, "dock-settings-changed", dockstore::settings());
                });
            }
        })
        .invoke_handler(tauri::generate_handler![get_active_app, glass_state, dock_items, dock_fit, dock_activate, dock_menu, dock_settings_menu, dock_settings, dock_reorder, dock_backdrop, launcher_search, launcher_launch])
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
            dockwin::start(app.handle());
            backdrop::start();
            launcher::warm_up();
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
