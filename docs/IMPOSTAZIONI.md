# Impostazioni di EosOS — specifica

Obiettivo: una finestra «Impostazioni di EosOS», in stile Impostazioni di Sistema di macOS, dove si cambiano **tutte** le preferenze di EosOS con il mouse. Oggi le preferenze ci sono, ma sono sparse e quasi nascoste: menu contestuali del Dock e della barra, voci del Launcher che accendono e spengono cose, e file JSON da modificare a mano.

Stato: specifica (8 nov 2026); **decisioni dell'utente (8 nov 2026): sì a tutto**, cioè questo è il passo successivo, l'avvio automatico con Windows è incluso (S4) e la finestra è senza bordi in vetro come Launcher e pannelli. **S1-S5 fatte.** La roadmap precedente è finita (Dock, Launcher, Indicatori, Mission Control, Menu globale); questo è il passo che ho scelto come successivo perché è il buco più evidente in uso quotidiano. 

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

## S1 — Elenco e validazione: FATTA (8 nov 2026)

**`settingsreg.rs`** (puro, 11 test): l'**elenco chiuso di 17 impostazioni** con chiave stabile, sezione, etichetta, aiuto, tipo e parole chiave per la ricerca; i valori (`Value`: sì/no, testo, elenco; dal JSON semplice `true`, `"standard"`, `["a","b"]`); la validazione per tipo; `set_value` (valida e poi scrive tramite un `Backend`); `snapshot` (lo stato di tutte, per la pagina) e `search`.

| Sezione | Chiavi |
|---|---|
| Dock | `dock.autohide`, `dock.reserve`, `dock.blur`, `dock.glass` (clear / standard / solid) |
| Barra dei menu | `bar.battery`, `bar.percent`, `bar.network`, `bar.volume`, `menu.enabled`, `menu.excluded` (elenco di app) |
| Launcher | `launcher.hotkey` (Alt+Spazio, Ctrl+Alt+Spazio, Ctrl+Maiusc+Spazio), `launcher.clipboard`, `launcher.folders` (elenco di cartelle) |
| Mission Control | `mc.hotkey`, `mc.corner`, `mc.alttab`, `mc.wintab` |

**Regole di validazione:** chiave non nell'elenco → rifiutata (anche una che sembra un percorso); tipo sbagliato (un testo al posto di un sì/no, un testo al posto di un elenco) → rifiutato; scelte (`Choice`) solo fra le opzioni; scorciatoia di Mission Control con le stesse regole delle altre di EosOS (serve un modificatore, Alt+F4 vietata) e scritta in **forma canonica** (`ctrl+alt+k` → `Ctrl+Alt+K`); elenchi **ripuliti** (spazi, doppi senza badare alle maiuscole, vuoti, elementi troppo lunghi; al massimo 100 app e 20 cartelle); cartelle di ricerca **assolute** (lettera di unità o `\\server`), senza `..`, senza caratteri vietati o di controllo, al massimo 260 caratteri; **una sola cartella non valida rifiuta tutto** e lo stato resta com'era. Un valore rifiutato non cambia mai lo stato.

**`settingslive.rs`:** il collegamento ai depositi veri. Ogni chiave passa dalle **stesse funzioni** che usano i menu contestuali e le voci del Launcher (`dockwin`, `indicatorstore`, `mcstore`, `launcherstore`, `menubar`, `mcwin`), quindi la finestra e quei menu restano sempre d'accordo e la modifica ha effetto subito dove può. Quattro funzioni di scrittura mancavano e sono state aggiunte con lo stesso schema robusto (`mcstore::set_hotkey`, `launcherstore::set_search_folders`, `menubar::set_menu_off`/`set_excluded`, `mcwin::set_corner`/`set_key`). Le **cartelle** si controllano anche per esistenza (la validazione pura guarda solo la forma). La scorciatoia del Launcher cambia subito e **si salva solo se la combinazione è libera** (altrimenti messaggio «già usata da un altro programma»). La scorciatoia di Mission Control **vale al prossimo avvio** (la risposta lo dice: `live: false`).

