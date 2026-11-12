//! Elenco delle app fissate nel Dock, salvato in `%LOCALAPPDATA%\EosOS\dock.json`.
//!
//! Al primo avvio l'elenco parte dalle app fissate sulla taskbar di Windows
//! (`dock::read_pinned`) e da lì in poi è indipendente: fissare o togliere
//! un'app dal Dock non tocca la taskbar.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::dock::{same_app, Pinned};

#[derive(Serialize, Deserialize, Default)]
struct File {
    version: u32,
    pinned: Vec<Entry>,
    /// Il Dock scompare e ricompare toccando il bordo inferiore dello schermo.
    #[serde(default)]
    autohide: bool,
    /// Opacità del vetro, da 0 (trasparente) a 1 (opaco). Come il cursore
    /// "Liquid Glass" di macOS 27, ma a tre livelli.
    #[serde(default = "default_glass")]
    glass: f64,
    /// Sfocatura dello sfondo (vedi `backdrop.rs`): acquisisce lo schermo dietro il
    /// Dock ed esclude il Dock da screenshot e registrazioni.
    #[serde(default = "default_blur")]
    blur: bool,
    /// Le finestre massimizzate si fermano sopra il Dock (appbar in basso). Non ha
    /// effetto con l'auto-nascondi.
    #[serde(default = "default_reserve")]
    reserve: bool,
}

fn default_reserve() -> bool {
    true
}

fn default_blur() -> bool {
    true
}

fn default_glass() -> f64 {
    GLASS_STANDARD
}

/// Livelli di vetro del menu: (id, etichetta, valore).
pub const GLASS_LEVELS: [(&str, &str, f64); 3] = [
    ("clear", "Vetro trasparente", 0.15),
    ("standard", "Vetro standard", GLASS_STANDARD),
    ("solid", "Vetro opaco", 1.0),
];
const GLASS_STANDARD: f64 = 0.7;

/// Impostazioni del Dock, come le legge il frontend.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Settings {
    pub autohide: bool,
    pub glass: f64,
    pub blur: bool,
    pub reserve: bool,
}

/// Le dimensioni del Dock: (id, etichetta, fattore). La media è quella di base.
pub const SIZES: &[(&str, &str, f64)] = &[("tiny", "Minuscolo", 0.65), ("small", "Piccolo", 0.82), ("medium", "Medio", 1.0), ("large", "Grande", 1.2), ("xl", "Molto grande", 1.4), ("xxl", "Gigante", 1.6)];

/// Le animazioni di un'app che si sta aprendo: (id, etichetta). Il rimbalzo è quello di base.
pub const ANIMS: &[(&str, &str)] = &[("bounce", "Rimbalzo"), ("pulse", "Pulsazione"), ("none", "Nessuna")];

/// L'animazione da un id salvato (vuoto o sconosciuto = rimbalzo). Pura, testata.
pub fn anim_id(saved: &str) -> &'static str {
    ANIMS.iter().find(|a| a.0 == saved).map_or("bounce", |a| a.0)
}

/// Il fattore di una dimensione dall'id (sconosciuto o vuoto = medio). Pura, testata.
pub fn size_factor(id: &str) -> f64 {
    SIZES.iter().find(|(i, _, _)| *i == id).map_or(1.0, |s| s.2)
}

/// Impostazioni del Dock per la pagina: quelle del file più la dimensione (che sta in `mission.json`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Ui {
    #[serde(flatten)]
    pub base: Settings,
    pub size: f64,
    /// Animazione di un'app che si apre («bounce», «pulse», «none»).
    pub anim: &'static str,
    /// L'ingrandimento delle icone al passaggio del mouse è acceso.
    pub magnify: bool,
    /// Icone nello stile di macOS (tessera arrotondata); falso = icone originali di Windows.
    pub icons: bool,
}

/// Le impostazioni complete per la pagina del Dock.
pub fn ui() -> Ui {
    let mc = crate::mcstore::load();
    Ui { base: Settings { blur: settings().blur && !crate::remote::blur_suspended(), ..settings() }, size: size_factor(&mc.dock_size), anim: anim_id(&mc.dock_anim), magnify: !mc.dock_magnify_off, icons: !mc.dock_icons_off }
}

/// Valore di un livello di vetro dall'id. Pura, testata.
pub fn glass_value(id: &str) -> Option<f64> {
    GLASS_LEVELS.iter().find(|(i, _, _)| *i == id).map(|(_, _, v)| *v)
}

