# Semafori in stile macOS al posto dei tasti di Windows (11 nov 2026)

Su tutte le finestre con barra del titolo, chiudi / riduci a icona / schermo intero diventano i tre cerchi rosso, giallo e verde di macOS. Il simbolo (×, −, frecce) compare passando il mouse; le finestre non attive hanno i cerchi grigi.

## Come funziona (e perché non tocca le altre app)
Windows non permette a un programma di cambiare i tasti della barra del titolo di un'altra app. EosOS quindi disegna, per ogni finestra, una finestrina «a strati» senza focus (`src-tauri/src/lights.rs`) **sopra** i tasti di Windows: ha il colore della barra (letto dallo schermo), copre i tasti veri e disegna i cerchi. Il clic manda alla finestra lo stesso comando dei tasti veri (`WM_SYSCOMMAND`: chiudi, riduci, ingrandisci/ripristina). Le finestrine seguono la finestra (spostamento, ordine, attivazione, riduzione) e spariscono con lei.

Se EosOS si chiude o si blocca, le finestrine spariscono e **i tasti di Windows tornano da soli**: non è stato modificato nulla nelle altre app.

## Impostazioni (Aspetto)
- «Semafori al posto dei tasti di Windows» (acceso di base).
- «Lato dei semafori»: **a destra** (di base, ordine riduci, schermo intero, chiudi: copre solo i tasti di Windows) o a sinistra come macOS (capsula con i cerchi, che però nasconde icona, menu o prime schede dell'app: in VS Code copre «File», in Chrome l'inizio della prima scheda).
- «Stile Finder per Esplora file»: toglie il bordo sottile alle finestre di Esplora file (restano arrotondate); si ripristina spegnendo l'opzione.

## Limiti dichiarati
- **A sinistra i cerchi coprono quello che c'è all'inizio della barra** (icona, menu, prima scheda): EosOS non può spostare il contenuto delle altre app, per questo di base stanno a destra.
- **App con la barra del titolo disegnata da sé** (VS Code, Discord, Spotify, app Electron): Windows non dice dove sono i loro tasti, quindi EosOS li copre alla posizione standard di Windows 11 (3 tasti da 46×30 pixel in alto a destra). Se un'app li mette altrove, o è più alta/bassa, la copertura può essere sbagliata: segnalamelo con uno screenshot.
- «Schermo intero» (verde) = ingrandisci/ripristina la finestra, come il tasto di Windows: lo schermo intero vero lo gestiscono le singole app (F11).
- Il colore della barra si legge dallo schermo: con barre sfumate o con immagini la finestrina può differire di poco.
- Niente semafori su finestre senza barra del titolo di sistema (app «senza cornice»), finestre strumento, finestre a schermo intero (video, giochi) e mentre è attiva la schermata di blocco.
- **Esplora file non è ridisegnato**: barra laterale, barra dei comandi e percorso li decide Windows. Cambia solo il bordo e, con i semafori, la barra del titolo.

## Prove
Rust: ordine e disposizione dei cerchi, clic, colori, simboli, angolo arrotondato, comandi (10 test). Dal vivo, con una finestra di prova e clic veri (build `test-hooks`): a sinistra e a destra i cerchi compaiono al posto dei tasti, verde ingrandisce e ripristina, giallo riduce a icona, rosso chiude.

## Aggiornamento 12 nov: tasti di Windows tolti davvero (finestre normali)
Per le finestre con la barra del titolo di sistema (Esplora file, Impostazioni classiche, app Win32/WinForms/WPF...) EosOS toglie lo stile `WS_SYSMENU`: i tasti di Windows (e l'icona della barra) spariscono e restano solo i tre cerchi, senza riquadro sopra e senza colore da leggere. Il clic agisce sulla finestra (`WM_CLOSE`, `ShowWindowAsync`). Si rimettono con lo spegnimento dell'impostazione, alla chiusura di EosOS e durante il blocco dello schermo. **Rischio dichiarato:** se EosOS viene ucciso di colpo le finestre aperte restano senza i tasti di Windows finche' non si riaprono.
Chrome, Firefox, app UWP e Qt (barra disegnata dall'app) continuano a usare la finestrina sopra i tasti.
Provato dal vivo: finestra di prova (tasti spariti, cerchi centrati sul titolo, rosso chiude).

## Aggiornamento 12 nov (sera): semafori dentro le finestre
Prima ogni semaforo era una finestrina a parte che EosOS spostava e riordinava «a mano» dietro la finestra vera: nei trascinamenti e nei cambi di finestra poteva restare indietro di un fotogramma (tasti di Windows che riaffioravano, cerchi sopra la finestra sbagliata). Ora la finestrina e' **legata alla finestra dell'app**:

- **App con la barra disegnata da se'** (Chrome, Edge, VS Code, Discord, app Electron, app moderne): i cerchi sono un riquadro *figlio* dentro la finestra. Windows lo muove, lo ordina, lo ritaglia e lo anima insieme a lei: niente piu' ritardi. I clic li raccoglie una finestrina invisibile *posseduta* dalla finestra, sopra i cerchi (queste app si prendono tutti i clic della loro barra, anche sopra un riquadro figlio).
- **Finestre con la barra di Windows**: la finestrina e' *posseduta* dalla finestra: Windows la tiene sempre subito sopra di lei e la nasconde con lei (prima l'ordine si aggiustava ogni 16 ms).
- Un thread per finestrina: una finestra legata a quella di un'altra app ne condivide la coda di tastiera e mouse; con un thread solo, un'app bloccata avrebbe fermato i semafori (e l'input) di tutte. Il thread non fa lavoro, passa i messaggi del mouse al thread dei semafori.
- Il manifesto dell'eseguibile (`src-tauri/app.manifest`) dichiara Windows 8+: serve per i riquadri figli «a strati».

Altre correzioni dello stesso giro:
- **Verde/giallo/rosso sui browser**: Chrome, Edge e le app Electron ignorano il comando della barra (`WM_SYSCOMMAND`) mandato da fuori: il verde non ingrandiva. Ora si agisce sulla finestra (`ShowWindowAsync`, `WM_CLOSE`) per tutte.
- **Colore sbagliato («patacca») sulle finestre coperte**: il colore della barra si leggeva dallo schermo anche quando sopra c'era un'altra finestra. Ora si legge solo se in quel punto si vede davvero la finestra giusta.
- **Tasti di Windows persi dopo un riavvio di EosOS**: le finestre a cui sono stati tolti i tasti portano una proprieta' (`EosOS.Bare`); un EosOS riavviato le riconosce, e il processo di guardia rimette i tasti se EosOS muore di colpo (prima restavano senza tasti, o con una pezza sopra).
- **Finestre avviate come amministratore**: non si lasciano toccare; niente semafori (prima comparivano ma non funzionavano).
- Passando il mouse i cerchi si colorano e mostrano i simboli anche sulle finestre non attive, come in macOS.
- Finestre ingrandite: i cerchi partono dalla parte visibile della barra.

Provato dal vivo su una finestra di Edge con profilo temporaneo: cerchi dentro la finestra con il colore giusto, simboli al passaggio, verde ingrandisce e ripristina, giallo riduce, rosso chiude.
