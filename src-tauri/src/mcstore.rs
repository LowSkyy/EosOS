//! Impostazioni di Mission Control, in `%LOCALAPPDATA%\EosOS\mission.json`: l'angolo attivo (spento di
//! default: muovere il mouse in un angolo non deve far partire nulla finché non lo si sceglie) e la
//! scorciatoia di apertura (predefinita Ctrl+Alt+Su).
//!
//! Stessa robustezza di `dock.json`, `launcher.json` e `indicators.json`: tolleranza del BOM, file non
//! valido conservato come `.bad` alla prima modifica, scrittura atomica sotto lock, nessuna riscrittura
//! dopo una lettura fallita.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::dockstore::{strip_bom, Read};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Angolo attivo: il cursore fermo nell'angolo in alto a sinistra di un monitor apre la panoramica.
    #[serde(rename = "hotCorner")]
    pub hot_corner: bool,
    /// Scorciatoia di apertura (per esempio "Ctrl+Alt+Up"); assente = predefinita.
    pub hotkey: Option<String>,
    /// Alt+Tab resta quello di Windows (di default lo sostituisce il selettore di Mission Control).
    #[serde(rename = "windowsAltTab")]
    pub windows_alt_tab: bool,
    /// Win+Tab resta la Visualizzazione attività di Windows (di default apre Mission Control).
    #[serde(rename = "windowsWinTab")]
    pub windows_win_tab: bool,
    /// Menu globale spento: la barra mostra solo il nome dell'app, come prima (di default è acceso).
    #[serde(rename = "menuOff")]
    pub menu_off: bool,
    /// App di cui non si legge né si mostra il menu (nome come appare nella barra).
    #[serde(rename = "menuExcluded")]
    pub menu_excluded: Vec<String>,
    /// Mostrare le notifiche di Windows nel Centro notifiche di EosOS (spento di default: sono testi privati).
    #[serde(rename = "notifRead")]
    pub notif_read: bool,
    /// Il calendario comincia la settimana da domenica (di default da lunedì).
    #[serde(rename = "calendarSunday")]
    pub calendar_sunday: bool,
    /// «Non disturbare» di EosOS: il pannello non legge né mostra le notifiche (quelle di Windows restano dove sono).
    #[serde(rename = "notifDnd")]
    pub notif_dnd: bool,
    /// Cartelle delle pile nel Dock (`None` = non scelte: c'è solo Download).
    #[serde(rename = "dockStacks")]
    pub dock_stacks: Option<Vec<String>>,
    /// Cerca aggiornamenti da solo all'avvio (spento di default: è una richiesta di rete).
    #[serde(rename = "updateAuto")]
    pub update_auto: bool,
    /// Widget «Oggi»: i promemoria dell'utente.
    pub reminders: Vec<crate::reminders::Reminder>,
    /// Widget «Oggi»: meteo acceso (spento di default: è una richiesta di rete) e città scelta.
    #[serde(rename = "weatherOn")]
    pub weather_on: bool,
    #[serde(rename = "weatherCity")]
    pub weather_city: Option<String>,
    /// Dimensione del Dock: "small", "medium" (predefinita, anche se vuoto), "large" o "xl".
    #[serde(rename = "dockSize")]
    pub dock_size: String,
    /// La barra in alto non mostra l'icona delle app in background (area di notifica); di base si vede quando ce n'è almeno una. Il nome del
    /// campo resta «dockTrayOff» (da quando l'icona stava nel Dock) per non perdere la scelta già salvata.
    #[serde(rename = "dockTrayOff")]
    pub dock_tray_off: bool,
    /// Animazione di un'app che si apre: "bounce" (vuoto), "pulse" o "none".
    #[serde(rename = "dockAnim")]
    pub dock_anim: String,
    /// L'ingrandimento delle icone del Dock al passaggio del mouse è spento (di base è acceso).
    #[serde(rename = "dockMagnifyOff")]
    pub dock_magnify_off: bool,
    /// Modificatore delle scorciatoie per la cattura dello schermo ("ctrl_shift", "ctrl_alt", "alt_shift"); vuoto = spente.
    #[serde(rename = "captureKeys")]
    pub capture_keys: String,
    /// I badge delle notifiche sulle icone del Dock sono spenti (di base si vedono, se la lettura delle notifiche è accesa).
    #[serde(rename = "dockBadgesOff")]
    pub dock_badges_off: bool,
    /// Modificatore delle scorciatoie per disporre le finestre ("ctrl_alt", "ctrl_shift", "alt_shift"); vuoto = spente.
    #[serde(rename = "snapKeys")]
    pub snap_keys: String,
    /// Il tasto Windows da solo NON apre il Launcher (di base lo apre, al posto del menu Start).
    #[serde(rename = "winKeyOff")]
    pub win_key_off: bool,
    /// Barra dei menu: mostra il nome dell'app in primo piano accanto al menu EosOS (di base NON si vede). Sostituisce il vecchio
    /// «menuHideName»: chi lo aveva salvato lo ignora, e il nome resta nascosto.
    #[serde(rename = "menuShowName")]
    pub menu_show_name: bool,
    /// Barra dei menu: mostra anche il menu «Finestra» (di base è nascosto).
    #[serde(rename = "menuShowWindow")]
    pub menu_show_window: bool,
    /// Lascia visibile la barra delle applicazioni di Windows (di base EosOS la nasconde: il Dock la sostituisce).
    #[serde(rename = "keepTaskbar")]
    pub keep_taskbar: bool,
    /// Il Dock sta solo sul monitor primario (di base ce n'è uno per monitor).
    #[serde(rename = "dockPrimaryOnly")]
    pub dock_primary_only: bool,
    /// Scorciatoie per aprire le app del Dock: "off", "ctrl_win", "ctrl_alt", "ctrl_shift" (vuoto = quella di base).
    #[serde(rename = "dockKeys")]
    pub dock_keys: String,
    /// La schermata di blocco di EosOS e' spenta: «Blocca lo schermo» usa quella di Windows (di base si usa quella di EosOS).
    #[serde(rename = "lockOff")]
    pub lock_off: bool,
    /// Blocca da solo dopo tanti minuti di inattivita' (0 = mai, di base).
    #[serde(rename = "lockIdle")]
    pub lock_idle: u32,
    /// Dopo tanti minuti di blocco EosOS passa il testimone al blocco di Windows (0 = mai); `None` = di base, 5 minuti.
    #[serde(rename = "lockWindows")]
    pub lock_windows: Option<u32>,
    /// I «semafori» macOS al posto dei tasti di Windows sono spenti (di base sono accesi).
    #[serde(rename = "lightsOff")]
    pub lights_off: bool,
    /// I semafori stanno a sinistra, come su macOS (di base stanno a destra, dove non coprono nulla: a sinistra nascondono menu e schede).
    #[serde(rename = "lightsLeft")]
    pub lights_left: bool,
    /// Le finestre di Esplora file NON prendono lo stile «Finder» (bordo tolto; di base lo prendono).
    #[serde(rename = "finderOff")]
    pub finder_off: bool,
    /// Con un controllo remoto in corso la sfocatura RESTA accesa (di base si spegne da sola, perche' la sfocatura rende Dock e barra
    /// invisibili a chi guarda da lontano).
    #[serde(rename = "remoteKeepBlur")]
    pub remote_keep_blur: bool,
}

