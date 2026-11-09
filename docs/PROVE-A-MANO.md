# Prove a mano (quelle che l'assistente non può fare)

Tutto il resto è coperto da prove automatiche (Rust 410, vitest 177, ~50 script in `scripts/`, tutti verdi al 9 nov 2026).
Qui sotto resta ciò che richiede una tastiera, un mouse o un monitor veri. Segna ogni riga: **ok**, o cosa non va (e su quale schermo).
Gli installer firmati sono in `src-tauri\target\release\bundle\` (NSIS e MSI); l'exe pulito in `src-tauri\target\release\eosos.exe`.

## 1. Tasto Windows → Launcher (nuovo)
- [ ] Un tocco del tasto Windows (sinistro) apre il Launcher; un secondo tocco lo chiude. Il menu Start **non** si apre.
- [ ] Lo stesso con il tasto Windows destro.
- [ ] Win+E (Esplora file), Win+D (desktop), Win+Tab (Mission Control di EosOS), Win+L (blocca) funzionano come prima, senza Launcher.
- [ ] Dopo Win+L e lo sblocco, un tocco di Windows apre ancora il Launcher.
- [ ] Win + clic del mouse, o Win + rotellina (Lente): **non** si apre il Launcher.
- [ ] Impostazioni → Launcher → «Tasto Windows apre il Launcher» spento: il tasto torna al menu Start, subito.
- [ ] Il mouse non rallenta né scatta (c'è un hook in sola lettura sui pulsanti).

## 2. Mission Control e tastiera
- [ ] Alt+Tab: scorre le finestre, rilasciando Alt attiva quella evidenziata, Esc annulla. Alt+Maiusc+Tab all'indietro.
- [ ] Win+Tab apre e richiude la panoramica; dopo, il menu Start non compare.
- [ ] Ctrl+Alt+1..9 aprono la 1ª..9ª app del Dock.

## 3. Dock e monitor
- [ ] Un Dock per monitor, centrato in basso su ciascuno; si ridimensiona dalle Impostazioni.
- [ ] Trascinando una finestra da uno schermo all'altro il Dock non lampeggia né salta.
- [ ] Scollegando e ricollegando il secondo monitor il Dock sparisce e riappare da solo.
- [ ] Rimuovendo un'app dal Dock non si blocca; il Dock di Windows (barra) resta nascosto.
- [ ] Il menu «mela» (a sinistra della barra) e il Launchpad (icona in testa al Dock) si aprono e chiudono bene.

## 4. Barra dei menu
- [ ] Effetto vetro leggibile su sfondi chiari e scuri; testo sempre leggibile.
- [ ] Con VS Code in primo piano non compare il menu ripetuto dell'app (se lasci le impostazioni di base).
- [ ] Impostazioni: nome app e «Finestra» si accendono e spengono.

## 5. Sistema
- [ ] Riavvio di Explorer: `powershell -File scripts\test-launcher.ps1 -WithExplorerRestart` (chiude le finestre di Esplora file: salva prima).
- [ ] Avvio automatico al prossimo login; controlla l'elenco in Gestione attività.
- [ ] Spegni… e Riavvia… dal menu EosOS mostrano la finestra Sì/No di Windows (rispondi **No** se non vuoi spegnere).
- [ ] Tema chiaro: pannelli, barra e Dock leggibili.

## 6. Notifiche, pile, aggiornamenti
- [ ] Apri un'app o un file vero da una notifica e da una pila del Dock.
- [ ] Clic reale su «Installa» in un aggiornamento (serve prima un host pubblico per `latest.json`, vedi `AGGIORNAMENTI.md`).

## Decisioni aperte (tue)
- Dove pubblicare `latest.json` (il repo `LowSkyy/EosOS` è privato: il feed non è leggibile).
- Una copia sicura di `C:\Users\Raffaele\.tauri\eosos-updater.key` (senza, le copie installate non accettano più aggiornamenti).

## Come riportare i problemi
Scrivi la riga e cosa vedi (es. «1: Win+E apre anche il Launcher»), e su quale schermo. Per i problemi di tasti basta il nome della combinazione:
il punto da guardare è `mckeys::step` in `src-tauri/src/mckeys.rs` (logica pura, testata).

## 7. Gestione finestre (nuovo, spenta di base)
- [ ] Impostazioni → Mission Control → «Scorciatoie per disporre le finestre» → Ctrl+Alt: con una finestra qualsiasi (Blocco note, Chrome) prova ←, →, T, B, U/I/J/K, D/F/G, Invio, C, Backspace.
- [ ] PagGiù / PagSu la sposta sull altro monitor; la metà sul secondo schermo usa il suo spazio.
- [ ] Su app moderne (Impostazioni di Windows, Esplora file) la finestra si dispone senza bordi vuoti o sovrapposizioni.
- [ ] Nessuna combinazione ti disturba in altre app (es. Ctrl+Alt+← in un editor): in tal caso scegli un altro modificatore o spegni.
