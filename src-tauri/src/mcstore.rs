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
    fn parse_tolerates_bom_and_rejects_garbage() {
        let with_bom = format!("{}{}", char::from_u32(0xfeff).unwrap(), r#"{"hotCorner":true}"#);
        assert!(parse(&with_bom).unwrap().hot_corner);
        for bad in ["", "non json", r#"{"hotCorner":"si"}"#, "[1]"] {
            assert!(parse(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn serialize_roundtrips() {
        for s in [Settings::default(), Settings { hot_corner: true, hotkey: Some("Ctrl+Alt+Up".into()), windows_alt_tab: true, windows_win_tab: false }] {
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
