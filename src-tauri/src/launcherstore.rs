//! Dati del Launcher, salvati in `%LOCALAPPDATA%\EosOS\launcher.json`: l'uso delle app (per la
//! "frecency") e le impostazioni (per ora il consenso alla cronologia degli appunti).
//!
//! Stessa robustezza di `dockstore`: tolleranza del BOM, file illeggibile conservato come `.bad`,
//! scrittura atomica sotto lock, e MAI una riscrittura dopo una lettura fallita (un errore di
//! accesso non è un file mancante). Qui non ci sono contenuti: per ogni app solo quante volte e
//! quando è stata avviata; **il testo degli appunti non finisce mai su disco**.

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
    /// Consenso alla cronologia degli appunti: assente = mai chiesto.
    #[serde(default)]
    clipboard: Option<bool>,
    /// Scorciatoia globale (per esempio "Alt+Space"); assente = predefinita.
    #[serde(default)]
    hotkey: Option<String>,
    /// Cartelle in più dove cercare i file (oltre a quelle personali).
    #[serde(default, rename = "searchFolders")]
    search_folders: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Entry {
    count: u32,
    last: u64,
}

type UsageMap = HashMap<String, Usage>;

/// Impostazioni del Launcher.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Settings {
    /// `None`: non ancora chiesto. `Some(true)`/`Some(false)`: scelta dell'utente.
    pub clipboard: Option<bool>,
    /// Scorciatoia scelta; `None` = predefinita (Alt+Spazio).
    pub hotkey: Option<String>,
    /// Cartelle di ricerca dei file in più.
    pub search_folders: Vec<String>,
}

/// Tutto ciò che sta nel file.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct State {
    pub usage: UsageMap,
    pub settings: Settings,
}

fn store_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\launcher.json"))
}

/// Lo stato dal testo del file (con o senza BOM), o `None` se non è JSON valido. Pura, testata.
pub fn parse(text: &str) -> Option<State> {
    let f: File = serde_json::from_str(strip_bom(text)).ok()?;
    Some(State {
        usage: f.usage.into_iter().map(|(k, e)| (k, Usage { count: e.count, last: e.last })).collect(),
        settings: Settings { clipboard: f.clipboard, hotkey: f.hotkey, search_folders: f.search_folders },
    })
}

