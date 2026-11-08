# Menu globale per app — specifica

Obiettivo: come su macOS, i menu dell'app in primo piano (File, Modifica, Visualizza...) compaiono **nella barra in alto di EosOS**, accanto al nome dell'app, invece che dentro la finestra. Un clic su una voce apre il menu a tendina e sceglierne una voce esegue il comando nell'app.

Stato: **solo specifica (8 nov 2026), niente codice.** Prima di scrivere la parte di invocazione c'è una fase di sola lettura (G1) che misura cosa si riesce a leggere davvero sulle app di questo PC: la fattibilità dipende da quel risultato.

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

## Fuori scopo

Modificare le finestre delle altre app o nasconderne la barra dei menu; gestire noi le scorciatoie; menu contestuali (clic destro) delle app; barra dei menu di app a schermo intero esclusiva.

## Rischi aperti

- Per molte app moderne non c'è un menu leggibile (da misurare in G1): l'ibrido serve proprio a questo.
- L'accesso a una finestra bloccata o lenta può rallentare la barra: letture con tempo massimo, mai nel thread dell'interfaccia.
- Voci di menu che cambiano mentre sono aperte (elenchi recenti): si rilegge all'apertura.
