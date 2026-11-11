# Controllo remoto (AnyDesk e simili): sfocatura sospesa da sola (11 nov 2026)

**Il problema.** Per sfocare lo sfondo, Dock, barra e pannelli di EosOS sono esclusi dalle catture dello schermo (`WDA_EXCLUDEFROMCAPTURE`). Un programma di controllo remoto cattura lo schermo come qualsiasi altro, quindi chi guarda da lontano non vedeva Dock, barra e pannelli (e la barra di Windows è nascosta di proposito).

**La soluzione.** `src-tauri/src/remote.rs` controlla ogni 2 secondi se c'è una sessione remota. Finché c'è, `dockwin::blur_enabled()` risponde «no»: le finestre tornano catturabili e il vetro resta semplice. L'impostazione salvata («Sfocatura dello sfondo», `"blur"` in `dock.json`) **non cambia**: finita la sessione la sfocatura torna da sola. Si può disattivare in Impostazioni → Aspetto → «Niente sfocatura durante il controllo remoto».

**Come si riconosce una sessione** (nessuna regola è infallibile):
- Desktop remoto di Windows: `SM_REMOTESESSION`.
- Processi che esistono solo durante una sessione: `TeamViewer_Desktop.exe`, `remoting_desktop.exe` (Chrome Remote Desktop), `UltraViewer_Desktop.exe`, `winvnc.exe`, `tvnserver.exe`.
- Programmi sempre attivi come servizio, contano solo con una **finestra visibile**: `AnyDesk.exe`, `RustDesk.exe`, `QuickAssist.exe`. Con AnyDesk la finestra c'è durante la connessione; aprire la finestra di AnyDesk in locale conta come sessione (sfocatura persa finché è aperta, nient'altro).

**Limiti.** Un programma non in elenco (o AnyDesk con la finestra nascosta) non viene riconosciuto: si spegne la sfocatura a mano dalle impostazioni o dal menu del Dock. Al cambio di stato Dock e barra rifanno lo sfondo subito; Launcher e pannelli si adeguano alla prossima apertura.

**Prova.** Test Rust sulle regole (4). Dal vivo sul PC di sviluppo, con una sessione AnyDesk in corso: tutte le finestre di EosOS risultano con `WDA_NONE` (catturabili) pur con `"blur": true` salvato.
