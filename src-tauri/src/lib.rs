mod appbar;
mod backdrop;
mod dock;
mod dockact;
mod dockgeom;
mod dockstore;
mod dockwin;
mod foreground;
mod glass;
mod indicators;
mod indicatorstore;
mod launcher;
mod launchercalc;
mod launcheract;
mod launcherclip;
mod launcherfiles;
mod launcherrank;
mod launcherstore;
mod launcherwin;
mod mc;
mod mclayout;
mod menubar;
mod autostart;
mod folderpick;
mod settingslive;
mod settingswin;
mod settingsreg;
mod menumodel;
mod menuread;
mod mchook;
mod mckeys;
mod mcstore;
mod mctrigger;
mod mcwin;
mod vdesk;
mod vdint;
mod vdprofile;
mod vdreport;
mod panelwin;
mod taskbar;
mod util;
mod watchdog;

#[cfg(test)]
mod tests;

use tauri::Manager;
use windows::core::BOOL;
use windows::Win32::System::Console::SetConsoleCtrlHandler;

/// La riga di menu corrente della barra (la pagina la chiede all'avvio, poi ascolta `menubar-row`).
#[tauri::command]
fn menubar_current() -> Option<menubar::Row> {
    menubar::current()
}

/// Clic su un titolo di menu della barra: apre (o chiude, se era già aperto) la tendina sotto il titolo.
#[tauri::command]
fn menubar_open(window: tauri::Window, app: tauri::AppHandle, index: usize, left: f64, top: f64, width: f64, height: f64) {
    let (Ok(pos), Ok(scale)) = (window.inner_position(), window.scale_factor()) else { return };
    if let Some(anchor) = panelwin::anchor_to_physical((pos.x, pos.y), scale, left, top, width, height) {
        panelwin::toggle(&app, &format!("menu:{index}"), anchor);
    }
}

/// Il mouse passa su un altro titolo mentre una tendina è aperta: la tendina passa a quel menu.
#[tauri::command]
fn menubar_hover(window: tauri::Window, app: tauri::AppHandle, index: usize, left: f64, top: f64, width: f64, height: f64) {
    if panelwin::open_menu().is_some_and(|open| open != index) {
        menubar_open(window, app, index, left, top, width, height);
    }
}

/// Le voci del menu `index` della riga corrente (se è pigro lo apre un istante nell'app per leggerle).
#[tauri::command(async)]
fn menubar_menu(app: tauri::AppHandle, index: usize) -> Option<menumodel::TopMenu> {
    let lazy = menubar::is_lazy(index);
    if lazy {
        // Leggendo le voci l'app apre il suo menu e si prende il focus: la tendina non deve chiudersi per questo.
        panelwin::busy_for(6000);
    }
    let menu = menubar::menu(index);
    if lazy {
        panelwin::refocus(&app);
        panelwin::busy_for(600);
    }
    menu
}

/// Sceglie una voce del menu (percorso nel modello). Dopo la scelta la tendina si chiude.
#[tauri::command(async)]
fn menubar_choose(app: tauri::AppHandle, index: usize, path: Vec<usize>) -> bool {
    // Prima si chiude la tendina (il primo piano deve passare all'app), poi si esegue la voce.
    panelwin::hide(&app);
    menubar::choose(index, &path)
}

/// Apre la finestra delle impostazioni (alla sezione indicata, se c'è).
#[tauri::command]
fn settings_open(app: tauri::AppHandle, section: Option<String>) {
    settingswin::open(&app, section.as_deref());
}

/// Apre il selettore di cartelle di Windows e restituisce il percorso scelto (o niente se si annulla).
#[tauri::command(async)]
fn settings_pick_folder(app: tauri::AppHandle) -> Option<String> {
    let owner = app.get_webview_window(settingswin::LABEL).and_then(|w| w.hwnd().ok()).map(|h| h.0 as isize);
    folderpick::pick(owner)
}

/// Chiude (nasconde) la finestra delle impostazioni.
#[tauri::command]
fn settings_close(app: tauri::AppHandle) {
    settingswin::hide(&app);
}

/// Lo stato di tutte le impostazioni (per la finestra delle impostazioni).
#[tauri::command(async)]
fn settings_all(app: tauri::AppHandle) -> Vec<settingsreg::Entry> {
    settingsreg::snapshot(&settingslive::Live::new(app))
}

