//! Pile nel Dock (fase P1): cartelle che stanno nel Dock a destra delle app aperte e, con un clic, si aprono in un
//! pannello sopra l'icona con i file più recenti in griglia. Qui: la parte pura (ordine, filtro dei file di servizio,
//! elenco delle pile, titolo) e la lettura della cartella. La pagina manda solo l'indice della pila o l'id di un file
//! che Rust le ha appena mostrato: i percorsi li decide Rust. I file che eseguono codice non si aprono direttamente
//! (si mostrano in Esplora risorse), come nel Launcher.
//!
//! Di default c'è una sola pila, Download. Le altre si aggiungono dalle Impostazioni (`dock.stacks`).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

/// File mostrati al massimo in una pila (i più recenti).
pub const MAX_ENTRIES: usize = 40;
/// Pile al massimo nel Dock.
pub const MAX_STACKS: usize = 5;

/// Un file o una cartella dentro una pila.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    /// Ultima modifica, in secondi dal 1970.
    pub mtime: i64,
    pub path: PathBuf,
}

/// File di servizio che non vanno mostrati. Pura, testata.
pub fn is_junk(name: &str) -> bool {
    let n = name.to_lowercase();
    n.starts_with('.') || n.starts_with("~$") || n == "desktop.ini" || n == "thumbs.db" || n.ends_with(".crdownload") || n.ends_with(".part")
}

/// Dal più recente al più vecchio; a parità di tempo, per nome senza badare alle maiuscole. Pura, testata.
pub fn sort_newest(v: &mut [Entry]) {
    v.sort_by(|a, b| b.mtime.cmp(&a.mtime).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
}

/// Il nome da mostrare per una cartella: l'ultimo pezzo del percorso (la radice di un disco resta com'è). Pura, testata.
pub fn title(path: &str) -> String {
    let p = path.trim().trim_end_matches(['\\', '/']);
    let last = p.rsplit(['\\', '/']).next().unwrap_or(p);
    if last.is_empty() { path.trim().to_string() } else { last.to_string() }
}

/// L'elenco delle pile da mostrare: quello scelto dall'utente, ripulito (solo cartelle valide e non di rete, senza
/// doppioni, al massimo `MAX_STACKS`), oppure `default` se non ha mai scelto. Pura, testata.
pub fn effective(chosen: &Option<Vec<String>>, default: Option<String>) -> Vec<String> {
    match chosen {
        None => default.into_iter().collect(),
        Some(list) => {
            let ok: Vec<String> = list.iter().filter(|p| crate::settingsreg::valid_folder(p) && !crate::launcherfiles::is_network_path(p)).cloned().collect();
            crate::settingsreg::clean_list(&ok, MAX_STACKS, crate::settingsreg::MAX_PATH_LEN)
        }
    }
}

/// Una pila nel Dock, come la vede la pagina.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct StackItem {
    pub index: usize,
    pub title: String,
    pub icon: Option<String>,
}

/// Un file nel pannello della pila.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct EntryOut {
    pub id: String,
    pub name: String,
    pub is_dir: bool,
    pub icon: Option<String>,
}

/// Contenuto del pannello di una pila.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct StackReply {
    pub title: String,
    /// "ok", "empty" (cartella vuota) o "missing" (non esiste o non si legge).
    pub state: &'static str,
    pub entries: Vec<EntryOut>,
    /// Quanti file ci sono in tutto (oltre a quelli mostrati).
    pub total: usize,
}

// ------------------------------------------------------------------ Windows

/// Gli ultimi file elencati: l'id che la pagina rimanda è l'indice qui dentro.
static KNOWN: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

fn id_of(i: usize) -> String {
    format!("stk:{i}")
}

fn index_of(id: &str) -> Option<usize> {
    id.strip_prefix("stk:")?.parse().ok()
}

