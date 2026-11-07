//! Uso delle app del Launcher (per la "frecency"), salvato in `%LOCALAPPDATA%\EosOS\launcher.json`.
//!
//! Stessa robustezza di `dockstore`: tolleranza del BOM, file illeggibile conservato come
//! `.bad`, scrittura atomica sotto lock, e MAI una riscrittura dopo una lettura fallita (un
//! errore di accesso non è un file mancante). Qui non ci sono appunti né contenuti: solo, per
//! ogni app, quante volte e quando è stata avviata.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::dockstore::{strip_bom, Read};
use crate::launcherrank::Usage;

/// Quante app ricordare al massimo: oltre, si scartano le meno recenti.
pub const MAX_ENTRIES: usize = 500;

#[derive(Serialize, Deserialize, Default)]
struct File {
    version: u32,
    #[serde(default)]
    usage: HashMap<String, Entry>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Entry {
    count: u32,
    last: u64,
}

type UsageMap = HashMap<String, Usage>;

fn store_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\launcher.json"))
}

/// L'uso dal testo del file (con o senza BOM), o `None` se non è JSON valido. Pura, testata.
pub fn parse(text: &str) -> Option<UsageMap> {
    let f: File = serde_json::from_str(strip_bom(text)).ok()?;
    Some(f.usage.into_iter().map(|(k, e)| (k, Usage { count: e.count, last: e.last })).collect())
}

fn serialize(usage: &UsageMap) -> String {
    let f = File {
        version: 1,
        usage: usage.iter().map(|(k, u)| (k.clone(), Entry { count: u.count, last: u.last })).collect(),
    };
    serde_json::to_string_pretty(&f).unwrap_or_default()
}

/// Registra un avvio di `id` all'istante `now`. Pura, testata.
pub fn record(map: &mut UsageMap, id: &str, now: u64) {
    let u = map.entry(id.to_string()).or_default();
    u.count = u.count.saturating_add(1);
    u.last = now;
}

/// Tiene al massimo `max` voci, scartando le meno recenti (a parità, quelle usate meno).
/// Restituisce se ha tolto qualcosa. Pura, testata.
pub fn prune(map: &mut UsageMap, max: usize) -> bool {
    if map.len() <= max {
        return false;
    }
    let mut keys: Vec<(&String, Usage)> = map.iter().map(|(k, u)| (k, *u)).collect();
    keys.sort_by(|a, b| b.1.last.cmp(&a.1.last).then(b.1.count.cmp(&a.1.count)).then_with(|| a.0.cmp(b.0)));
    let drop: Vec<String> = keys[max..].iter().map(|(k, _)| (*k).clone()).collect();
    for k in drop {
        map.remove(&k);
    }
    true
}

/// Cosa fare dopo aver letto il file.
#[derive(Debug, PartialEq)]
pub enum Decision {
    /// Il file è valido.
    Use(UsageMap),
    /// Il file manca: si riparte da vuoto.
    Fresh,
    /// Il file esiste ma non è JSON valido: si conserva come `.bad` e si riparte da vuoto.
    BackupAndFresh,
    /// Il file non si legge ora: NON si tocca e si usa l'ultimo stato buono.
    KeepLast(UsageMap),
}

impl Decision {
    /// I dati sono quelli del file (o il file va ricreato)? `false` solo per `KeepLast`.
    pub fn is_reliable(&self) -> bool {
        !matches!(self, Decision::KeepLast(_))
    }
}

/// Decide come usare l'esito della lettura. Pura, testata.
pub fn decide(read: Read, last_good: Option<&UsageMap>) -> Decision {
    match read {
        Read::Text(t) => parse(&t).map_or(Decision::BackupAndFresh, Decision::Use),
        Read::Missing => Decision::Fresh,
        Read::Failed => Decision::KeepLast(last_good.cloned().unwrap_or_default()),
    }
}

static STORE_LOCK: Mutex<()> = Mutex::new(());
static LAST: Mutex<Option<UsageMap>> = Mutex::new(None);

