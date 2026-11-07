//! Funzioni di utilità condivise.

use std::path::PathBuf;

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
