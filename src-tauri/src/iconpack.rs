//! Icone di cartelle e file di Esplora file nello stile di macOS (chiare o scure), scritte nel registro dell'UTENTE.
//!
//! Windows non lascia cambiare le icone dentro le app di altri; per le cartelle e i tipi di file pero' le icone si leggono da chiavi
//! `DefaultIcon` che l'utente puo' sovrascrivere sotto `HKCU\Software\Classes` (senza amministratore e senza toccare file di sistema).
//! Qui si scrivono le `.ico` in `%LOCALAPPDATA%\EosOS\icons` e si puntano quelle chiavi. Ogni chiave toccata si ricorda in
//! `%LOCALAPPDATA%\EosOS\iconpack.json` (valore precedente, se c'era; se la chiave l'abbiamo creata noi): `revert` rimette le cose
//! come stavano e non toglie niente che non abbiamo scritto.
//!
//! Le `.ico` si generano con `scripts/build_icons.py` dai disegni di `src/components/iconart.tsx`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use windows::core::HSTRING;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
    KEY_SET_VALUE, REG_OPEN_CREATE_OPTIONS, REG_SZ, REG_VALUE_TYPE,
};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;
use windows::Win32::System::Threading::{WaitForSingleObject, INFINITE};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

macro_rules! ico {
    ($($id:literal),*) => { &[$(($id, include_bytes!(concat!("../iconpack/", $id, "-light.ico")), include_bytes!(concat!("../iconpack/", $id, "-dark.ico")))),*] };
}

/// (id, icona chiara, icona scura).
const ICONS: &[(&str, &[u8], &[u8])] = ico!("folder", "file", "text", "image", "audio", "video", "zip", "pdf", "word", "excel", "powerpoint", "code");

/// Quali estensioni hanno quale icona. Le altre restano com'erano.
pub const EXTENSIONS: &[(&str, &[&str])] = &[
    ("text", &[".txt", ".log", ".md", ".ini", ".cfg", ".rtf"]),
    ("image", &[".png", ".jpg", ".jpeg", ".gif", ".bmp", ".webp", ".tif", ".tiff", ".heic", ".svg"]),
    ("audio", &[".mp3", ".wav", ".flac", ".m4a", ".ogg", ".aac", ".wma"]),
    ("video", &[".mp4", ".mkv", ".avi", ".mov", ".wmv", ".webm", ".m4v"]),
    ("zip", &[".zip", ".rar", ".7z", ".tar", ".gz"]),
    ("pdf", &[".pdf"]),
    ("word", &[".doc", ".docx", ".odt"]),
    ("excel", &[".xls", ".xlsx", ".csv", ".ods"]),
    ("powerpoint", &[".ppt", ".pptx", ".odp"]),
    ("code", &[".js", ".ts", ".tsx", ".jsx", ".py", ".rs", ".json", ".html", ".css", ".xml", ".cs", ".cpp", ".c", ".h", ".java", ".sh", ".ps1", ".bat", ".yml", ".yaml", ".toml"]),
];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
struct Entry {
    /// Chiave `DefaultIcon` sotto HKCU.
    key: String,
    /// Valore che c'era prima (se la chiave esisteva).
    prev: Option<String>,
    /// La chiave madre non esisteva: l'abbiamo creata noi.
    parent_created: bool,
}

