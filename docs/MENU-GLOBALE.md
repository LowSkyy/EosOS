# Menu globale per app — specifica

Obiettivo: come su macOS, i menu dell'app in primo piano (File, Modifica, Visualizza...) compaiono **nella barra in alto di EosOS**, accanto al nome dell'app, invece che dentro la finestra. Un clic su una voce apre il menu a tendina e sceglierne una voce esegue il comando nell'app.

Stato: specifica (8 nov 2026) e **G1-G5 fatte**. **Scelta dell'utente: opzione 1, ibrido.** Resta G6.

## Cosa fa macOS (da replicare)

Non ho fatto una ricerca nuova per questa parte: è comportamento noto della barra dei menu di macOS, da confermare contro le fonti prima di costruire i dettagli.

- A sinistra, dopo il logo, c'è il **nome dell'app in primo piano in grassetto** e poi i suoi menu; cambia app e cambia tutta la fila.
- Il primo menu (quello col nome dell'app) contiene "Informazioni su...", impostazioni ed "Esci da...". Poi File, Modifica, Visualizza, Finestra, Aiuto, con le **scorciatoie** scritte a destra di ogni voce, **voci disattivate** in grigio, **segni di spunta**, **sottomenu** e separatori.
- Il menu si apre con un clic; con un menu aperto, passando sopra gli altri titoli si apre quello. Esc chiude.
- Senza finestre dell'app (solo Finder) restano i menu minimi.

## Cosa permette Windows (e dove sta il rischio)

Fonti trovate: la documentazione di Microsoft su [UI Automation, tipo di controllo Menu Bar](https://msdn.microsoft.com/de-de/library/ms752322(v=vs.85)) e [MenuItem](https://msdn.microsoft.com/en-us/library/ms746680): una barra dei menu è un contenitore di voci di menu leggibile da un altro processo, con un modello di invocazione. **Quello che la ricerca non ha confermato, e che quindi do solo come ipotesi da misurare:**

- **App con menu Win32 classici** (Blocco note, Esplora file vecchio stile, molte app storiche): dovrebbero essere leggibili, o con la finestra di UI Automation o direttamente dall'`HMENU` della finestra (`GetMenu`, `GetMenuItemInfo`).
- **App moderne senza barra dei menu** (Impostazioni, Calcolatrice, molte app dello Store), **app con il Ribbon** (Office) e **app Electron/Chromium** (VS Code, Discord, Teams): è probabile che non espongano nulla di simile a un menu leggibile, o lo espongano a modo loro (menu disegnati dall'app, non menu di sistema). **Non l'ho verificato.**
- Quindi, a differenza di macOS dove il menu è un obbligo per ogni app, qui **per una parte delle app non c'è nessun menu da portare in alto**.
- Rischi di sicurezza e correttezza: leggere e invocare voci di menu di altre app è automazione di altri processi. Con finestre di processi con privilegi più alti (UIPI) non funziona (come l'hook di M6), e invocare una voce a nome dell'utente va fatto solo su un clic esplicito, mai in automatico.

## Decisione che serve da te prima di costruire

La fattibilità cambia a seconda dell'ambizione. Le opzioni, con la mia raccomandazione:

1. **Ibrido (consigliato):** per le app di cui si riesce a leggere il menu, la fila completa nella barra; per tutte le altre, **solo il nome dell'app + un menu "Finestra" fatto da EosOS** (Riduci a icona, Ingrandisci, Chiudi, Sposta su un altro desktop, Mostra nel Dock) e il menu dell'app (Esci). Funziona sempre e non ti lascia una barra vuota.
2. **Solo nome + menu fatto da EosOS** per tutte le app: nessuna dipendenza da come sono fatte le app; poco "menu globale", ma affidabile.
3. **Fedeltà completa a tutti i costi:** provare a leggere anche Electron/Ribbon con trucchi (accessibilità, Alt per attivare la barra...): fragile, dipende da ogni app, non lo raccomando.

Domande aperte: (a) l'ibrido va bene? (b) il menu dell'app deve **sostituire** completamente la barra Alt dell'app (nasconderla non si può senza modificare la finestra dell'app: quindi la lascerei dov'è)? (c) scorciatoie: solo mostrarle o anche gestirle? (propongo solo mostrarle: l'app le gestisce già).

## Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova)

1. **G1 — Misura in sola lettura.** Uno strumento che, per una finestra, prova a leggere il menu in due modi (`HMENU` e UI Automation) e riferisce *cosa* si riesce a leggere, quanto ci mette e dove fallisce, **su app di prova e sulle app comuni del PC (stampando solo conteggi e nomi di programma, mai i testi dei menu dell'utente)**. Esito: una tabella "app tipo → leggibile sì/no" nella specifica. Se i numeri sono pessimi, ripieghiamo sull'opzione 2.
2. **G2 — Modello e disposizione (puri).** Albero di voci (titolo, scorciatoia, attiva, spuntata, sottomenu, separatore), taglio e ordinamento, i menu minimi di EosOS; tutto testato senza Windows.
3. **G3 — La barra.** Nome dell'app in grassetto e la fila dei menu accanto al logo/menu di EosOS, aggiornata al cambio dell'app in primo piano (la barra ha già il nome, vedi `foreground.rs`), con cache e senza ritardi percepibili.
4. **G4 — Tendina.** Apertura, passaggio fra i titoli, sottomenu, voci disattivate, spunte, Esc e tastiera.
5. **G5 — Invocazione.** Eseguire la voce scelta nell'app (solo su clic dell'utente), con verifica che la finestra sia ancora l'app giusta; errori gestiti senza chiudere né bloccare l'app bersaglio.
6. **G6 — Impostazioni e rifiniture:** attivazione/disattivazione, app escluse, scale diverse e secondo monitor, ripartenza di Explorer, revisione del codice.

## G1 — Misura in sola lettura: FATTA (8 nov 2026)

`menuread.rs` misura per ogni finestra, senza invocare nulla, due letture: l'`HMENU` classico (`GetMenu`) e la barra dei menu di UI Automation (con un tempo massimo di 1,5 s, in un thread a parte per poter abbandonare una finestra che non risponde). Il risultato contiene solo conteggi, tempi e nomi tecnici delle barre, mai titoli di finestra né testi di menu. Il comando `menu_probe_all` lo espone alle prove; `scripts/test-menu-probe.ps1` lo prova su finestre mie e sulle tue, riportando le tue **per programma** (solo nome del programma e conteggi).

**Difetti trovati misurando (e corretti):**
- UI Automation espone anche il **menu di sistema della barra del titolo** come "barra dei menu" con una sola voce (Calcolatrice: identificatore `SystemMenuBar`; console classica ed Esplora risorse: identificatore generico `MenuBar` con una voce). Contarla avrebbe dato un "menu" falso a quasi ogni app. Ora si scarta `SystemMenuBar` per nome e **una barra con meno di 2 voci non conta come menu dell'app**.
- Il Blocco note nuovo mostra due barre `MenuBar` nell'albero: si prende quella con più voci.

**Risultati su questo PC (build 26300.9818):**

| App | Come | Esito |
|---|---|---|
| Finestra con menu classico (WinForms `MainMenu`) | `HMENU` | **Leggibile**: 4 voci su 4, tutte con titolo, istantaneo; UI Automation ne vede le stesse 4 |
| Finestra con barra `MenuStrip` (.NET) | UI Automation | **Leggibile**: 3 voci su 3, nessun `HMENU` |
| **Blocco note** (Windows 11, WinUI) | UI Automation | **Leggibile**: 3 voci (File, Modifica, Visualizza), 54 ms |
| **Visual Studio Code** (Electron) | UI Automation | **Leggibile** (barra con voci, 12 ms) |
| Google Chrome, Discord | — | Nessun menu leggibile |
| Esplora risorse | — | Nessuno (la barra dei comandi non è un menu) |
| Calcolatrice (UWP) | — | Nessuno (solo il menu di sistema) |
| Console classica, `cmd` | — | Nessuno |

Tempi: UI Automation fra 3 e 112 ms, **nessuna finestra ha fatto scadere il limite** su circa 15 misurate. `mspaint.exe` e `wordpad.exe` non esistono su questo PC (non misurati).

**Cosa dice questo per l'ibrido:** la lettura funziona per una parte reale delle app, comprese alcune moderne (Blocco note) e una Electron (VS Code), contro la mia ipotesi prudente. Per le altre (browser, chat, Esplora, app dello Store) l'ibrido dà il nome più il menu fatto da EosOS, come previsto.

**Limiti della misura:** campione piccolo (aperte c'erano 3 tue finestre e le mie di prova): non so ancora come vadano Office, JetBrains, app Qt, Firefox, Teams e simili. Si legge la **struttura di primo livello**: i sottomenu e lo stato (attivo/spuntato/scorciatoia) non sono ancora misurati e si leggeranno in G2/G4. **Invocare una voce non è stato provato** (G5). Non so quanto siano stabili le letture quando l'app cambia menu mentre è in uso.

## G2 — Modello e disposizione (puri): FATTA (8 nov 2026)

`menumodel.rs`, 11 test, nessun Windows. Il modello è pronto ma **non è ancora collegato a nulla** (la barra è G3, la tendina G4): fino ad allora solo i test lo chiamano.

- **Voce di menu** (`Item`): titolo, scorciatoia (solo testo, la gestisce l'app), attiva, spuntata, separatore, sottomenu e azione, che è un comando di EosOS (`Cmd`: esci, riduci a icona, ingrandisci, ripristina, chiudi, mostra nel Dock) oppure `App(percorso)`, la posizione della voce nel menu originale dell'app, da invocare in G5.
- **Pulizia dei titoli** (`split_label`): toglie gli acceleratori `&` (`&&` resta una e commerciale vera), separa la scorciatoia dopo la tabulazione (`"&Apri...	Ctrl+O"` → `Apri...` e `Ctrl+O`), toglie i caratteri di controllo e accorcia a 60 caratteri con «…» senza spezzare un carattere.
- **Limiti** (`normalize`): voci senza titolo scartate; separatori iniziali, finali e ripetuti tolti; al massimo 40 voci per livello e 3 livelli (oltre, un sottomenu diventa voce semplice); i percorsi delle azioni non cambiano, così G5 trova sempre la voce giusta anche dopo i tagli.
- **L'ibrido** (`compose`): la fila della barra è il menu col **nome dell'app in grassetto** («Mostra nel Dock», «Esci da ...»), poi i menu letti dall'app (puliti; un menu vuoto o senza titolo non si mostra; al massimo 12 in tutto), poi «**Finestra**» fatto da EosOS (Riduci a icona, Ingrandisci o Ripristina secondo lo stato, Chiudi) **salvo che l'app ne abbia già uno** (riconosciuto in italiano, inglese, francese e tedesco). Senza un menu leggibile restano solo il nome dell'app e «Finestra»: la barra non è mai vuota.

Scelte da confermare: i comandi della prima versione sono quelli sopra; «Sposta su un altro desktop» è escluso (serve l'elenco dei desktop e il backend di M5, non ancora collaudato in scrittura).

## G3 — La barra: FATTA (8 nov 2026)

La barra in alto mostra ora il **nome dell'app in primo piano in grassetto** e, dopo, la **fila dei suoi menu**, con «Finestra» fatto da EosOS in fondo (l'ibrido). I titoli sono ancora solo etichette (la tendina è G4); i menu che si potranno aprire hanno già l'evidenziazione al passaggio del mouse.

**Come funziona**
- **Lettura** (`menuread::read_menus`, solo lettura): prima l'`HMENU` classico, **con tutte le voci** (sottomenu fino a 4 livelli, 100 voci per livello, separatori, voci disattivate e spuntate); se non c'è, la barra di UI Automation, di cui si leggono **solo i titoli di primo livello** e i menu restano «pigri» (voci vuote, da leggere all'apertura in G4). Tempo massimo 1,5 s per la lettura con UI Automation, in un thread a parte.
- **Conversione** (`menumodel::raw_to_top`, pura, testata): titoli ripuliti dagli acceleratori `&`, scorciatoia separata dopo la tabulazione, sottomenu senza azione, e per ogni voce di azione il **percorso nella posizione originale del menu dell'app** (primo indice = il menu di primo livello), che resta valido dopo ogni taglio del modello. I menu «pigri» restano nella barra anche con l'elenco vuoto.
- **Barra** (`menubar.rs`): quando cambia la finestra in primo piano si emette subito una riga di ripiego (nome + «Finestra»), poi un thread legge il menu e, se la finestra è ancora quella, emette la riga completa (evento `menubar-row`; comando `menubar_current` per la pagina che parte dopo). Ogni nuova richiesta scavalca quelle vecchie (cambiando finestra in fretta non si leggono quelle di mezzo); cache a tempo (5 s se il menu c'è, 3 s se non c'era). Sul desktop solo il nome. **L'hook del primo piano non fa altro che annotare e inviare**: leggere finestre altrui o emettere eventi da lì potrebbe bloccarlo.
- **Pagina**: `AppMenus.tsx` sostituisce il vecchio nome dell'app; la funzione pura `acceptRow` impedisce che una riga di ripiego sovrascriva una già letta della stessa finestra.

**Difetto vecchio trovato e corretto: il nome dell'app in barra poteva restare indietro per decine di secondi.** L'evento di sistema del primo piano (`SetWinEventHook`) in certi casi non arrivava (finestre rese attive da un altro processo): provato dal vivo, le prime tre finestre portate in primo piano non producevano nessun evento, mentre le successive sì; non ne ho trovato la causa. Ora un thread guarda la finestra in primo piano ogni 300 ms e aggiorna come farebbe l'evento (`update` è idempotente): la barra, e con lei il Dock che usa la stessa informazione, non dipendono più dall'evento da solo.

**Verifica dal vivo** (`scripts/test-menubar.ps1`, 17 controlli, finestre di prova mie; nessun clic, nessun tasto, nessuna voce invocata): finestra con menu classico → nome, File, Modifica, Visualizza, Aiuto, Finestra, con acceleratore tolto, primo menu in grassetto, «File» con 3 voci («Apri» con scorciatoia Ctrl+O, separatore, «Esci»), percorso dell'azione `0,0`, e la barra mostra gli stessi titoli; finestra con `MenuStrip` → File, Modifica, Aiuto «pigri»; console → solo nome e Finestra; tre cambi rapidi → vince l'ultima finestra e la barra mostra la sua riga; tornare su una finestra già letta → riga completa in 74 ms. Con Visual Studio Code in primo piano la barra mostra davvero i suoi menu (File, Edit, Selection, View, Go, Run, Terminal, Help) senza aver fatto altro. Regressioni: hook di tastiera 41, panoramica 58, misura dei menu: tutti OK. Rust 297 test (3 a richiesta), pagina 105, clippy a 0 avvisi.

**Non provato:** la barra con due monitor (una barra per monitor, stesso evento: dovrebbe coincidere, non l'ho guardato); il comportamento di app con menu che cambiano mentre sono in primo piano (la cache dura 5 s e si riprova solo al cambio di finestra); aspetto con tema chiaro (i colori vengono dalle variabili della barra); lettura di app molto grandi o lente oltre VS Code.

## G4 — La tendina: FATTA (8 nov 2026)

Un clic su un titolo della barra apre sotto di esso la **tendina del menu**; passando col mouse su un altro titolo mentre è aperta la tendina passa a quel menu; un secondo clic sullo stesso titolo, Esc o un clic fuori la chiudono. La tendina riusa la **finestra del pannello degli indicatori** (stessa sfocatura, stessi comportamenti di focus), con la chiave `menu:<indice>`: niente nuova finestra e nessun nuovo canale di cattura.

**Cosa mostra:** voci con la **scorciatoia a destra** (solo testo), **separatori**, voci **disattivate** in grigio e non selezionabili, **segno di spunta**, **sottomenu** che si aprono sul posto, rientrati (non a lato: la finestra è larga 300 px, un secondo livello a fianco richiederebbe una finestra in più; scelta da rivedere se si vuole la fedeltà a macOS), a tastiera frecce su/giù (saltano separatori e voci disattivate, senza girare in tondo), destra/sinistra per aprire e chiudere i sottomenu, Invio e Spazio per scegliere, Esc per chiudere. La logica di selezione sta in funzioni pure testate (`flatRows`, `moveSelection`, `firstSelectable`).

**Dove sta la decisione:** la pagina manda a Rust solo l'**indice del menu e il percorso della voce**; Rust ritrova la voce nella propria riga (`item_at`), controlla che esista, sia attiva e non sia un sottomenu né un separatore, e decide cosa fare. Indici o percorsi inventati vengono rifiutati.

**Comandi di EosOS, già funzionanti** (agiscono sulla finestra a cui si riferisce la riga, non su quella in primo piano, che durante la tendina è EosOS): *Riduci a icona*, *Ingrandisci*/*Ripristina* (la voce cambia con lo stato della finestra), *Chiudi* (`WM_CLOSE`, come il pulsante X) ed *Esci da ...* (`WM_CLOSE` a tutte le finestre visibili di primo livello dell'app: il modo gentile, ogni finestra può chiedere di salvare; mai si termina un processo). Tolta la voce «Mostra nel Dock», che non aveva un significato chiaro (il Dock mostra già le app aperte).

**Menu «pigri» (UI Automation):** le loro voci si leggono quando l'utente apre il menu: **si apre un istante il menu nell'app (ExpandCollapse) e si richiude**. È l'unico modo che l'accessibilità offre per le app che non espongono le voci a menu chiuso, ed è visibile come un lampo del menu nell'app (e, provato con Visual Studio Code, sposta il focus sull'app). Per questo il pannello ignora la perdita del focus mentre legge e lo riprende dopo (`busy_for`/`refocus`). Si legge **un solo livello** (le voci come discendenti del menu); i sottomenu di un'app UI Automation non sono ancora esplorati.

**Difetto trovato e corretto dalla prova:** l'apertura di un menu pigro faceva perdere il focus alla tendina, che si chiudeva da sola mezzo secondo dopo averla aperta; l'avrei attribuito al test, ma la causa vera era l'app che si prendeva il focus.

**Verifica dal vivo** (`scripts/test-menudrop.ps1`, 40 controlli, finestre di prova mie): tendina di un menu classico (4 voci con titoli giusti, separatore, scorciatoia `Ctrl+O`, voce disattivata, voce spuntata), apertura e chiusura dei sottomenu, passaggio da un titolo all'altro, secondo clic che chiude, **clic col mouse vero sul titolo** (verificato prima che sotto il cursore ci fosse proprio quel titolo), tastiera (frecce che saltano separatore e voce disattivata, Esc), scelta di una voce dell'app (per ora non fa nulla e non rompe nulla; indici e percorsi inventati, voci disattivate e sottomenu rifiutati), menu pigri di una finestra con `MenuStrip` (File mostra «Prima voce» e «Seconda voce» in 225 ms, la finestra resta viva), comandi di Finestra sulla console di prova (ingrandisce, ripristina, riduce a icona, chiude), «Esci da ...» che chiude solo l'app di prova e lascia viva l'altra. Regressioni: barra 17, pannello degli indicatori, hook di tastiera 41, tutti OK. Rust 300 test (3 a richiesta), pagina 108, clippy a 0 avvisi.

**Non provato:** il lampo del menu pigro su app vere diverse da Visual Studio Code e dalla finestra di prova (Blocco note di Windows 11: la lettura delle voci dopo l'espansione non è stata verificata; è il prossimo candidato); tendina vicino al bordo destro dello schermo e con due monitor (il posizionamento è quello del pannello, già provato); menu molto lunghi (il pannello scorre oltre 400 px: non l'ho riempito); menu con tema chiaro.

## G5 — Invocazione: FATTA (8 nov 2026)

Scegliere una voce dell'app dalla tendina la **esegue nell'app**. Solo su clic dell'utente e solo dopo aver controllato che la voce sia ancora quella mostrata.

**Come si esegue**
- **Menu classico (`HMENU`):** si rilegge la voce seguendo il percorso (posizione per posizione, sottomenu compresi), si controlla che non sia un separatore, un sottomenu o disattivata, e che il **titolo coincida ancora con quello mostrato**; poi si manda alla finestra il `WM_COMMAND` del suo identificatore, come se l'utente l'avesse scelta (`wm_command_wparam`: serve un id da 1 a 65535).
- **Barra di UI Automation:** si apre un istante il menu, si ritrova la voce (stesso controllo del titolo e dell'abilitazione) e la si invoca con `InvokePattern`; il menu si chiude da solo. Con tempo massimo di 2,5 s.
- **Prima** si chiude la tendina e si riporta in primo piano l'app (una finestra di dialogo che si apre deve comparire davanti), **poi** si esegue la voce.
- **Se il menu è cambiato** dopo che l'abbiamo letto (la voce a quel posto ha un altro titolo, o non c'è più) **non si esegue niente** e si dimentica la cache, così al prossimo cambio di finestra la barra rilegge. Dopo una voce eseguita si dimentica la cache lo stesso (le app cambiano spunte ed elenchi recenti).

**Limiti noti** (dichiarati, non nascosti): con UI Automation si eseguono solo le voci del **primo livello** di ogni menu (i sottomenu di quelle app non sono esplorati); l'`HMENU` ha invece tutti i livelli. Il comando arriva all'app come `WM_COMMAND` senza `WM_INITMENUPOPUP`: un'app che aggiorna lo stato delle voci solo all'apertura del menu potrebbe eseguire una voce già non valida (per le disattivate ci pensa il controllo di stato letto adesso). Le voci con id oltre 16 bit, o senza id (menu disegnati dall'app), non si eseguono.

**Verifica dal vivo** (`scripts/test-menuinvoke.ps1`, 23 controlli, **solo su due app di prova mie** che scrivono una riga in un log quando una voce viene eseguita): menu classico → scegliere «Apri» scrive la riga; la tendina si chiude e il primo piano torna all'app; una voce **dentro un sottomenu** («Recenti» → «uno», percorso di tre livelli) si esegue; la voce **disattivata** non si può scegliere (pulsante disattivato) **né chiamando il comando a mano** e non si esegue mai; **un'app che rinomina da sola una voce mentre la tendina è aperta**: scegliere la vecchia voce non esegue niente, cambiando finestra la barra rilegge, e la voce col nome nuovo si esegue; barra `MenuStrip` (UI Automation) → «Seconda voce» e poi «Prima voce» si eseguono ognuna una volta, l'app resta viva. Regressioni: tendina 40, barra 17, tutti OK. Rust 302 test (3 a richiesta), clippy a 0 avvisi.

**Non provato:** app vere (Blocco note di Windows 11, Visual Studio Code, Office): la lettura funziona, ma **non ho invocato voci di app dell'utente** (nessuna esecuzione su finestre vere, per la regola di non toccarle); come reagiscono le app a un `WM_COMMAND` senza apertura del menu (alcune app Win32 molto vecchie controllano lo stato in `WM_INITMENUPOPUP`); app con privilegi più alti (UIPI: il messaggio non arriva e `PostMessage` non lo segnala).

## Fuori scopo

Modificare le finestre delle altre app o nasconderne la barra dei menu; gestire noi le scorciatoie; menu contestuali (clic destro) delle app; barra dei menu di app a schermo intero esclusiva.

## Rischi aperti

- Per molte app moderne non c'è un menu leggibile (da misurare in G1): l'ibrido serve proprio a questo.
- L'accesso a una finestra bloccata o lenta può rallentare la barra: letture con tempo massimo, mai nel thread dell'interfaccia.
- Voci di menu che cambiano mentre sono aperte (elenchi recenti): si rilegge all'apertura.
