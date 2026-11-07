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
}

#[derive(Serialize, Deserialize)]
struct Entry {
    name: String,
    path: String,
}

fn store_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\dock.json"))
}

fn parse(text: &str) -> Option<Vec<Pinned>> {
    let f: File = serde_json::from_str(text).ok()?;
    Some(f.pinned.into_iter().map(|e| Pinned { name: e.name, path: PathBuf::from(e.path) }).collect())
}

fn serialize(pinned: &[Pinned]) -> String {
    let f = File {
        version: 1,
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
    let Some(path) = store_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    // Scrittura atomica: un crollo a metà non lascia un file troncato.
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, serialize(pinned)).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

/// App fissate. Se il file manca o è illeggibile si riparte dalla taskbar.
pub fn load() -> Vec<Pinned> {
    if let Some(text) = store_path().and_then(|p| std::fs::read_to_string(p).ok())
        && let Some(list) = parse(&text)
    {
        let clean = dedup(list.clone());
        if clean.len() != list.len() {
            save(&clean); // ripulisce i file scritti prima che i doppioni fossero esclusi
        }
        return clean;
    }
    let seeded = dedup(crate::dock::read_pinned());
    save(&seeded);
    seeded
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
        let back = parse(&serialize(&list)).expect("json valido");
        assert_eq!(back, list);
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
    fn remove_drops_matching_entry_only() {
        let mut l = vec![pin("A", r"C:\x\a.exe"), pin("B", r"C:\y\b.exe")];
        assert!(remove(&mut l, Path::new(r"c:\X\A.exe")));
        assert_eq!(l.len(), 1);
        assert!(!remove(&mut l, Path::new(r"C:\x\a.exe")));
    }
}