/// Risposta di `settings_set`.
#[derive(serde::Serialize)]
struct SetReply {
    ok: bool,
    /// Ha effetto subito (altrimenti al prossimo avvio).
    live: bool,
    error: Option<String>,
}

/// Cambia un'impostazione. La chiave e il valore li valida Rust: quel che non è nell'elenco chiuso si rifiuta.
#[tauri::command(async)]
fn settings_set(app: tauri::AppHandle, key: String, value: settingsreg::Value) -> SetReply {
    let mut live = settingslive::Live::new(app.clone());
    match settingsreg::set_value(&mut live, &key, &value) {
        Ok(a) => {
            let _ = tauri::Emitter::emit(&app, "settings-changed", &key);
            SetReply { ok: true, live: a.live, error: None }
        }
        Err(e) => SetReply { ok: false, live: false, error: Some(e.0) },
    }
}

/// Preme un pulsante della finestra delle impostazioni (per esempio il rapporto sui desktop virtuali).
#[tauri::command(async)]
fn settings_action(app: tauri::AppHandle, key: String) -> SetReply {
    let mut live = settingslive::Live::new(app);
    match settingsreg::run_action(&mut live, &key) {
        Ok(()) => SetReply { ok: true, live: true, error: None },
        Err(e) => SetReply { ok: false, live: false, error: Some(e.0) },
    }
}

/// Le chiavi delle impostazioni che corrispondono alla ricerca.
#[tauri::command]
fn settings_search(query: String) -> Vec<&'static str> {
    settingsreg::search(&query)
}

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
    tauri::ipc::Response::new(backdrop::latest(backdrop::Channel::Dock, since))
}

/// Ricerca del Launcher tra le app. `have`: gli id di cui il frontend ha già l'icona. Lavoro
/// bloccante (shell, icone): fuori dal thread UI.
#[tauri::command(async)]
fn launcher_search(query: String, have: Option<Vec<String>>, filter: Option<String>) -> Vec<launcher::LauncherItem> {
    launcher::search(&query, &have.unwrap_or_default(), filter.as_deref())
}

/// Ricerca dei file del Launcher (indice di Windows Search). Bloccante e più lenta: la pagina la
/// chiama a parte dopo una breve pausa.
#[tauri::command(async)]
fn launcher_files(query: String, have: Option<Vec<String>>) -> Vec<launcher::LauncherItem> {
    launcher::files(&query, &have.unwrap_or_default())
}

/// Apre un file del Launcher, o lo mostra in Esplora risorse (`reveal`). Solo se Rust lo ha
/// appena mostrato come risultato: un percorso arbitrario è rifiutato.
#[tauri::command(async)]
fn launcher_open_file(id: String, reveal: bool) -> bool {
    launcherfiles::open(&id, reveal)
}

/// Consenso alla cronologia degli appunti (si salva solo il sì/no; il testo copiato resta in
/// memoria). Rifiutare svuota subito la cronologia.
#[tauri::command]
fn launcher_clip_set(on: bool) {
    launcher::set_clipboard(on);
}

/// Rimette negli appunti una voce della cronologia (Rust ha il testo; la pagina manda solo l'id).
#[tauri::command(async)]
fn launcher_clip_use(id: String) -> bool {
    launcher::clip_use(&id)
}

/// Toglie una voce dalla cronologia degli appunti.
#[tauri::command]
fn launcher_clip_delete(id: String) -> bool {
    launcher::clip_delete(&id)
}

/// Stato corrente di batteria, rete e volume (le pagine che si caricano dopo lo chiedono qui; i
/// cambi successivi arrivano con l'evento `indicators-changed`).
#[tauri::command]
fn indicators_get() -> indicators::Indicators {
    indicators::current()
}

/// Clic su un indicatore della barra: apre la pagina di Impostazioni giusta. La pagina manda solo
/// l'id ("battery", "network", "sound"); l'indirizzo lo sceglie Rust.
#[tauri::command(async)]
fn indicator_open(id: String) -> bool {
    indicators::open_settings(&id)
}

/// Clic su un indicatore: apre, cambia o chiude il pannello sotto di esso. La pagina descrive il
/// rettangolo dell'indicatore in px CSS rispetto alla propria finestra; Rust lo converte in pixel
/// dello schermo con posizione e scala della finestra che ha chiamato.
#[tauri::command]
fn panel_toggle(window: tauri::Window, app: tauri::AppHandle, key: String, left: f64, top: f64, width: f64, height: f64) {
    let (Ok(pos), Ok(scale)) = (window.outer_position(), window.scale_factor()) else { return };
    if let Some(anchor) = panelwin::anchor_to_physical((pos.x, pos.y), scale, left, top, width, height) {
        panelwin::toggle(&app, &key, anchor);
    }
}

