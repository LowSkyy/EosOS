//! Quali voci dell'indice delle app sono vere **applicazioni** da mostrare nel Launchpad (come in macOS), e quali no.
//!
//! L'indice (`shell:AppsFolder`) contiene di tutto: strumenti di sistema (Defrag, ODBC, Narrator…), il prompt dei comandi e PowerShell, Python
//! e Node a riga di comando, debugger e kit di sviluppo, manuali e licenze, aggiornatori e segnalatori di errori. Il Launchpad le toglie; la
//! ricerca del Launcher (Alt+Spazio) continua a trovare tutto.
//!
//! Si giudica da: la destinazione (cartelle della shell e strumenti di sistema non sono app), il **tipo di eseguibile** (un programma a riga di
//! comando, sottosistema «console», non è un'app) e il nome (manuali, aggiornatori…). Nel dubbio una voce **resta** (meglio una in più che una
//! app vera nascosta). La parte pura è testata; sotto c'è la lettura dei file.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::core::GUID;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT};

use crate::launcher::AppEntry;

/// Sottosistema degli eseguibili: finestra (GUI) e riga di comando (console).
pub const SUBSYSTEM_CONSOLE: u16 = 3;

/// `{GUID}\resto` (cartella nota di Windows + percorso relativo) → (GUID, resto). Pura, testata.
pub fn split_known_folder(id: &str) -> Option<(&str, &str)> {
    let rest = id.strip_prefix('{')?;
    let (guid, tail) = rest.split_once('}')?;
    let tail = tail.strip_prefix('\\')?;
    (!tail.is_empty()).then_some((guid, tail))
}