impl Settings {
    /// Minuti di blocco EosOS dopo i quali scatta anche quello di Windows (0 = mai).
    pub fn lock_windows_minutes(&self) -> u32 {
        self.lock_windows.unwrap_or(5)
    }
}

fn store_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\mission.json"))
}

/// Le impostazioni dal testo del file (con o senza BOM), o `None` se non è JSON valido. Campi mancanti =
/// predefiniti. Pura, testata.
pub fn parse(text: &str) -> Option<Settings> {
    serde_json::from_str(strip_bom(text)).ok()
}

fn serialize(s: &Settings) -> String {
    #[derive(Serialize)]
    struct File<'a> {
        version: u32,
        #[serde(flatten)]
        s: &'a Settings,
    }
    serde_json::to_string_pretty(&File { version: 1, s }).unwrap_or_default()
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Use(Settings),
    Fresh,
    BackupAndFresh,
    KeepLast(Settings),
}

impl Decision {
    pub fn is_reliable(&self) -> bool {
        !matches!(self, Decision::KeepLast(_))
    }
}

/// Decide come usare l'esito della lettura: un errore di accesso non è un file mancante. Pura, testata.
pub fn decide(read: Read, last_good: Option<&Settings>) -> Decision {
    match read {
        Read::Text(t) => parse(&t).map_or(Decision::BackupAndFresh, Decision::Use),
        Read::Missing => Decision::Fresh,
        Read::Failed => Decision::KeepLast(last_good.cloned().unwrap_or_default()),
    }
}