/// Il livello più vicino a un valore salvato (per spuntare la voce giusta nel
/// menu anche se il file è stato modificato a mano). Pura, testata.
pub fn nearest_glass_id(v: f64) -> &'static str {
    GLASS_LEVELS
        .iter()
        .min_by(|a, b| (a.2 - v).abs().total_cmp(&(b.2 - v).abs()))
        .map_or("standard", |l| l.0)
}

/// Porta un valore letto dal file dentro 0..=1 (NaN compreso). Pura, testata.
pub fn clamp_glass(v: f64) -> f64 {
    if v.is_finite() { v.clamp(0.0, 1.0) } else { GLASS_STANDARD }
}

#[derive(Serialize, Deserialize)]
struct Entry {
    name: String,
    path: String,
}

fn store_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\dock.json"))
}

/// Toglie l'eventuale BOM UTF-8 iniziale: il Blocco note e PowerShell 5 lo
/// aggiungono quando si salva a mano. Pura, testata.
pub fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// Elenco e impostazioni dal testo del file (con o senza BOM), o `None` se non è JSON valido.
fn parse_full(text: &str) -> Option<(Vec<Pinned>, Settings)> {
    let f: File = serde_json::from_str(strip_bom(text)).ok()?;
    let st = settings_of(&f);
    Some((f.pinned.into_iter().map(|e| Pinned { name: e.name, path: PathBuf::from(e.path) }).collect(), st))
}

#[cfg(test)]
fn parse(text: &str) -> Option<Vec<Pinned>> {
    parse_full(text).map(|(list, _)| list)
}

fn serialize(pinned: &[Pinned], st: Settings) -> String {
    let f = File {
        version: 1,
        autohide: st.autohide,
        glass: st.glass,
        blur: st.blur,
        reserve: st.reserve,
        pinned: pinned
            .iter()
            .map(|p| Entry { name: p.name.clone(), path: p.path.to_string_lossy().to_string() })
            .collect(),
    };
    serde_json::to_string_pretty(&f).unwrap_or_default()
}

/// Aggiunge `p` se la stessa app non c'è già. Pura, testata.
pub fn add(list: &mut Vec<Pinned>, p: Pinned) -> bool {
    if list.iter().any(|x| same_app(&x.path, &p.path)) {
        return false;
    }
    list.push(p);
    true
}

/// Toglie le voci che corrispondono a `path`. Pura, testata.
pub fn remove(list: &mut Vec<Pinned>, path: &Path) -> bool {
    let before = list.len();
    list.retain(|x| !same_app(&x.path, path));
    list.len() != before
}

/// Toglie i doppioni (più collegamenti verso lo stesso programma), tenendo il
/// primo. Pura, testata.
pub fn dedup(list: Vec<Pinned>) -> Vec<Pinned> {
    let mut out = Vec::new();
    for p in list {
        add(&mut out, p);
    }
    out
}

/// Un solo scrittore alla volta: ogni modifica legge il file, cambia e lo riscrive, e due
/// modifiche simultanee (menu, trascinamento, ciclo di controllo) si perderebbero a vicenda.
static STORE_LOCK: Mutex<()> = Mutex::new(());

/// Ultimo elenco e ultime impostazioni letti o scritti con successo. Separati: `settings()` e
/// `load()` leggono il file in momenti diversi e ognuno aggiorna solo la propria parte.
static LAST_LIST: Mutex<Option<Vec<Pinned>>> = Mutex::new(None);
static LAST_SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);

