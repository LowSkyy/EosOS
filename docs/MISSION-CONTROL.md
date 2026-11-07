# Mission Control — specifica

Obiettivo: una panoramica in stile Mission Control di macOS 27 "Golden Gate" per EosOS su Windows 11: un tasto (o un angolo dello schermo) mostra tutte le finestre aperte, ridotte e disposte in una griglia sullo sfondo sfocato; si clicca una finestra per portarla avanti, in alto c'è la fascia dei desktop (Spaces).

## Cosa fa macOS (da replicare) e da dove viene

Fonti: [Apple Support, glossario "Mission Control"](https://support.apple.com/en-sa/guide/mac-help/aside/glos9edf2d32/mac), [AppleInsider, usare Mission Control](https://appleinsider.com/inside/macos/tips/how-to-use-mission-control-on-mac-to-enhance-your-productivity), [MacPaw, usare Mission Control](https://macpaw.com/how-to/use-mission-control-mac), [Apple Discussions, miniature dei Space](https://discussions.apple.com/thread/256239285).

**Limite della ricerca**: le fonti che ho trovato descrivono macOS 26 (Tahoe) e versioni precedenti. Un messaggio nelle Discussioni Apple parla di una modifica di macOS 27 (la fascia dei Space mostrerebbe i titoli delle finestre), ma è un'unica fonte non verificata: non la prendo come requisito. Anche gli angoli attivi sono descritti solo in modo generico.

- Si apre con F3, Ctrl+Freccia su, un gesto del trackpad o un angolo attivo.
- Tutte le finestre aperte sono ridotte e disposte in una griglia ("a mosaico") senza sovrapporsi, sullo sfondo del desktop attenuato; muovendo il cursore su una finestra si evidenzia con il suo titolo.
- Un clic su una finestra la porta in primo piano e chiude la panoramica; Esc chiude senza scegliere.
- In alto la **fascia dei Space**: una miniatura per ogni desktop e per ogni app a schermo intero; un clic passa a quel desktop; si trascina una finestra su un'altra miniatura per spostarla; il pulsante + in alto a destra aggiunge un desktop.
- Animazione: le finestre scivolano dalla loro posizione a quella nella griglia e viceversa.

## Cosa permette Windows (e dove sta il rischio)

- **Elenco delle finestre**: lo sappiamo già fare (il Dock enumera le finestre di primo livello visibili, non nascoste da DWM, non "tool window", con la gestione dei gusci delle app UWP in `dock.rs`). Va estratto e riusato.
- **Miniature vive**: l'API documentata `DwmRegisterThumbnail` disegna una finestra qualunque, **in tempo reale**, dentro una finestra del nostro processo, con un rettangolo di destinazione che si può cambiare a ogni fotogramma. È l'unico modo economico di avere miniature vive. Due conseguenze: (1) le miniature sono composte **sopra** il contenuto della nostra finestra, quindi la pagina web non può disegnare sopra una miniatura (niente pulsante di chiusura sovrapposto: lo mettiamo nella barra del titolo sotto o sopra la miniatura); (2) una finestra **minimizzata** non ha un'immagine viva: si mostra l'icona dell'app con il titolo.
- **Desktop virtuali**: qui il limite è serio.
  - L'interfaccia **documentata** `IVirtualDesktopManager` sa solo dire se una finestra è sul desktop corrente (`IsWindowOnCurrentVirtualDesktop`), a quale desktop appartiene (`GetWindowDesktopId`) e spostare **solo finestre del proprio processo**.
  - Elencare, creare, rimuovere, nominare i desktop, passare da uno all'altro e spostare finestre di altri programmi richiede interfacce **non documentate** (`IVirtualDesktopManagerInternal`): i loro identificatori e le firme **cambiano da una versione di Windows all'altra**. Fonti: il codice del crate Rust [winvd](https://docs.rs/crate/winvd/latest/source/src/interfaces.rs), che dichiara di funzionare dalla build 26100.2605, e i progetti .NET come [Slions.VirtualDesktop](https://packages.nuget.org/packages/Slions.VirtualDesktop/6.2.0), che avvertono che gli ID cambiano molto fra le versioni. Non ho trovato un elenco delle modifiche build per build. Questo PC ha la build 26300.
  - L'elenco dei desktop si può leggere dal registro (`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops`), ma è un formato non documentato.
- **Gesti del trackpad e tasto F3 di macOS**: non esistono su Windows. Servono una scorciatoia globale e un angolo attivo.
- **Win+Tab** è riservata a Windows (Visualizzazione attività) e non si può registrare.

## Scelte tecniche (proposte)

- **Una finestra panoramica per monitor**: finestra Tauri senza bordi, a tutto schermo del monitor, sempre in primo piano (sopra il Dock), con la stessa sfocatura del Launcher (fotogramma dello schermo dietro, finestra esclusa dalla cattura) più un'attenuazione scura. Mostra le finestre che stanno su **quel** monitor, come fa macOS con i Space separati per schermo.
- **Elenco e filtro (Rust, puro e testato)**: le finestre candidate (visibili o minimizzate, sul desktop corrente, non finestre di EosOS né della shell, con titolo o app riconoscibile) passano da un filtro estratto dal codice del Dock; ogni voce porta id, titolo, app, icona, rettangolo attuale e se è minimizzata. La pagina vede solo `id`; l'attivazione la fa Rust (riusando `dockact::focus`, con lo stesso ripiego se Windows nega il primo piano). Nessun HWND dalla pagina.
- **Disposizione a griglia (Rust, pura e testata)**: dato il rettangolo disponibile e le proporzioni di ogni finestra, calcola i rettangoli senza sovrapposizioni, con margini e dimensione massima, mantenendo l'ordine stabile (la stessa finestra resta nello stesso punto se l'elenco non cambia), per qualunque numero di finestre da 1 a molte decine. È la parte più sensibile alla qualità visiva e la più semplice da testare.
- **Miniature**: `DwmRegisterThumbnail` con rettangolo di destinazione animato da Rust (interpolazione dalla posizione reale alla cella, circa 200 ms); ritaglio ai bordi della cella; una finestra che si chiude o cambia mentre la panoramica è aperta aggiorna la griglia.
- **Chrome sulla pagina**: titolo, icona e pulsante di chiusura **fuori** dall'area della miniatura (per il limite di sovrapposizione), evidenziazione al passaggio del cursore con un bordo che circonda la miniatura. Chiusura di una finestra dalla panoramica: `WM_CLOSE` alla finestra (mai terminare un processo).
- **Come si apre**: tre modi, tutti attivabili e configurabili. (a) scorciatoia globale con `RegisterHotKey` (proposta più sotto), (b) **angolo attivo** (cursore nell'angolo in alto a sinistra del monitor per mezzo secondo; si disattiva con un'impostazione e si ignora se c'è un'app a schermo intero o una finestra in trascinamento), (c) voce nel Launcher ("Mission Control") e icona nel Dock se serve.
- **Fascia dei Space (in due livelli, per non legarsi a interfacce instabili)**:
  - *Livello sicuro, nelle fasi iniziali*: mostra solo le finestre del desktop corrente e il numero dei desktop letto dal registro; un clic su un altro desktop passa a quel desktop simulando la scorciatoia di Windows (Win+Ctrl+Freccia); il + crea un desktop con Win+Ctrl+D. Nessuna interfaccia non documentata, ma niente spostamento di finestre fra desktop.
  - *Livello completo, solo con l'ok dell'utente*: spostare finestre fra desktop e rinominarli con `IVirtualDesktopManagerInternal`, **solo su un elenco di build conosciute** (verificate dal vivo); su una build sconosciuta la fascia resta in sola lettura e lo dice. Va deciso a parte perché può smettere di funzionare con un aggiornamento di Windows.
- **Impostazioni** in `%LOCALAPPDATA%\EosOS` con la stessa robustezza degli altri file (BOM, `.bad`, scrittura atomica, nessuna riscrittura dopo una lettura fallita): scorciatoia, angolo attivo (acceso/spento, quale angolo), durata dell'animazione.
- **Sicurezza e privacy**: nessuna miniatura viene salvata o inviata; le miniature vive sono composte da Windows e la pagina non le vede mai (non finiscono nei log né nei file). Le finestre di EosOS e quelle marcate come escluse dalla cattura (per esempio i gestori di password) non si mostrano.

## Fasi

1. **M1 — Dati e disposizione (Rust, nessuna finestra)**: filtro delle finestre estratto dal Dock, modello dei dati, disposizione a griglia, hit-test e ordine stabile, comando `mc_windows`. Test unitari su disposizione (1, 2, 3, 7, 20, 60 finestre; proporzioni estreme; area minuscola) e su filtro (con elenchi finti).
2. **M2 — Panoramica**: finestra per monitor con sfocatura e attenuazione, miniature vive con `DwmRegisterThumbnail`, titoli e icone, evidenziazione, clic per attivare, Esc per chiudere, chiusura di una finestra dalla panoramica.
3. **M3 — Apertura e animazione**: scorciatoia globale con gestione del conflitto, angolo attivo, voce nel Launcher, animazione di ingresso e uscita.
4. **M4 — Più monitor, scale e Spaces sicuro**: una panoramica per monitor con la scala giusta, fascia dei desktop in sola lettura più passaggio con scorciatoia di Windows, finestre minimizzate con l'icona.
5. **M5 — (solo se decidi di farlo) Spaces completo** con interfacce non documentate e allow-list di build: spostare finestre fra desktop.
6. **M6 — Rifiniture**: impostazioni, prove su scale diverse e dopo il riavvio di Explorer, revisione del codice.

## Fuori scopo

Gesti del trackpad, App Exposé e "mostra desktop" con le finestre che si spostano sul bordo, app a schermo intero come Space separati, assegnazione delle app ai desktop, sfondo diverso per desktop, finestre di altri utenti o di sessioni remote.

## Rischi aperti

- **Miniature sopra la pagina**: `DwmRegisterThumbnail` compone sopra la nostra finestra, quindi niente elementi sovrapposti alle miniature. Se non basta, l'alternativa è catturare i fotogrammi con `Windows.Graphics.Capture` e disegnarli nella pagina: più pesante e più lavoro, da valutare solo se servisse.
- **Finestre particolari**: app UWP dietro `ApplicationFrameHost`, finestre "cloaked", finestre di sistema, finestre a schermo intero esclusivo, finestre con DPI diverso da quello del monitor, finestre senza titolo.
- **Focus e primo piano**: portare una finestra avanti dopo il clic può essere negato da Windows (ripiego già noto in `dockact::focus`).
- **Desktop virtuali**: vedi sopra. Il livello sicuro non basta per spostare finestre; il livello completo dipende dalla build e può rompersi con un aggiornamento.
- **Conflitti**: Win+Tab è riservata; una scorciatoia diversa può essere già presa (lo si dice, come per il Launcher). L'angolo attivo può scattare per sbaglio: tempo di permanenza, disattivabile.
- **Prestazioni**: molte finestre (60+) con miniature vive e animazione a 60 fps; si misura e, se serve, si riduce l'aggiornamento delle miniature fuori dal cursore.
- **Prove dal vivo sul PC reale**: la panoramica mostra anche le finestre vere dell'utente (solo a video, non vengono né salvate né catturate dalle prove). Le prove **non** cliccano sulle finestre vere: si usano finestre di prova create apposta (Blocco note vuoto, Calcolatrice, la finestra a strisce delle prove) e si verifica l'handle del bersaglio prima di ogni clic. Niente `SendKeys`; i tasti solo se la panoramica è in primo piano. Le prove sui desktop virtuali, se ci saranno, creano e rimuovono desktop **solo** di prova e solo con l'ok esplicito.