/// Nasconde il pannello degli indicatori (Esc).
#[tauri::command]
fn panel_hide(app: tauri::AppHandle) {
    panelwin::hide(&app);
}

/// Altezza che serve al pannello, in px CSS.
#[tauri::command]
fn panel_fit(app: tauri::AppHandle, height: f64) {
    panelwin::fit(&app, height);
}

/// Ultimo fotogramma dello sfondo da sfocare dietro il pannello.
#[tauri::command(async)]
fn panel_backdrop(since: u64) -> tauri::ipc::Response {
    tauri::ipc::Response::new(backdrop::latest(backdrop::Channel::Panel, since))
}

/// Imposta il volume del dispositivo di uscita (0-100; NaN rifiutato). Nelle build di prova con la
/// sorgente simulata scrive solo nel file finto, mai sull'audio vero.
#[tauri::command(async)]
fn volume_set(percent: f64) -> bool {
    indicators::set_volume(Some(percent), None)
}

/// Attiva o disattiva il silenzio del dispositivo di uscita.
#[tauri::command(async)]
fn volume_mute(on: bool) -> bool {
    indicators::set_volume(None, Some(on))
}

/// Menu del clic destro sulla barra: quali indicatori mostrare e se la batteria mostra la
/// percentuale. Le scelte arrivano a `on_menu_event` (id con prefisso `ind:`).
#[tauri::command(async)]
fn indicators_menu(app: tauri::AppHandle, window: tauri::Window) -> tauri::Result<()> {
    use indicatorstore::{menu_id, Toggle};
    use tauri::menu::{CheckMenuItem, Menu, PredefinedMenuItem};
    let s = indicatorstore::load();
    let item = |t: Toggle, label: &str, on: bool| CheckMenuItem::with_id(&app, menu_id(t), label, true, on, None::<&str>);
    let battery = item(Toggle::Battery, "Mostra la batteria", s.battery)?;
    let percent = item(Toggle::Percent, "Percentuale della batteria", s.percent)?;
    let network = item(Toggle::Network, "Mostra la rete", s.network)?;
    let volume = item(Toggle::Volume, "Mostra l'audio", s.volume)?;
    let sep = PredefinedMenuItem::separator(&app)?;
    let sep2 = PredefinedMenuItem::separator(&app)?;
    let settings = tauri::menu::MenuItem::with_id(&app, settingswin::MENU_ID, settingswin::MENU_LABEL, true, None::<&str>)?;
    let menu = Menu::with_items(&app, &[&battery, &percent, &sep, &network, &volume, &sep2, &settings])?;
    window.popup_menu(&menu)
}

/// Finestre aperte (desktop corrente) per monitor, già disposte a griglia per la panoramica di
/// Mission Control. Sola lettura: non muove né attiva nulla. Lavoro bloccante (COM, processi).
#[tauri::command(async)]
fn mc_windows(app: tauri::AppHandle) -> Vec<mc::McMonitor> {
    mc::snapshot(&app)
}

/// Apre la panoramica di Mission Control su tutti i monitor.
#[tauri::command(async)]
fn mc_open(app: tauri::AppHandle) {
    mcwin::open(&app);
}

/// Chiude la panoramica (Esc, clic sullo sfondo).
#[tauri::command(async)]
fn mc_close(app: tauri::AppHandle) {
    mcwin::close_animated(&app, None);
}

/// Porta in primo piano la finestra `id` (la ripristina se è minimizzata) e chiude la panoramica. Solo id
/// elencati dalla panoramica e ancora esistenti.
#[tauri::command(async)]
fn mc_activate(app: tauri::AppHandle, id: u64) -> bool {
    mcwin::activate(&app, id)
}

/// Chiede alla finestra `id` di chiudersi (come il pulsante X). Solo id elencati.
#[tauri::command(async)]
fn mc_close_window(app: tauri::AppHandle, id: u64) -> bool {
    mcwin::close_window(&app, id)
}

/// Lo scatto dello sfondo del monitor `index` (vuoto se non c'è), stesso formato di `dock_backdrop`.
#[tauri::command]
fn mc_backdrop(index: usize) -> tauri::ipc::Response {
    tauri::ipc::Response::new(mcwin::backdrop(index))
}

