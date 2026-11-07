//! Impostazioni degli indicatori di sistema, in `%LOCALAPPDATA%\EosOS\indicators.json`: quali
//! indicatori mostrare (batteria, rete, audio) e se la batteria mostra la percentuale. Di
//! default è tutto acceso.
//!
//! Stessa robustezza di `dock.json` e `launcher.json`: tolleranza del BOM, file illeggibile
//! conservato come `.bad`, scrittura atomica sotto lock, e MAI una riscrittura dopo una lettura
//! fallita (un errore di accesso non è un file mancante).

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::dockstore::{strip_bom, Read};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub battery: bool,
    pub network: bool,
    pub volume: bool,
    /// La batteria mostra la percentuale accanto all'icona.
    pub percent: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { battery: true, network: true, volume: true, percent: true }
    }
}

/// Una scelta del menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    Battery,
    Network,
    Volume,
    Percent,
}

const PREFIX: &str = "ind";

/// Id di una voce di menu per la scelta `t`. Pura, testata.
pub fn menu_id(t: Toggle) -> String {
    let name = match t {
        Toggle::Battery => "battery",
        Toggle::Network => "network",
        Toggle::Volume => "volume",
        Toggle::Percent => "percent",
    };
    format!("{PREFIX}:{name}")
}

/// Inverso di `menu_id`: `None` per qualunque altro id (anche quelli del Dock). Pura, testata.
pub fn parse_menu_id(raw: &str) -> Option<Toggle> {
    match raw.strip_prefix("ind:")? {
        "battery" => Some(Toggle::Battery),
        "network" => Some(Toggle::Network),
        "volume" => Some(Toggle::Volume),
        "percent" => Some(Toggle::Percent),
        _ => None,
    }
}

/// Le impostazioni con la scelta `t` invertita. Pura, testata.
pub fn toggled(s: Settings, t: Toggle) -> Settings {
    let mut n = s;
    match t {
        Toggle::Battery => n.battery = !s.battery,
        Toggle::Network => n.network = !s.network,
        Toggle::Volume => n.volume = !s.volume,
        Toggle::Percent => n.percent = !s.percent,
    }
    n
}

fn store_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\indicators.json"))
}

/// Le impostazioni dal testo del file (con o senza BOM), o `None` se non è JSON valido. Campi
/// mancanti = acceso. Pura, testata.
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
    /// I dati sono quelli del file (o il file va ricreato)? `false` solo per `KeepLast`.
    pub fn is_reliable(&self) -> bool {
        !matches!(self, Decision::KeepLast(_))
    }
}

/// Decide come usare l'esito della lettura. Pura, testata.
pub fn decide(read: Read, last_good: Option<Settings>) -> Decision {
    match read {
        Read::Text(t) => parse(&t).map_or(Decision::BackupAndFresh, Decision::Use),
        Read::Missing => Decision::Fresh,
        Read::Failed => Decision::KeepLast(last_good.unwrap_or_default()),
    }
}

static LOCK: Mutex<()> = Mutex::new(());
static LAST: Mutex<Option<Settings>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn last() -> Option<Settings> {
    *LAST.lock().unwrap_or_else(|e| e.into_inner())
}

fn remember(s: Settings) {
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(s);
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
            remember(*s);
        } else {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}

/// Impostazioni correnti e se sono affidabili. Un file mancante NON viene creato qui (si crea alla
/// prima modifica); un file rotto si conserva come `.bad` alla prima modifica.
fn load_full() -> (Settings, bool) {
    let decision = decide(read_store(), last());
    let reliable = decision.is_reliable();
    match decision {
        Decision::Use(s) => {
            remember(s);
            (s, reliable)
        }
        Decision::KeepLast(s) => (s, reliable),
        Decision::Fresh | Decision::BackupAndFresh => (Settings::default(), reliable),
    }
}

/// Le impostazioni correnti (file mancante o rotto: tutto acceso; non leggibile: le ultime note).
pub fn load() -> Settings {
    load_full().0
}

/// Inverte una scelta e salva. Se il file non si è potuto leggere non scrive nulla. Un file
/// non valido viene prima conservato come `.bad`. Restituisce le impostazioni risultanti.
pub fn toggle(t: Toggle) -> Settings {
    let _g = lock();
    let text = read_store();
    let broken = matches!(&text, Read::Text(t) if parse(t).is_none());
    let (cur, reliable) = load_full();
    if !reliable {
        return cur;
    }
    if broken
        && let Some(path) = store_path()
    {
        let _ = std::fs::copy(&path, path.with_extension("json.bad"));
    }
    let new = toggled(cur, t);
    write_file(&new);
    new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_show_everything_with_the_percentage() {
        assert_eq!(Settings::default(), Settings { battery: true, network: true, volume: true, percent: true });
        assert_eq!(parse("{}"), Some(Settings::default()), "campi mancanti: acceso");
        assert_eq!(parse(r#"{"version":1,"network":false}"#), Some(Settings { network: false, ..Default::default() }));
    }

    #[test]
    fn parse_tolerates_bom_and_rejects_garbage() {
        let with_bom = format!("{}{}", char::from_u32(0xfeff).unwrap(), r#"{"percent":false}"#);
        assert_eq!(parse(&with_bom), Some(Settings { percent: false, ..Default::default() }));
        assert_eq!(parse("non json"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse(r#"{"battery":"si"}"#), None, "tipo sbagliato");
    }

    #[test]
    fn serialize_roundtrips() {
        for s in [Settings::default(), Settings { battery: false, network: true, volume: false, percent: false }] {
            assert_eq!(parse(&serialize(&s)), Some(s));
        }
        assert!(serialize(&Settings::default()).contains("\"version\": 1"));
    }

    #[test]
    fn toggling_flips_exactly_one_field() {
        let s = Settings::default();
        assert_eq!(toggled(s, Toggle::Battery), Settings { battery: false, ..s });
        assert_eq!(toggled(s, Toggle::Network), Settings { network: false, ..s });
        assert_eq!(toggled(s, Toggle::Volume), Settings { volume: false, ..s });
        assert_eq!(toggled(s, Toggle::Percent), Settings { percent: false, ..s });
        assert_eq!(toggled(toggled(s, Toggle::Volume), Toggle::Volume), s);
    }

    #[test]
    fn menu_ids_roundtrip_and_ignore_other_menus() {
        for t in [Toggle::Battery, Toggle::Network, Toggle::Volume, Toggle::Percent] {
            assert_eq!(parse_menu_id(&menu_id(t)), Some(t));
        }
        for bad in ["", "ind:", "ind:boom", "dock:autohide:-", "ind", "xind:battery", "ind:battery:x"] {
            assert_eq!(parse_menu_id(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn a_failed_read_is_never_treated_as_missing() {
        let last = Settings { volume: false, ..Default::default() };
        assert_eq!(decide(Read::Failed, Some(last)), Decision::KeepLast(last));
        assert_eq!(decide(Read::Failed, None), Decision::KeepLast(Settings::default()));
        assert!(!decide(Read::Failed, None).is_reliable());
        assert_eq!(decide(Read::Missing, Some(last)), Decision::Fresh);
        assert_eq!(decide(Read::Text("rotto".into()), Some(last)), Decision::BackupAndFresh);
        assert_eq!(decide(Read::Text(r#"{"battery":false}"#.into()), None), Decision::Use(Settings { battery: false, ..Default::default() }));
        assert!(Decision::Fresh.is_reliable() && Decision::BackupAndFresh.is_reliable());
    }
}
