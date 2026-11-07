//! Rapporto dei desktop virtuali su file: `%LOCALAPPDATA%\\EosOS\\vdesk-report.json`, aperto con il programma
//! predefinito. Contiene solo dati tecnici (build, profili provati, esito dei controlli), mai dati dell'utente.

use std::path::PathBuf;

use windows::core::{w, HSTRING};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

fn path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\vdesk-report.json"))
}

/// Rifà la selezione, scrive il rapporto (file atomico) e lo apre. `false` se non riesce a scriverlo.
pub fn write_and_open() -> bool {
    let Some(p) = path() else { return false };
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let report = crate::vdint::report(true);
    let Ok(text) = serde_json::to_string_pretty(&report) else { return false };
    let tmp = p.with_extension(format!("json.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, text).is_err() || std::fs::rename(&tmp, &p).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    let target = HSTRING::from(p.to_string_lossy().as_ref());
    let r = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    r.0 as usize > 32
}