/// Stato della panoramica per la diagnostica e le prove.
#[tauri::command]
fn mc_state() -> mcwin::Diag {
    mcwin::diag()
}

/// I desktop virtuali (dal registro, in sola lettura) e quale è il corrente.
#[tauri::command(async)]
fn mc_desktops() -> Vec<vdesk::Desktop> {
    vdesk::list()
}

/// Passa a un altro desktop virtuale (solo mentre la panoramica è aperta e solo con un indice valido).
#[tauri::command(async)]
fn mc_switch_desktop(app: tauri::AppHandle, index: usize) -> bool {
    mcwin::switch_desktop(&app, index)
}

/// Crea un nuovo desktop virtuale (solo mentre la panoramica è aperta).
#[tauri::command(async)]
fn mc_new_desktop(app: tauri::AppHandle) -> bool {
    mcwin::new_desktop(&app)
}

/// Sposta una finestra su un altro desktop (backend interno, solo se ha superato la verifica).
#[tauri::command(async)]
fn mc_move_window(app: tauri::AppHandle, id: u64, index: usize) -> bool {
    mcwin::move_window_to(&app, id, index)
}

/// Rimuove un desktop virtuale.
#[tauri::command(async)]
fn mc_remove_desktop(app: tauri::AppHandle, index: usize) -> bool {
    mcwin::remove_desktop(&app, index)
}

/// Dà un nome a un desktop virtuale (backend interno, solo se ha superato la verifica).
#[tauri::command(async)]
fn mc_rename_desktop(app: tauri::AppHandle, index: usize, name: String) -> bool {
    mcwin::rename_desktop(&app, index, &name)
}

/// Nasconde o mostra la miniatura viva di una finestra durante un trascinamento.
#[tauri::command]
fn mc_thumb_visible(id: u64, visible: bool) -> bool {
    mcwin::set_thumb_visible(id, visible)
}

/// Il rapporto diagnostico dei desktop virtuali (build, profili provati, esito di ogni controllo). Con
/// `refresh` rifà la selezione. Niente dati dell'utente.
#[tauri::command(async)]
fn mc_vd_report(refresh: Option<bool>) -> vdint::Report {
    vdint::report(refresh.unwrap_or(false))
}

/// Menu globale G1 (misura, sola lettura): misura in sola lettura cosa si legge dei menu di ogni finestra aperta. Niente
/// titoli di finestra né testi di menu: solo nome del programma, conteggi e tempi.
#[derive(serde::Serialize)]
struct MenuProbeRow {
    id: u64,
    app: String,
    readability: menuread::Readability,
    probe: menuread::Probe,
}

#[tauri::command(async)]
fn menu_probe_all(app: tauri::AppHandle) -> Vec<MenuProbeRow> {
    let wins: Vec<(u64, String)> =
        mc::snapshot(&app).into_iter().flat_map(|m| m.windows).map(|w| (w.id, w.app)).collect();
    wins.into_iter()
        .filter_map(|(id, name)| {
            let probe = menuread::probe(id as usize)?;
            Some(MenuProbeRow { id, app: name, readability: menuread::classify(&probe), probe })
        })
        .collect()
}

/// Esegue un'azione del Launcher. Le azioni che spengono o riavviano rispondono "confirm" alla
/// prima richiesta ed eseguono alla seconda (stessa azione, entro 10 secondi): la conferma è di
/// Rust, non della pagina. Risposte: "done", "confirm", "unknown", "failed".
#[tauri::command(async)]
fn launcher_action(id: String) -> &'static str {
    match launcheract::run(&id) {
        launcheract::Outcome::Done => "done",
        launcheract::Outcome::Confirm => "confirm",
        launcheract::Outcome::Unknown => "unknown",
        launcheract::Outcome::Failed => "failed",
    }
}

/// Avvia un'app del Launcher. Il frontend manda solo l'`id` di un risultato che ha ricevuto;
/// ciò che si esegue lo ricava Rust dall'indice.
#[tauri::command(async)]
fn launcher_launch(id: String) -> bool {
    launcher::launch(&id)
}

/// Ultimo fotogramma dello sfondo da sfocare dietro il Launcher (vuoto se la sfocatura è spenta,
/// il Launcher è nascosto o non c'è nulla di nuovo). Stesso formato di `dock_backdrop`.
#[tauri::command(async)]
fn launcher_backdrop(since: u64) -> tauri::ipc::Response {
    tauri::ipc::Response::new(backdrop::latest(backdrop::Channel::Launcher, since))
}

