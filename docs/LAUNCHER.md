# Launcher — specifica

Obiettivo: un Launcher in stile Spotlight di macOS 27 "Golden Gate" per EosOS su Windows 11: una scorciatoia da tastiera apre un campo di ricerca al centro dello schermo e da lì si avviano app, si trovano file, si eseguono azioni e si ritrova ciò che si è copiato.

## Cosa fa Spotlight (da replicare) e da dove viene

Fonti: [Apple Support: cronologia degli appunti](https://support.apple.com/guide/mac-help/search-your-clipboard-history-mchl40d5b86b/mac), [Paste](https://pasteapp.io/blog/macos-tahoe-clipboard-history), [MacMost](https://macmost.com/how-to-use-the-spotlight-clipboard-history-in-macos-tahoe.html).

**Limite della ricerca**: per macOS 27 non ho trovato novità di Spotlight verificabili; le fonti descrivono Spotlight di macOS 26 (Tahoe) e dicono che in macOS 27 la cronologia degli appunti resta invariata. Quindi la specifica replica quel comportamento, non novità non verificate.

- Un pannello di vetro, in alto rispetto al centro dello schermo, con un campo di ricerca grande. Compare con una scorciatoia e sparisce con Esc o cliccando fuori.
- Risultati a elenco sotto il campo, con icona, nome e tipo; si naviga con le frecce e si apre con Invio.
- Quattro schede: **App**, **File**, **Azioni**, **Appunti**, richiamabili da tastiera (Cmd+1…4 su Mac; qui Ctrl+1…4).
- Calcoli e conversioni digitando nel campo (`12*7`, `20% di 340`).
- Cronologia degli appunti breve (ore, non archivio), con richiesta di permesso al primo uso.
- Le app usate più spesso salgono in cima ("frecency": frequenza e recenza).

## Scelte tecniche (proposte)

- **Finestra**: nuova finestra Tauri "launcher", senza bordi, trasparente, sempre in primo piano, centrata a un terzo dall'alto del monitor dove sta il cursore. A differenza del Dock **deve poter ricevere la tastiera**: niente `WS_EX_NOACTIVATE`; all'apertura si porta in primo piano (permesso, perché parte da una scorciatoia premuta dall'utente) e quando perde il focus si nasconde.
- **Vetro**: la stessa pipeline del Dock (`backdrop.rs`: cattura dello schermo dietro, sfocatura e ritaglio nel CSS, finestra esclusa dalla cattura). Per un pannello rettangolare arrotondato si può riusare così com'è.
- **Scorciatoia globale**: `RegisterHotKey`. Predefinita **Alt+Spazio** (come PowerToys Run; sostituisce il menu di sistema della finestra, che quasi nessuno usa). Va resa configurabile e, se la registrazione fallisce (combinazione già presa), il Launcher deve dirlo invece di non aprirsi in silenzio. `Win+Spazio` e `Ctrl+Spazio` sono occupate (lingua di input).
- **App**: si elenca `shell:AppsFolder` (programmi classici e app dello Store insieme, con nome, icona e identificatore), la stessa fonte che il Dock usa già per l'avvio e le icone (`SHCreateItemFromParsingName`, `IShellItemImageFactory`, `ShellExecute` su `shell:AppsFolder\...`). L'indice si ricostruisce in un thread, non all'apertura.
- **Sicurezza come nel Dock**: il frontend manda solo l'`id` di un risultato che il Launcher gli ha mostrato; il comando o il percorso da eseguire li ricava Rust. La pagina non può far eseguire stringhe arbitrarie.
- **Classifica**: corrispondenza approssimata (inizio di parola, sigle: `vsc` → Visual Studio Code, sottosequenza) più frecency salvata in `%LOCALAPPDATA%\EosOS\launcher.json`. Funzioni pure e testate, come `merge` nel Dock.
- **File**: partire da ciò che Windows già indicizza (Windows Search tramite `ISearchQueryHelper`/OLE DB `SystemIndex`) invece di riscansionare il disco; se l'indice non c'è si cade su file recenti e cartelle note. Mai scansioni ricorsive sul thread dell'interfaccia.
- **Azioni**: calcolatrice (parser di espressioni in Rust, senza `eval`), comandi di sistema (blocca, sospendi, riavvia, spegni: questi ultimi con conferma), apertura di Impostazioni e cartelle note.
- **Appunti**: `AddClipboardFormatListener` su una finestra nascosta; solo testo nella prima versione; cronologia **in memoria**, non su disco, con scadenza (ore) e limite di voci. Si ignorano le copie che i gestori di password marcano con `ExcludeClipboardContentFromMonitorProcessing` / `CanIncludeInClipboardHistory = 0`. Nessun contenuto negli log.
- **Impostazioni** in `launcher.json` (scorciatoia, schede attive, durata della cronologia), con la stessa robustezza di `dock.json`: tolleranza del BOM, file illeggibile conservato come `.bad`, scrittura atomica sotto lock, nessuna riscrittura dopo una lettura fallita.

## Decisioni dell'utente (7 nov 2026)

- Scorciatoia predefinita **Alt+Spazio**, configurabile (L2 la registra, L6 la rende configurabile).
- Cronologia degli appunti **solo in memoria**: mai su disco, con scadenza e limite di voci, e senza le copie dei gestori di password (L5).

## Fasi

1. **L1 – Dati e classifica (Rust)**: indice delle app da `shell:AppsFolder` (nome, id, icona), corrispondenza approssimata, frecency, comando `launcher_search`. Test unitari su classifica e punteggi.
2. **L2 – Finestra e scorciatoia**: finestra del Launcher con vetro, campo di ricerca e risultati con le frecce, scorciatoia globale con gestione del conflitto, apertura/chiusura (Esc, perdita del focus), avvio di un'app con Invio.
3. **L3 – Azioni e calcolatrice**: parser di espressioni, comandi di sistema con conferma, schede da tastiera.
4. **L4 – File**: ricerca nell'indice di Windows Search con ripiego.
5. **L5 – Appunti**: cronologia in memoria con esclusioni, scheda dedicata, richiesta di consenso al primo uso.
6. **L6 – Rifiniture**: scorciatoia configurabile, animazioni, più monitor (si apre dove sta il cursore), DPI, riavvio di Explorer, revisione di codice.

## L1 — Dati e classifica: FATTA (7 nov 2026)

Solo Rust, nessuna finestra ancora. Tre moduli:

- `launcherrank.rs` (puro, 14 test): `fold` (minuscole e senza accenti), `initials` (sigle, anche dalle maiuscole interne), `match_score`, `frecency`/`bonus`, `rank`.
  - Livelli di corrispondenza: nome identico 1000, inizio del nome 900, inizio di una parola 800, sigla 750 (`vsc` → Visual Studio Code), sottostringa 600, sottosequenza 100-560.
  - La sottosequenza è l'ultimo ripiego e ha limiti contro il rumore: la prima lettera deve cadere a inizio parola e le lettere trovate devono stare vicine (al più 3 volte la lunghezza della query). Senza questi limiti `paint` dava "Pannello di controllo" e `calc` "PC Health Check" (visto sull'indice reale). Un refuso (`calclatrice`) si perdona, una lettera estranea no.
  - Con una lettera sola contano solo l'inizio del nome e delle parole.
  - Frecency: logaritmica nel numero di usi, dimezzata ogni 14 giorni, bonus con tetto 120 punti: sposta un'app tra livelli vicini ma non scavalca una corrispondenza molto migliore (un'app identica al testo resta prima anche se un'altra è usatissima). A query vuota compaiono solo le app già usate, per frecency. A parità: nome più corto, poi alfabetico (ordine deterministico).
- `launcherstore.rs` (7 test): `launcher.json` con, per ogni app, solo numero di avvii e ultimo avvio (mai contenuti). Stessa robustezza di `dock.json`: BOM tollerato, file non JSON conservato come `.bad`, scrittura atomica con nome temporaneo per processo sotto lock, una lettura fallita NON è un file mancante e non porta a riscrivere nulla (`decide`/`KeepLast`). Si ricordano al massimo 500 app (`prune` scarta le meno recenti).
- `launcher.rs` (3 test + 2 sulla shell vera): indice da `shell:AppsFolder` (`IEnumShellItems`; id = nome di parsing relativo, nome = nome visualizzato), ripulito da voci vuote e doppioni, riletto ogni 5 minuti; una rilettura vuota non cancella un indice buono; riscaldato in un thread all'avvio. Icone dalla stessa cache del Dock.
- Comandi Tauri: `launcher_search(query, have)` e `launcher_launch(id)`. **Sicurezza**: la pagina manda solo un `id`; `launch` lo cerca nell'indice e avvia `shell:AppsFolder\<id>` solo se c'è (un percorso come `C:\Windows\System32\cmd.exe` non è avviabile). L'uso si registra solo se l'avvio è riuscito.

Verifica dal vivo: sull'indice reale (185 voci) `calc` → Calcolatrice, `note` → Blocco note, `vsc` → Visual Studio Code, `chr` → Google Chrome, `paint`/`zzzzqq` → nessun risultato; avvio vero della Calcolatrice (app di prova) con registrazione dell'uso e comparsa in cima a query vuota; poi chiusa e file di prova rimosso. Test totali 106 verdi (2 a richiesta), clippy a 0 avvisi.

Limiti noti: l'indice include anche voci di servizio (Cestino, manuali, "License Agreement", cartelle degli strumenti): vanno filtrate o declassate in L6, quando si vede come appaiono nella finestra.

## L2 — Finestra e scorciatoia: FATTA (7 nov 2026)

- `launcherwin.rs` (5 test sulle parti pure): finestra Tauri `launcher` creata nascosta all'avvio, senza bordi, trasparente, sempre in primo piano. Stili: `WS_EX_TOOLWINDOW` (niente Alt+Tab) e **senza** `WS_EX_NOACTIVATE`, perché deve ricevere la tastiera. Si apre sul monitor dove sta il cursore, 680 px CSS di larghezza (scalati con il monitor), centrata in orizzontale e a un quinto dell'area di lavoro dall'alto, mai fuori dall'area (`place`, pura). L'altezza la comunica la pagina (`launcher_fit`, limitata da `clamp_height`, che regge NaN e infiniti) e la cima resta ferma.
- **Scorciatoia**: `RegisterHotKey` Alt+Spazio (`MOD_NOREPEAT`) su un thread con ciclo di messaggi proprio; apre se chiuso e chiude se aperto. **Conflitto**: se la combinazione è già presa da un altro programma compare una sola volta un avviso (titolo "EosOS") e si riprova ogni 30 secondi, finché si libera; niente apertura muta.
- **Focus**: all'apertura `dockact::focus` (con l'Alt finto solo se Windows nega il primo piano) più `set_focus`; si ignora una perdita di focus nei primi 350 ms (oscillazione dell'apertura); dopo 500 ms, se il primo piano non è arrivato, il Launcher si richiude invece di restare inutilizzabile. Perdere il focus, Esc, Invio su un risultato o Alt+F4 lo nascondono (Alt+F4 non chiude EosOS: la chiusura della finestra del Launcher è intercettata).
- Pagina (`Launcher.tsx`, `lib/launcher.ts`, 12 test): campo di ricerca, elenco con icona, nome e tipo; frecce con giro, Invio, Esc, mouse (passaggio = selezione, clic = avvio). Ricerca a ogni tasto con numero di sequenza (una risposta in ritardo non sovrascrive la nuova), cache delle icone che solo cresce, riga "Nessun risultato", ruoli ARIA `combobox`/`listbox`. Durante la composizione IME Invio e frecce non vengono intercettati. A campo vuoto compaiono le app più usate. All'apertura il campo si azzera (evento `launcher-open`).
- Sicurezza invariata: la pagina manda solo l'`id`; `launcher_hide` e `launcher_fit` non accettano nulla di eseguibile.

**Scelta: vetro senza sfocatura reale per ora.** Il pannello usa la tinta del vetro del Dock (stessi token, tema chiaro/scuro) ma non lo sfondo sfocato catturato: `backdrop.rs` è oggi legato alla finestra del Dock (cattura intorno alla sua posizione, esclusione dalla cattura). Generalizzarlo a una seconda finestra va fatto con cura (e ha l'effetto collaterale di escludere il Launcher da screenshot e registrazioni), quindi è rimandato a L6.

Verifica dal vivo (`scripts/test-launcher.ps1`, build di prova, **tasti veri** e nessun clic; i tasti partono solo se il Launcher è davvero in primo piano): finestra creata nascosta con gli stili giusti; Alt+Spazio la apre, 680 px, centrata, a 213 px dall'alto (un quinto dell'area), in primo piano con il campo a fuoco; "calc" digitato → Calcolatrice prima riga e selezionata, altezza 140 px; Esc chiude; la riapertura azzera il campo; Invio avvia la Calcolatrice (app di prova), chiude il Launcher e registra l'uso; portare un'altra finestra in primo piano nasconde il Launcher; la stessa scorciatoia lo chiude. 22 controlli OK. Conflitto: con un altro processo che occupa Alt+Spazio compare l'avviso (una volta sola); liberata, il Launcher si apre dopo il nuovo tentativo.

Non provati: più monitor con scale diverse e riavvio di Explorer (previsti in L6), tastiere con editor di metodo di input, schermo con il tema scuro.

## L3 — Azioni e calcolatrice: FATTA (7 nov 2026)

- **Calcolatrice** (`launchercalc.rs`, parser a discesa ricorsiva, 10 test): `+ - * / ^`, parentesi, `x`/`×`/`÷`, decimali con virgola o punto, `sqrt()`, `abs()`, `pi`, e percentuali come Spotlight (`20% di 340` = 68, `340 + 10%` = 374). Nessun `eval`: legge solo numeri, operatori e due funzioni; profondità e lunghezza limitate (input ostile di 5000 parentesi o segni meno non manda in crisi nulla). Serve almeno un operatore o una funzione, quindi `42`, `50%`, `7-zip`, `chrome` non producono un "risultato". Divisione per zero, radice di negativi e risultati non finiti: nessun risultato. Il risultato si mostra con la virgola decimale (`84,5`), al massimo 10 decimali, notazione scientifica oltre 1e15; **Invio lo copia negli appunti** e chiude. Le conversioni di unità restano fuori (non richieste per ora).
- **Azioni** (`launcheract.rs`, 10 test): tabella fissa in Rust: Blocca lo schermo, Sospendi, Riavvia il computer, Spegni il computer, Impostazioni, cartelle Download/Documenti/Scrivania; cercabili per nome o parola chiave, in italiano e inglese (`reboot`, `settings`). La pagina manda solo l'id; comando e argomenti sono fissi nella tabella (un id sconosciuto o un percorso non si esegue).
- **Conferma gestita da Rust**, non dalla pagina: Riavvia e Spegni rispondono "confirm" al primo Invio e eseguono al secondo, solo se è la stessa azione ed entro 10 secondi. Si annulla se si digita, si cambia riga o si esegue un'altra azione; una conferma scaduta, un'altra azione o un orologio che torna indietro non confermano. La riga diventa rossa con "Invio per confermare".
- **Schede**: Ctrl+1 = solo App, Ctrl+2 = solo Azioni; ripremere la scheda attiva torna a tutto. Un'etichetta accanto al campo mostra la scheda attiva. (File e Appunti avranno le loro in L4 e L5.)
- **Ordine dei risultati**: il calcolo prima, poi app e azioni per punteggio; un'azione con lo stesso nome di un'app (Impostazioni) non compare due volte.
- **Sottosequenza più rigorosa** (`launcherrank.rs`): trovando rumore nell'indice reale (`calc` → "Cartella Scrivania", `chr` → "Crosshair X") la regola è ora: prima lettera a inizio parola e ogni lettera successiva consecutiva, oppure a inizio parola, oppure l'unico salto di una lettera (refuso); si cerca la combinazione migliore. `vscode` → Visual Studio Code e `clcolatrice` → Calcolatrice funzionano, `calc` non trova più cartelle.

**Sicurezza delle prove**: nelle build con la feature `test-hooks` le azioni di alimentazione (blocca, sospendi, riavvia, spegni) NON si eseguono mai: scrivono una riga in `%TEMP%\eosos-actions.log`. `scripts/test-launcher.ps1` rifiuta di partire se l'eseguibile non contiene quella protezione; i test unitari non chiamano mai l'esecuzione reale di azioni di alimentazione. Il comando reale è verificato solo come dato (tabella testata).

Verifica dal vivo (tasti veri, 50 controlli totali con L2): `12*7` → 84 copiato negli appunti (poi ripristinati); Ctrl+2 mostra solo azioni con l'etichetta, Ctrl+2 di nuovo la toglie, Ctrl+1 solo app; "riavvia" + Invio → riga rossa di conferma e nulla eseguito; digitare annulla la conferma e serve di nuovo; il secondo Invio registra l'azione una sola volta (non eseguita, build di prova); "Impostazioni" si apre senza conferma e la finestra è stata chiusa. Test: 131 Rust, 63 pagina, clippy a 0 avvisi.

Non provato dal vivo: l'esecuzione reale di blocca/sospendi/riavvia/spegni (volutamente: i comandi sono fissi e testati come dati); `Sospendi` usa `SetSuspendState` e su PC con ibernazione attiva può ibernare invece di sospendere (limite noto di quel comando).

## Fuori scopo

Anteprime dei file (Quick Look), Siri/IA, ricerca nel contenuto dei file, sincronizzazione degli appunti tra dispositivi, plugin di terze parti.

## Rischi aperti

- **Focus**: portare una finestra in primo piano da una scorciatoia globale funziona di norma, ma su alcune configurazioni Windows lo nega; serve il ripiego già usato dal Dock (`dockact::focus`), senza l'Alt finto se non serve.
- **IME e tastiera**: il campo di ricerca deve funzionare con le tastiere internazionali e gli editor di metodo di input (accenti, cinese/giapponese).
- **Indice dei file**: Windows Search può essere disattivato o incompleto (OLE DB `SystemIndex` non sempre disponibile).
- **Appunti**: privacy; cronologia solo in memoria e sempre svuotabile. Non deve mai registrare password.
- **Scorciatoia**: `Alt+Spazio` può essere già presa da altri programmi (PowerToys Run, launcher di terzi).
- **Tutti i test dal vivo sul PC reale** seguono la regola già nota: app di prova, nessun clic su icone vere dell'utente, bersaglio controllato prima di ogni clic.
