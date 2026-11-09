//! App in background (area di notifica): il «mini menu» di Windows con le icone nascoste, per EosOS.
//!
//! Con la barra di Windows nascosta non si vedono più le icone dell'area di notifica (NVIDIA Broadcast, G HUB, AnyDesk…). Windows 11 non dà un
//! modo affidabile di leggere le icone vive, ma tiene l'elenco delle app che ne hanno registrata una in
//! `HKCU\\Control Panel\\NotifyIconSettings`. Qui si incrocia quell'elenco con i **processi in esecuzione adesso** e si tolgono le app che hanno già
//! una finestra o un posto nel Dock: restano quelle che girano «ridotte» e non hanno altro modo di essere raggiunte. Un clic le riporta davanti
//! (se hanno una finestra) o le avvia di nuovo (le app a istanza unica mostrano la loro finestra).
//!
//! Il frontend vede solo un identificativo e il nome; il percorso da avviare lo ricava Rust dall'elenco del momento.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use windows::core::{HSTRING, PWSTR};
use windows::Win32::System::Registry::{RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE};

use crate::dock::{cached_icon, key, same_app};

const SETTINGS_PATH: &str = r"Control Panel\NotifyIconSettings";
const TTL: Duration = Duration::from_secs(3);

/// Un'app in background per la pagina.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TrayApp {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
}

/// Il percorso (come lo scrive la voce del registro) è un candidato? Non lo sono le componenti di Windows (cartella di Windows, Esplora risorse,
/// Gestione attività, avvisi di sistema), EosOS stessa e i file che non sono `.exe`. Pura, testata.
pub fn is_candidate(path: &Path, windows_dir: &str, own_exe: &str) -> bool {
    let p = path.to_string_lossy().to_lowercase();
    if p.is_empty() || (!windows_dir.is_empty() && p.starts_with(windows_dir)) || p == own_exe {
        return false;
    }
    let ext_ok = path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("exe"));
    ext_ok && !p.ends_with(r"\explorer.exe") && !p.ends_with(r"\taskmgr.exe")
}

/// Il nome da mostrare: la descrizione del file se c'è, altrimenti il nome dell'eseguibile con la prima lettera maiuscola. Pura, testata.
pub fn pretty_name(description: Option<&str>, stem: &str) -> String {
    if let Some(d) = description.map(str::trim).filter(|d| !d.is_empty()) {
        return d.to_string();
    }
    let mut c = stem.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => "App".to_string(),
    }
}

/// Mette in ordine alfabetico (senza badare alle maiuscole) e toglie i doppioni di nome (la stessa app con più eseguibili: si tiene il primo). Pura, testata.
pub fn tidy(mut apps: Vec<TrayApp>) -> Vec<TrayApp> {
    apps.sort_by_key(|a| a.name.to_lowercase());
    let mut seen = HashSet::new();
    apps.retain(|a| seen.insert(a.name.to_lowercase()));
    apps
}

// ------------------------------------------------------------------ registro

fn wide_buf(len: usize) -> Vec<u16> {
    vec![0u16; len]
}

fn read_string(key_handle: HKEY, name: &str) -> Option<String> {
    let n = HSTRING::from(name);
    let (mut ty, mut len) = (REG_VALUE_TYPE(0), 0u32);
    unsafe {
        RegQueryValueExW(key_handle, &n, None, Some(&mut ty), None, Some(&mut len)).ok().ok()?;
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        RegQueryValueExW(key_handle, &n, None, Some(&mut ty), Some(buf.as_mut_ptr()), Some(&mut len)).ok().ok()?;
        let units: Vec<u16> = buf[..len as usize].as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
        let s = String::from_utf16_lossy(&units);
        Some(s.trim_end_matches('\0').to_string())
    }
}

/// I percorsi degli eseguibili che hanno registrato un'icona nell'area di notifica (anche in passato).
fn registered_executables() -> Vec<String> {
    let mut out = Vec::new();
    unsafe {
        let mut root = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, &HSTRING::from(SETTINGS_PATH), None, KEY_READ, &mut root).ok().is_err() {
            return out;
        }
        for i in 0..2000u32 {
            let mut name = wide_buf(260);
            let mut len = name.len() as u32;
            if RegEnumKeyExW(root, i, Some(PWSTR(name.as_mut_ptr())), &mut len, None, None, None, None).ok().is_err() {
                break;
            }
            let sub = String::from_utf16_lossy(&name[..len as usize]);
            let mut k = HKEY::default();
            if RegOpenKeyExW(root, &HSTRING::from(sub.as_str()), None, KEY_READ, &mut k).ok().is_ok() {
                if let Some(p) = read_string(k, "ExecutablePath") {
                    out.push(p);
                }
                let _ = RegCloseKey(k);
            }
        }
        let _ = RegCloseKey(root);
    }
    out
}

fn windows_dir() -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    format!("{}\\", root.trim_end_matches('\\').to_lowercase())
}