/// Le cartelle delle pile adesso (impostazione dell'utente o Download di default).
pub fn folders() -> Vec<String> {
    effective(&crate::mcstore::load().dock_stacks, crate::launcherfiles::downloads_folder().map(|p| p.to_string_lossy().to_string()))
}

/// Le pile per il Dock, con l'icona della cartella.
pub fn items() -> Vec<StackItem> {
    folders()
        .into_iter()
        .enumerate()
        .map(|(index, f)| StackItem { index, title: title(&f), icon: crate::dock::cached_icon(Path::new(&f)) })
        .collect()
}

/// Legge una cartella: file visibili (non nascosti, non di sistema, non di servizio), con data di modifica.
fn read_entries(dir: &Path) -> Option<Vec<Entry>> {
    use std::os::windows::fs::MetadataExt;
    const HIDDEN_OR_SYSTEM: u32 = 0x2 | 0x4;
    let rd = std::fs::read_dir(dir).ok()?;
    let mut out = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let Ok(md) = e.metadata() else { continue };
        if md.file_attributes() & HIDDEN_OR_SYSTEM != 0 || is_junk(&name) {
            continue;
        }
        let mtime = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs() as i64);
        out.push(Entry { name, is_dir: md.is_dir(), mtime, path: e.path() });
    }
    Some(out)
}

/// Il contenuto della pila `index`: i file più recenti, con le icone, e ricorda i percorsi per `open`.
pub fn list(index: usize) -> StackReply {
    let Some(folder) = folders().into_iter().nth(index) else {
        return StackReply { title: String::new(), state: "missing", entries: Vec::new(), total: 0 };
    };
    let t = title(&folder);
    let Some(mut entries) = read_entries(Path::new(&folder)) else {
        *KNOWN.lock().unwrap_or_else(|e| e.into_inner()) = Vec::new();
        return StackReply { title: t, state: "missing", entries: Vec::new(), total: 0 };
    };
    sort_newest(&mut entries);
    let total = entries.len();
    entries.truncate(MAX_ENTRIES);
    let out: Vec<EntryOut> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| EntryOut { id: id_of(i), name: e.name.clone(), is_dir: e.is_dir, icon: crate::dock::cached_icon(&e.path) })
        .collect();
    *KNOWN.lock().unwrap_or_else(|e| e.into_inner()) = entries.into_iter().map(|e| e.path).collect();
    StackReply { title: t, state: if total == 0 { "empty" } else { "ok" }, entries: out, total }
}

fn shell_open(target: &str, args: Option<&str>) -> bool {
    use windows::core::{w, HSTRING};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let t = HSTRING::from(target);
    let r = match args {
        Some(a) => {
            let a = HSTRING::from(a);
            unsafe { ShellExecuteW(None, w!("open"), &t, &a, None, SW_SHOWNORMAL) }
        }
        None => unsafe { ShellExecuteW(None, w!("open"), &t, None, None, SW_SHOWNORMAL) },
    };
    r.0 as usize > 32
}

/// Apre un file della pila (solo se Rust lo ha appena mostrato): le cartelle si aprono in Esplora risorse; i file che
/// eseguono codice (exe, script, collegamenti...) si mostrano soltanto in Esplora risorse; gli altri si aprono col loro
/// programma. Nelle build di prova (`EOSOS_TEST_NO_OPEN`) registra la richiesta senza lanciare niente.
pub fn open(id: &str) -> bool {
    let Some(i) = index_of(id) else { return false };
    let Some(path) = KNOWN.lock().unwrap_or_else(|e| e.into_inner()).get(i).cloned() else { return false };
    if !path.exists() {
        return false;
    }
    let reveal = !path.is_dir() && crate::launcherfiles::is_risky_file(&path);
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("EOSOS_TEST_NO_OPEN").is_some() {
        *LAST_OPEN.lock().unwrap_or_else(|e| e.into_inner()) = Some((path.to_string_lossy().to_string(), reveal));
        return true;
    }
    if reveal {
        shell_open("explorer.exe", Some(&format!("/select,\"{}\"", path.to_string_lossy())))
    } else {
        shell_open(&path.to_string_lossy(), None)
    }
}