**Comandi per la pagina:** `settings_all` (stato di tutte), `settings_set(key, value)` (risposta `{ok, live, error}`; un valore di forma assurda non passa nemmeno la deserializzazione), `settings_search(query)`; evento `settings-changed` con la chiave dopo ogni scrittura riuscita.

**Verifica dal vivo** (`scripts/test-settings.ps1`, **67 controlli**, i quattro file dell'utente salvati e **ripristinati identici**, comprese le assenze): 17 impostazioni tutte con un valore e nell'ordine delle sezioni; valori predefiniti giusti; ogni interruttore scritto, riletto e rimesso com'era (dal vivo anche `dock.autohide`); livelli di vetro; scorciatoia del Launcher cambiata e **rimessa alla predefinita senza lasciare traccia nel file**, Alt+F4 rifiutata; app escluse ripulite, testo al posto di elenco rifiutato; cartella esistente accettata, **inesistente e relativa rifiutate con un messaggio**, stato invariato dopo i rifiuti; scorciatoia di Mission Control in forma canonica e segnata come non immediata, tasto nudo rifiutato; chiave sconosciuta, chiave «tipo percorso» e valori sbagliati rifiutati; ricerca («batteria» → indicatore per primo); evento `settings-changed` ricevuto. Rust 320 test (3 a richiesta), clippy a 0 avvisi.

**Non provato:** il rifiuto per **conflitto reale** della scorciatoia del Launcher (nessun programma di prova che occupi una delle tre combinazioni; la funzione è quella già provata in L4); `launcher.clipboard` acceso a lungo (si è solo acceso e spento); la reazione di altri programmi in ascolto agli eventi `dock-settings-changed` (solo il Dock).

## S2 — La finestra: FATTA (8 nov 2026)

Una finestra «Impostazioni di EosOS» senza bordi (angoli arrotondati disegnati dalla pagina, barra del titolo trascinabile, pulsante di chiusura), 780 × 560 px CSS, **al centro del monitor dove sta il cursore** (ridotta se il monitor è piccolo, mai fuori dai bordi: `settingswin::place`, pura e testata). Si apre dall'azione del Launcher «**Impostazioni di EosOS**» (parole chiave: preferenze, opzioni, dock, vetro, scorciatoie, menu globale, configurazione) o dal comando `settings_open(section)`; se è già aperta la porta davanti e passa alla sezione richiesta. **Esc** chiude (prima svuota la ricerca, se c'è), come il pulsante e Alt+F4: la nascondono soltanto, EosOS resta in esecuzione. Non compare in Mission Control né nella taskbar.

**Pagina** (`Settings.tsx`, funzioni pure in `lib/settings.ts` con 8 test): barra laterale con la ricerca e le quattro sezioni; a destra le impostazioni della sezione con etichetta, aiuto e controllo — **interruttore** per i sì/no, **menu a tendina** per le scelte (livello di vetro, scorciatoia del Launcher), **elenco con «×» e campo «Aggiungi»** per le app escluse dal menu globale; scorciatoia di Mission Control e cartelle di ricerca mostrate come testo (si modificano in S3). Ogni modifica agisce **subito** e passa da `settings_set`: se Rust la rifiuta la pagina mostra il messaggio sotto la riga e **rilegge lo stato vero**; se vale solo al prossimo avvio lo dice. La pagina non decide niente.

**Si aggiorna da sola** quando la stessa cosa cambia altrove: ascolta `settings-changed` (ogni scrittura da `settings_set`), `dock-settings-changed` (i menu del Dock) e rilegge quando la finestra torna in primo piano (gli indicatori e le voci del Launcher non emettono un evento proprio).

**Ricerca:** Rust dà le chiavi (`settings_search`), la pagina le mostra nell'ordine ricevuto con il nome della sezione accanto; senza risultati lo dice.