fn lock_store() -> MutexGuard<'static, ()> {
    STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn last_list() -> Option<Vec<Pinned>> {
    LAST_LIST.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn last_settings() -> Option<Settings> {
    *LAST_SETTINGS.lock().unwrap_or_else(|e| e.into_inner())
}

fn remember(list: &[Pinned], st: Settings) {
    *LAST_LIST.lock().unwrap_or_else(|e| e.into_inner()) = Some(list.to_vec());
    remember_settings(st);
}

fn remember_settings(st: Settings) {
    *LAST_SETTINGS.lock().unwrap_or_else(|e| e.into_inner()) = Some(st);
}

/// Esito della lettura del file.
#[derive(Debug, PartialEq)]
pub enum Read {
    Text(String),
    /// Il file non esiste (primo avvio).
    Missing,
    /// Il file non si è potuto leggere (accesso negato, condivisione violata...).
    Failed,
}

/// Legge il file, riprovando brevemente se è inaccessibile: mentre un altro thread lo
/// sostituisce (rinomina) o un antivirus lo scandisce, la lettura può fallire per un attimo.
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

/// Cosa fare dopo aver letto il file.
#[derive(Debug, PartialEq)]
pub enum Decision {
    /// Il file è valido: si usano questo elenco e queste impostazioni.
    Use(Vec<Pinned>, Settings),
    /// Il file manca: si riparte dalle app fissate sulla taskbar.
    Seed,
    /// Il file esiste ma non è JSON valido: si conserva come `dock.json.bad` e si riparte.
    BackupAndSeed,
    /// Il file non si legge in questo momento: NON si tocca e si usa l'ultimo elenco buono.
    KeepLast(Vec<Pinned>),
}

impl Decision {
    /// I dati sono quelli del file (o il file va ricreato)? `false` solo per `KeepLast`: una
    /// lettura fallita. Su dati non affidabili non si deve scrivere nulla.
    pub fn is_reliable(&self) -> bool {
        !matches!(self, Decision::KeepLast(_))
    }
}

/// Decide come usare il risultato della lettura. Un errore di accesso non è un file
/// mancante: trattarlo così riscriverebbe `dock.json` con l'elenco della taskbar e
/// butterebbe via le app fissate e il loro ordine. Pura, testata.
pub fn decide(read: Read, last_good: Option<&[Pinned]>) -> Decision {
    match read {
        Read::Text(t) => match parse_full(&t) {
            Some((list, st)) => Decision::Use(list, st),
            None => Decision::BackupAndSeed,
        },
        Read::Missing => Decision::Seed,
        Read::Failed => Decision::KeepLast(last_good.map(<[Pinned]>::to_vec).unwrap_or_default()),
    }
}

fn default_settings() -> Settings {
    Settings { autohide: false, glass: GLASS_STANDARD, blur: true, reserve: true }
}

fn settings_of(f: &File) -> Settings {
    Settings { autohide: f.autohide, glass: clamp_glass(f.glass), blur: f.blur, reserve: f.reserve }
}

/// Impostazioni salvate. Se il file manca, quelle di default; se non si legge, quelle
/// dell'ultima lettura riuscita. Ogni lettura riuscita aggiorna la memoria.
pub fn settings() -> Settings {
    match read_store() {
        Read::Text(t) => match parse_full(&t) {
            Some((_, st)) => {
                remember_settings(st);
                st
            }
            None => default_settings(),
        },
        Read::Missing => default_settings(),
        Read::Failed => last_settings().unwrap_or_else(default_settings),
    }
}

/// Scrive il file in modo atomico. Chi chiama tiene `STORE_LOCK`. Il file temporaneo ha un
/// nome proprio del processo: due istanze di EosOS non si pestano i piedi.
fn write_file(pinned: &[Pinned], st: Settings) {
    let Some(path) = store_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, serialize(pinned, st)).is_ok() {
        if std::fs::rename(&tmp, &path).is_ok() {
            remember(pinned, st);
            crate::dock::invalidate();
        } else {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}

/// App fissate. Se il file manca si riparte dalla taskbar. Se esiste ma non è JSON
/// valido lo si conserva come `dock.json.bad` prima di ripartire. Se non si riesce a
/// leggerlo si usa l'ultimo elenco buono senza toccare nulla (vedi `decide`).
pub fn load() -> Vec<Pinned> {
    load_full(false).list
}

/// Elenco e impostazioni letti dal file, con l'indicazione se sono affidabili.
struct Loaded {
    list: Vec<Pinned>,
    settings: Settings,
    /// `false` se il file non si è potuto leggere: i dati sono l'ultimo stato noto e chi li
    /// modifica NON deve riscriverli sul file.
    reliable: bool,
}

/// `locked`: chi chiama tiene già `STORE_LOCK` (le modifiche, che leggono e riscrivono).
fn load_full(locked: bool) -> Loaded {
    let list = last_list();
    let decision = decide(read_store(), list.as_deref());
    let reliable = decision.is_reliable();
    match decision {
        Decision::Use(items, st) => {
            let clean = dedup(items.clone());
            if clean.len() != items.len() {
                // Ripulisce i file scritti prima che i doppioni fossero esclusi.
                let _g = (!locked).then(lock_store);
                write_file(&clean, st);
            } else {
                remember(&clean, st);
            }
            Loaded { list: clean, settings: st, reliable }
        }
        Decision::KeepLast(items) => {
            Loaded { list: items, settings: last_settings().unwrap_or_else(default_settings), reliable }
        }
        Decision::Seed | Decision::BackupAndSeed => {
            let _g = (!locked).then(lock_store);
            // Un altro thread può aver creato il file mentre si aspettava il lock.
            let again = decide(read_store(), list.as_deref());
            let reliable = again.is_reliable();
            match again {
                Decision::Use(items, st) => {
                    let clean = dedup(items);
                    remember(&clean, st);
                    Loaded { list: clean, settings: st, reliable }
                }
                Decision::KeepLast(items) => {
                    Loaded { list: items, settings: last_settings().unwrap_or_else(default_settings), reliable }
                }
                other => {
                    if other == Decision::BackupAndSeed
                        && let Some(path) = store_path()
                    {
                        let _ = std::fs::copy(&path, path.with_extension("json.bad"));
                    }
                    let seeded = dedup(crate::dock::read_pinned());
                    let st = last_settings().unwrap_or_else(default_settings);
                    write_file(&seeded, st);
                    Loaded { list: seeded, settings: st, reliable }
                }
            }
        }
    }
}

/// Legge, modifica e riscrive il file sotto il lock. `f` dice se qualcosa è cambiato. Se la
/// lettura è fallita non si scrive nulla: riscrivere l'ultimo stato noto (o un elenco vuoto, se
/// non ce n'è uno) cancellerebbe app fissate e impostazioni. L'azione dell'utente va persa,
/// ma è un caso raro (file bloccato per qualche istante) e meglio di perdere tutto.
fn modify(f: impl FnOnce(&mut Vec<Pinned>, &mut Settings) -> bool) {
    let _g = lock_store();
    let loaded = load_full(true);
    if !loaded.reliable {
        return;
    }
    let (mut list, mut st) = (loaded.list, loaded.settings);
    if f(&mut list, &mut st) {
        write_file(&list, st);
    }
}

pub fn set_autohide(on: bool) {
    modify(|_, st| std::mem::replace(&mut st.autohide, on) != on);
}

pub fn set_reserve(on: bool) {
    modify(|_, st| std::mem::replace(&mut st.reserve, on) != on);
}

pub fn set_blur(on: bool) {
    modify(|_, st| std::mem::replace(&mut st.blur, on) != on);
}

pub fn set_glass(v: f64) {
    let v = clamp_glass(v);
    modify(|_, st| std::mem::replace(&mut st.glass, v) != v);
}

/// Mette `path` nella posizione `to` tra le fissate (contata senza di essa). Se
/// non era fissata la inserisce usando `fallback`; senza `fallback` non fa nulla.
/// Restituisce se l'elenco è cambiato. Pura, testata.
pub fn place(list: &mut Vec<Pinned>, path: &Path, fallback: Option<Pinned>, to: usize) -> bool {
    let before = list.clone();
    let entry = match list.iter().position(|x| same_app(&x.path, path)) {
        Some(i) => list.remove(i),
        None => match fallback {
            Some(p) => p,
            None => return false,
        },
    };
    let to = to.min(list.len());
    list.insert(to, entry);
    *list != before
}

/// Sposta o fissa in `to` la voce con questo percorso, e salva.
pub fn move_to(path: &Path, fallback: Option<Pinned>, to: usize) {
    modify(|list, _| place(list, path, fallback, to));
}

pub fn pin(p: Pinned) {
    modify(|list, _| add(list, p));
}

pub fn unpin(path: &Path) {
    modify(|list, _| remove(list, path));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(name: &str, path: &str) -> Pinned {
        Pinned { name: name.into(), path: PathBuf::from(path) }
    }

    #[test]
    fn dock_sizes_are_known_and_default_to_medium() {
        assert_eq!(size_factor(""), 1.0);
        assert_eq!(size_factor("medium"), 1.0);
        assert_eq!(size_factor("nonesiste"), 1.0);
        assert!(size_factor("small") < 1.0 && size_factor("large") > 1.0 && size_factor("xl") > size_factor("large"));
        let ids: Vec<_> = SIZES.iter().map(|s| s.0).collect();
        assert_eq!(ids.len(), 6);
        assert!(SIZES.windows(2).all(|w| w[0].2 < w[1].2), "in ordine crescente");
    }

    #[test]
    fn the_app_animation_defaults_to_the_bounce() {
        assert_eq!(anim_id(""), "bounce");
        assert_eq!(anim_id("boh"), "bounce");
        assert_eq!(anim_id("none"), "none");
        assert_eq!(anim_id("pulse"), "pulse");
        assert_eq!(ANIMS.len(), 3);
    }

    #[test]
    fn roundtrip_through_json() {
        let list = vec![pin("A", r"C:\x\a.exe"), pin("B", r"C:\y\b.exe")];
        let back = parse(&serialize(&list, Settings { autohide: true, glass: 0.4, blur: false, reserve: true })).expect("json valido");
        assert_eq!(back, list);
    }

    #[test]
    fn autohide_defaults_to_false_for_old_files() {
        // File scritti prima dell'impostazione: nessun campo `autohide`.
        let f: File = serde_json::from_str(r#"{"version":1,"pinned":[]}"#).unwrap();
        assert!(!f.autohide);
        let f: File = serde_json::from_str(&serialize(&[], Settings { autohide: true, glass: 0.4, blur: false, reserve: true })).unwrap();
        assert!(f.autohide);
    }

    #[test]
    fn blur_is_on_by_default_and_roundtrips() {
        let f: File = serde_json::from_str(r#"{"version":1,"pinned":[]}"#).unwrap();
        assert!(f.blur, "file vecchi senza il campo: sfocatura attiva");
        let off: File = serde_json::from_str(&serialize(&[], Settings { autohide: false, glass: 0.7, blur: false, reserve: true })).unwrap();
        assert!(!off.blur);
    }

    #[test]
    fn reserve_is_on_by_default_and_roundtrips() {
        let f: File = serde_json::from_str(r#"{"version":1,"pinned":[]}"#).unwrap();
        assert!(f.reserve, "file vecchi senza il campo: spazio riservato");
        let st = Settings { autohide: false, glass: 0.7, blur: true, reserve: false };
        let off: File = serde_json::from_str(&serialize(&[], st)).unwrap();
        assert!(!off.reserve);
    }

    #[test]
    fn glass_defaults_and_levels() {
        // File vecchi senza il campo `glass`: vetro standard.
        let f: File = serde_json::from_str(r#"{"version":1,"pinned":[]}"#).unwrap();
        assert_eq!(f.glass, GLASS_STANDARD);
        assert_eq!(glass_value("clear"), Some(0.15));
        assert_eq!(glass_value("solid"), Some(1.0));
        assert_eq!(glass_value("boh"), None);
        // Valori scritti a mano: si spunta il livello più vicino e si limita il range.
        assert_eq!(nearest_glass_id(0.0), "clear");
        assert_eq!(nearest_glass_id(0.66), "standard");
        assert_eq!(nearest_glass_id(0.9), "solid");
        assert_eq!(clamp_glass(5.0), 1.0);
        assert_eq!(clamp_glass(-2.0), 0.0);
        assert_eq!(clamp_glass(f64::NAN), GLASS_STANDARD);
    }

    #[test]
    fn a_leading_bom_is_tolerated() {
        let text = format!("\u{feff}{}", serialize(&[pin("A", "C:/x/a.exe")], Settings { autohide: true, glass: 0.3, blur: true, reserve: true }));
        assert_eq!(parse(&text).expect("json con BOM").len(), 1);
        assert_eq!(strip_bom("\u{feff}abc"), "abc");
        assert_eq!(strip_bom("abc"), "abc");
    }

    #[test]
    fn a_read_error_never_reseeds_or_overwrites_the_file() {
        let last = [pin("A", "C:/x/a.exe"), pin("B", "C:/y/b.exe")];
        // Errore di accesso (rinomina in corso, antivirus): si tiene l'ultimo elenco buono.
        assert_eq!(decide(Read::Failed, Some(&last)), Decision::KeepLast(last.to_vec()));
        // Senza un ultimo elenco buono: vuoto, ma comunque nessuna riscrittura dalla taskbar.
        assert_eq!(decide(Read::Failed, None), Decision::KeepLast(vec![]));
        // File mancante: primo avvio, si riparte dalla taskbar.
        assert_eq!(decide(Read::Missing, Some(&last)), Decision::Seed);
        // File presente ma non valido: si conserva come .bad e si riparte.
        assert_eq!(decide(Read::Text("{ rotto".into()), Some(&last)), Decision::BackupAndSeed);
        // File valido, anche con il BOM.
        let st = Settings { autohide: true, glass: 0.15, blur: false, reserve: false };
        let good = serialize(&last, st);
        assert_eq!(decide(Read::Text(good.clone()), None), Decision::Use(last.to_vec(), st));
        assert_eq!(decide(Read::Text(format!("\u{feff}{good}")), None), Decision::Use(last.to_vec(), st));
    }

    #[test]
    fn only_a_failed_read_is_unreliable() {
        let st = Settings { autohide: false, glass: 0.7, blur: true, reserve: true };
        assert!(Decision::Use(vec![], st).is_reliable());
        assert!(Decision::Seed.is_reliable());
        assert!(Decision::BackupAndSeed.is_reliable());
        // Con una lettura fallita non si deve scrivere nemmeno se c'e' un ultimo elenco buono.
        assert!(!Decision::KeepLast(vec![pin("A", "C:/a.exe")]).is_reliable());
        assert!(!Decision::KeepLast(vec![]).is_reliable());
        assert!(!decide(Read::Failed, None).is_reliable());
        assert!(decide(Read::Missing, None).is_reliable());
    }

    #[test]
    fn the_settings_read_from_the_file_are_the_ones_remembered() {
        // Prima la memoria teneva quelle precedenti (di default), non quelle lette.
        let list = [pin("A", "C:/a.exe")];
        let st = Settings { autohide: true, glass: 0.15, blur: false, reserve: false };
        let (l, got) = parse_full(&serialize(&list, st)).expect("json valido");
        assert_eq!(l, list.to_vec());
        assert_eq!(got, st);
        assert!(parse_full("{ rotto").is_none());
    }

    #[test]
    fn corrupt_json_is_rejected() {
        assert!(parse("{ non json").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn add_ignores_duplicates_including_versioned_paths() {
        let mut l = vec![pin("Discord", r"C:\U\Discord\Discord.exe")];
        assert!(!add(&mut l, pin("Discord", r"C:\U\Discord\app-1.0\Discord.exe")));
        assert!(add(&mut l, pin("Altra", r"C:\U\altra.exe")));
        assert_eq!(l.len(), 2);
    }

    #[test]
    fn dedup_keeps_the_first_of_each_program() {
        let l = dedup(vec![
            pin("Chrome", r"C:\c\chrome.exe"),
            pin("Chrome (2)", r"C:\c\chrome.exe"),
            pin("Altro", r"C:/altra/altro.exe"),
            pin("Chrome (3)", r"c:\C\CHROME.exe"),
        ]);
        assert_eq!(l.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["Chrome", "Altro"]);
    }

    #[test]
    fn place_reorders_within_the_pinned_list() {
        let names = |l: &[Pinned]| l.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
        let mut l = vec![pin("A", "C:/a.exe"), pin("B", "C:/b.exe"), pin("C", "C:/c.exe")];
        assert!(place(&mut l, Path::new("C:/a.exe"), None, 2));
        assert_eq!(names(&l), ["B", "C", "A"]);
        assert!(place(&mut l, Path::new("c:/C.EXE"), None, 0));
        assert_eq!(names(&l), ["C", "B", "A"]);
        // Rilasciata dov'è già: nessun cambiamento (non si riscrive il file).
        assert!(!place(&mut l, Path::new("C:/b.exe"), None, 1));
        // Posizione oltre la fine: va in coda.
        assert!(place(&mut l, Path::new("C:/c.exe"), None, 99));
        assert_eq!(names(&l), ["B", "A", "C"]);
    }

    #[test]
    fn place_pins_an_unpinned_app_at_the_given_position() {
        let mut l = vec![pin("A", "C:/a.exe"), pin("B", "C:/b.exe")];
        assert!(place(&mut l, Path::new("C:/x.exe"), Some(pin("X", "C:/x.exe")), 1));
        assert_eq!(l.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["A", "X", "B"]);
        // Sconosciuta e senza ripiego: niente.
        assert!(!place(&mut l, Path::new("C:/zzz.exe"), None, 0));
        assert_eq!(l.len(), 3);
    }

    #[test]
    fn remove_drops_matching_entry_only() {
        let mut l = vec![pin("A", r"C:\x\a.exe"), pin("B", r"C:\y\b.exe")];
        assert!(remove(&mut l, Path::new(r"c:\X\A.exe")));
        assert_eq!(l.len(), 1);
        assert!(!remove(&mut l, Path::new(r"C:\x\a.exe")));
    }
}
