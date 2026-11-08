//! Impostazioni di EosOS, fase S4: l'avvio automatico con Windows. Scrive e toglie UN solo valore, chiamato
//! `EosOS`, nella chiave `Run` dell'UTENTE corrente (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`):
//! niente chiavi di sistema, niente altri valori, niente attività pianificate. Rispetta anche la scelta fatta
//! da Gestione attività (`StartupApproved`): se l'utente l'ha disattivata da lì, per le impostazioni è spenta, e
//! riaccenderla da qui toglie quel blocco. Le funzioni lavorano su chiavi passate dal chiamante così le prove
//! usano una chiave di prova e non toccano mai quella vera.

use std::path::Path;

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPEN_CREATE_OPTIONS, REG_SZ, REG_VALUE_TYPE,
};

/// Il nome del valore.
pub const NAME: &str = "EosOS";
const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

/// Le due chiavi (sotto HKCU) su cui si lavora.
#[derive(Clone, Debug)]
pub struct Keys {
    pub run: String,
    pub approved: String,
}

/// Le chiavi vere dell'utente.
pub fn real_keys() -> Keys {
    Keys { run: RUN.to_string(), approved: APPROVED.to_string() }
}

/// Il testo da scrivere nel valore: il percorso dell'eseguibile fra virgolette (così uno spazio nel percorso
/// non lo spezza). Pura, testata.
pub fn run_value(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

/// Il blocco di Gestione attività: `StartupApproved` ha 12 byte; il primo, dispari (03), vuol dire «disattivata»,
/// pari (02 o 06) «attivata». Pura, testata.
pub fn blocked_by_user(approved: Option<&[u8]>) -> bool {
    approved.and_then(|b| b.first()).is_some_and(|b| b % 2 == 1)
}

/// Il percorso scritto in un valore `Run` (tolte le virgolette e gli eventuali argomenti), per confrontarlo. Pura, testata.
#[cfg(test)]
pub fn exe_of(value: &str) -> String {
    let v = value.trim();
    if let Some(rest) = v.strip_prefix('"') {
        return rest.split('"').next().unwrap_or("").to_string();
    }
    v.split(" -").next().unwrap_or(v).trim().to_string()
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn open(path: &str, access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Option<HKEY> {
    let mut key = HKEY::default();
    unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, &HSTRING::from(path), None, access, &mut key) }.ok().ok()?;
    Some(key)
}

fn read(path: &str, name: &str) -> Option<(REG_VALUE_TYPE, Vec<u8>)> {
    let key = open(path, KEY_QUERY_VALUE)?;
    let n = HSTRING::from(name);
    let (mut ty, mut len) = (REG_VALUE_TYPE(0), 0u32);
    let out = unsafe {
        let sized = RegQueryValueExW(key, &n, None, Some(&mut ty), None, Some(&mut len)).ok().is_ok();
        if sized && len > 0 {
            let mut buf = vec![0u8; len as usize];
            RegQueryValueExW(key, &n, None, Some(&mut ty), Some(buf.as_mut_ptr()), Some(&mut len)).ok().ok().map(|_| {
                buf.truncate(len as usize);
                (ty, buf)
            })
        } else {
            None
        }
    };
    unsafe {
        let _ = RegCloseKey(key);
    }
    out
}

fn text_of(bytes: &[u8]) -> String {
    let u: Vec<u16> = bytes.iter().step_by(2).zip(bytes.iter().skip(1).step_by(2)).map(|(a, b)| u16::from_le_bytes([*a, *b])).collect();
    String::from_utf16_lossy(&u).trim_end_matches('\0').to_string()
}

/// Il valore `Run` c'è (e non è stato bloccato da Gestione attività)?
pub fn is_enabled(k: &Keys, name: &str) -> bool {
    let present = read(&k.run, name).is_some_and(|(ty, b)| ty == REG_SZ && !text_of(&b).trim().is_empty());
    present && !blocked_by_user(read(&k.approved, name).map(|(_, b)| b).as_deref())
}

/// Il percorso dell'eseguibile scritto adesso nel valore, se c'è (serve alle prove).
#[cfg(test)]
pub fn current_target(k: &Keys, name: &str) -> Option<String> {
    read(&k.run, name).filter(|(ty, _)| *ty == REG_SZ).map(|(_, b)| exe_of(&text_of(&b)))
}

fn delete_value(path: &str, name: &str) -> Result<(), String> {
    let Some(key) = open(path, KEY_SET_VALUE) else { return Ok(()) }; // la chiave non c'è: niente da togliere
    let r = unsafe { RegDeleteValueW(key, &HSTRING::from(name)) };
    unsafe {
        let _ = RegCloseKey(key);
    }
    if r == WIN32_ERROR(0) || r == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(format!("registro: errore {}", r.0))
    }
}

