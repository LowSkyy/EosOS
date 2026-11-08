# Passata finale (8 nov 2026)

Revisione di sicurezza e misura dei consumi a riposo. Nessuna funzione nuova.

## Sicurezza (revisione statica di un agente, poi corretta)
**Corretto** (`9c658fd`): una pagina compromessa poteva impostare una cartella di rete come cartella di ricerca del Launcher, cercarci un file e farlo aprire con `ShellExecuteW` (e l'enumerazione di un percorso UNC manda le credenziali Windows a quel server). Ora `valid_folder` rifiuta i percorsi di rete, `search_roots` ignora quelli già salvati e il Launcher non apre file che eseguono codice (`is_risky_file`: exe, bat, cmd, ps1, lnk, msi, js, ...): si possono solo mostrare in Esplora risorse. Test puri; nessuna prova dal vivo.

**Controllato e a posto:** avvio delle app (l'id si risolve sempre nell'indice di Rust), Dock, azioni del Launcher (tabelle statiche), apertura da notifica (`valid_app_id`, id verificati), chiave Run dell'avvio automatico, processo `--keyhook` (non emette mai codici di tasto), nessun testo di notifica su disco né nei log, CSP rigida (`script-src 'self'`).

**I tre rischi rimasti: corretti** (8 nov 2026):
- **Comandi per finestra** (`cmdpolicy.rs`, tabella pura e testata): le impostazioni (`settings_*`) si usano solo dalla finestra «settings»; `notif_*` solo dal pannello; `launcher_*` solo dal Launcher. Eccezioni minime: il pannello sotto l'orologio può leggere le impostazioni (per la settimana del calendario) e cambiare **solo** `notif.read` e `notif.dnd`. Dalla barra, dal Dock o da Mission Control non si arriva più a niente di tutto questo.
- **Consensi:** accendere la lettura delle notifiche o la cronologia degli appunti da una pagina qualunque non è più possibile (solo dal pannello i due interruttori delle notifiche, solo dal Launcher gli appunti, e dalle Impostazioni tutto).
- **Spegni/Riavvia:** oltre alla doppia pressione (che una pagina potrebbe fare da sola) ora serve un clic su «Sì» in una **finestra di Windows** («No» predefinito, sempre in primo piano) che la pagina non può premere.
- **Prova dal vivo** (`scripts/test-cmd-policy.ps1`, con `EOSOS_TEST_STRICT=1` che accende il controllo anche nella build di prova, dove di norma è spento per lasciare ai test un canale qualunque): dalla barra tutti i comandi sensibili sono bloccati; dal pannello solo notif.read/notif.dnd passano (launcher.clipboard, dock.pinned, general.autostart no); dal Launcher le impostazioni e le notifiche no, ma i suoi comandi sì; Riavvia/Spegni: conferma, poi «Sì» -> eseguita (nella build di prova solo una riga di registro), «No» -> non eseguita. Con il controllo acceso le finestre vere funzionano ancora: Launcher, pannello e calendario passano; nella finestra Impostazioni gli interruttori e le scelte funzionano (i controlli che fallivano leggevano o scrivevano dalla finestra del Launcher, ora vietato come previsto).
- **Non provato:** la finestra di Windows «Sì/No» con un clic vero (nelle prove non compare: si prova solo l'esito sì/no); la build di produzione completa con LTO.

## Consumi a riposo (`scripts/measure-idle.ps1`, build release ottimizzata, 30 s, nessuna interazione)
| | prima | dopo |
|---|---|---|
| CPU processo principale | 1,1–1,5 s / 30 s | **0,48 s / 30 s** (1,6 % di un core) |
| CPU totale (con i WebView2) | 2,2–2,6 s / 30 s | **1,38 s / 30 s** (4,6 %) |
| Memoria | ~868 MB in 15 processi | invariata (~867 MB) |

**Causa trovata:** il thread della sfocatura (`backdrop.rs`) catturava e confrontava lo schermo ogni 40 ms anche quando lo sfondo era fermo. Ora, dopo 1 s senza cambiamenti, cattura ogni 120 ms e torna a 40 ms appena qualcosa cambia (nella build non ottimizzata il thread è sceso da 1,08 s a 0,33 s ogni 20 s). Il ciclo del Dock a 16 ms non era il colpevole (provato: nessun cambiamento) e non è stato modificato.

**Non ridotta:** la memoria. Sono ~9 pagine WebView2 (barre, Dock, Launcher, pannello, Mission Control, Impostazioni) da 55–85 MB ciascuna; ridurla vorrebbe dire creare le finestre solo quando servono (lavoro architetturale, non fatto). Le maniglie sono stabili nel processo principale; nei renderer crescono di poco nei primi secondi.

## Altro emerso
- `scripts/test-dock-reserve.ps1`: il controllo «Blocco note massimizzato finisce sopra la capsula» **fallisce anche senza le mie modifiche** (verificato) — probabilmente il Blocco note di Windows 11 non rispetta più l'area di lavoro in questo modo; da riguardare. Il test usa di default `target\debug` se non gli si passa `-Exe`.
