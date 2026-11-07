//! Funzioni di utilità condivise.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::core::PWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{QueryFullProcessImageNameW, PROCESS_NAME_WIN32};

/// Percorso dell'eseguibile di un processo aperto (serve almeno
/// PROCESS_QUERY_LIMITED_INFORMATION).
pub fn image_path(h: HANDLE) -> Option<PathBuf> {
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).ok()?;
    }
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..len as usize])))
}

/// Cache a scadenza: chiave -> (quando e' stata riempita, valore o `None` se non trovato).
pub type TimedCache = Mutex<HashMap<String, (Instant, Option<String>)>>;

/// Quanto resta valida una voce di cache trovata (nome o icona di un'app).
pub const CACHE_FOUND_TTL: Duration = Duration::from_secs(600);
/// Quanto resta valida una voce di cache NON trovata: breve, perché un fallimento può essere
/// passeggero (shell non ancora pronta, per esempio subito dopo il riavvio di Explorer).
pub const CACHE_MISSING_TTL: Duration = Duration::from_secs(20);

/// La voce di cache creata `age` fa vale ancora? Pura, testata.
pub fn is_fresh(age: Duration, found: bool) -> bool {
    age < if found { CACHE_FOUND_TTL } else { CACHE_MISSING_TTL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failures_expire_quickly_and_successes_slowly() {
        assert!(is_fresh(Duration::from_secs(5), false));
        assert!(!is_fresh(Duration::from_secs(25), false), "un fallimento si ritenta presto");
        assert!(is_fresh(Duration::from_secs(25), true));
        assert!(is_fresh(Duration::from_secs(599), true));
        assert!(!is_fresh(Duration::from_secs(601), true), "anche un successo si rinfresca");
    }
}