/// Un valore di `Shell Icons` (HKLM) che abbiamo cambiato: il nome ("3" cartella chiusa, "4" aperta) e quello che c'era.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
struct ShellVal {
    name: String,
    prev: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct State {
    entries: Vec<Entry>,
    /// Cartelle: Windows 11 non legge `Folder\DefaultIcon` per utente, solo `Shell Icons` in HKLM (serve l'amministratore).
    #[serde(default)]
    shell: Vec<ShellVal>,
}

const SHELL_ICONS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons";
/// Cartella chiusa e aperta.
const SHELL_FOLDER_IDS: [&str; 2] = ["3", "4"];

/// Esegue `cmd /c <args>` con i permessi di amministratore (richiesta UAC) e aspetta la fine. `false` se rifiutata o fallita.
fn run_elevated(args: &str) -> bool {
    let verb = wide("runas");
    let file = wide("cmd.exe");
    let params = wide(&format!("/c {args}"));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: windows::core::PCWSTR(verb.as_ptr()),
        lpFile: windows::core::PCWSTR(file.as_ptr()),
        lpParameters: windows::core::PCWSTR(params.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    unsafe {
        if ShellExecuteExW(&mut info).is_err() || info.hProcess.is_invalid() {
            return false;
        }
        WaitForSingleObject(info.hProcess, INFINITE);
        let _ = CloseHandle(info.hProcess);
    }
    true
}

fn shell_icon_value(name: &str) -> Option<String> {
    get_string(HKEY_LOCAL_MACHINE, SHELL_ICONS, name)
}

/// I comandi `reg` che puntano le cartelle alla nostra icona. Pura, testata.
pub fn shell_add_command(icon: &str) -> String {
    SHELL_FOLDER_IDS.iter().map(|n| format!(r#"reg add "HKLM\{SHELL_ICONS}" /v {n} /t REG_SZ /d "{icon}" /f"#)).collect::<Vec<_>>().join(" & ")
}

/// I comandi che rimettono le cartelle com'erano (valore di prima, o via il valore se non c'era). Pura, testata.
pub fn shell_restore_command(vals: &[(String, Option<String>)]) -> String {
    vals.iter()
        .map(|(n, prev)| match prev {
            Some(v) => format!(r#"reg add "HKLM\{SHELL_ICONS}" /v {n} /t REG_SZ /d "{v}" /f"#),
            None => format!(r#"reg delete "HKLM\{SHELL_ICONS}" /v {n} /f"#),
        })
        .collect::<Vec<_>>()
        .join(" & ")
}

fn data_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("EosOS"))
}

fn state_path() -> Option<PathBuf> {
    Some(data_dir()?.join("iconpack.json"))
}

fn load_state() -> State {
    state_path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save_state(st: &State) {
    if let Some(p) = state_path() {
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Ok(s) = serde_json::to_string_pretty(st) {
            let _ = std::fs::write(p, s);
        }
    }
}

/// Le icone sono applicate adesso?
pub fn is_applied() -> bool {
    !load_state().entries.is_empty()
}

/// Il file dell'icona (nome stabile: cambiando tema si riscrive il file e le chiavi non cambiano).
fn ico_path(id: &str) -> Option<PathBuf> {
    Some(data_dir()?.join("icons").join(format!("{id}.ico")))
}

/// Scrive le `.ico` del tema (chiaro/scuro) in `%LOCALAPPDATA%\EosOS\icons`.
pub fn write_icons(dark: bool) -> bool {
    let Some(dir) = ico_path("x").and_then(|p| p.parent().map(Path::to_path_buf)) else { return false };
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    ICONS.iter().all(|(id, light, darkb)| std::fs::write(dir.join(format!("{id}.ico")), if dark { darkb } else { light }).is_ok())
}

fn wide(s: &str) -> HSTRING {
    HSTRING::from(s)
}

fn open(root: HKEY, path: &str, access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Option<HKEY> {
    let mut key = HKEY::default();
    (unsafe { RegOpenKeyExW(root, &wide(path), None, access, &mut key) } == ERROR_SUCCESS).then_some(key)
}

fn get_string(root: HKEY, path: &str, name: &str) -> Option<String> {
    let key = open(root, path, KEY_QUERY_VALUE)?;
    let out = unsafe {
        let (mut ty, mut len) = (REG_VALUE_TYPE(0), 0u32);
        let n = wide(name);
        if RegQueryValueExW(key, &n, None, Some(&mut ty), None, Some(&mut len)) == ERROR_SUCCESS && len >= 2 && ty == REG_SZ {
            let mut buf = vec![0u8; len as usize];
            if RegQueryValueExW(key, &n, None, Some(&mut ty), Some(buf.as_mut_ptr()), Some(&mut len)) == ERROR_SUCCESS {
                let u: Vec<u16> = buf[..len as usize].as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
                Some(String::from_utf16_lossy(&u).trim_end_matches('\0').to_string())
            } else {
                None
            }
        } else {
            None
        }
    };
    unsafe {
        let _ = RegCloseKey(key);
    }
    out.filter(|s| !s.is_empty())
}

fn get_dword(root: HKEY, path: &str, name: &str) -> Option<u32> {
    let key = open(root, path, KEY_QUERY_VALUE)?;
    let mut v = 0u32;
    let mut len = 4u32;
    let mut ty = REG_VALUE_TYPE(0);
    let ok = unsafe { RegQueryValueExW(key, &wide(name), None, Some(&mut ty), Some(&mut v as *mut u32 as *mut u8), Some(&mut len)) } == ERROR_SUCCESS;
    unsafe {
        let _ = RegCloseKey(key);
    }
    ok.then_some(v)
}

/// Windows e' in tema scuro per le app? (Per l'aspetto «automatico» delle icone.)
pub fn system_is_dark() -> bool {
    get_dword(HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "AppsUseLightTheme") == Some(0)
}

/// Imposta il valore predefinito di una chiave sotto HKCU (creandola).
fn set_default(path: &str, value: &str) -> bool {
    let mut key = HKEY::default();
    unsafe {
        if RegCreateKeyExW(HKEY_CURRENT_USER, &wide(path), None, None, REG_OPEN_CREATE_OPTIONS(0), KEY_SET_VALUE, None, &mut key, None) != ERROR_SUCCESS {
            return false;
        }
        let bytes: Vec<u8> = value.encode_utf16().chain(std::iter::once(0)).flat_map(u16::to_le_bytes).collect();
        let ok = RegSetValueExW(key, None, None, REG_SZ, Some(&bytes)) == ERROR_SUCCESS;
        let _ = RegCloseKey(key);
        ok
    }
}

/// La ProgID che Windows usa adesso per l'estensione: la scelta dell'utente, altrimenti quella registrata.
fn prog_id(ext: &str) -> Option<String> {
    get_string(HKEY_CURRENT_USER, &format!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\{ext}\UserChoice"), "ProgId").or_else(|| get_string(HKEY_CLASSES_ROOT, ext, ""))
}

/// Le chiavi `DefaultIcon` da scrivere: (chiave sotto HKCU\Software\Classes, id icona). Pura rispetto al resolver, testata.
pub fn targets(resolve: &dyn Fn(&str) -> Option<String>) -> Vec<(String, &'static str)> {
    let mut out: Vec<(String, &'static str)> = vec![(r"Software\Classes\Folder\DefaultIcon".to_string(), "folder")];
    for (id, exts) in EXTENSIONS {
        for ext in *exts {
            // Una ProgID condivisa fra due estensioni prende la prima icona; senza ProgID si usa la chiave dell'estensione.
            let base = resolve(ext).unwrap_or_else(|| (*ext).to_string());
            let key = format!(r"Software\Classes\{base}\DefaultIcon");
            if !out.iter().any(|(k, _)| k.eq_ignore_ascii_case(&key)) {
                out.push((key, id));
            }
        }
    }
    out
}

fn notify() {
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
}

/// Applica le icone. `dark`: versione scura. Ripetibile: le chiavi gia' nostre si aggiornano senza perdere il valore originale.
pub fn apply(dark: bool) -> Result<usize, String> {
    if !write_icons(dark) {
        return Err("Non riesco a scrivere le icone in %LOCALAPPDATA%\\EosOS\\icons.".into());
    }
    let mut st = load_state();
    let mut written = 0;
    for (key, id) in targets(&prog_id) {
        let Some(path) = ico_path(id) else { continue };
        let ours = st.entries.iter().any(|e| e.key.eq_ignore_ascii_case(&key));
        if !ours {
            let parent = key.rsplit_once('\\').map(|(p, _)| p.to_string()).unwrap_or_default();
            let parent_created = open(HKEY_CURRENT_USER, &parent, KEY_QUERY_VALUE).is_none();
            let prev = get_string(HKEY_CURRENT_USER, &key, "");
            if !set_default(&key, &path.to_string_lossy()) {
                continue;
            }
            st.entries.push(Entry { key, prev, parent_created });
        } else if !set_default(&key, &path.to_string_lossy()) {
            continue;
        }
        written += 1;
    }
    // Cartelle: una sola richiesta UAC, solo la prima volta (poi il valore e' gia' nostro e il file .ico si riscrive e basta).
    if st.shell.is_empty()
        && let Some(path) = ico_path("folder")
    {
        let icon = path.to_string_lossy().to_string();
        let prev: Vec<ShellVal> = SHELL_FOLDER_IDS.iter().map(|n| ShellVal { name: (*n).to_string(), prev: shell_icon_value(n) }).collect();
        if run_elevated(&shell_add_command(&icon)) && shell_icon_value("3").as_deref() == Some(icon.as_str()) {
            st.shell = prev;
            written += 1;
        }
    }
    save_state(&st);
    notify();
    Ok(written)
}

/// Rimette le cose come stavano.
pub fn revert() -> usize {
    let st = load_state();
    let mut n = 0;
    for e in &st.entries {
        match &e.prev {
            Some(v) => {
                set_default(&e.key, v);
            }
            None => unsafe {
                let _ = RegDeleteKeyW(HKEY_CURRENT_USER, &wide(&e.key));
                if e.parent_created
                    && let Some((parent, _)) = e.key.rsplit_once('\\')
                {
                    let _ = RegDeleteKeyW(HKEY_CURRENT_USER, &wide(parent));
                }
            },
        }
        n += 1;
    }
    if !st.shell.is_empty() {
        let vals: Vec<(String, Option<String>)> = st.shell.iter().map(|v| (v.name.clone(), v.prev.clone())).collect();
        run_elevated(&shell_restore_command(&vals));
        n += 1;
    }
    if let Some(p) = state_path() {
        let _ = std::fs::remove_file(p);
    }
    notify();
    n
}

/// Il tema da usare: la scelta salvata, e per «auto» quello di Windows.
pub fn want_dark(choice: &str) -> bool {
    match choice {
        "dark" => true,
        "light" => false,
        _ => system_is_dark(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_commands_cover_closed_and_open_folder_and_restore_exactly() {
        let add = shell_add_command(r"C:\x\folder.ico");
        assert!(add.contains("/v 3 ") && add.contains("/v 4 ") && add.contains(r#"/d "C:\x\folder.ico""#));
        let back = shell_restore_command(&[("3".into(), Some(r"C:\old.ico".into())), ("4".into(), None)]);
        assert!(back.contains("reg add") && back.contains(r"C:\old.ico") && back.contains("reg delete") && back.contains("/v 4 /f"));
    }

    #[test]
    fn every_extension_group_has_an_icon_file() {
        for (id, _) in EXTENSIONS {
            assert!(ICONS.iter().any(|(i, ..)| i == id), "manca l'icona {id}");
        }
        assert!(ICONS.iter().all(|(_, l, d)| l.len() > 100 && d.len() > 100 && l[..4] == [0, 0, 1, 0] && d[..4] == [0, 0, 1, 0]), "devono essere .ico veri");
    }

    #[test]
    fn targets_use_the_prog_id_and_never_repeat_a_key() {
        let t = targets(&|ext| match ext {
            ".jpg" | ".jpeg" => Some("jpegfile".to_string()),
            ".txt" => Some("txtfile".to_string()),
            _ => None,
        });
        assert_eq!(t[0], (r"Software\Classes\Folder\DefaultIcon".to_string(), "folder"));
        assert!(t.iter().any(|(k, i)| k == r"Software\Classes\jpegfile\DefaultIcon" && *i == "image"));
        assert_eq!(t.iter().filter(|(k, _)| k == r"Software\Classes\jpegfile\DefaultIcon").count(), 1);
        // senza ProgID si usa la chiave dell'estensione
        assert!(t.iter().any(|(k, i)| k == r"Software\Classes\.mp3\DefaultIcon" && *i == "audio"));
    }

    /// Prova sul registro VERO dell'utente (si lancia a mano con `--ignored`): applica, controlla e rimette tutto come stava.
    #[test]
    #[ignore]
    fn real_registry_round_trip() {
        let before = get_string(HKEY_CURRENT_USER, r"Software\Classes\Folder\DefaultIcon", "");
        let n = apply(true).expect("apply");
        assert!(n > 10, "scritte {n}");
        assert!(is_applied());
        let now = get_string(HKEY_CURRENT_USER, r"Software\Classes\Folder\DefaultIcon", "").expect("valore scritto");
        assert!(now.ends_with(r"icons\folder.ico"), "{now}");
        assert!(std::path::Path::new(&now).exists());
        let reverted = revert();
        assert!(reverted >= n);
        assert!(!is_applied());
        assert_eq!(get_string(HKEY_CURRENT_USER, r"Software\Classes\Folder\DefaultIcon", ""), before);
    }

    #[test]
    fn the_theme_choice_wins_over_windows() {
        assert!(want_dark("dark"));
        assert!(!want_dark("light"));
    }
}