/// Un GUID scritto come `1AC14E77-02E7-4E5D-B744-2EB1AE5198B7` (senza graffe). Pura, testata.
pub fn parse_guid(s: &str) -> Option<GUID> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 || [8, 4, 4, 4, 12].iter().zip(&parts).any(|(n, p)| p.len() != *n) {
        return None;
    }
    let d1 = u32::from_str_radix(parts[0], 16).ok()?;
    let d2 = u16::from_str_radix(parts[1], 16).ok()?;
    let d3 = u16::from_str_radix(parts[2], 16).ok()?;
    let mut d4 = [0u8; 8];
    let tail = format!("{}{}", parts[3], parts[4]);
    for (i, b) in d4.iter_mut().enumerate() {
        *b = u8::from_str_radix(&tail[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(GUID::from_values(d1, d2, d3, d4))
}

/// Il sottosistema di un eseguibile dalle prime righe del file (intestazione PE): 2 = finestra, 3 = console. `None` se non è un PE. Pura, testata.
pub fn subsystem_of(head: &[u8]) -> Option<u16> {
    if head.len() < 0x40 || &head[0..2] != b"MZ" {
        return None;
    }
    let pe = u32::from_le_bytes(head[0x3C..0x40].try_into().ok()?) as usize;
    let opt = pe.checked_add(24)?;
    if head.len() < opt + 70 || &head[pe..pe + 4] != b"PE\0\0" {
        return None;
    }
    Some(u16::from_le_bytes([head[opt + 68], head[opt + 69]]))
}

/// Nomi che non sono applicazioni: manuali, licenze, siti, prompt dei comandi, aggiornatori, strumenti da sviluppatore e di amministrazione. Pura, testata.
pub fn name_is_not_an_app(name: &str) -> bool {
    const TOKENS: &[&str] = &[
        "command prompt", "prompt dei comandi", "powershell", "documentation", "documentazione", "manual", "manuale", "module docs", "pydoc", "website",
        "sito web", "readme", "leggimi", "license", "licenza", "release notes", "novità", "novita", "help", "guida", "uninstall", "disinstalla", "updater",
        "aggiornamenti ", "error reporter", "crash", "idle (", "debugging", "global flags", "application verifier", "gpuview", "iscsi", "origini dati",
        "odbc", "windows kits", "sdk", "app recovery", "criteri di sicurezza", "gestione computer", "monitoraggio risorse", "performance monitor",
        "visualizzatore eventi", "pianificazione attività", "install additional tools", "strumenti di windows", "utilità di pianificazione", "software development kit",
        "install manager", "scelta della lingua", "informazioni di base", "azioni con un clic", "pdfsaver",
    ];
    let l = name.trim().to_lowercase();
    TOKENS.iter().any(|t| l.contains(t))
}

/// Il nome del file eseguibile dice che non è l'app vera (installatori, aggiornatori, segnalatori, programmi di supporto). Pura, testata.
pub fn exe_stem_is_support(stem: &str) -> bool {
    const TOKENS: &[&str] = &["updater", "update", "uninst", "unins0", "setup", "install", "crashreport", "errorreporter", "crashpad", "helper", "bootstrap", "system_tray", "elevate"];
    let l = stem.to_lowercase();
    TOKENS.iter().any(|t| l.contains(t))
}

/// I fatti noti di una voce. `exe`: il file a cui punta, se è un percorso; `subsystem`: letto dal file; `windows_dir`: la cartella di Windows in minuscolo con la barra finale.
pub struct Facts<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub exe: Option<&'a Path>,
    pub subsystem: Option<u16>,
    pub windows_dir: &'a str,
}

/// La voce è un'applicazione da mostrare nel Launchpad? Pura, testata.
pub fn is_app(f: &Facts) -> bool {
    let id = f.id.trim();
    if id.is_empty() || f.name.trim().is_empty() {
        return false;
    }
    // Cartelle della shell (Cestino, Pannello di controllo come cartella…): «::{GUID}» o un GUID da solo.
    if id.starts_with("::") || (id.starts_with('{') && !id.contains('\\')) {
        return false;
    }
    let lid = id.to_lowercase();
    // La finestra «Esegui» e il Pannello di controllo classico sono utilità di sistema, non app.
    if lid.contains("rundialog") || lid == "microsoft.windows.controlpanel" {
        return false;
    }
    if name_is_not_an_app(f.name) {
        return false;
    }
    if let Some(exe) = f.exe {
        let p = exe.to_string_lossy().to_lowercase();
        if (!f.windows_dir.is_empty() && p.starts_with(f.windows_dir)) || p.contains(r"\windows kits\") {
            return false; // strumenti di sistema e kit di sviluppo
        }
        if exe.extension().and_then(|e| e.to_str()).is_none_or(|e| !e.eq_ignore_ascii_case("exe")) {
            return false; // .msc, .cpl, .bat, documenti…
        }
        if f.subsystem == Some(SUBSYSTEM_CONSOLE) {
            return false; // programmi a riga di comando: cmd, python, node, wsl…
        }
        if exe.file_stem().and_then(|s| s.to_str()).is_some_and(exe_stem_is_support) {
            return false;
        }
    }
    true
}

// ------------------------------------------------------------------ lettura dei file

fn known_folder(guid: &GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(guid, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(PathBuf::from)
    }
}

/// Il file a cui punta un id dell'indice, se è un percorso: `{GUID}\resto` (cartella nota) o un percorso assoluto. Gli identificatori di app
/// (Store, «Chrome»…) non hanno un file noto: `None`.
pub(crate) fn exe_path_of(id: &str) -> Option<PathBuf> {
    let id = id.trim();
    if let Some((guid, rest)) = split_known_folder(id) {
        return known_folder(&parse_guid(guid)?).map(|b| b.join(rest));
    }
    let p = Path::new(id);
    (p.is_absolute() && id.len() > 3).then(|| p.to_path_buf())
}

fn subsystem_of_file(path: &Path) -> Option<u16> {
    use std::io::Read;
    let mut buf = vec![0u8; 4096];
    let n = std::fs::File::open(path).ok()?.read(&mut buf).ok()?;
    subsystem_of(&buf[..n])
}

fn windows_dir() -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    format!("{}\\", root.trim_end_matches('\\').to_lowercase())
}

/// Decide su una voce leggendo il suo file.
fn decide(e: &AppEntry, wd: &str) -> bool {
    let exe = exe_path_of(&e.id);
    let subsystem = exe.as_deref().and_then(subsystem_of_file);
    is_app(&Facts { id: &e.id, name: &e.name, exe: exe.as_deref(), subsystem, windows_dir: wd })
}

static CACHE: Mutex<Option<(Instant, usize, Vec<AppEntry>)>> = Mutex::new(None);
const TTL: Duration = Duration::from_secs(120);