**Accessibilità:** ruoli `dialog`, `switch` con `aria-checked`, `navigation` per le sezioni con `aria-current`, `role="alert"` per gli errori, ogni riga etichettata, controlli raggiungibili con Tab con contorno di focus visibile.

**Verifica dal vivo** (`scripts/test-settingswin.ps1`, **35 controlli**, i quattro file delle impostazioni salvati e ripristinati identici): non si vede all'inizio; l'azione del Launcher la apre, **al centro di un monitor**, 780 × 560, con il primo piano; sezioni e righe giuste con etichetta e aiuto; **cliccare l'interruttore della sfocatura la cambia davvero nel deposito** (e un secondo clic la rimette); la scelta «Vetro opaco» e il ritorno a «standard»; **un cambio fatto da fuori compare nell'interruttore**; sezione Barra dei menu con le sei righe; **aggiungere un'app all'elenco delle escluse la scrive nel deposito, toglierla la toglie**; Launcher con tre scorciatoie; ricerca «batteria» (primo risultato con la sua sezione, nessuna sezione selezionata), ricerca senza risultati, Esc che svuota la ricerca senza chiudere; il pulsante la chiude, EosOS resta in esecuzione, `settings_open` con sezione la riapre a «Launcher», **Esc con la tastiera vera la chiude** (solo dopo aver verificato che fosse in primo piano); non compare tra le finestre di Mission Control. Controllato anche con uno screenshot reale della pagina (tema scuro, sezione Barra dei menu). Rust 322 test (3 a richiesta), pagina 116, clippy a 0 avvisi. Regressioni: impostazioni (S1) 67, Launcher, pannello degli indicatori: OK. Il test del Launcher ora accetta due azioni per «impost» (Impostazioni di Windows e di EosOS).

**Scelta di stile da conoscere:** a differenza di Launcher e pannelli la finestra **non ha lo sfondo sfocato** (la cattura dello schermo per la sfocatura è un canale condiviso con Dock, Launcher e pannello, e una finestra grande e trascinabile lo metterebbe sotto sforzo): usa il fondo del tema, opaco. Se lo vuoi sfocato si può aggiungere in S6.

