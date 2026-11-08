# Impostazioni di EosOS — specifica

Obiettivo: una finestra «Impostazioni di EosOS», in stile Impostazioni di Sistema di macOS, dove si cambiano **tutte** le preferenze di EosOS con il mouse. Oggi le preferenze ci sono, ma sono sparse e quasi nascoste: menu contestuali del Dock e della barra, voci del Launcher che accendono e spengono cose, e file JSON da modificare a mano.

Stato: **solo specifica (8 nov 2026), niente codice.** La roadmap precedente è finita (Dock, Launcher, Indicatori, Mission Control, Menu globale); questo è il passo che ho scelto come successivo perché è il buco più evidente in uso quotidiano. Se preferisci un altro (per esempio il Centro notifiche), dimmelo e riscrivo.

## Cosa fa macOS (da replicare) e da dove viene

Fonti trovate: [MacRumors, 50 novità di macOS Tahoe](https://www.macrumors.com/2025/09/24/all-the-new-macos-tahoe-features/), [TWiT, controllo della barra dei menu di Tahoe](https://twit.tv/posts/tech/take-full-control-your-macos-tahoe-menu-bar), [SlashGear, impostazioni da cambiare in Tahoe](https://www.slashgear.com/1977378/macos-tahoe-26-settings-to-change/), [Apple Support, glossario](https://support.apple.com/ta-in/guide/mac-help/glos200755a1/mac).

**Limite della ricerca:** le fonti sono blog e guide, non la documentazione Apple; non ho trovato dove stiano esattamente «Scrivania e Dock» e «Mission Control» nella barra laterale, e le fonti si contraddicono su dove stia l'opzione «barra dei menu opaca» (Aspetto o Barra dei menu). **Non riproduco l'ordine esatto delle sezioni di macOS**: uso la struttura che serve a EosOS, ispirata a quella.

- Una finestra con **barra laterale** a sinistra (sezioni con icona) e il contenuto della sezione a destra; una **ricerca** in cima alla barra laterale.
- In Tahoe la **Barra dei menu è una sezione propria** (controlli da mostrare, sfondo della barra); **Aspetto** ha tema e Liquid Glass; **Generale** ha le voci di login.
- Interruttori che agiscono subito, senza «Applica» né «Salva».

## Cosa c'è oggi (inventario)

| Cosa | Dove sta | Come si cambia ora |
|---|---|---|
| Dock: nascondi automatico, riserva spazio, livello di vetro, sfocatura, app fissate e ordine | `dock.json` | menu contestuale del Dock (Impostazioni), trascinamento |
| Indicatori: batteria, rete, volume, percentuale | `indicators.json` | clic destro sulla barra |
| Launcher: scorciatoia, cronologia appunti, cartelle di ricerca in più | `launcher.json` | voci del Launcher (3 scorciatoie fisse), cartelle **solo a mano** nel file |
| Mission Control: angolo attivo, scorciatoia (Ctrl+Alt+Su), Win+Tab e Alt+Tab di EosOS o di Windows | `mission.json` | voci del Launcher; scorciatoia **solo a mano** |
| Menu globale: acceso/spento, app escluse | `mission.json` | voci del Launcher; l'elenco delle escluse **non si vede da nessuna parte** |
| Profili dei desktop virtuali, rapporto diagnostico | `vdesk-profiles.json`, `vdesk-report.json` | file a mano / voce del Launcher |

Mancano del tutto: **avvio automatico con Windows**, un posto dove vedere cosa è attivo, una scorciatoia di Mission Control scelta dall'utente da interfaccia, la gestione dell'elenco delle app escluse dal menu.

## Principi (gli stessi del resto del progetto)

- **Rust decide, la pagina mostra.** La pagina manda a Rust solo una chiave e un valore; Rust ha un **elenco chiuso di impostazioni** (chiave, tipo, valori ammessi), valida ogni richiesta e la inoltra al deposito giusto. Una chiave o un valore sconosciuto viene rifiutato.
- **Un solo posto per ogni verità:** la finestra non ha un proprio file. Legge e scrive gli stessi depositi di oggi (`dock.json`, `launcher.json`, `indicators.json`, `mission.json`), con la stessa robustezza (BOM, copia `.bad`, scrittura atomica, niente riscrittura dopo lettura fallita). Le voci dei menu contestuali e del Launcher continuano a funzionare e restano coerenti, perché passano dagli stessi depositi.
- **Subito, senza «Applica».** Ogni interruttore usa il percorso che già applica la modifica dal vivo (è lo stesso dei menu di oggi).
- **Funzioni pure testate** per la parte logica (elenco delle impostazioni, validazione, ricerca); prove dal vivo con la finestra reale e con scale diverse; clippy a 0.

## Contenuto proposto (sezioni)

1. **Generale** — avvio automatico con Windows (nuovo, scrive la chiave `Run` dell'utente, mai di sistema; reversibile), lingua (solo italiano per ora, la voce c'è per dire che è così), «Mostra rapporto diagnostico».
2. **Aspetto** — livello di vetro e sfocatura (valgono per Dock, barra, Launcher e pannelli).
3. **Dock** — nascondi automaticamente, riserva spazio, app fissate (elenco con rimozione e ordine).
4. **Barra dei menu** — indicatori (batteria, rete, volume), percentuale della batteria, **menu globale** acceso/spento, **app escluse dal menu globale** (elenco visibile, con «Rimuovi»).
5. **Launcher** — scorciatoia (scelta fra quelle sicure, con la verifica che non sia già usata da altri), cronologia degli appunti (con il consenso come oggi), **cartelle di ricerca** (aggiungi/rimuovi con selettore di cartelle).
6. **Mission Control** — scorciatoia, angolo attivo, Win+Tab e Alt+Tab (EosOS o Windows), **desktop virtuali: stato del backend** (profilo scelto, esito dei controlli, apri il rapporto).

## Come si apre

- Voce «Impostazioni di EosOS...» nel menu col nome dell'app quando l'app in primo piano è il Desktop (oggi mostra solo il nome) e nei menu contestuali del Dock e della barra (oggi hanno ognuno le sue voci).
- Azione del Launcher «Impostazioni di EosOS» (con le sezioni come sinonimi: «dock», «vetro», «scorciatoia»...).
- Una sola finestra: se è già aperta, la porta davanti e passa alla sezione richiesta.

## Fasi

1. **S1 — Elenco e validazione (puri).** L'elenco chiuso delle impostazioni con chiave, tipo, valori ammessi e sezione; funzioni `get`/`set` validate sui depositi esistenti; ricerca di testo sull'elenco. Test per ogni chiave e per ogni valore non valido.
2. **S2 — La finestra.** Finestra Tauri con barra laterale, sezione Dock e Barra dei menu (le più usate), aggiornamento dal vivo quando la stessa cosa cambia da un menu contestuale; stile coerente con vetro e tema.
3. **S3 — Le altre sezioni** (Launcher con selettore di cartelle, Mission Control con scelta della scorciatoia e verifica dei conflitti, Aspetto).
4. **S4 — Avvio automatico** e sezione Generale, con verifica che si aggiunga e si tolga solo la chiave di EosOS.
5. **S5 — Ricerca e punti di ingresso** (Launcher, Dock, barra, menu del desktop).
6. **S6 — Rifiniture:** tastiera e accessibilità (etichette per lo screen reader, navigazione a tab), scale diverse, riavvio di Explorer, revisione del codice.

## Decisioni che servono da te

1. **Il prossimo passo è questo?** Alternativa: Centro notifiche (le notifiche di Windows nella barra, richiede il permesso di sistema per leggerle) o gestione delle finestre in stile Stage Manager. La mia raccomandazione resta questo: tutto il resto è già configurabile solo con trucchi.
2. **Avvio automatico con Windows:** lo includo (consigliato, riguarda solo la tua chiave utente e si disattiva da lì), oppure lo lascio fuori?
3. **Una finestra normale** (con barra del titolo) o **una finestra senza bordi in vetro** come Launcher e pannelli? Consiglio senza bordi in vetro, ma con una barra del titolo trascinabile e il pulsante di chiusura: coerente con il resto.

## Fuori scopo

Sincronizzare le impostazioni fra PC; temi personalizzati; modificare impostazioni di Windows (restano nelle Impostazioni di Windows: la finestra può solo aprirle); esportare/importare le preferenze.

## Rischi aperti

- Le impostazioni cambiate **da fuori** (file modificato a mano mentre la finestra è aperta) vanno rilette: serve un evento o una rilettura alla messa a fuoco.
- Scegliere una scorciatoia già usata da un altro programma: `RegisterHotKey` fallisce; la finestra deve dirlo prima di salvare, non dopo.
- Il selettore di cartelle di Windows è una finestra di un altro processo/thread: va aperto senza bloccare l'interfaccia.
