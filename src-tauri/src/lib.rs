mod appbar;
mod capture;
mod capturewin;
mod chromeguard;
mod control;
mod dockbadge;
mod dockkeys;
mod winsnap;
mod winsettings;
mod reminders;
mod weather;
mod updates;
mod webmem;
mod stacks;
mod cmdpolicy;
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
mod launchpadfilter;
mod trayapps;
mod winsys;
mod mc;
mod mclayout;
mod menubar;
mod notifmodel;
mod notifprobe;
mod notifread;
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
mod lights;
mod gamemode;
mod iconpack;
mod remote;
mod lockscreen;
mod taskbar;
mod util;
mod watchdog;

#[cfg(test)]
mod tests;

use tauri::Manager;

/// Esce subito (con `default`) se la finestra che chiama non può usare il comando (vedi `cmdpolicy`).
macro_rules! guard {
    ($w:expr, $cmd:expr) => {
        if cmdpolicy::guard($w.label(), $cmd, None).is_err() {
            return;
        }
    };
    ($w:expr, $cmd:expr, $default:expr) => {
        if cmdpolicy::guard($w.label(), $cmd, None).is_err() {
            return $default;
        }
    };
}
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

/// Una sola lettura alla volta dei menu «pigri» dell'app in primo piano: leggerli significa aprirli davvero nell'app, e con clic rapidi piu' letture
/// insieme si sovrapponevano (menu aperti e richiusi a caso nell'altra app, pannello che si chiudeva o non si apriva).
static LAZY_READ: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Le voci del menu `index` della riga corrente (se è pigro lo apre un istante nell'app per leggerle).
#[tauri::command(async)]
fn menubar_menu(app: tauri::AppHandle, index: usize) -> Option<menumodel::TopMenu> {
    if !menubar::is_lazy(index) {
        return menubar::menu(index);
    }
    let _one_at_a_time = LAZY_READ.lock().unwrap_or_else(|e| e.into_inner());
    // Mentre si aspettava, un'altra lettura puo' aver gia' riempito questo menu.
    if !menubar::is_lazy(index) {
        return menubar::menu(index);
    }
    // Se il pannello e' gia' passato a un altro menu (o si e' chiuso) non si apre niente nell'app per un menu che nessuno guarda piu'.
    if panelwin::open_menu() != Some(index) {
        return None;
    }
    // Leggendo le voci l'app apre il suo menu e si prende il focus: la tendina non deve chiudersi per questo.
    panelwin::busy_begin();
    let menu = menubar::menu(index);
    panelwin::refocus(&app);
    panelwin::busy_end();
    menu
}

/// Sceglie una voce del menu (percorso nel modello). Dopo la scelta la tendina si chiude.
#[tauri::command(async)]
fn menubar_choose(window: tauri::Window, app: tauri::AppHandle, index: usize, path: Vec<usize>) -> bool {
    guard!(window, "menubar_choose", false);
    // Prima si chiude la tendina (il primo piano deve passare all'app), poi si esegue la voce.
    panelwin::hide(&app);
    menubar::choose(index, &path)
}

/// Apre la finestra delle impostazioni (alla sezione indicata, se c'è).
#[tauri::command]
fn settings_open(window: tauri::Window, app: tauri::AppHandle, section: Option<String>) {
    guard!(window, "settings_open");
    settingswin::open(&app, section.as_deref());
}

/// Apre il selettore di cartelle di Windows e restituisce il percorso scelto (o niente se si annulla).
#[tauri::command(async)]
fn settings_pick_folder(window: tauri::Window, app: tauri::AppHandle) -> Option<String> {
    guard!(window, "settings_pick_folder", None);
    let owner = app.get_webview_window(settingswin::LABEL).and_then(|w| w.hwnd().ok()).map(|h| h.0 as isize);
    folderpick::pick(owner)
}

/// Chiude (nasconde) la finestra delle impostazioni.
#[tauri::command]
fn settings_close(window: tauri::Window, app: tauri::AppHandle) {
    guard!(window, "settings_close");
    settingswin::hide(&app);
}

/// I promemoria del widget «Oggi», già in ordine. Solo il pannello.
#[tauri::command]
fn oggi_reminders(window: tauri::Window) -> Vec<reminders::Reminder> {
    guard!(window, "oggi_reminders", Vec::new());
    reminders::sorted(&mcstore::load().reminders)
}

/// Aggiunge un promemoria (testo e ora facoltativa «HH:MM»). Risponde con la lista aggiornata o con l'errore.
#[tauri::command]
fn oggi_add(window: tauri::Window, text: String, time: Option<String>) -> Result<Vec<reminders::Reminder>, String> {
    cmdpolicy::guard(window.label(), "oggi_add", None)?;
    let mut err = None;
    let list = mcstore::update_reminders(|l| {
        if let Err(e) = reminders::add(l, &text, time.as_deref()) {
            err = Some(e);
        }
    });
    err.map_or(Ok(reminders::sorted(&list)), Err)
}

/// Segna un promemoria come fatto (o da fare).
#[tauri::command]
fn oggi_toggle(window: tauri::Window, id: u32) -> Vec<reminders::Reminder> {
    guard!(window, "oggi_toggle", Vec::new());
    reminders::sorted(&mcstore::update_reminders(|l| {
        reminders::toggle(l, id);
    }))
}

/// Toglie un promemoria.
#[tauri::command]
fn oggi_remove(window: tauri::Window, id: u32) -> Vec<reminders::Reminder> {
    guard!(window, "oggi_remove", Vec::new());
    reminders::sorted(&mcstore::update_reminders(|l| {
        reminders::remove(l, id);
    }))
}

/// Toglie tutti i promemoria fatti.
#[tauri::command]
fn oggi_clear_done(window: tauri::Window) -> Vec<reminders::Reminder> {
    guard!(window, "oggi_clear_done", Vec::new());
    reminders::sorted(&mcstore::update_reminders(|l| {
        reminders::clear_done(l);
    }))
}

/// Il meteo di oggi (solo se acceso nelle Impostazioni e con una città scelta; altrimenti nessuna richiesta di rete).
#[tauri::command(async)]
fn oggi_weather(window: tauri::Window) -> weather::Reply {
    if cmdpolicy::guard(window.label(), "oggi_weather", None).is_err() {
        return weather::Reply::plain("off", None);
    }
    let s = mcstore::load();
    weather::current(s.weather_on, s.weather_city.as_deref())
}

/// Le voci del menu EosOS (il «menu mela» a sinistra della barra): id fissi, scelti dalla pagina del pannello. Spegni e Riavvia
/// chiedono comunque il «Sì» in una finestra di Windows. Risponde `true` se la voce è stata eseguita.
#[tauri::command(async)]
fn eosmenu_run(window: tauri::Window, app: tauri::AppHandle, id: String) -> bool {
    guard!(window, "eosmenu_run", false);
    let act = |a: &str| matches!(launcheract::run_now(a), launcheract::Outcome::Done);
    match id.as_str() {
        "settings" => {
            settingswin::open(&app, None);
            true
        }
        "winsettings" => winsettings::open("home"),
        "launchpad" => {
            launcherwin::show_grid(&app);
            true
        }
        "mission" => act("action:mission-control"),
        "lock" => {
            lockscreen::request_lock(&app);
            true
        }
        "sleep" => act("action:sleep"),
        "restart" => act("action:restart"),
        "shutdown" => act("action:shutdown"),
        "quit" => {
            app.exit(0);
            true
        }
        _ => false,
    }
}

/// La schermata di blocco: nome, avatar, modo di accesso e attesa residua (solo per le sue finestre).
#[tauri::command]
fn lock_info(window: tauri::Window) -> Option<lockscreen::Info> {
    guard!(window, "lock_info", None);
    Some(lockscreen::info())
}

/// I byte dello sfondo del desktop per la schermata di blocco (vuoti se non si trova).
#[tauri::command(async)]
fn lock_wallpaper(window: tauri::Window) -> tauri::ipc::Response {
    guard!(window, "lock_wallpaper", tauri::ipc::Response::new(Vec::<u8>::new()));
    tauri::ipc::Response::new(lockscreen::wallpaper())
}

/// Prova una password: la verifica Windows (`LogonUser`); dopo troppi errori impone un'attesa.
#[tauri::command(async)]
fn lock_try_password(window: tauri::Window, app: tauri::AppHandle, password: String) -> lockscreen::Attempt {
    guard!(window, "lock_try_password", lockscreen::Attempt { ok: false, wait_secs: 0 });
    lockscreen::try_password(&app, &password)
}

/// Account senza password: un clic sblocca.
#[tauri::command]
fn lock_try_blank(window: tauri::Window, app: tauri::AppHandle) -> bool {
    guard!(window, "lock_try_blank", false);
    lockscreen::try_blank(&app)
}

/// Windows Hello / PIN: la verifica la fa Windows con la sua finestra.
#[tauri::command(async)]
fn lock_try_hello(window: tauri::Window, app: tauri::AppHandle) -> bool {
    guard!(window, "lock_try_hello", false);
    lockscreen::try_hello(&app)
}

/// Le informazioni per «Informazioni su EosOS» nel menu.
#[derive(serde::Serialize)]
struct About {
    version: String,
    windows: String,
}

#[tauri::command]
fn eosmenu_about(window: tauri::Window, app: tauri::AppHandle) -> Option<About> {
    if cmdpolicy::guard(window.label(), "eosmenu_about", None).is_err() {
        return None;
    }
    use std::os::windows::process::CommandExt;
    let windows = std::process::Command::new("cmd")
        .args(["/c", "ver"])
        .creation_flags(0x0800_0000)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    Some(About { version: app.package_info().version.to_string(), windows })
}

/// L'elenco delle pagine delle Impostazioni di Windows (senza gli indirizzi). Solo la finestra Impostazioni.
#[tauri::command]
fn winsettings_list(window: tauri::Window) -> Vec<winsettings::Entry> {
    guard!(window, "winsettings_list", Vec::new());
    winsettings::entries()
}

/// Apre una pagina delle Impostazioni di Windows (solo gli id della tabella fissa).
#[tauri::command(async)]
fn winsettings_open(window: tauri::Window, id: String) -> bool {
    guard!(window, "winsettings_open", false);
    winsettings::open(&id)
}

/// L'ultimo indirizzo aperto (solo build di prova con `EOSOS_TEST_NO_OPEN`).
#[tauri::command]
fn winsettings_last_open() -> Option<String> {
    #[cfg(feature = "test-hooks")]
    return winsettings::last_open();
    #[cfg(not(feature = "test-hooks"))]
    None
}

/// Lo stato degli aggiornamenti (versione in uso, ultima ricerca). Solo le Impostazioni.
#[tauri::command]
fn update_status(window: tauri::Window, app: tauri::AppHandle) -> Option<updates::Status> {
    if cmdpolicy::guard(window.label(), "update_status", None).is_err() {
        return None;
    }
    Some(updates::status(&app))
}

/// Cerca una versione nuova (una richiesta di rete al sito degli aggiornamenti). Non scarica né installa.
#[tauri::command(async)]
async fn update_check(window: tauri::Window, app: tauri::AppHandle) -> Option<updates::Status> {
    if cmdpolicy::guard(window.label(), "update_check", None).is_err() {
        return None;
    }
    Some(updates::check(&app).await)
}

/// Scarica e installa la versione trovata (la firma si controlla prima). Su Windows EosOS si chiude e si aggiorna.
#[tauri::command(async)]
async fn update_install(window: tauri::Window, app: tauri::AppHandle) -> Result<(), String> {
    cmdpolicy::guard(window.label(), "update_install", None)?;
    updates::install(&app).await
}

/// Solo build di prova: esegue come se fosse premuta la scorciatoia del numero `n` (senza tastiera).
#[tauri::command(async)]
fn dockkeys_fire(n: usize) -> bool {
    #[cfg(feature = "test-hooks")]
    return dockkeys::open_nth(n);
    #[cfg(not(feature = "test-hooks"))]
    {
        let _ = n;
        false
    }
}

/// Solo build di prova: esegue l'azione di finestre `name` (come «left», «maximize») su una finestra scelta (`hwnd`, un numero) o, se
/// manca, su quella in primo piano. `false` se la finestra non si può toccare.
#[tauri::command(async)]
fn winsnap_fire(name: String, hwnd: Option<isize>) -> bool {
    #[cfg(feature = "test-hooks")]
    {
        let Some(action) = winsnap::action_named(&name) else { return false };
        return match hwnd {
            Some(h) => winsnap::apply(windows::Win32::Foundation::HWND(h as *mut _), action),
            None => winsnap::apply_foreground(action),
        };
    }
    #[cfg(not(feature = "test-hooks"))]
    {
        let _ = (name, hwnd);
        false
    }
}

/// Centro di controllo: lo stato degli interruttori (Wi-Fi e Bluetooth se il PC li ha, mantieni sveglio).
#[tauri::command(async)]
fn control_state(window: tauri::Window) -> Option<control::ControlState> {
    guard!(window, "control_state", None);
    Some(control::state())
}

/// Centro di controllo: cambia l'interruttore `id` («wifi», «bluetooth», «awake»). `false` se non si può.
#[tauri::command(async)]
fn control_set(window: tauri::Window, id: String, on: bool) -> bool {
    guard!(window, "control_set", false);
    control::set(&id, on)
}

/// Cattura dello schermo dal pannello: `kind` è «screen» (schermo intero) o «region» (selezione). Parte dopo un attimo, perché il pannello
/// si chiuda e non finisca nell'immagine.
#[tauri::command(async)]
fn capture_run(window: tauri::Window, kind: String) -> bool {
    guard!(window, "capture_run", false);
    match capture::kind_named(&kind) {
        Some(k) if k != capture::Kind::Window => {
            capture::trigger(k, 350);
            true
        }
        _ => false,
    }
}

/// Il rettangolo scelto nella finestra di selezione (px CSS del monitor e scala): scatta e restituisce dove è stata salvata.
#[tauri::command(async)]
fn capture_region(window: tauri::Window, app: tauri::AppHandle, x: f64, y: f64, w: f64, h: f64, dpr: f64) -> Option<capture::Shot> {
    guard!(window, "capture_region", None);
    capturewin::finish(&app, x, y, w, h, dpr)
}

/// La selezione è annullata (Esc, o un clic senza trascinare).
#[tauri::command]
fn capture_cancel(window: tauri::Window, app: tauri::AppHandle) {
    guard!(window, "capture_cancel");
    capturewin::end(&app);
}

/// Solo build di prova: scatta `kind` («screen» o «window»; per la finestra si può dare l'handle) subito e dice dove ha salvato.
#[tauri::command(async)]
fn capture_fire(kind: String, hwnd: Option<isize>) -> Option<capture::Shot> {
    #[cfg(feature = "test-hooks")]
    return match (capture::kind_named(&kind)?, hwnd) {
        (capture::Kind::Window, Some(h)) => capture::run_window(windows::Win32::Foundation::HWND(h as *mut _)),
        (k, _) => capture::run(k),
    };
    #[cfg(not(feature = "test-hooks"))]
    {
        let _ = (kind, hwnd);
        None
    }
}

/// Quante scorciatoie della cattura sono registrate adesso (per le prove).
#[tauri::command]
fn capture_registered() -> usize {
    capture::registered_count()
}

/// Quante scorciatoie per le finestre sono registrate adesso (per le prove).
#[tauri::command]
fn winsnap_registered() -> usize {
    winsnap::registered_count()
}

/// Quante scorciatoie del Dock sono registrate adesso (per le prove).
#[tauri::command]
fn dockkeys_registered() -> usize {
    dockkeys::registered_count()
}

/// Badge delle notifiche per le icone del Dock: la pagina manda le sue icone (id, nome, percorso) e riceve solo i conteggi. Vuoto
/// se la lettura delle notifiche è spenta, se c'è Non disturbare o se i badge sono spenti. Solo il Dock.
#[tauri::command(async)]
fn dock_badges(window: tauri::Window, items: Vec<dockbadge::Item>) -> Vec<dockbadge::Badge> {
    guard!(window, "dock_badges", Vec::new());
    dockbadge::badges(&items)
}

/// Quante app girano in background (area di notifica) senza finestre né posto nel Dock: l'icona della barra in alto compare solo se sono almeno una.
#[tauri::command(async)]
fn tray_count(window: tauri::Window) -> usize {
    guard!(window, "tray_count", 0);
    if mcstore::load().dock_tray_off {
        return 0;
    }
    trayapps::count()
}

/// Le app in background per il pannello (id, nome, icona).
#[tauri::command(async)]
fn tray_apps(window: tauri::Window) -> Vec<trayapps::TrayApp> {
    guard!(window, "tray_apps", Vec::new());
    trayapps::apps()
}

/// Porta davanti (o avvia) l'app in background `id`, uno di quelli appena elencati.
#[tauri::command(async)]
fn tray_open(window: tauri::Window, id: String) -> bool {
    guard!(window, "tray_open", false);
    trayapps::open(&id)
}

/// Impostazioni di Windows dentro EosOS: informazioni sul sistema.
#[tauri::command(async)]
fn winsys_info(window: tauri::Window) -> Option<winsys::Info> {
    guard!(window, "winsys_info", None);
    Some(winsys::info())
}

/// I dischi fissi con spazio totale e libero.
#[tauri::command(async)]
fn winsys_drives(window: tauri::Window) -> Vec<winsys::Drive> {
    guard!(window, "winsys_drives", Vec::new());
    winsys::drives()
}

/// Gli schermi collegati (misura, scala, posizione).
#[tauri::command(async)]
fn winsys_displays(window: tauri::Window, app: tauri::AppHandle) -> Vec<winsys::Display> {
    guard!(window, "winsys_displays", Vec::new());
    winsys::displays(&app)
}

/// Il tema di Windows (app, sistema, trasparenza).
#[tauri::command(async)]
fn winsys_theme(window: tauri::Window) -> Option<winsys::Theme> {
    guard!(window, "winsys_theme", None);
    Some(winsys::theme())
}

/// Cambia un valore del tema: `which` è «apps», «system» o «transparency»; `on` vero = chiaro (o trasparenza accesa).
#[tauri::command(async)]
fn winsys_theme_set(window: tauri::Window, which: String, on: bool) -> bool {
    guard!(window, "winsys_theme_set", false);
    winsys::set_theme(&which, on)
}

/// Tempi di spegnimento dello schermo e di sospensione.
#[tauri::command(async)]
fn winsys_power(window: tauri::Window) -> Option<winsys::Power> {
    guard!(window, "winsys_power", None);
    Some(winsys::power())
}

/// Cambia un tempo di alimentazione: `which` come «screen-ac», `minutes` fra quelli offerti (0 = mai).
#[tauri::command(async)]
fn winsys_power_set(window: tauri::Window, which: String, minutes: u32) -> bool {
    guard!(window, "winsys_power_set", false);
    winsys::set_power(&which, minutes)
}

/// Le app che partono con Windows (voci dell'utente).
#[tauri::command(async)]
fn winsys_startup(window: tauri::Window) -> Vec<winsys::StartupApp> {
    guard!(window, "winsys_startup", Vec::new());
    winsys::startup_apps()
}

/// Accende o spegne una app all'avvio (solo una di quelle elencate).
#[tauri::command(async)]
fn winsys_startup_set(window: tauri::Window, name: String, on: bool) -> bool {
    guard!(window, "winsys_startup_set", false);
    winsys::set_startup(&name, on)
}

/// Le pile del Dock (cartelle con la loro icona). Solo il Dock.
#[tauri::command(async)]
fn dock_stacks(window: tauri::Window) -> Vec<stacks::StackItem> {
    guard!(window, "dock_stacks", Vec::new());
    stacks::items()
}

/// I file più recenti di una pila, per il pannello sopra l'icona. Solo il pannello.
#[tauri::command(async)]
fn stack_list(window: tauri::Window, index: usize) -> stacks::StackReply {
    if cmdpolicy::guard(window.label(), "stack_list", None).is_err() {
        return stacks::StackReply { title: String::new(), state: "missing", entries: Vec::new(), total: 0 };
    }
    stacks::list(index)
}

/// Apre un file della pila appena mostrato (i programmi e gli script si mostrano solo in Esplora risorse).
#[tauri::command(async)]
fn stack_open(window: tauri::Window, id: String) -> bool {
    guard!(window, "stack_open", false);
    stacks::open(&id)
}

/// Apre la cartella della pila in Esplora risorse.
#[tauri::command(async)]
fn stack_open_folder(window: tauri::Window, index: usize) -> bool {
    guard!(window, "stack_open_folder", false);
    stacks::open_folder(index)
}

/// L'ultima apertura registrata (solo build di prova con `EOSOS_TEST_NO_OPEN`): (percorso, solo-mostra).
#[tauri::command]
fn stack_last_open() -> Option<(String, bool)> {
    #[cfg(feature = "test-hooks")]
    return stacks::last_open();
    #[cfg(not(feature = "test-hooks"))]
    None
}

/// Lo stato di tutte le impostazioni (per la finestra delle impostazioni).
#[tauri::command(async)]
fn settings_all(window: tauri::Window, app: tauri::AppHandle) -> Vec<settingsreg::Entry> {
    guard!(window, "settings_all", Vec::new());
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
fn settings_set(window: tauri::Window, app: tauri::AppHandle, key: String, value: settingsreg::Value) -> SetReply {
    if let Err(e) = cmdpolicy::guard(window.label(), "settings_set", Some(&key)) {
        return SetReply { ok: false, live: false, error: Some(e) };
    }
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
fn settings_action(window: tauri::Window, app: tauri::AppHandle, key: String) -> SetReply {
    if let Err(e) = cmdpolicy::guard(window.label(), "settings_action", None) {
        return SetReply { ok: false, live: false, error: Some(e) };
    }
    let mut live = settingslive::Live::new(app);
    match settingsreg::run_action(&mut live, &key) {
        Ok(()) => SetReply { ok: true, live: true, error: None },
        Err(e) => SetReply { ok: false, live: false, error: Some(e.0) },
    }
}

/// Centro notifiche N1: prova di fattibilità in sola lettura (vedi `notifprobe.rs`).
#[tauri::command(async)]
fn notif_probe() -> notifprobe::Probe {
    notifprobe::probe()
}

/// Risposta di `notif_list`.
#[derive(serde::Serialize)]
struct NotifReply {
    /// "off" (lettura spenta dall'utente), "ok" o "error".
    state: &'static str,
    groups: Vec<notifmodel::Group>,
    /// Quante notifiche in tutto (comprese quelle oltre il tetto per gruppo).
    total: usize,
    message: Option<String>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// Lettura accesa e «Non disturbare» spento.
fn notif_allowed() -> bool {
    let s = mcstore::load();
    notifmodel::read_gate(s.notif_read, s.notif_dnd) == "go"
}

/// Le notifiche di Windows, già raggruppate per app. Solo se l'utente ha acceso «Mostra le notifiche»: altrimenti
/// risponde "off" senza leggere niente. I testi restano in memoria.
#[tauri::command(async)]
fn notif_list(window: tauri::Window) -> NotifReply {
    if cmdpolicy::guard(window.label(), "notif_list", None).is_err() {
        return NotifReply { state: "off", groups: Vec::new(), total: 0, message: None };
    }
    let s = mcstore::load();
    let gate = notifmodel::read_gate(s.notif_read, s.notif_dnd);
    if gate != "go" {
        return NotifReply { state: gate, groups: Vec::new(), total: 0, message: None };
    }
    match notifread::read_all() {
        Ok(raw) => {
            let groups = notifmodel::build_groups(&raw, now_ms());
            NotifReply { state: "ok", total: notifmodel::total(&groups), groups, message: None }
        }
        Err(m) => NotifReply { state: "error", groups: Vec::new(), total: 0, message: Some(m) },
    }
}

/// Toglie una notifica dal Centro notifiche di Windows. Solo se la lettura è accesa e l'id è fra quelli che ci sono
/// adesso (un id inventato si rifiuta).
#[tauri::command(async)]
fn notif_dismiss(window: tauri::Window, id: u32) -> bool {
    guard!(window, "notif_dismiss", false);
    if !notif_allowed() {
        return false;
    }
    let Ok(raw) = notifread::read_all() else { return false };
    if !notifmodel::contains_id(&notifmodel::build_groups(&raw, now_ms()), id) {
        return false;
    }
    notifread::remove(id).is_ok()
}

/// Come un clic sulla notifica: apre l'app che l'ha mandata e, se si apre, toglie la notifica. Solo con la lettura accesa
/// e per un id che c'è adesso.
#[tauri::command(async)]
fn notif_open(window: tauri::Window, id: u32) -> bool {
    guard!(window, "notif_open", false);
    if !notif_allowed() {
        return false;
    }
    let Ok(raw) = notifread::read_all() else { return false };
    if !notifmodel::contains_id(&notifmodel::build_groups(&raw, now_ms()), id) {
        return false;
    }
    let Some(n) = raw.iter().find(|r| r.id == id) else { return false };
    let opened = notifread::open_app(&n.app_id);
    if opened {
        let _ = notifread::remove(id);
    }
    opened
}

/// Toglie TUTTE le notifiche di un'app (il «×» del gruppo). L'app deve essere fra quelle che hanno notifiche adesso.
/// Risponde quante ne ha tolte.
#[tauri::command(async)]
fn notif_clear_app(window: tauri::Window, app_id: String) -> usize {
    guard!(window, "notif_clear_app", 0);
    if !notif_allowed() {
        return 0;
    }
    let Ok(raw) = notifread::read_all() else { return 0 };
    notifmodel::ids_of_app(&raw, &app_id).into_iter().filter(|id| notifread::remove(*id).is_ok()).count()
}

/// L'identificatore dell'ultima app «aperta» da una notifica (solo build di prova con `EOSOS_TEST_NO_OPEN`).
#[tauri::command]
fn notif_last_opened() -> Option<String> {
    #[cfg(feature = "test-hooks")]
    return notifread::last_opened();
    #[cfg(not(feature = "test-hooks"))]
    None
}

/// Quante notifiche toast di una app precisa ci sono adesso (solo per le prove: conta, non legge niente).
#[tauri::command(async)]
fn notif_count_for(aumid: String) -> Option<u32> {
    // Nelle build normali non fa niente: guarda gli identificatori delle app che hanno notifiche, e questo serve
    // solo alle prove (build con `test-hooks`).
    #[cfg(feature = "test-hooks")]
    return notifprobe::count_for_app(&aumid);
    #[cfg(not(feature = "test-hooks"))]
    {
        let _ = aumid;
        None
    }
}

/// Le chiavi delle impostazioni che corrispondono alla ricerca.
#[tauri::command]
fn settings_search(window: tauri::Window, query: String) -> Vec<&'static str> {
    guard!(window, "settings_search", Vec::new());
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
fn dock_activate(window: tauri::Window, id: String) {
    guard!(window, "dock_activate");
    if let Some((item, windows)) = dock::find_with_windows(&id) {
        dockact::activate(&item, &windows);
    }
    dock::invalidate();
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
fn dock_fit(window: tauri::Window, app: tauri::AppHandle, fit: dockgeom::FitArgs) {
    if !dockwin::is_dock_label(window.label()) {
        return;
    }
    if let Some(fit) = dockgeom::sanitize(fit) {
        dockwin::fit(&app, window.label(), fit);
    }
}

/// Ultimo fotogramma dello sfondo da sfocare, se più recente di `since` (vuoto
/// altrimenti). Risposta binaria: intestazione + pixel RGBA, vedi `backdrop::encode`.
#[tauri::command(async)]
fn dock_backdrop(window: tauri::Window, since: u64) -> tauri::ipc::Response {
    // Ogni Dock (uno per monitor) ha il suo canale di sfondo.
    let ch = dockwin::dock_index(window.label()).and_then(backdrop::Channel::dock);
    tauri::ipc::Response::new(ch.map_or_else(Vec::new, |c| backdrop::latest(c, since)))
}

/// Ricerca del Launcher tra le app. `have`: gli id di cui il frontend ha già l'icona. Lavoro
/// bloccante (shell, icone): fuori dal thread UI.
#[tauri::command(async)]
fn launcher_search(window: tauri::Window, query: String, have: Option<Vec<String>>, filter: Option<String>) -> Vec<launcher::LauncherItem> {
    guard!(window, "launcher_search", Vec::new());
    launcher::search(&query, &have.unwrap_or_default(), filter.as_deref())
}

/// Ricerca dei file del Launcher (indice di Windows Search). Bloccante e più lenta: la pagina la
/// chiama a parte dopo una breve pausa.
#[tauri::command(async)]
fn launcher_files(window: tauri::Window, query: String, have: Option<Vec<String>>) -> Vec<launcher::LauncherItem> {
    guard!(window, "launcher_files", Vec::new());
    launcher::files(&query, &have.unwrap_or_default())
}

/// Apre un file del Launcher, o lo mostra in Esplora risorse (`reveal`). Solo se Rust lo ha
/// appena mostrato come risultato: un percorso arbitrario è rifiutato.
#[tauri::command(async)]
fn launcher_open_file(window: tauri::Window, id: String, reveal: bool) -> bool {
    guard!(window, "launcher_open_file", false);
    launcherfiles::open(&id, reveal)
}

/// Consenso alla cronologia degli appunti (si salva solo il sì/no; il testo copiato resta in
/// memoria). Rifiutare svuota subito la cronologia.
#[tauri::command]
fn launcher_clip_set(window: tauri::Window, on: bool) {
    guard!(window, "launcher_clip_set");
    launcher::set_clipboard(on);
}

/// Rimette negli appunti una voce della cronologia (Rust ha il testo; la pagina manda solo l'id).
#[tauri::command(async)]
fn launcher_clip_use(window: tauri::Window, id: String) -> bool {
    guard!(window, "launcher_clip_use", false);
    launcher::clip_use(&id)
}

/// Toglie una voce dalla cronologia degli appunti.
#[tauri::command]
fn launcher_clip_delete(window: tauri::Window, id: String) -> bool {
    guard!(window, "launcher_clip_delete", false);
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
fn indicator_open(window: tauri::Window, id: String) -> bool {
    guard!(window, "indicator_open", false);
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
fn volume_set(window: tauri::Window, percent: f64) -> bool {
    guard!(window, "volume_set", false);
    indicators::set_volume(Some(percent), None)
}

/// Attiva o disattiva il silenzio del dispositivo di uscita.
#[tauri::command(async)]
fn volume_mute(window: tauri::Window, on: bool) -> bool {
    guard!(window, "volume_mute", false);
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
fn mc_activate(window: tauri::Window, app: tauri::AppHandle, id: u64) -> bool {
    guard!(window, "mc_activate", false);
    mcwin::activate(&app, id)
}

/// Chiede alla finestra `id` di chiudersi (come il pulsante X). Solo id elencati.
#[tauri::command(async)]
fn mc_close_window(window: tauri::Window, app: tauri::AppHandle, id: u64) -> bool {
    guard!(window, "mc_close_window", false);
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
fn mc_switch_desktop(window: tauri::Window, app: tauri::AppHandle, index: usize) -> bool {
    guard!(window, "mc_switch_desktop", false);
    mcwin::switch_desktop(&app, index)
}

/// Crea un nuovo desktop virtuale (solo mentre la panoramica è aperta).
#[tauri::command(async)]
fn mc_new_desktop(window: tauri::Window, app: tauri::AppHandle) -> bool {
    guard!(window, "mc_new_desktop", false);
    mcwin::new_desktop(&app)
}

/// Sposta una finestra su un altro desktop (backend interno, solo se ha superato la verifica).
#[tauri::command(async)]
fn mc_move_window(window: tauri::Window, app: tauri::AppHandle, id: u64, index: usize) -> bool {
    guard!(window, "mc_move_window", false);
    mcwin::move_window_to(&app, id, index)
}

/// Rimuove un desktop virtuale.
#[tauri::command(async)]
fn mc_remove_desktop(window: tauri::Window, app: tauri::AppHandle, index: usize) -> bool {
    guard!(window, "mc_remove_desktop", false);
    mcwin::remove_desktop(&app, index)
}

/// Dà un nome a un desktop virtuale (backend interno, solo se ha superato la verifica).
#[tauri::command(async)]
fn mc_rename_desktop(window: tauri::Window, app: tauri::AppHandle, index: usize, name: String) -> bool {
    guard!(window, "mc_rename_desktop", false);
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
fn launcher_action(window: tauri::Window, app: tauri::AppHandle, id: String) -> &'static str {
    guard!(window, "launcher_action", "unknown");
    // «Blocca» dal Launcher e' lo stesso blocco del menu: quello di EosOS (o di Windows, se la schermata EosOS e' spenta nelle Impostazioni).
    if id == "action:lock" {
        lockscreen::request_lock(&app);
        return "done";
    }
    match launcheract::run(&id) {
        launcheract::Outcome::Done => "done",
        launcheract::Outcome::Confirm => "confirm",
        launcheract::Outcome::Unknown => "unknown",
        launcheract::Outcome::Failed => "failed",
    }
}

/// Una pagina dell'elenco di tutte le app, in ordine alfabetico, per il Launchpad. Solo il Launcher.
#[tauri::command(async)]
fn launcher_grid(window: tauri::Window, offset: usize, limit: usize, have: Option<Vec<String>>) -> launcher::GridPage {
    if cmdpolicy::guard(window.label(), "launcher_grid", None).is_err() {
        return launcher::GridPage { total: 0, items: Vec::new() };
    }
    launcher::all_page(offset, limit, &have.unwrap_or_default())
}

/// L'icona del Launchpad nel Dock apre il Launchpad. Solo i Dock.
#[tauri::command]
fn dock_launchpad(window: tauri::Window, app: tauri::AppHandle) {
    guard!(window, "dock_launchpad");
    launcherwin::show_grid(&app);
}

/// Avvia un'app del Launcher. Il frontend manda solo l'`id` di un risultato che ha ricevuto;
/// ciò che si esegue lo ricava Rust dall'indice.
#[tauri::command(async)]
fn launcher_launch(window: tauri::Window, id: String) -> bool {
    guard!(window, "launcher_launch", false);
    launcher::launch(&id)
}

/// Ultimo fotogramma dello sfondo da sfocare dietro la barra dei menu che lo chiede (una per monitor). Vuoto se la
/// sfocatura è spenta o non c'è niente di nuovo. Sola lettura: nessuna restrizione per finestra.
#[tauri::command(async)]
fn bar_backdrop(window: tauri::Window, since: u64) -> tauri::ipc::Response {
    let ch = appbar::bar_index(window.label()).and_then(backdrop::Channel::bar);
    tauri::ipc::Response::new(ch.map_or_else(Vec::new, |c| backdrop::latest(c, since)))
}

/// Ultimo fotogramma dello sfondo da sfocare dietro il Launcher (vuoto se la sfocatura è spenta,
/// il Launcher è nascosto o non c'è nulla di nuovo). Stesso formato di `dock_backdrop`.
#[tauri::command(async)]
fn launcher_backdrop(window: tauri::Window, since: u64) -> tauri::ipc::Response {
    guard!(window, "launcher_backdrop", tauri::ipc::Response::new(Vec::<u8>::new()));
    tauri::ipc::Response::new(backdrop::latest(backdrop::Channel::Launcher, since))
}

/// Nasconde il Launcher (Esc, o dopo aver avviato qualcosa).
#[tauri::command]
fn launcher_hide(window: tauri::Window, app: tauri::AppHandle) {
    guard!(window, "launcher_hide");
    launcheract::cancel_pending();
    launcherwin::hide(&app);
}

/// Altezza che serve al Launcher (campo + risultati), in px CSS.
#[tauri::command]
fn launcher_fit(window: tauri::Window, app: tauri::AppHandle, height: f64) {
    guard!(window, "launcher_fit");
    launcherwin::fit(&app, height);
}

/// Trascinamento di un'icona: la mette nella posizione `index` tra le fissate
/// (contata senza di essa). Un'app aperta non fissata viene fissata lì. Come
/// sempre, il frontend manda solo l'`id` di una voce che il Dock sta mostrando.
#[tauri::command(async)]
fn dock_reorder(window: tauri::Window, id: String, index: usize) {
    guard!(window, "dock_reorder");
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
        dockwin::blur_setting(),
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
    let sep_size = tauri::menu::PredefinedMenuItem::separator(&app)?;
    let cur_size = match mcstore::load().dock_size.as_str() {
        "" => "medium".to_string(),
        s => s.to_string(),
    };
    let sizes = dockstore::SIZES
        .iter()
        .map(|(id, label, _)| CheckMenuItem::with_id(&app, menu_id(MenuAction::SetSize, id), format!("Dock {}", label.to_lowercase()), true, *id == cur_size, None::<&str>))
        .collect::<tauri::Result<Vec<_>>>()?;
    let all_screens = CheckMenuItem::with_id(&app, menu_id(MenuAction::ToggleAllScreens, "-"), "Dock su tutti gli schermi", true, !mcstore::load().dock_primary_only, None::<&str>)?;
    let sep2 = tauri::menu::PredefinedMenuItem::separator(&app)?;
    let settings = tauri::menu::MenuItem::with_id(&app, settingswin::MENU_ID, settingswin::MENU_LABEL, true, None::<&str>)?;
    let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&hide, &reserve, &blur, &sep];
    items.extend(levels.iter().map(|l| l as &dyn tauri::menu::IsMenuItem<tauri::Wry>));
    items.push(&sep_size);
    items.extend(sizes.iter().map(|l| l as &dyn tauri::menu::IsMenuItem<tauri::Wry>));
    items.push(&all_screens);
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
fn dock_settings() -> dockstore::Ui {
    dockstore::ui()
}

#[tauri::command]
fn glass_state() -> &'static str {
    glass::state().as_str()
}

/// Decide se nascondere la taskbar di Windows. Pura, testata: `env` è il valore di `EOSOS_HIDE_TASKBAR` (se c'è),
/// `keep` è la scelta dell'utente di lasciarla visibile, `test_build` dice se è una build di prova.
/// «1» e «0» nell'ambiente vincono su tutto; altrimenti di base si nasconde (il Dock la sostituisce), tranne nelle
/// build di prova, dove i test non devono mai nascondere la taskbar vera.
fn should_hide(env: Option<&str>, keep: bool, test_build: bool) -> bool {
    match env {
        Some("1") => true,
        Some("0") => false,
        _ => !keep && !test_build,
    }
}

fn hide_requested() -> bool {
    should_hide(std::env::var("EOSOS_HIDE_TASKBAR").ok().as_deref(), mcstore::load().keep_taskbar, cfg!(feature = "test-hooks"))
}

/// Annulla tutto ciò che EosOS ha cambiato nel sistema. Idempotente, chiamabile
/// da qualunque percorso di uscita (uscita normale, panic, Ctrl+C...).
fn cleanup() {
    appbar::remove_if_registered();
    taskbar::restore_if_hidden();
    lights::restore_all();
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
    // Memoria: tutte le pagine di EosOS (stesso sito) stanno in un solo processo di rendering invece di uno per finestra (circa -200 MB).
    // Prima di creare qualunque finestra; le opzioni eventualmente gia' impostate (per esempio per il debug) restano.
    if args.get(1).is_none_or(|a| !a.starts_with("--")) {
        let key = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";
        let cur = std::env::var(key).unwrap_or_default();
        if !cur.contains("--process-per-site") {
            // SAFETY: si e' ancora a inizio programma, in un solo thread, prima di creare finestre o thread.
            unsafe { std::env::set_var(key, format!("{cur} --process-per-site").trim()) };
        }
    }
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
        .plugin(tauri_plugin_updater::Builder::new().build())
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
                    let _ = tauri::Emitter::emit(&app, "dock-settings-changed", dockstore::ui());
                });
            }
        })
        .invoke_handler(tauri::generate_handler![lock_info, lock_wallpaper, lock_try_password, lock_try_blank, lock_try_hello, get_active_app, notif_list, notif_dismiss, notif_open, notif_clear_app, notif_last_opened, notif_probe, notif_count_for, settings_open, settings_close, settings_pick_folder, settings_action, settings_all, settings_set, settings_search, menubar_current, menubar_open, menubar_hover, menubar_menu, menubar_choose, glass_state, dock_items, dock_fit, dock_activate, dock_menu, dock_settings_menu, dock_settings, dock_reorder, dock_stacks, launcher_grid, dock_launchpad, dock_badges, tray_count, tray_apps, tray_open, winsys_info, winsys_drives, winsys_displays, winsys_theme, winsys_theme_set, winsys_power, winsys_power_set, winsys_startup, winsys_startup_set, eosmenu_run, eosmenu_about, dockkeys_fire, dockkeys_registered, winsnap_fire, winsnap_registered, capture_run, capture_region, capture_cancel, capture_fire, capture_registered, control_state, control_set, winsettings_list, winsettings_open, winsettings_last_open, bar_backdrop, oggi_reminders, oggi_add, oggi_toggle, oggi_remove, oggi_clear_done, oggi_weather, update_status, update_check, update_install, stack_list, stack_open, stack_open_folder, stack_last_open, dock_backdrop, launcher_search, launcher_launch, launcher_files, launcher_open_file, launcher_clip_set, launcher_clip_use, launcher_clip_delete, indicators_get, mc_windows, mc_open, mc_close, mc_activate, mc_close_window, mc_backdrop, mc_state, mc_desktops, mc_switch_desktop, mc_new_desktop, mc_move_window, mc_remove_desktop, mc_rename_desktop, mc_thumb_visible, mc_vd_report, menu_probe_all, indicators_menu, indicator_open, panel_toggle, panel_hide, panel_fit, panel_backdrop, volume_set, volume_mute, launcher_action, launcher_hide, launcher_fit, launcher_backdrop])
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
            // La finestra «main» (la barra del monitor principale) nasce dalla configurazione di Tauri: la si protegge dalla cornice come le altre.
            if let Some(w) = app.get_webview_window("main") {
                chromeguard::protect(&w);
            }
            appbar::start(app.handle());
            dockwin::start(app.handle());
            dockkeys::start();
            winsnap::start();
            chromeguard::start();
            capture::start(app.handle());
            backdrop::start(app.handle().clone());
            // Aggiornamenti: solo se l'utente ha acceso la ricerca all'avvio; cerca soltanto, non installa mai.
            if mcstore::load().update_auto {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(updates::AUTO_DELAY);
                    let _ = tauri::async_runtime::block_on(updates::check(&handle));
                });
            }
            // Solo build di prova: aggiornamento automatico completo (cerca e, se c'è, installa) senza premere i
            // pulsanti, per provarlo nella Sandbox dove non c'è la porta di debug. Passa dalle stesse funzioni dei comandi.
            #[cfg(feature = "test-hooks")]
            if std::env::var_os("EOSOS_TEST_AUTOUPDATE").is_some() {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(15));
                    // Fuori dalla cartella di installazione: l'aggiornamento la sostituisce e il registro andrebbe perso.
                    let log = |m: String| {
                        let path = std::env::temp_dir().join("eosos-autoupdate.log");
                        let _ = std::fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| {
                            use std::io::Write;
                            writeln!(f, "{m}")
                        });
                    };
                    let st = tauri::async_runtime::block_on(updates::check(&handle));
                    log(format!("check: {} ({})", st.state, st.text));
                    if st.state == "available" {
                        let r = tauri::async_runtime::block_on(updates::install(&handle));
                        log(format!("install: {r:?}"));
                    }
                });
            }
            launcher::warm_up();
            launcherclip::start();
            indicators::start(app.handle().clone());
            launcherwin::start(app.handle());
            lockscreen::start(app.handle());
            lights::start();
            remote::start(app.handle().clone());
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
