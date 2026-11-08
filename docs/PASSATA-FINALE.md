# Passata finale (8 nov 2026)

Revisione di sicurezza e misura dei consumi a riposo. Nessuna funzione nuova.

## Sicurezza (revisione statica di un agente, poi corretta)
**Corretto** (`9c658fd`): una pagina compromessa poteva impostare una cartella di rete come cartella di ricerca del Launcher, cercarci un file e farlo aprire con `ShellExecuteW` (e l'enumerazione di un percorso UNC manda le credenziali Windows a quel server). Ora `valid_folder` rifiuta i percorsi di rete, `search_roots` ignora quelli già salvati e il Launcher non apre file che eseguono codice (`is_risky_file`: exe, bat, cmd, ps1, lnk, msi, js, ...): si possono solo mostrare in Esplora risorse. Test puri; nessuna prova dal vivo.

**Controllato e a posto:** avvio delle app (l'id si risolve sempre nell'indice di Rust), Dock, azioni del Launcher (tabelle statiche), apertura da notifica (`valid_app_id`, id verificati), chiave Run dell'avvio automatico, processo `--keyhook` (non emette mai codici di tasto), nessun testo di notifica su disco né nei log, CSP rigida (`script-src 'self'`).

**Rischi accettati (non corretti):**
- `notif.read`, `notif.dnd` e gli appunti del Launcher si possono cambiare da qualunque finestra: limitarli alla sola finestra Impostazioni romperebbe il pulsante «Mostra le notifiche» del pannello e lo spegnimento degli appunti dal Launcher. Serve già una pagina compromessa.
- Nessuna separazione dei comandi per finestra (capability uguale per tutte).
- La conferma di spegnimento/riavvio vive in Rust ma una pagina compromessa può chiamare l'azione due volte.

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