/// Accende l'avvio automatico: scrive il valore (chiave creata se manca) e toglie un eventuale blocco di Gestione
/// attività. Scrive solo `name`.
pub fn enable(k: &Keys, name: &str, exe: &Path) -> Result<(), String> {
    if !exe.is_absolute() || !exe.is_file() {
        return Err("Non trovo il file di EosOS da avviare.".into());
    }
    let mut key = HKEY::default();
    let r = unsafe {
        RegCreateKeyExW(HKEY_CURRENT_USER, &HSTRING::from(k.run.as_str()), None, PCWSTR::null(), REG_OPEN_CREATE_OPTIONS(0), KEY_SET_VALUE, None, &mut key, None)
    };
    if r != WIN32_ERROR(0) {
        return Err(format!("registro: errore {}", r.0));
    }
    let data: Vec<u8> = wide(&run_value(exe)).iter().flat_map(|u| u.to_le_bytes()).collect();
    let r = unsafe { RegSetValueExW(key, &HSTRING::from(name), None, REG_SZ, Some(&data)) };
    unsafe {
        let _ = RegCloseKey(key);
    }
    if r != WIN32_ERROR(0) {
        return Err(format!("registro: errore {}", r.0));
    }
    delete_value(&k.approved, name)
}

/// Spegne l'avvio automatico: toglie il valore `name` (e il suo blocco di Gestione attività). Se non c'è, va bene.
pub fn disable(k: &Keys, name: &str) -> Result<(), String> {
    delete_value(&k.run, name)?;
    delete_value(&k.approved, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn the_value_is_the_quoted_path() {
        assert_eq!(run_value(Path::new(r"C:\Program Files\EosOS\eosos.exe")), r#""C:\Program Files\EosOS\eosos.exe""#);
    }

    #[test]
    fn the_path_is_recovered_from_a_run_value() {
        assert_eq!(exe_of(r#""C:\a b\eosos.exe""#), r"C:\a b\eosos.exe");
        assert_eq!(exe_of(r#""C:\a\x.exe" --flag"#), r"C:\a\x.exe");
        assert_eq!(exe_of(r"C:\a\x.exe -min"), r"C:\a\x.exe");
        assert_eq!(exe_of("  "), "");
    }

    #[test]
    fn task_manager_blocks_are_read_from_the_first_byte() {
        assert!(blocked_by_user(Some(&[3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])));
        assert!(!blocked_by_user(Some(&[2, 0, 0, 0])), "02 = attivata");
        assert!(!blocked_by_user(Some(&[6, 0, 0, 0])), "06 = attivata");
        assert!(!blocked_by_user(Some(&[])) && !blocked_by_user(None));
    }

    /// Una chiave di prova sotto HKCU, mai quella vera di Windows.
    fn scratch(tag: &str) -> Keys {
        let base = format!(r"Software\EosOS-Test-{}-{tag}", std::process::id());
        Keys { run: format!(r"{base}\Run"), approved: format!(r"{base}\Approved") }
    }
    fn clean(k: &Keys) {
        let base = k.run.rsplit_once('\\').unwrap().0.to_string();
        let _ = std::process::Command::new("reg").args(["delete", &format!(r"HKCU\{base}"), "/f"]).output();
    }
    fn this_exe() -> PathBuf {
        std::env::current_exe().unwrap()
    }

    #[test]
    fn enable_and_disable_touch_only_our_value() {
        let k = scratch("a");
        // Un altro programma ha già il suo valore nella chiave: non deve mai cambiare.
        let other = std::process::Command::new("reg").args(["add", &format!(r"HKCU\{}", k.run), "/v", "Altro", "/d", r"C:\altro.exe", "/f"]).output().unwrap();
        assert!(other.status.success());
        assert!(!is_enabled(&k, NAME));
        enable(&k, NAME, &this_exe()).unwrap();
        assert!(is_enabled(&k, NAME));
        assert_eq!(current_target(&k, NAME).unwrap(), this_exe().display().to_string());
        assert_eq!(current_target(&k, "Altro").unwrap(), r"C:\altro.exe", "l'altro valore e' intatto");
        enable(&k, NAME, &this_exe()).unwrap(); // idempotente
        assert!(is_enabled(&k, NAME));
        disable(&k, NAME).unwrap();
        assert!(!is_enabled(&k, NAME));
        assert_eq!(current_target(&k, "Altro").unwrap(), r"C:\altro.exe", "ancora intatto dopo averlo spento");
        disable(&k, NAME).unwrap(); // spegnere due volte va bene
        clean(&k);
    }

    #[test]
    fn a_task_manager_block_counts_as_off_and_turning_on_clears_it() {
        let k = scratch("b");
        enable(&k, NAME, &this_exe()).unwrap();
        let blocked = std::process::Command::new("reg")
            .args(["add", &format!(r"HKCU\{}", k.approved), "/v", NAME, "/t", "REG_BINARY", "/d", "030000000000000000000000", "/f"])
            .output()
            .unwrap();
        assert!(blocked.status.success());
        assert!(!is_enabled(&k, NAME), "disattivata da Gestione attivita'");
        enable(&k, NAME, &this_exe()).unwrap();
        assert!(is_enabled(&k, NAME), "riaccenderla toglie il blocco");
        disable(&k, NAME).unwrap();
        clean(&k);
    }

    #[test]
    fn a_missing_or_relative_exe_is_refused_and_nothing_is_written() {
        let k = scratch("c");
        assert!(enable(&k, NAME, Path::new("eosos.exe")).is_err());
        assert!(enable(&k, NAME, Path::new(r"C:\non\esiste\mai.exe")).is_err());
        assert!(!is_enabled(&k, NAME));
        assert!(read(&k.run, NAME).is_none(), "la chiave non e' stata nemmeno creata");
        clean(&k);
    }
}
