//! Impostazioni di EosOS, fase S3: il selettore di cartelle di Windows (la finestra «Seleziona cartella» di sistema,
//! la stessa di ogni altro programma). Si apre in un thread a parte con il suo COM, legato alla finestra delle
//! impostazioni (`owner`), così non blocca nient'altro. Restituisce il percorso scelto o `None` se si annulla.
//! Nessun test automatico (è una finestra di sistema): la verifica è dal vivo, vedi `scripts/test-settings-s3.ps1`.

use windows::core::w;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{FileOpenDialog, IFileOpenDialog, FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, SIGDN_FILESYSPATH};

fn pick_blocking(owner: Option<isize>) -> Option<String> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let out = (|| -> windows::core::Result<Option<String>> {
            let dlg: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dlg.SetOptions(dlg.GetOptions()? | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST)?;
            dlg.SetTitle(w!("Scegli una cartella"))?;
            // Annullare restituisce un errore (HRESULT di "annullato"): non è un guasto.
            if dlg.Show(owner.map(|h| HWND(h as *mut _))).is_err() {
                return Ok(None);
            }
            let item = dlg.GetResult()?;
            let name = item.GetDisplayName(SIGDN_FILESYSPATH)?;
            let s = name.to_string().ok();
            CoTaskMemFree(Some(name.0 as *const _));
            Ok(s)
        })();
        if init {
            CoUninitialize();
        }
        out.ok().flatten().filter(|p| !p.trim().is_empty())
    }
}

/// Apre il selettore e aspetta la scelta. Nelle build di prova, `EOSOS_TEST_PICK_FOLDER=<percorso>` restituisce
/// quel percorso senza aprire la finestra.
pub fn pick(owner: Option<isize>) -> Option<String> {
    #[cfg(feature = "test-hooks")]
    if let Some(p) = std::env::var_os("EOSOS_TEST_PICK_FOLDER").filter(|v| !v.is_empty()) {
        return Some(p.to_string_lossy().to_string());
    }
    std::thread::spawn(move || pick_blocking(owner)).join().ok().flatten()
}