/// Solo le applicazioni fra le voci date (stesso ordine). Il risultato si ricorda per due minuti (le pagine del Launchpad lo chiedono più volte).
pub fn apps_only(entries: &[AppEntry]) -> Vec<AppEntry> {
    if let Some((at, n, v)) = &*CACHE.lock().unwrap_or_else(|e| e.into_inner())
        && *n == entries.len()
        && at.elapsed() < TTL
    {
        return v.clone();
    }
    let wd = windows_dir();
    let kept: Vec<AppEntry> = entries.iter().filter(|e| decide(e, &wd)).cloned().collect();
    *CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), entries.len(), kept.clone()));
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    const WD: &str = r"c:\windows\";

    fn app(id: &str, name: &str, exe: Option<&str>, sub: Option<u16>) -> bool {
        let p = exe.map(PathBuf::from);
        is_app(&Facts { id, name, exe: p.as_deref(), subsystem: sub, windows_dir: WD })
    }

    #[test]
    fn real_apps_stay() {
        assert!(app("Microsoft.WindowsNotepad_8wekyb3d8bbwe!App", "Blocco note", None, None), "app dello Store");
        assert!(app("Chrome", "Google Chrome", None, None), "identificatore semplice");
        assert!(app("com.squirrel.Discord.Discord", "Discord", None, None));
        assert!(app("Microsoft.Windows.Explorer", "Esplora file", None, None));
        assert!(app("Microsoft.Office.EXCEL.EXE.15", "Excel", None, None));
        assert!(app(r"C:\Users\x\AppData\Local\Programs\Ollama\ollama app.exe", "Ollama", Some(r"C:\Users\x\AppData\Local\Programs\Ollama\ollama app.exe"), Some(2)));
        assert!(app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\NVIDIA Corporation\NVIDIA Broadcast\NVIDIA Broadcast.exe", "NVIDIA Broadcast", Some(r"C:\Program Files\NVIDIA Corporation\NVIDIA Broadcast\NVIDIA Broadcast.exe"), Some(2)));
        assert!(app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\LGHUB\lghub.exe", "LGHUB", Some(r"C:\Program Files\LGHUB\lghub.exe"), Some(2)));
        assert!(app("Microsoft.AutoGenerated.{10D58619-54C7-8BD3-934F-1CF2D4472400}", "Gestione attività", None, None), "nel dubbio resta");
    }

    #[test]
    fn command_line_programs_are_not_apps() {
        assert!(!app(r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\cmd.exe", "Prompt dei comandi", Some(r"C:\Windows\System32\cmd.exe"), Some(3)));
        assert!(!app(r"C:\Python314\python.exe", "Python 3.14 (64-bit)", Some(r"C:\Python314\python.exe"), Some(3)), "console");
        assert!(!app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\nodejs\node.exe", "Node.js", Some(r"C:\Program Files\nodejs\node.exe"), Some(3)));
        assert!(!app("Microsoft.AutoGenerated.{B01147A4-E554-761F-B533-FF3E10213F31}", "Developer Command Prompt for VS", None, None));
        assert!(!app("Microsoft.AutoGenerated.{E10538EF-CD1A-F9C1-3145-3BC12E11E1F8}", "Developer PowerShell for VS", None, None));
        assert!(!app("Microsoft.AutoGenerated.{499EE7CA-7ED5-EBC9-AB09-80DC143D7B90}", "Node.js command prompt", None, None));
    }

    #[test]
    fn system_tools_and_dev_kits_are_not_apps() {
        for (id, name, exe) in [
            (r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\dfrgui.exe", "Deframmenta e ottimizza unità", r"C:\Windows\System32\dfrgui.exe"),
            (r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\odbcad32.exe", "Origini dati ODBC (64 bit)", r"C:\Windows\System32\odbcad32.exe"),
            (r"{F38BF404-1D43-42F2-9305-67DE0B28FC23}\regedit.exe", "Editor del Registro di sistema", r"C:\Windows\regedit.exe"),
            (r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\printmanagement.msc", "Gestione stampa", r"C:\Windows\System32\printmanagement.msc"),
            (r"{7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E}\Windows Kits\10\Debuggers\x64\gflags.exe", "Global Flags (X64)", r"C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\gflags.exe"),
        ] {
            assert!(!app(id, name, Some(exe), Some(2)), "{name}");
        }
    }

    #[test]
    fn manuals_licenses_websites_and_updaters_are_not_apps() {
        assert!(!app("https://nodejs.org/", "Node.js website", None, None));
        assert!(!app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\WinRAR\WinRAR.chm", "Guida a WinRAR", Some(r"C:\Program Files\WinRAR\WinRAR.chm"), None));
        assert!(!app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\PDF-XChange\PDF Editor\PXCLicense.pdf", "PDF-XChange Editor License Agreement", Some(r"C:\x\PXCLicense.pdf"), None));
        assert!(!app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\Electronic Arts\EA Desktop\EA Desktop\EAUpdater.exe", "Aggiornamenti EA app", Some(r"C:\Program Files\Electronic Arts\EAUpdater.exe"), Some(2)));
        assert!(!app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\Electronic Arts\EA Desktop\EA Desktop\ErrorReporter.exe", "EA Error Reporter", Some(r"C:\x\ErrorReporter.exe"), Some(2)));
        assert!(!app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\PDF-XChange\Updater\PXCUpdater.exe", "PDF-XChange Updater", Some(r"C:\x\PXCUpdater.exe"), Some(2)));
        assert!(!app(r"{6D809377-6AF0-444B-8957-A3773F02200E}\LGHUB\system_tray\lghub_system_tray.exe", "Logitech G HUB", Some(r"C:\Program Files\LGHUB\system_tray\lghub_system_tray.exe"), Some(2)), "programma di supporto della barra di sistema");
    }

    #[test]
    fn shell_folders_and_system_dialogs_are_not_apps() {
        assert!(!app("::{645FF040-5081-101B-9F08-00AA002F954E}", "Cestino", None, None));
        assert!(!app("{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}", "Qualcosa", None, None), "GUID da solo");
        assert!(!app("Microsoft.Windows.Shell.RunDialog", "Esegui", None, None));
        assert!(!app("Microsoft.Windows.ControlPanel", "Pannello di controllo", None, None));
        assert!(!app("Microsoft.AutoGenerated.{AAAA}", "Strumenti di Windows", None, None) && !app("x", "Utilità di pianificazione", None, None));
        assert!(!app("", "Vuota", None, None) && !app("x", "  ", None, None));
    }

    #[test]
    fn the_pe_subsystem_is_read_from_the_header() {
        // MZ + e_lfanew a 0x40, poi «PE\0\0», 20 byte di intestazione di file, e il sottosistema a 68 byte dall'inizio dell'opzionale.
        let mut h = vec![0u8; 0x200];
        h[0] = b'M';
        h[1] = b'Z';
        h[0x3C] = 0x40;
        h[0x40..0x44].copy_from_slice(b"PE\0\0");
        let opt = 0x40 + 24;
        h[opt + 68] = 3;
        assert_eq!(subsystem_of(&h), Some(3));
        h[opt + 68] = 2;
        assert_eq!(subsystem_of(&h), Some(2));
        assert_eq!(subsystem_of(&h[..0x20]), None, "troppo corto");
        h[0] = b'X';
        assert_eq!(subsystem_of(&h), None, "non e un eseguibile");
        h[0] = b'M';
        h[0x3C] = 0xFF;
        h[0x3D] = 0xFF;
        assert_eq!(subsystem_of(&h), None, "puntatore fuori dal file");
    }

    #[test]
    fn known_folder_ids_and_guids_are_parsed() {
        assert_eq!(split_known_folder(r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\cmd.exe"), Some(("1AC14E77-02E7-4E5D-B744-2EB1AE5198B7", "cmd.exe")));
        assert_eq!(split_known_folder(r"C:\x.exe"), None);
        assert_eq!(split_known_folder("{abc}"), None);
        assert_eq!(split_known_folder(r"{abc}\"), None);
        let g = parse_guid("1AC14E77-02E7-4E5D-B744-2EB1AE5198B7").unwrap();
        assert_eq!(g, GUID::from_values(0x1AC14E77, 0x02E7, 0x4E5D, [0xB7, 0x44, 0x2E, 0xB1, 0xAE, 0x51, 0x98, 0xB7]));
        assert!(parse_guid("zzzz").is_none() && parse_guid("1AC14E77-02E7-4E5D-B744").is_none());
    }

    #[test]
    fn real_machine_smoke_test_keeps_the_common_apps() {
        // Sul PC vero: nessun panico e l'elenco filtrato è più corto dell'indice ma non vuoto (se l'indice non lo è).
        let all = crate::launcher::apps();
        let kept = apps_only(&all);
        println!("indice {} -> launchpad {}", all.len(), kept.len());
        let keep_ids: std::collections::HashSet<&str> = kept.iter().map(|e| e.id.as_str()).collect();
        let mut dropped: Vec<&str> = all.iter().filter(|e| !keep_ids.contains(e.id.as_str())).map(|e| e.name.as_str()).collect();
        dropped.sort_unstable();
        println!("TOLTE: {}", dropped.join(" | "));
        let mut kn: Vec<&str> = kept.iter().map(|e| e.name.as_str()).collect();
        kn.sort_unstable();
        println!("RESTANO: {}", kn.join(" | "));
        assert!(kept.len() <= all.len());
        if !all.is_empty() {
            assert!(!kept.is_empty());
        }
    }
}
