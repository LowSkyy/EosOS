//! Memoria delle pagine nascoste. Il Launcher, il pannello, le Impostazioni e Mission Control sono finestre WebView2
//! create all'avvio e quasi sempre nascoste: ognuna tiene un processo di rendering da 50-80 MB. WebView2 sa ridurre la
//! memoria di una pagina nascosta (`MemoryUsageTargetLevel` = basso: libera cache e memoria non usata e la tiene
//! compressa); tornando a mostrarla si rimette il livello normale. Non cambia niente di ciò che si vede: la pagina resta
//! viva e riceve gli eventi come prima. Se l'interfaccia non c'è (WebView2 vecchio) non succede nulla.

/// Mette la pagina a basso (`low`) o normale uso di memoria. Sicura da chiamare da qualunque thread.
pub fn set_low(win: &tauri::WebviewWindow, low: bool) {
    // `EOSOS_WEBMEM=0` spegne la riduzione (per misurare quanto costa alla prima apertura).
    if low && std::env::var("EOSOS_WEBMEM").is_ok_and(|v| v == "0") {
        return;
    }
    let _ = win.with_webview(move |wv| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        };
        use windows::core::Interface;
        let Ok(core) = wv.controller().CoreWebView2() else { return };
        let Ok(core19) = core.cast::<ICoreWebView2_19>() else { return };
        let level = if low { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW } else { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL };
        let _ = core19.SetMemoryUsageTargetLevel(level);
    });
}

/// Una finestra appena creata (nascosta) o appena nascosta.
///
/// Provata anche la sospensione completa (`TrySuspend`) delle pagine nascoste: la memoria non scendeva di più (la base
/// di ogni processo di rendering resta) e si rischiava di perdere gli eventi arrivati a pagina sospesa, quindi non c'è.
pub fn hidden(win: &tauri::WebviewWindow) {
    set_low(win, true);
}

/// Una finestra che sta per essere mostrata.
pub fn shown(win: &tauri::WebviewWindow) {
    set_low(win, false);
}