fn serialize(st: &State) -> String {
    let f = File {
        version: 1,
        usage: st.usage.iter().map(|(k, u)| (k.clone(), Entry { count: u.count, last: u.last })).collect(),
        clipboard: st.settings.clipboard,
        hotkey: st.settings.hotkey.clone(),
        search_folders: st.settings.search_folders.clone(),
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
    Use(State),
    /// Il file manca: si riparte da vuoto.
    Fresh,
    /// Il file esiste ma non è JSON valido: si conserva come `.bad` e si riparte da vuoto.
    BackupAndFresh,
    /// Il file non si legge ora: NON si tocca e si usa l'ultimo stato buono.
    KeepLast(State),
}

impl Decision {
    /// I dati sono quelli del file (o il file va ricreato)? `false` solo per `KeepLast`.
    pub fn is_reliable(&self) -> bool {
        !matches!(self, Decision::KeepLast(_))
    }
}

/// Decide come usare l'esito della lettura. Pura, testata.
pub fn decide(read: Read, last_good: Option<&State>) -> Decision {
    match read {
        Read::Text(t) => parse(&t).map_or(Decision::BackupAndFresh, Decision::Use),
        Read::Missing => Decision::Fresh,
        Read::Failed => Decision::KeepLast(last_good.cloned().unwrap_or_default()),
    }
}

static STORE_LOCK: Mutex<()> = Mutex::new(());
static LAST: Mutex<Option<State>> = Mutex::new(None);

fn lock_store() -> MutexGuard<'static, ()> {
    STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn last() -> Option<State> {
    LAST.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn remember(st: &State) {
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(st.clone());
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

fn write_file(st: &State) {
    let Some(path) = store_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, serialize(st)).is_ok() {
        if std::fs::rename(&tmp, &path).is_ok() {
            remember(st);
        } else {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}

/// Stato corrente e se è affidabile (chi modifica non deve riscrivere dati non affidabili).
/// `locked`: chi chiama tiene già `STORE_LOCK`.
fn load_full(locked: bool) -> (State, bool) {
    let prev = last();
    let decision = decide(read_store(), prev.as_ref());
    let reliable = decision.is_reliable();
    match decision {
        Decision::Use(st) => {
            remember(&st);
            (st, reliable)
        }
        Decision::KeepLast(st) => (st, reliable),
        Decision::Fresh | Decision::BackupAndFresh => {
            let _g = (!locked).then(lock_store);
            // Un altro thread può aver creato il file mentre si aspettava il lock.
            let again = decide(read_store(), prev.as_ref());
            let reliable = again.is_reliable();
            match again {
                Decision::Use(st) => {
                    remember(&st);
                    (st, reliable)
                }
                Decision::KeepLast(st) => (st, reliable),
                other => {
                    if other == Decision::BackupAndFresh
                        && let Some(path) = store_path()
                    {
                        let _ = std::fs::copy(&path, path.with_extension("json.bad"));
                    }
                    let empty = State::default();
                    write_file(&empty);
                    (empty, reliable)
                }
            }
        }
    }
}

/// L'uso di tutte le app, per la classifica.
pub fn load() -> UsageMap {
    load_full(false).0.usage
}

/// Le impostazioni correnti (se il file non si legge, quelle dell'ultima lettura buona).
pub fn settings() -> Settings {
    load_full(false).0.settings
}

/// Legge, modifica e riscrive sotto il lock. `f` dice se qualcosa è cambiato. Se la lettura è
/// fallita non si scrive nulla.
fn modify(f: impl FnOnce(&mut State) -> bool) {
    let _g = lock_store();
    let (mut st, reliable) = load_full(true);
    if !reliable {
        return;
    }
    if f(&mut st) {
        write_file(&st);
    }
}

/// Registra l'avvio di `id` e salva. Se il file non si è potuto leggere non scrive nulla.
pub fn record_launch(id: &str, now: u64) {
    modify(|st| {
        record(&mut st.usage, id, now);
        prune(&mut st.usage, MAX_ENTRIES);
        true
    });
}

/// Salva la scorciatoia scelta (`None` = torna alla predefinita).
pub fn set_hotkey(h: Option<String>) {
    modify(|st| {
        let changed = st.settings.hotkey != h;
        st.settings.hotkey = h;
        changed
    });
}

/// Salva le cartelle di ricerca dei file in più.
pub fn set_search_folders(list: Vec<String>) {
    modify(|st| {
        let changed = st.settings.search_folders != list;
        st.settings.search_folders = list;
        changed
    });
}

/// Salva la scelta dell'utente sulla cronologia degli appunti.
pub fn set_clipboard_consent(on: bool) {
    modify(|st| st.settings.clipboard.replace(on) != Some(on));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(items: &[(&str, u32, u64)]) -> UsageMap {
        items.iter().map(|(k, c, l)| (k.to_string(), Usage { count: *c, last: *l })).collect()
    }
    fn state(items: &[(&str, u32, u64)], clip: Option<bool>) -> State {
        State { usage: map(items), settings: Settings { clipboard: clip, ..Default::default() } }
    }

    #[test]
    fn parse_accepts_bom_and_rejects_garbage() {
        let json = r#"{"version":1,"usage":{"a":{"count":3,"last":10}}}"#;
        assert_eq!(parse(json).unwrap().usage["a"], Usage { count: 3, last: 10 });
        assert!(parse(&format!("\u{feff}{json}")).is_some(), "BOM tollerato");
        assert!(parse("non json").is_none());
        assert!(parse("").is_none());
        assert!(parse(r#"{"version":1}"#).unwrap().usage.is_empty(), "senza uso: vuoto");
        assert!(parse(r#"{"version":1,"usage":{"a":{"count":-1,"last":1}}}"#).is_none(), "conteggio negativo");
    }

    #[test]
    fn clipboard_consent_defaults_to_unasked_and_roundtrips() {
        assert_eq!(parse(r#"{"version":1}"#).unwrap().settings.clipboard, None, "file vecchio: mai chiesto");
        for c in [None, Some(true), Some(false)] {
            let st = state(&[("x", 2, 5)], c);
            assert_eq!(parse(&serialize(&st)).unwrap(), st);
        }
        assert_eq!(parse(r#"{"version":1,"clipboard":false}"#).unwrap().settings.clipboard, Some(false));
    }

    #[test]
    fn serialized_file_never_contains_clipboard_text_fields() {
        // Il file ha solo uso e consenso: nessun campo che possa contenere testo copiato.
        let v: serde_json::Value = serde_json::from_str(&serialize(&state(&[("a", 1, 1)], Some(true)))).unwrap();
        let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        // Solo uso e impostazioni (mai testo copiato): i nomi dei campi sono questi e nessun altro.
        assert_eq!(keys, ["clipboard", "hotkey", "searchFolders", "usage", "version"]);
    }

    #[test]
    fn hotkey_and_search_folders_roundtrip_and_default() {
        let old = parse(r#"{"version":1,"usage":{}}"#).unwrap();
        assert_eq!(old.settings.hotkey, None);
        assert!(old.settings.search_folders.is_empty());
        let json = r#"{"version":1,"hotkey":"Ctrl+Alt+Space","searchFolders":["D:\\Progetti","E:\\Dati"]}"#;
        let st = parse(json).unwrap();
        assert_eq!(st.settings.hotkey.as_deref(), Some("Ctrl+Alt+Space"));
        assert_eq!(st.settings.search_folders, vec![r"D:\Progetti".to_string(), r"E:\Dati".to_string()]);
        assert_eq!(parse(&serialize(&st)).unwrap(), st);
    }

    #[test]
    fn serialize_roundtrips() {
        let st = state(&[("x", 2, 5), ("y", 9, 7)], Some(true));
        assert_eq!(parse(&serialize(&st)).unwrap(), st);
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
        let last = state(&[("a", 1, 1)], Some(false));
        assert_eq!(decide(Read::Failed, Some(&last)), Decision::KeepLast(last.clone()));
        assert_eq!(decide(Read::Failed, None), Decision::KeepLast(State::default()));
        assert!(!decide(Read::Failed, None).is_reliable());
        assert_eq!(decide(Read::Missing, Some(&last)), Decision::Fresh);
        assert_eq!(decide(Read::Text("rotto".into()), Some(&last)), Decision::BackupAndFresh);
        let ok = decide(Read::Text(r#"{"version":1,"usage":{"a":{"count":1,"last":2}},"clipboard":true}"#.into()), None);
        assert_eq!(ok, Decision::Use(state(&[("a", 1, 2)], Some(true))));
        assert!(ok.is_reliable() && Decision::Fresh.is_reliable() && Decision::BackupAndFresh.is_reliable());
    }
}