fn lock_store() -> MutexGuard<'static, ()> {
    STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn last() -> Option<UsageMap> {
    LAST.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn remember(map: &UsageMap) {
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(map.clone());
}

/// Legge il file riprovando brevemente (antivirus, rinomina in corso).
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

fn write_file(map: &UsageMap) {
    let Some(path) = store_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, serialize(map)).is_ok() {
        if std::fs::rename(&tmp, &path).is_ok() {
            remember(map);
        } else {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}

/// Uso corrente e se è affidabile (chi modifica non deve riscrivere dati non affidabili).
/// `locked`: chi chiama tiene già `STORE_LOCK`.
fn load_full(locked: bool) -> (UsageMap, bool) {
    let prev = last();
    let decision = decide(read_store(), prev.as_ref());
    let reliable = decision.is_reliable();
    match decision {
        Decision::Use(m) => {
            remember(&m);
            (m, reliable)
        }
        Decision::KeepLast(m) => (m, reliable),
        Decision::Fresh | Decision::BackupAndFresh => {
            let _g = (!locked).then(lock_store);
            // Un altro thread può aver creato il file mentre si aspettava il lock.
            let again = decide(read_store(), prev.as_ref());
            let reliable = again.is_reliable();
            match again {
                Decision::Use(m) => {
                    remember(&m);
                    (m, reliable)
                }
                Decision::KeepLast(m) => (m, reliable),
                other => {
                    if other == Decision::BackupAndFresh
                        && let Some(path) = store_path()
                    {
                        let _ = std::fs::copy(&path, path.with_extension("json.bad"));
                    }
                    let empty = UsageMap::new();
                    write_file(&empty);
                    (empty, reliable)
                }
            }
        }
    }
}

/// L'uso di tutte le app, per la classifica.
pub fn load() -> UsageMap {
    load_full(false).0
}

/// Registra l'avvio di `id` e salva. Se il file non si è potuto leggere non scrive nulla.
pub fn record_launch(id: &str, now: u64) {
    let _g = lock_store();
    let (mut map, reliable) = load_full(true);
    if !reliable {
        return;
    }
    record(&mut map, id, now);
    prune(&mut map, MAX_ENTRIES);
    write_file(&map);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(items: &[(&str, u32, u64)]) -> UsageMap {
        items.iter().map(|(k, c, l)| (k.to_string(), Usage { count: *c, last: *l })).collect()
    }

    #[test]
    fn parse_accepts_bom_and_rejects_garbage() {
        let json = r#"{"version":1,"usage":{"a":{"count":3,"last":10}}}"#;
        assert_eq!(parse(json).unwrap()["a"], Usage { count: 3, last: 10 });
        assert!(parse(&format!("\u{feff}{json}")).is_some(), "BOM tollerato");
        assert!(parse("non json").is_none());
        assert!(parse("").is_none());
        assert!(parse(r#"{"version":1}"#).unwrap().is_empty(), "senza uso: vuoto");
        assert!(parse(r#"{"version":1,"usage":{"a":{"count":-1,"last":1}}}"#).is_none(), "conteggio negativo");
    }

    #[test]
    fn serialize_roundtrips() {
        let m = map(&[("x", 2, 5), ("y", 9, 7)]);
        assert_eq!(parse(&serialize(&m)).unwrap(), m);
    }

    #[test]
    fn record_counts_and_stamps() {
        let mut m = UsageMap::new();
        record(&mut m, "a", 100);
        record(&mut m, "a", 200);
        record(&mut m, "b", 150);
        assert_eq!(m["a"], Usage { count: 2, last: 200 });
        assert_eq!(m["b"], Usage { count: 1, last: 150 });
        let mut full = map(&[("z", u32::MAX, 1)]);
        record(&mut full, "z", 2);
        assert_eq!(full["z"].count, u32::MAX, "nessun overflow");
    }

    #[test]
    fn prune_drops_least_recent() {
        let mut m = map(&[("old", 50, 1), ("mid", 1, 5), ("new", 1, 9), ("same_a", 3, 5)]);
        assert!(!prune(&mut m.clone(), 4));
        assert!(prune(&mut m, 2));
        assert_eq!(m.len(), 2);
        assert!(m.contains_key("new"));
        assert!(m.contains_key("same_a"), "a pari recenza resta quello usato di piu'");
        assert!(!m.contains_key("old"), "il vecchio va via anche se molto usato");
    }

    #[test]
    fn decide_never_treats_a_failed_read_as_missing() {
        let last = map(&[("a", 1, 1)]);
        assert_eq!(decide(Read::Failed, Some(&last)), Decision::KeepLast(last.clone()));
        assert_eq!(decide(Read::Failed, None), Decision::KeepLast(UsageMap::new()));
        assert!(!decide(Read::Failed, None).is_reliable());
        assert_eq!(decide(Read::Missing, Some(&last)), Decision::Fresh);
        assert_eq!(decide(Read::Text("rotto".into()), Some(&last)), Decision::BackupAndFresh);
        let ok = decide(Read::Text(r#"{"version":1,"usage":{"a":{"count":1,"last":2}}}"#.into()), None);
        assert_eq!(ok, Decision::Use(map(&[("a", 1, 2)])));
        assert!(ok.is_reliable() && Decision::Fresh.is_reliable() && Decision::BackupAndFresh.is_reliable());
    }
}