/// Nasconde il Launcher (Esc, o dopo aver avviato qualcosa).
#[tauri::command]
fn launcher_hide(app: tauri::AppHandle) {
    launcheract::cancel_pending();
    launcherwin::hide(&app);
}

/// Altezza che serve al Launcher (campo + risultati), in px CSS.
#[tauri::command]
fn launcher_fit(app: tauri::AppHandle, height: f64) {
    launcherwin::fit(&app, height);
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
    let sep2 = tauri::menu::PredefinedMenuItem::separator(&app)?;
    let settings = tauri::menu::MenuItem::with_id(&app, settingswin::MENU_ID, settingswin::MENU_LABEL, true, None::<&str>)?;
    let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&hide, &reserve, &blur, &sep];
    items.extend(levels.iter().map(|l| l as &dyn tauri::menu::IsMenuItem<tauri::Wry>));
    items.push(&sep2);
    items.push(&settings);
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

    // Modalità hook di tastiera (Win+Tab e Alt+Tab di Mission Control): processo senza finestre.
    if args.get(1).map(String::as_str) == Some("--keyhook") {
        mchook::run_child();
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
            if settingswin::is_menu_id(event.id().as_ref()) {
                settingswin::open_soon(None);
            } else if let Some(t) = indicatorstore::parse_menu_id(event.id().as_ref()) {
                // Fuori dal thread UI: scrive il file e fa rileggere lo stato.
                std::thread::spawn(move || {
                    indicatorstore::toggle(t);
                    indicators::refresh_now();
                });
            } else if let Some((action, id)) = dockact::parse_menu_id(event.id().as_ref()) {
                let app = app.clone();
                // Fuori dal thread UI: le azioni enumerano finestre e avviano processi.
                std::thread::spawn(move || {
                    dockact::run_menu_action(action, &id);
                    // Il frontend del Dock aggiorna subito vetro e impostazioni.
                    let _ = tauri::Emitter::emit(&app, "dock-settings-changed", dockstore::settings());
                });
            }
        })
        .invoke_handler(tauri::generate_handler![get_active_app, settings_open, settings_close, settings_pick_folder, settings_action, settings_all, settings_set, settings_search, menubar_current, menubar_open, menubar_hover, menubar_menu, menubar_choose, glass_state, dock_items, dock_fit, dock_activate, dock_menu, dock_settings_menu, dock_settings, dock_reorder, dock_backdrop, launcher_search, launcher_launch, launcher_files, launcher_open_file, launcher_clip_set, launcher_clip_use, launcher_clip_delete, indicators_get, mc_windows, mc_open, mc_close, mc_activate, mc_close_window, mc_backdrop, mc_state, mc_desktops, mc_switch_desktop, mc_new_desktop, mc_move_window, mc_remove_desktop, mc_rename_desktop, mc_thumb_visible, mc_vd_report, menu_probe_all, indicators_menu, indicator_open, panel_toggle, panel_hide, panel_fit, panel_backdrop, volume_set, volume_mute, launcher_action, launcher_hide, launcher_fit, launcher_backdrop])
        // Chiudere una barra (es. Alt+F4) chiude l'intera app e fa scattare il
        // ripristino. Le barre rimosse da `reconcile` usano destroy(), che non
        // passa di qui.
        .on_window_event(|window, event| {
            if launcherwin::on_window_event(window, event) || panelwin::on_window_event(window, event) || mcwin::on_window_event(window, event) || settingswin::on_window_event(window, event) {
                return;
            }
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window.app_handle().exit(0);
            }
        })
        .setup(|app| {
            appbar::start(app.handle());
            dockwin::start(app.handle());
            backdrop::start();
            launcher::warm_up();
            launcherclip::start();
            indicators::start(app.handle().clone());
            launcherwin::start(app.handle());
            panelwin::start(app.handle());
            mcwin::start(app.handle());
            settingswin::remember(app.handle());
            settingswin::start(app.handle());
            launcheract::set_eos_settings(|| settingswin::open_soon(None));
            launcheract::set_mc_open(mcwin::open_soon);
            launcheract::set_mc_corner(mcwin::toggle_corner);
            launcheract::set_mc_keys(mcwin::toggle_key);
            launcheract::set_menu_actions(|which| match which {
                "toggle" => {
                    menubar::toggle_menu_off();
                    true
                }
                "exclude" => menubar::toggle_current_excluded(),
                _ => false,
            });
            menubar::start(app.handle().clone());
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