/// Le app in background adesso: registrate nell'area di notifica, con un processo in esecuzione, senza finestre e non nel Dock.
fn compute() -> Found {
    let wd = windows_dir();
    let own = std::env::current_exe().map(|p| p.to_string_lossy().to_lowercase()).unwrap_or_default();
    // Percorsi risolti (i `{GUID}\\...` diventano percorsi veri) e candidati.
    let mut candidates: Vec<PathBuf> = Vec::new();
    for raw in registered_executables() {
        let path = crate::launchpadfilter::exe_path_of(&raw).unwrap_or_else(|| PathBuf::from(&raw));
        if is_candidate(&path, &wd, &own) && !candidates.iter().any(|c| key(c) == key(&path)) {
            candidates.push(path);
        }
    }
    let names: Vec<String> = candidates.iter().filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase())).collect::<HashSet<_>>().into_iter().collect();
    let running = crate::dock::processes_named(&names);
    // Quelle che hanno già una finestra o un posto nel Dock non servono qui.
    let (items, windows) = crate::dock::items_and_windows();
    let mut out: Found = Vec::new();
    for path in candidates {
        let is_running = running.iter().any(|r| key(r) == key(&path));
        let has_window = windows.iter().any(|(_, w)| same_app(w, &path));
        let in_dock = items.iter().any(|i| same_app(Path::new(&i.path), &path));
        if !is_running || has_window || in_dock {
            continue;
        }
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let name = pretty_name(crate::foreground::cached_description(&path).as_deref(), &stem);
        out.push((TrayApp { id: key(&path), name, icon: cached_icon(&path) }, path));
    }
    let tidy_apps = tidy(out.iter().map(|(a, _)| a.clone()).collect());
    tidy_apps.into_iter().filter_map(|a| out.iter().find(|(x, _)| x.id == a.id).cloned()).collect()
}

/// Le app con il percorso che serve ad avviarle.
type Found = Vec<(TrayApp, PathBuf)>;

static CACHE: Mutex<Option<(Instant, Found)>> = Mutex::new(None);

fn current() -> Found {
    if let Some((at, v)) = &*CACHE.lock().unwrap_or_else(|e| e.into_inner())
        && at.elapsed() < TTL
    {
        return v.clone();
    }
    let v = compute();
    *CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), v.clone()));
    v
}

/// Le app in background per il pannello.
pub fn apps() -> Vec<TrayApp> {
    current().into_iter().map(|(a, _)| a).collect()
}

/// Quante sono (per mostrare l'icona nel Dock solo quando serve).
pub fn count() -> usize {
    current().len()
}

/// Porta davanti l'app `id` (o la avvia: le app a istanza unica mostrano la loro finestra). `false` se non è fra quelle del momento.
pub fn open(id: &str) -> bool {
    let Some((_, path)) = current().into_iter().find(|(a, _)| a.id == id) else { return false };
    crate::dockact::open_path(&path);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const WD: &str = r"c:\windows\";

    #[test]
    fn windows_components_and_our_own_exe_are_not_candidates() {
        assert!(is_candidate(Path::new(r"C:\Program Files\NVIDIA Corporation\NVIDIA Broadcast\NVIDIA Broadcast.exe"), WD, ""));
        assert!(is_candidate(Path::new(r"C:\Users\x\Downloads\AnyDesk.exe"), WD, ""));
        assert!(!is_candidate(Path::new(r"C:\Windows\explorer.exe"), WD, ""));
        assert!(!is_candidate(Path::new(r"C:\Windows\UUS\amd64\MoNotificationUx.exe"), WD, ""));
        assert!(!is_candidate(Path::new(r"C:\Windows\System32\Taskmgr.exe"), WD, ""));
        assert!(!is_candidate(Path::new(r"D:\EosOS\eosos.exe"), WD, r"d:\eosos\eosos.exe"));
        assert!(!is_candidate(Path::new(r"C:\Users\x\AppData\Local\FiveM\FiveM.app\data\cache\subprocess\FiveM_ROSLauncher"), WD, ""), "non e un .exe");
        assert!(!is_candidate(Path::new(""), WD, ""));
    }

    #[test]
    fn the_name_comes_from_the_file_description_or_the_exe_name() {
        assert_eq!(pretty_name(Some("  Logitech G HUB "), "lghub"), "Logitech G HUB");
        assert_eq!(pretty_name(Some(""), "anydesk"), "Anydesk");
        assert_eq!(pretty_name(None, "ISLC"), "ISLC");
        assert_eq!(pretty_name(None, ""), "App");
    }

    #[test]
    fn the_list_is_alphabetical_without_duplicate_names() {
        let a = |id: &str, name: &str| TrayApp { id: id.into(), name: name.into(), icon: None };
        let v = tidy(vec![a("3", "Zeta"), a("1", "anydesk"), a("2", "AnyDesk"), a("4", "Beta")]);
        assert_eq!(v.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["anydesk", "Beta", "Zeta"]);
        assert!(tidy(vec![]).is_empty());
    }

    #[test]
    fn the_real_registry_is_read_without_crashing() {
        let regs = registered_executables();
        let list = compute();
        println!("registrate {} -> in background adesso {}: {:?}", regs.len(), list.len(), list.iter().map(|(a, _)| a.name.clone()).collect::<Vec<_>>());
        assert!(list.len() <= regs.len().max(1));
    }
}