/// Apre la cartella della pila in Esplora risorse.
pub fn open_folder(index: usize) -> bool {
    let Some(folder) = folders().into_iter().nth(index) else { return false };
    if !Path::new(&folder).is_dir() {
        return false;
    }
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("EOSOS_TEST_NO_OPEN").is_some() {
        *LAST_OPEN.lock().unwrap_or_else(|e| e.into_inner()) = Some((folder, false));
        return true;
    }
    shell_open(&folder, None)
}

/// L'ultima richiesta di apertura (percorso, solo-mostra) registrata nelle build di prova.
#[cfg(feature = "test-hooks")]
static LAST_OPEN: Mutex<Option<(String, bool)>> = Mutex::new(None);

#[cfg(feature = "test-hooks")]
pub fn last_open() -> Option<(String, bool)> {
    LAST_OPEN.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(name: &str, mtime: i64) -> Entry {
        Entry { name: name.into(), is_dir: false, mtime, path: PathBuf::from(name) }
    }

    #[test]
    fn junk_files_are_recognised() {
        for j in ["desktop.ini", "Desktop.INI", "Thumbs.db", ".DS_Store", "~$relazione.docx", "film.mp4.crdownload", "x.part", ".hidden"] {
            assert!(is_junk(j), "{j}");
        }
        for ok in ["relazione.docx", "foto.jpg", "setup.exe", "Cartella", "a.partial.txt", "notes.txt"] {
            assert!(!is_junk(ok), "{ok}");
        }
    }

    #[test]
    fn newest_first_then_by_name() {
        let mut v = vec![e("b", 10), e("A", 30), e("c", 20), e("a", 30), e("Z", 10)];
        sort_newest(&mut v);
        assert_eq!(v.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["A", "a", "c", "b", "Z"]);
    }

    #[test]
    fn titles_come_from_the_last_component() {
        assert_eq!(title(r"C:\Users\x\Downloads"), "Downloads");
        assert_eq!(title(r"C:\Users\x\Downloads\"), "Downloads");
        assert_eq!(title("D:/Progetti/EosOS"), "EosOS");
        assert_eq!(title(r"D:\"), r"D:");
        assert_eq!(title("  C:\\a b  "), "a b");
    }

    #[test]
    fn the_stack_list_is_clean_and_defaults_to_downloads() {
        let dl = Some(r"C:\Users\x\Downloads".to_string());
        assert_eq!(effective(&None, dl.clone()), vec![r"C:\Users\x\Downloads".to_string()]);
        assert!(effective(&None, None).is_empty());
        // Scelta esplicita: anche la lista vuota (nessuna pila) vale.
        assert!(effective(&Some(vec![]), dl.clone()).is_empty());
        let chosen = Some(vec![r"D:\Dati".into(), "relativo".into(), r"\\server\share".into(), r"d:\dati".into(), r"C:\a\..\b".into(), r"E:\Foto".into()]);
        assert_eq!(effective(&chosen, dl), vec![r"D:\Dati".to_string(), r"E:\Foto".to_string()]);
    }

    #[test]
    fn at_most_five_stacks() {
        let many: Vec<String> = (0..9).map(|i| format!(r"D:\cartella{i}")).collect();
        assert_eq!(effective(&Some(many), None).len(), MAX_STACKS);
    }

    #[test]
    fn ids_round_trip_and_reject_garbage() {
        assert_eq!(index_of(&id_of(7)), Some(7));
        for bad in ["", "stk:", "stk:x", "7", "stk:-1", "../x"] {
            assert_eq!(index_of(bad), None, "{bad}");
        }
    }

    #[test]
    fn open_refuses_ids_it_never_showed() {
        // Un indice enorme non manda in panico e un percorso al posto dell'id non vale.
            assert!(!open("stk:999999"));
        assert!(!open("C:\\Windows\\notepad.exe"));
    }
}
