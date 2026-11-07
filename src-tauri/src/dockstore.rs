//! Elenco delle app fissate nel Dock, salvato in `%LOCALAPPDATA%\EosOS\dock.json`.
//!
//! Al primo avvio l'elenco parte dalle app fissate sulla taskbar di Windows
//! (`dock::read_pinned`) e da lì in poi è indipendente: fissare o togliere
//! un'app dal Dock non tocca la taskbar.

use std::path::{Path, PathBuf};

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

fn parse(text: &str) -> Option<Vec<Pinned>> {
    let f: File = serde_json::from_str(strip_bom(text)).ok()?;
    Some(f.pinned.into_iter().map(|e| Pinned { name: e.name, path: PathBuf::from(e.path) }).collect())
}

fn serialize(pinned: &[Pinned], st: Settings) -> String {
    let f = File {
        version: 1,
        autohide: st.autohide,
        glass: st.glass,
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

fn save(pinned: &[Pinned]) {
    save_with(pinned, settings());
}

/// Impostazioni salvate (quelle di default se il file manca o è illeggibile).
pub fn settings() -> Settings {
    store_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<File>(strip_bom(&t)).ok())
        .map_or(Settings { autohide: false, glass: GLASS_STANDARD }, |f| Settings {
            autohide: f.autohide,
            glass: clamp_glass(f.glass),
        })
}

/// Impostazione "nascondi automaticamente".
pub fn autohide() -> bool {
    settings().autohide
}

pub fn set_autohide(on: bool) {
    save_with(&load(), Settings { autohide: on, ..settings() });
}

pub fn set_glass(v: f64) {
    save_with(&load(), Settings { glass: clamp_glass(v), ..settings() });
}

fn save_with(pinned: &[Pinned], st: Settings) {
    let Some(path) = store_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    // Scrittura atomica: un crollo a metà non lascia un file troncato.
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, serialize(pinned, st)).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

/// App fissate. Se il file manca si riparte dalla taskbar. Se esiste ma non si
/// legge (per esempio un errore di battitura dopo una modifica a mano) lo si
/// mette da parte come `dock.json.bad` prima di ripartire, invece di perderlo.
pub fn load() -> Vec<Pinned> {
    let text = store_path().and_then(|p| std::fs::read_to_string(p).ok());
    if let Some(text) = &text
        && let Some(list) = parse(text)
    {
        let clean = dedup(list.clone());
        if clean.len() != list.len() {
            save(&clean); // ripulisce i file scritti prima che i doppioni fossero esclusi
        }
        return clean;
    }
    if text.is_some()
        && let Some(path) = store_path()
    {
        let _ = std::fs::copy(&path, path.with_extension("json.bad"));
    }
    let seeded = dedup(crate::dock::read_pinned());
    save(&seeded);
    seeded
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
    let mut list = load();
    if place(&mut list, path, fallback, to) {
        save(&list);
    }
}

pub fn pin(p: Pinned) {
    let mut list = load();
    if add(&mut list, p) {
        save(&list);
    }
}

pub fn unpin(path: &Path) {
    let mut list = load();
    if remove(&mut list, path) {
        save(&list);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(name: &str, path: &str) -> Pinned {
        Pinned { name: name.into(), path: PathBuf::from(path) }
    }

    #[test]
    fn roundtrip_through_json() {
        let list = vec![pin("A", r"C:\x\a.exe"), pin("B", r"C:\y\b.exe")];
        let back = parse(&serialize(&list, Settings { autohide: true, glass: 0.4 })).expect("json valido");
        assert_eq!(back, list);
    }

    #[test]
    fn autohide_defaults_to_false_for_old_files() {
        // File scritti prima dell'impostazione: nessun campo `autohide`.
        let f: File = serde_json::from_str(r#"{"version":1,"pinned":[]}"#).unwrap();
        assert!(!f.autohide);
        let f: File = serde_json::from_str(&serialize(&[], Settings { autohide: true, glass: 0.4 })).unwrap();
        assert!(f.autohide);
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
        let text = format!("\u{feff}{}", serialize(&[pin("A", "C:/x/a.exe")], Settings { autohide: true, glass: 0.3 }));
        assert_eq!(parse(&text).expect("json con BOM").len(), 1);
        assert_eq!(strip_bom("\u{feff}abc"), "abc");
        assert_eq!(strip_bom("abc"), "abc");
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