**Non provato:** tema chiaro dal vivo (le variabili del tema ci sono, non l'ho guardato), scale diverse dal 100% (S6), trascinamento della finestra col mouse vero (l'attributo c'è e il permesso è concesso, ma non ho trascinato), lettore di schermo vero.

## S3 — Le altre sezioni: FATTA (8 nov 2026)

**Sezioni ora cinque:** Aspetto (sfocatura e livello di vetro, che valgono per Dock, barra e pannelli: spostate qui dal Dock, le chiavi `dock.blur` e `dock.glass` restano le stesse), Dock, Barra dei menu, Launcher, Mission Control. Le impostazioni sono **18** (si è aggiunta `dock.pinned`).

- **Scorciatoia di Mission Control, cambiata a caldo.** Nella pagina: «Cambia» → «Premi la nuova scorciatoia…» → si preme la combinazione (Esc annulla senza chiudere la finestra; un tasto da solo, o con il solo Maiusc, non basta e si resta in ascolto). La pagina scrive la combinazione dal codice fisico del tasto (`comboFromEvent`, pura e testata), Rust la valida con le regole di sempre e **prova davvero a registrarla**: se è occupata (da un altro programma o **dal Launcher**) resta quella di prima e compare il messaggio «Questa scorciatoia è già usata da un altro programma (o dal Launcher)». Lo stesso meccanismo del Launcher (messaggio al thread della scorciatoia, ritorno alla vecchia se la nuova non si registra). Non serve più riavviare.
- **Cartelle di ricerca del Launcher.** Elenco con «×» e **«Aggiungi cartella…»** che apre il **selettore di cartelle di Windows** (`folderpick.rs`, in un thread a parte con il suo COM, legato alla finestra delle impostazioni: non blocca nient'altro; annullarlo non è un errore). La stessa cartella scelta due volte non si duplica. Rust controlla che la cartella esista.
- **App fissate del Dock.** Elenco con ↑ ↓ ×: si riordina e si toglie; **non si aggiunge** da qui (per fissare un'app la si trascina nel Dock). Rust accetta solo nomi che sono davvero nel Dock (`plan_pins`, pura e testata: due app con lo stesso nome si distinguono per ordine, nessun nome inventato, nessun uso doppio) e applica togliendo e riordinando con le stesse funzioni del Dock.
- **Scorciatoia del Launcher:** le tre scorciatoie del Launcher restano un menu a tendina di scelte sicure (Alt+Spazio, Ctrl+Alt+Spazio, Ctrl+Maiusc+Spazio).

**Verifica dal vivo** (`scripts/test-settings-s3.ps1`, **35 controlli**, i quattro file dell'utente salvati e ripristinati identici; per le app fissate si usa un `dock.json` di prova con tre programmi di Windows):
- **Aspetto** è la prima sezione con due righe.
- **App fissate:** le tre app nell'ordine del file, frecce disattivate ai bordi; ↓ e ↑ cambiano l'ordine **nel deposito e nel file del Dock**; «×» toglie un'app; un nome inventato è rifiutato e non cambia nulla.
- **Cartelle:** il selettore (finto, per la parte di pagina) aggiunge la cartella scelta, compare con l'ultimo pezzo del percorso e finisce nel file del Launcher; una seconda scelta uguale non la duplica; «×» la toglie.
- **Scorciatoia:** si mostra quella attuale; un tasto da solo resta in ascolto; **Esc annulla senza chiudere la finestra**; Ctrl+Alt+K è accettata, mostrata e scritta nel file, **senza la nota del riavvio**; **dal vivo, premendo Ctrl+Alt+K con tasti veri si apre Mission Control** (con la finestra delle impostazioni in primo piano); Alt+Spazio (del Launcher) rifiutata con il messaggio e Alt+F4 rifiutata, e lo stato resta Ctrl+Alt+K; registrando Alt+Spazio dalla pagina compare l'errore; si torna alla predefinita Ctrl+Alt+Su, il file non la conserva più e **Ctrl+Alt+Su apre di nuovo Mission Control**.
- **Selettore vero** (app riavviata senza il selettore finto): la finestra «Scegli una cartella» di Windows compare davvero, e chiudendola il comando risponde «nessuna cartella» con EosOS ancora vivo.

Regressioni (aggiornate dove cambiavano le sezioni): impostazioni S1 (67), finestra S2 (35, ora con le sezioni nuove), Mission Control apertura/scorciatoie (39), Launcher: tutte OK. Rust 323 test (3 a richiesta), pagina 122, clippy a 0 avvisi.

**Non provato:** un vero conflitto con un programma esterno su una scorciatoia di Mission Control (il rifiuto è provato con quella del Launcher, che è dello stesso processo ma usa lo stesso meccanismo del sistema); il selettore con una cartella vera scelta col mouse (si è provata la pagina con il selettore finto e il selettore vero solo aperto e chiuso); riordino delle app fissate con il Dock che le ridisegna dal vivo (si è verificato il deposito e il file).

## S4 — Generale e avvio automatico: FATTA (8 nov 2026)

**Sezione «Generale»** (la prima della finestra, ora sei sezioni e **21** impostazioni):
- **«Apri EosOS all'accesso»** (`general.autostart`): scrive e toglie UN solo valore, `EosOS`, nella chiave `Run` **dell'utente** (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`): niente chiavi di sistema, niente attività pianificate, nessun altro valore. Il valore è il percorso di questo eseguibile fra virgolette. Rispetta Gestione attività: se la voce è stata disattivata da lì (`StartupApproved`), per le impostazioni risulta **spenta**, e riaccenderla da qui toglie il blocco. Rifiuta di accendersi se non trova il file di EosOS. Spegnere due volte, o accendere due volte, va bene.
- **«Lingua»** (`general.language`): solo Italiano; c'è per dire com'è.
- **«Rapporto sui desktop virtuali»** (`general.report`): un **pulsante** (nuovo tipo `Action`, senza valore) che scrive e apre il rapporto tecnico di M5 (versione di Windows ed esito dei controlli; nessun dato personale). `settings_action(key)` ha il suo comando e la sua validazione: un interruttore non si può premere come pulsante e un pulsante non si imposta come un valore.

**Logica in `autostart.rs`** (6 test con una chiave di prova sotto HKCU, mai quella vera): funzioni che lavorano su chiavi passate dal chiamante; scrivere/togliere tocca solo il valore `EosOS` (provato con un altro valore nella stessa chiave che resta identico), blocco di Gestione attività letto dal primo byte (dispari = disattivata), percorso relativo o inesistente rifiutato senza nemmeno creare la chiave.

**Verifica dal vivo** (`scripts/test-settings-s4.ps1`, **21 controlli**): Generale è la prima sezione con le tre righe e la lingua ha solo «Italiano»; all'inizio spento; **cliccando l'interruttore compare il valore `EosOS` nella chiave `Run` dell'utente, fra virgolette e con il percorso di questo eseguibile**; gli altri valori della chiave `Run` restano identici; con il blocco di Gestione attività risulta spento, riaccenderlo lo toglie; spegnere toglie il valore; valori non booleani e lingue inesistenti rifiutati; il rapporto viene scritto (senza aprirlo, con `EOSOS_TEST_NO_OPEN` nelle build di prova), contiene versione di Windows e controlli e **non contiene il nome utente**; ricerca «avvio automatico» → prima l'avvio all'accesso. Rust 331 test (3 a richiesta), pagina 122, clippy a 0 avvisi; le prove S1, S2, S3 aggiornate e passate.

### Incidente durante la prova (8 nov 2026)

**La prima esecuzione di `test-settings-s4.ps1` ha cancellato i valori di `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`.** Lo script usava `New-Item -Path <chiave> -Force` su quella chiave, che esisteva già: in PowerShell `-Force` su una chiave esistente **la ricrea vuota**. Quella chiave contiene, per ogni programma che parte all'accesso, un valore che Gestione attività usa per sapere se è **attivato o disattivato**. Prima della prova c'erano 9 valori (Medal, Spotify, Docker Desktop, electron.app.Vaydeer keyboard, Gemini, GoogleUpdaterTaskUser156.0.8067.0, Discord, LGHUB, GoogleUpdaterTaskUser157.0.8089.0); ora la chiave è vuota. **Un valore mancante vale «attivata»**: i programmi della chiave `Run` partono come prima, ma **quelli che l'utente aveva disattivato da Gestione attività ripartirebbero** al prossimo accesso. Non avevo una copia completa (salvavo solo il valore `EosOS`), e non c'è una copia di ripristino accessibile: **non si può ricostruire quali fossero disattivati**. Da rifare a mano: Gestione attività → App di avvio → disattivare quelle che non si vogliono. Lo script è corretto: niente `-Force` su chiavi esistenti, **esportazione completa delle chiavi con `reg export` prima** e `reg import` dalla copia se alla fine il registro non coincide. Regola salvata in memoria per le prove future.

## S5 — Punti d'ingresso: FATTA (8 nov 2026)

La finestra si apre ora da quattro posti (la stessa finestra: se è già aperta la porta davanti):
1. **Launcher:** azione «Impostazioni di EosOS» (già da S2).
2. **Barra, menu del nome:** la voce «**Impostazioni di EosOS…**» compare nel menu col nome quando **non ci sono menu dell'app da mostrare**: sul **Desktop**, con il **menu globale spento** o per un'**app esclusa**. Per questo ultimo caso è importante: da lì si può sempre riaccenderlo (prima, con il menu spento, il nome non apriva niente). Nelle altre app il menu del nome resta «Esci da ...».
3. **Barra, clic destro:** in fondo al menu degli indicatori, dopo un separatore.
4. **Dock, clic destro sullo sfondo:** in fondo al menu delle impostazioni del Dock, dopo un separatore.

Dettagli: la voce dei menu nativi ha un id fisso (`eos:settings`) e il gestore dei menu riconosce solo quell'id esatto (`is_menu_id`, testato); la voce del menu del nome è un comando di EosOS (`Cmd::OpenSettings`) che non riguarda nessuna finestra (si esegue anche se la finestra del desktop non risponde ai comandi). Si apre con 250 ms di ritardo, perché il menu o la tendina da cui parte finisca di chiudersi e non si riprenda il primo piano.

**Verifica dal vivo** (`scripts/test-settings-s5.ps1`, **12 controlli**, quattro file delle impostazioni ripristinati identici): con il menu globale spento la riga di una finestra di prova è solo il nome, si apre e ha l'unica voce, scegliendola si apre la finestra delle impostazioni; con il **Desktop** in primo piano la riga è «Desktop» con la stessa voce e scegliendola si apre; **menu nativo della barra e menu nativo del Dock**: si aprono davvero (verificato che il menu a comparsa sia di un processo EosOS prima di fare qualunque cosa), **si clicca col mouse la loro ultima riga** e si apre la finestra delle impostazioni; il menu non resta aperto e nessuna impostazione cambia per sbaglio. (La tastiera, Fine e Invio, non funzionava sul menu nativo perché la barra non è la finestra in primo piano: la prova usa il mouse.) Rust 332 test (3 a richiesta), clippy a 0 avvisi. Regressioni: barra, tendina, indicatori e impostazioni: OK. `test-menusettings.ps1` è fallito UNA volta su tre esecuzioni (7 controlli, tutti legati al ritardo con cui la barra segue la finestra in primo piano) e poi passato due volte di fila: **ha un'instabilità di tempi nota** (la barra impiega fino a 300 ms ad accorgersi del cambio di finestra), da irrobustire nelle rifiniture.

**Non provato:** la voce nel menu a tendina del Dock aperto da un clic reale (la prova chiama lo stesso comando e clicca il menu vero); l'apertura da tastiera dei menu nativi.

## Decisioni (prese dall'utente, 8 nov 2026)

1. **Il prossimo passo è questo?** Alternativa: Centro notifiche (le notifiche di Windows nella barra, richiede il permesso di sistema per leggerle) o gestione delle finestre in stile Stage Manager. La mia raccomandazione resta questo: tutto il resto è già configurabile solo con trucchi.
2. **Avvio automatico con Windows:** lo includo (consigliato, riguarda solo la tua chiave utente e si disattiva da lì), oppure lo lascio fuori?
3. **Una finestra normale** (con barra del titolo) o **una finestra senza bordi in vetro** come Launcher e pannelli? Consiglio senza bordi in vetro, ma con una barra del titolo trascinabile e il pulsante di chiusura: coerente con il resto.

## Fuori scopo

Sincronizzare le impostazioni fra PC; temi personalizzati; modificare impostazioni di Windows (restano nelle Impostazioni di Windows: la finestra può solo aprirle); esportare/importare le preferenze.

## Rischi aperti

- Le impostazioni cambiate **da fuori** (file modificato a mano mentre la finestra è aperta) vanno rilette: serve un evento o una rilettura alla messa a fuoco.
- Scegliere una scorciatoia già usata da un altro programma: `RegisterHotKey` fallisce; la finestra deve dirlo prima di salvare, non dopo.
- Il selettore di cartelle di Windows è una finestra di un altro processo/thread: va aperto senza bloccare l'interfaccia.
