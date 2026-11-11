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

## 8. Centro di controllo (nuovo)
- [ ] Il pulsante a due cursori (a destra, vicino all orologio) apre il pannello; si chiude con Esc o un clic fuori.
- [ ] Wi-Fi e Bluetooth: appaiono solo se il PC li ha; spegnere e riaccendere funziona (attenzione: spegnere il Wi-Fi taglia la rete).
- [ ] Non disturbare e Mantieni sveglio cambiano stato; con Mantieni sveglio acceso lo schermo non si spegne per inattività.
- [ ] Il cursore del volume muove il volume vero.

## 9. Badge delle notifiche sul Dock (nuovo)
- [ ] Impostazioni → Notifiche → «Mostra le notifiche» acceso: ricevi una notifica da un app fissata nel Dock (es. Discord): compare la pallina rossa col numero.
- [ ] Leggere o chiudere le notifiche la fa sparire (entro qualche secondo).
- [ ] Con Non disturbare acceso i badge spariscono; spegnendolo tornano.
- [ ] Un app che NON ha il badge pur avendo notifiche: dimmi quale (nome e icona), l abbinamento si puo allargare.

## 10. Cattura dello schermo (nuovo)
- [ ] Centro di controllo → «Schermo intero»: nasce un PNG in Immagini\Screenshots, si sente un suono e l immagine e negli appunti (incollala in Paint). Nell immagine ci sono anche Dock e barra dei menu.
- [ ] «Selezione…»: il velo scuro compare, si trascina un rettangolo, la misura si vede; Esc annulla; il file ha la misura scelta.
- [ ] Impostazioni → Mission Control → «Scorciatoie per la cattura dello schermo» → Ctrl+Maiusc: Ctrl+Maiusc+3 / 4 / 5 (schermo, selezione, finestra in primo piano).
- [ ] Su un secondo monitor, e con scala diversa dal 100

## 10. Cattura dello schermo (nuovo)
- [ ] Centro di controllo → «Schermo intero»: nasce un PNG in Immagini\Screenshots, si sente un suono e l'immagine è negli appunti (incollala in Paint). Nell'immagine ci sono anche Dock e barra dei menu.
- [ ] «Selezione…»: il velo scuro compare, si trascina un rettangolo, la misura si vede; Esc annulla; il file ha la misura scelta.
- [ ] Impostazioni → Mission Control → «Scorciatoie per la cattura dello schermo» → Ctrl+Maiusc: Ctrl+Maiusc+3 / 4 / 5 (schermo, selezione, finestra in primo piano).
- [ ] Su un secondo monitor, e con scala diversa dal 100%: la selezione ha la misura giusta.
- [ ] Nessun conflitto con altre app (Ctrl+Maiusc+3/4/5 in un editor): se ti dà fastidio scegli un altro modificatore.

## 11. Correzioni del 10 novembre (da provare a mano)
- [ ] Alt+Tab: il pulsante per chiudere una finestra ha una X vera.
- [x] Clic su un'app del Dock: nessun rettangolo «EosOS Dock» (provato dall'utente il 10 nov 2026 dopo la correzione di `chromeguard.rs`, vedi `docs/NOVITA-10-NOV.md`). Resta da guardare: passando il mouse sull'icona compare il nome in un'etichetta sopra di lei.
- [ ] Barra in alto: accanto al menu EosOS non c'è più il nome dell'app attiva (si riaccende da Impostazioni → Barra dei menu).
- [ ] Impostazioni: i menu a tendina (dimensione del Dock, scorciatoie…) si leggono bene sia con il tema scuro sia con quello chiaro.
- [ ] Dock: sei dimensioni; animazione all'apertura (rimbalzo / pulsazione / nessuna); ingrandimento al passaggio del mouse acceso/spento.
- [ ] Launchpad: solo applicazioni (niente cmd, PowerShell, Python, manuali, strumenti); pagine con puntini, rotellina, frecce, PagSu/PagGiù; Esc cancella la ricerca e poi chiude.
- [ ] Menu in alto a sinistra (il logo EosOS): le otto voci (Informazioni su EosOS, Impostazioni di EosOS…, Impostazioni di Windows…, Blocca lo schermo, Sospendi, Riavvia…, Spegni…, Esci da EosOS); il Centro di controllo non ha piu' i link alle impostazioni. Riavvia e Spegni chiedono conferma: rispondi «No».
- [ ] Barra in alto: compare l'icona «App in background» (una freccia su, a sinistra degli indicatori) con NVIDIA Broadcast, G HUB, AnyDesk…; un clic apre il pannello sotto l'icona e un clic su un'app la porta davanti. Nel Dock non c'è più. Si spegne in Impostazioni → Barra dei menu.
- [ ] Impostazioni → Windows: le schede (informazioni, aspetto, alimentazione, app all'avvio, archiviazione, schermi) funzionano; cambia il tema chiaro/scuro (attenzione: cambia davvero Windows) e rimettilo.

## 12. Schermata di blocco (nuova, 11 nov)
Prima di provarla: la via d'uscita di emergenza e' `Ctrl+Alt+Maiusc+Esc` (chiude EosOS e rimette la barra di Windows). Se qualcosa va storto, usala.
- [ ] Menu EosOS → «Blocca lo schermo» (o `Ctrl+Alt+L`): compare la schermata su **entrambi i monitor** (sfondo sfocato, data, orologio grande). Il mouse o un tasto fanno comparire avatar, nome e campo password.
- [ ] **Sblocco con la tua password VERA di Windows** (la prova che io non posso fare): scrivila e premi Invio: lo schermo si sblocca. Una password sbagliata fa «Password errata» e scuote il campo; dopo 5 errori dice «Riprova tra 5 secondi» e anche la password giusta aspetta.
- [ ] Mentre e' bloccato: il tasto Windows, `Alt+Tab`, `Alt+F4`, `Ctrl+Esc` e `Ctrl+Maiusc+Esc` non fanno niente; non si apre il Launcher (`Alt+Spazio`), ne' Mission Control, ne' le Impostazioni.
- [ ] `Ctrl+Alt+Canc` apre comunque la schermata di Windows (e' cosi' per tutti i programmi): lo sblocco di EosOS non protegge da quello. Con il Task Manager chiuso EosOS lo schermo si sblocca: e' il limite dichiarato in `docs/BLOCCO-SCHERMO.md`.
- [ ] Impostazioni → Generali: «Poi blocca anche Windows dopo» a 1 minuto: bloccando e aspettando, dopo un minuto la schermata EosOS sparisce e compare quella di Windows (accedi con la tua password).
- [ ] Impostazioni → Generali: «Blocca dopo l'inattivita'» a 5 minuti: senza toccare niente il blocco parte da solo.
- [ ] Se spegni «Schermata di blocco EosOS», «Blocca lo schermo» mostra la schermata di Windows.
- [ ] Windows Hello / PIN (solo su un PC che li ha): compare «Usa PIN o Windows Hello» e funziona. *Mai provato dal vivo.*