static LOCK: Mutex<()> = Mutex::new(());
static LAST: Mutex<Option<Settings>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn last() -> Option<Settings> {
    LAST.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn remember(s: &Settings) {
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(s.clone());
}

fn read_store() -> Read {
    let Some(path) = store_path() else { return Read::Missing };
    for attempt in 0..4u64 {
        match std::fs::read_to_string(&path) {
            Ok(t) => return Read::Text(t),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Read::Missing,
            Err(_) => std::thread::sleep(Duration::from_millis(15 * (attempt + 1))),
        }
    }
    Read::Failed
}

fn write_file(s: &Settings) {
    let Some(path) = store_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, serialize(s)).is_ok() {
        if std::fs::rename(&tmp, &path).is_ok() {
            remember(s);
        } else {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}

fn load_full() -> (Settings, bool) {
    let decision = decide(read_store(), last().as_ref());
    let reliable = decision.is_reliable();
    match decision {
        Decision::Use(s) => {
            remember(&s);
            (s, reliable)
        }
        Decision::KeepLast(s) => (s, reliable),
        Decision::Fresh | Decision::BackupAndFresh => (Settings::default(), reliable),
    }
}

/// Le impostazioni correnti (file mancante o rotto: predefinite; non leggibile: le ultime note).
pub fn load() -> Settings {
    load_full().0
}

/// Applica `f` alle impostazioni e salva. Se il file non si è potuto leggere non scrive nulla. Un file
/// non valido viene prima conservato come `.bad`. Restituisce le impostazioni risultanti.
fn update(f: impl FnOnce(&mut Settings)) -> Settings {
    let _g = lock();
    let broken = matches!(read_store(), Read::Text(t) if parse(&t).is_none());
    let (mut cur, reliable) = load_full();
    if !reliable {
        return cur;
    }
    if broken
        && let Some(path) = store_path()
    {
        let _ = std::fs::copy(&path, path.with_extension("json.bad"));
    }
    f(&mut cur);
    write_file(&cur);
    cur
}

/// Accende o spegne l'angolo attivo e salva.
pub fn set_hot_corner(on: bool) -> Settings {
    update(|s| s.hot_corner = on)
}

/// Sceglie se Alt+Tab resta quello di Windows e salva.
pub fn set_windows_alt_tab(keep: bool) -> Settings {
    update(|s| s.windows_alt_tab = keep)
}

/// Salva la scorciatoia di apertura di Mission Control (`None` = la predefinita).
pub fn set_hotkey(h: Option<String>) -> Settings {
    update(|s| s.hotkey = h)
}

/// Sceglie se il calendario comincia la settimana da domenica e salva.
pub fn set_calendar_sunday(on: bool) -> Settings {
    update(|s| s.calendar_sunday = on)
}

/// Sceglie le scorciatoie per le app del Dock e salva.
pub fn set_dock_keys(id: &str) -> Settings {
    update(|s| s.dock_keys = id.to_string())
}

/// Sceglie se il Dock sta su tutti i monitor (`all`) o solo sul primario, e salva.
pub fn set_dock_all_screens(all: bool) -> Settings {
    update(|s| s.dock_primary_only = !all)
}

/// Sceglie se EosOS nasconde la barra delle applicazioni di Windows (vale dal prossimo avvio) e salva.
pub fn set_hide_taskbar(hide: bool) -> Settings {
    update(|s| s.keep_taskbar = !hide)
}

/// Barra dei menu: mostrare il nome dell'app e il menu «Finestra».
pub fn set_menu_view(show_name: Option<bool>, show_window: Option<bool>) -> Settings {
    update(|s| {
        if let Some(v) = show_name {
            s.menu_show_name = v;
        }
        if let Some(v) = show_window {
            s.menu_show_window = v;
        }
    })
}

/// Sceglie la dimensione del Dock e salva.
pub fn set_dock_size(id: &str) -> Settings {
    update(|s| s.dock_size = id.to_string())
}

/// Applica `f` ai promemoria e salva; restituisce la lista risultante.
pub fn update_reminders(f: impl FnOnce(&mut Vec<crate::reminders::Reminder>)) -> Vec<crate::reminders::Reminder> {
    update(|s| f(&mut s.reminders)).reminders
}

/// Accende o spegne il meteo e salva.
pub fn set_weather_on(on: bool) -> Settings {
    update(|s| s.weather_on = on)
}

/// Salva la città del meteo (`None` = nessuna).
pub fn set_weather_city(c: Option<String>) -> Settings {
    update(|s| s.weather_city = c)
}

/// Accende o spegne la ricerca di aggiornamenti all'avvio e salva.
pub fn set_update_auto(on: bool) -> Settings {
    update(|s| s.update_auto = on)
}

/// Salva le cartelle delle pile nel Dock (`None` = di nuovo solo Download).
pub fn set_dock_stacks(v: Option<Vec<String>>) -> Settings {
    update(|s| s.dock_stacks = v)
}

/// Accende o spegne «Non disturbare» di EosOS e salva.
pub fn set_notif_dnd(on: bool) -> Settings {
    update(|s| s.notif_dnd = on)
}

/// Accende o spegne la lettura delle notifiche e salva.
pub fn set_notif_read(on: bool) -> Settings {
    update(|s| s.notif_read = on)
}

/// Accende o spegne il menu globale e salva.
pub fn set_menu_off(off: bool) -> Settings {
    update(|s| s.menu_off = off)
}

/// Mette l'elenco delle app escluse dal menu globale e salva.
pub fn set_menu_excluded(list: Vec<String>) -> Settings {
    update(|s| s.menu_excluded = list)
}

/// Sceglie se il tasto Windows da solo apre il Launcher (`on` vero) e salva.
pub fn set_win_key_launcher(on: bool) -> Settings {
    update(|s| s.win_key_off = !on)
}

/// Sceglie il modificatore delle scorciatoie per le finestre (vuoto o "off" = spente) e salva.
pub fn set_lock_eos(on: bool) -> Settings {
    update(|s| s.lock_off = !on)
}

pub fn set_lights(on: bool) -> Settings {
    update(|s| s.lights_off = !on)
}

pub fn set_lights_left(left: bool) -> Settings {
    update(|s| s.lights_left = left)
}

pub fn set_remote_blur_off(on: bool) -> Settings {
    update(|s| s.remote_keep_blur = !on)
}

pub fn set_finder(on: bool) -> Settings {
    update(|s| s.finder_off = !on)
}

pub fn set_lock_idle(minutes: u32) -> Settings {
    update(|s| s.lock_idle = minutes)
}

pub fn set_lock_windows(minutes: u32) -> Settings {
    update(|s| s.lock_windows = Some(minutes))
}

pub fn set_snap_keys(id: &str) -> Settings {
    update(|s| s.snap_keys = if id == "off" { String::new() } else { id.to_string() })
}

/// Accende (`on` vero) o spegne i badge delle notifiche sulle icone del Dock e salva.
pub fn set_dock_badges(on: bool) -> Settings {
    update(|s| s.dock_badges_off = !on)
}

/// Sceglie il modificatore delle scorciatoie per la cattura dello schermo (vuoto o "off" = spente) e salva.
pub fn set_capture_keys(id: &str) -> Settings {
    update(|s| s.capture_keys = if id == "off" { String::new() } else { id.to_string() })
}

/// Sceglie l'animazione delle app che si aprono ("bounce" = quella di base) e salva.
pub fn set_dock_anim(id: &str) -> Settings {
    update(|s| s.dock_anim = if id == "bounce" { String::new() } else { id.to_string() })
}

/// Accende o spegne l'ingrandimento delle icone del Dock al passaggio del mouse e salva.
pub fn set_dock_magnify(on: bool) -> Settings {
    update(|s| s.dock_magnify_off = !on)
}

/// Accende o spegne l'icona delle app in background nel Dock e salva.
pub fn set_bar_tray(on: bool) -> Settings {
    update(|s| s.dock_tray_off = !on)
}

/// Sceglie se Win+Tab resta quello di Windows e salva.
pub fn set_windows_win_tab(keep: bool) -> Settings {
    update(|s| s.windows_win_tab = keep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_the_corner_off_and_the_default_hotkey() {
        assert_eq!(Settings::default(), Settings::default());
        assert_eq!(parse("{}"), Some(Settings::default()));
        assert_eq!(parse(r#"{"hotCorner":true}"#), Some(Settings { hot_corner: true, ..Settings::default() }));
        assert_eq!(parse(r#"{"hotkey":"Ctrl+Alt+K"}"#).unwrap().hotkey.as_deref(), Some("Ctrl+Alt+K"));
    }

    #[test]
    fn the_replacements_of_alt_tab_and_win_tab_are_on_by_default() {
        let d = Settings::default();
        assert!(!d.windows_alt_tab && !d.windows_win_tab, "di default sostituiscono quelli di Windows");
        let s = parse(r#"{"windowsAltTab":true}"#).unwrap();
        assert!(s.windows_alt_tab && !s.windows_win_tab);
    }

    #[test]
    fn the_global_menu_is_on_by_default_with_no_exclusions() {
        let d = Settings::default();
        assert!(!d.menu_off && d.menu_excluded.is_empty());
        assert!(!d.notif_read, "la lettura delle notifiche e' spenta di default");
        assert!(d.reminders.is_empty() && !d.weather_on && d.weather_city.is_none(), "il widget Oggi parte vuoto e senza rete");
        assert!(d.dock_keys.is_empty(), "scorciatoie del Dock: quelle di base");
        assert!(!d.dock_badges_off, "i badge del Dock sono accesi di base");
        assert!(d.dock_anim.is_empty() && !d.dock_magnify_off, "Dock: rimbalzo e ingrandimento di base");
        assert!(!d.dock_tray_off, "app in background nella barra: acceso di base");
        assert!(d.capture_keys.is_empty(), "scorciatoie della cattura: spente di base");
        assert!(d.snap_keys.is_empty(), "scorciatoie delle finestre: spente di base");
        assert!(!d.dock_primary_only, "di base il Dock c'e' su ogni monitor");
        assert!(!d.keep_taskbar, "di base EosOS nasconde la barra delle applicazioni di Windows");
        assert!(!d.menu_show_name && !d.menu_show_window, "di base: nome dell'app e menu Finestra nascosti");
        assert!(d.dock_size.is_empty(), "dimensione del Dock non scelta: quella media");
        assert!(!d.update_auto, "nessuna richiesta di rete da sola di default");
        assert!(d.dock_stacks.is_none(), "di default nessuna scelta: c'e' solo Download");
        assert!(!d.notif_dnd, "non disturbare e' spento di default");
        assert!(!d.calendar_sunday, "la settimana comincia da lunedi' di default");
        let s = parse(r#"{"menuOff":true,"menuExcluded":["Chrome","Discord"]}"#).unwrap();
        assert!(s.menu_off && s.menu_excluded == vec!["Chrome".to_string(), "Discord".to_string()]);
        assert!(parse(r#"{"menuExcluded":"Chrome"}"#).is_none(), "una lista che non e una lista e' un file non valido");
    }

    #[test]
    fn parse_tolerates_bom_and_rejects_garbage() {
        let with_bom = format!("{}{}", char::from_u32(0xfeff).unwrap(), r#"{"hotCorner":true}"#);
        assert!(parse(&with_bom).unwrap().hot_corner);
        for bad in ["", "non json", r#"{"hotCorner":"si"}"#, "[1]"] {
            assert!(parse(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn serialize_roundtrips() {
        for s in [Settings::default(), Settings { hot_corner: true, hotkey: Some("Ctrl+Alt+Up".into()), windows_alt_tab: true, windows_win_tab: false, menu_off: true, menu_excluded: vec!["Chrome".into()], notif_read: true, calendar_sunday: true, notif_dnd: true, dock_stacks: Some(vec!["D:\\Dati".into()]), update_auto: true, reminders: vec![crate::reminders::Reminder { id: 1, text: "x".into(), time: Some("09:30".into()), done: true }], weather_on: true, weather_city: Some("Roma".into()), dock_size: "large".into(), dock_tray_off: true, dock_anim: "none".into(), dock_magnify_off: true, capture_keys: "ctrl_shift".into(), dock_badges_off: true, snap_keys: "alt_shift".into(), win_key_off: true, menu_show_name: true, menu_show_window: true, keep_taskbar: true, dock_primary_only: true, dock_keys: "ctrl_alt".into(), lock_off: true, lock_idle: 10, lock_windows: Some(15), lights_off: true, lights_left: true, finder_off: true, remote_keep_blur: true }] {
            assert_eq!(parse(&serialize(&s)), Some(s));
        }
    }

    #[test]
    fn a_failed_read_is_never_treated_as_missing() {
        let last = Settings { hot_corner: true, ..Settings::default() };
        assert_eq!(decide(Read::Failed, Some(&last)), Decision::KeepLast(last.clone()));
        assert_eq!(decide(Read::Failed, None), Decision::KeepLast(Settings::default()));
        assert!(!decide(Read::Failed, None).is_reliable());
        assert_eq!(decide(Read::Missing, Some(&last)), Decision::Fresh);
        assert_eq!(decide(Read::Text("rotto".into()), Some(&last)), Decision::BackupAndFresh);
        assert_eq!(decide(Read::Text(r#"{"hotCorner":true}"#.into()), None), Decision::Use(last));
        assert!(Decision::Fresh.is_reliable() && Decision::BackupAndFresh.is_reliable());
    }
}
