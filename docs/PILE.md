# Pile nel Dock

Stato: **fatte** (8 nov 2026). Decisioni dell'utente: il pannello si apre come **griglia sopra l'icona**; all'inizio c'è **solo la pila Download**.

## Cosa fa
- Una **pila** è una cartella che sta nel Dock a destra delle app aperte, dopo un separatore (`.dock-separator-stacks`), con l'icona della cartella.
- Un **clic** apre il pannello (la stessa finestra `panel` degli indicatori, chiave `stack:N`) **sopra l'icona**: titolo, «Apri cartella» e una **griglia di 4 colonne** con i **40 file più recenti** (icona vera, nome su due righe); «+N altri» se ce ne sono di più. Un clic fuori, Esc o un secondo clic sull'icona lo chiudono.
- Si mostrano i file **visibili**: niente nascosti/di sistema, `desktop.ini`, `thumbs.db`, `.DS_Store`, `~$…`, `.crdownload`, `.part`. Dal più recente (data di modifica), a parità per nome.
- Un clic su un file lo **apre col suo programma**; le **cartelle** si aprono in Esplora risorse; i file che **eseguono codice** (exe, script, collegamenti, msi… — stessa lista del Launcher, `is_risky_file`) si **mostrano soltanto** in Esplora risorse.
- Fino a **5 pile**. Si scelgono dalle Impostazioni → Dock → **Pile** (elenco di cartelle con il selettore di Windows). Percorsi di rete e non assoluti sono rifiutati (come per il Launcher). Lista vuota = nessuna pila; mai scelto = solo Download.

## Come è fatto
- Rust: `stacks.rs` (parte pura testata: filtro, ordine, titolo, elenco effettivo; lettura cartella; id `stk:N` che valgono solo per gli ultimi file mostrati), comandi `dock_stacks` (solo Dock), `stack_list`, `stack_open`, `stack_open_folder` (solo pannello, vedi `cmdpolicy.rs`), impostazione `dock.stacks` (salvata in `mission.json` come `dockStacks`).
- Pagina: `StackIcon` in `Dock.tsx` (anche nella copia invisibile che misura la capsula), `StackPanel.tsx`, `lib/stacks.ts`.
- **Sistemato strada facendo:** un pannello ancorato sopra l'icona cresceva verso il basso coprendo l'icona e uscendo dallo schermo. Ora `fit` ricalcola la posizione con l'ancora salvata (`place` è pura e c'è un test: il fondo non si sposta quando cresce).

## Verifica
- `scripts/test-stacks.ps1` (30 controlli, cartella MIA in `%TEMP%`; la cartella Download vera non viene mai letta): pila di default presente; la mia cartella compare; pannello **sopra** l'icona e dentro lo schermo; ordine corretto; file di servizio e nascosti assenti; apertura di un file normale (registrata, non lanciata: `EOSOS_TEST_NO_OPEN`), di un programma (solo mostrato), «Apri cartella»; id inventati o percorsi al posto dell'id rifiutati; cartella vuota, mancante, 45 file (40 + «+5 altri»); percorso di rete e relativo rifiutati; massimo cinque pile; lista vuota. Regressioni: pannello, calendario, notifiche, indicatori, menu, impostazioni: OK. Rust 366 test, pagina 157, clippy 0.
- **Scale** (`scripts/test-stacks-dpi.ps1`, scala vera del primario a 100/125/150/175%, ripristinata a 100%): su ogni scala il pannello si apre sopra l'icona, dentro il monitor, largo 300 px CSS per la scala, centrato sull'icona (a 175% si ferma contro il bordo dello schermo), con la griglia dentro i bordi.
- **Non provato:** aprire davvero un file dal pannello (le prove non lanciano nulla); icone di cartelle con tanti file veri; cartelle su dischi lenti o di rete (rifiutate); un secondo monitor; clic destro sull'icona di una pila (non c'è un menu: la si toglie dalle Impostazioni); trascinare file nella pila.
