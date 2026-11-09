# Rifiniture dopo le prove a mano (9 nov 2026)

Dalle prove a mano dell'utente: Dock «grossolano», angoli della capsula brutti, icone «buttate su», Dock che «si blocca»
quando si toglie un'app, barra dei menu senza il vetro liquido, barra di Windows ancora visibile, Impostazioni di Windows
da avere dentro quelle di EosOS, Dock su più schermi e ridimensionabile, barra dei menu con troppe voci (nome dell'app,
Finestra, menu di Visual Studio Code ripetuti), reattività generale.

## Dock
- **Bordi lisci.** La capsula aveva gli angoli «a gradini» perché la finestra era ritagliata con una regione arrotondata di
  Windows, che non ha bordi lisci. Ora la zona cliccabile è un rettangolo appena più largo (3 px) e gli angoli arrotondati li
  disegna il CSS con i bordi lisci (provato con uno zoom sulla cattura dello schermo prima e dopo).
- **Icone più curate.** Misure di base riviste (icona 50 px, margine 12, spazio 8, raggio 24) e **angoli arrotondati** sulle
  icone con sfondo quadrato pieno (molte app di Windows): le icone con forma propria (cerchi, cartelle) non cambiano.
- **Dimensione a scelta:** Piccolo / Medio / Grande / Molto grande (0,82 / 1 / 1,2 / 1,4), da Impostazioni → Dock e dal menu del
  clic destro sulla capsula; vale per tutti i Dock. Le misure restano a pixel interi.
- **Un Dock per monitor.** «Dock su tutti gli schermi» (acceso di base; solo sul primario se lo spegni, anche dal menu del clic
  destro): ogni monitor ha il suo Dock centrato in basso, con la sua sfocatura dello sfondo (canale proprio) e la sua riserva di
  spazio (una fascia appbar per Dock). Le stesse app e le stesse pile su tutti. Il Dock di un monitor scollegato sparisce.
- **«Si blocca togliendo un'app».** Non si bloccava: l'icona sparisce solo al giro successivo dell'elenco (ogni 2 s) e il Dock
  sembrava fermo. Misurato: **1769 ms prima, 158 ms dopo**. Ora l'elenco si rilegge subito a ogni cambio delle impostazioni, a ogni
  cambio di app in primo piano (evento di sistema) e ogni secondo (era ogni 2).

## Barra dei menu
- **Vetro liquido** come nel Dock e nei pannelli: lo schermo dietro ogni barra (una per monitor) viene catturato, sfocato e
  saturato nella pagina, con una tinta leggera che **si adatta alla luminosità dello sfondo** (testo sempre leggibile) e un
  filo di luce sul bordo basso. Con la sfocatura accesa (impostazione unica del Dock, anche «Aspetto → Sfocatura dello sfondo») la
  barra è esclusa dalle catture dello schermo, come il Dock; spenta torna l'acrilico di Windows.
- **Meno voci:** «Mostra il nome dell'app» (acceso di base), «Mostra il menu Finestra» (**spento di base**, vale anche per quello
  dell'app), e le app che disegnano il menu dentro la finestra (**Visual Studio Code, Cursor, Slack, Spotify, Obsidian, Notion,
  Figma, Postman, WhatsApp, Telegram, Signal…**) non vengono più lette: restano nome e basta. «Menu globale» (già c'era) spegne tutti
  i menu delle app.

## Windows
- **Barra delle applicazioni di Windows nascosta di base** (impostazione «Nascondi la barra delle applicazioni di Windows», vale
  dal prossimo avvio; Ctrl+Alt+Maiusc+Esc la rimette in ogni momento; il watchdog la ripristina se EosOS si chiude). Le build di
  prova non la nascondono mai da sole.
- **Impostazioni di Windows dentro quelle di EosOS:** nuova sezione con 41 pagine raggruppate come in Windows 11 e una ricerca
  (senza badare ad accenti e maiuscole); un clic apre la pagina vera (`ms-settings:`). Le pagine di Windows non si possono incorporare:
  EosOS le apre. L'elenco è una tabella fissa in Rust (la pagina manda solo un id).

## Reattività
Misurato (build di prova, non ottimizzata): apertura del pannello sotto l'orologio ~150 ms (con e senza la riduzione di memoria
delle pagine nascoste); `launcher_search` 2–4 ms; `dock_items` ~10 ms; **fluidità del Dock al passaggio del mouse 6,9 ms per
fotogramma (144 Hz), zero fotogrammi oltre 25 ms, con la sfocatura accesa e spenta**. Il resto della «poca reattività» era il ritardo
dell'elenco delle app (sopra). Costo del vetro della barra a riposo: circa +0,5 s di CPU ogni 30 s sul processo principale (build
ottimizzata), per questo le barre si catturano al massimo ogni 0,5 s.

- **Build di produzione provata a mano (9 nov 2026):** senza nessuna variabile la barra di Windows risulta nascosta dopo l'avvio e, uccidendo il processo principale, il watchdog la rimette **visibile** entro 6 secondi (nessun processo rimasto).

## Prove
`test-dock-remove.ps1` (1769 → 158 ms), `test-dock-multi.ps1` (2 monitor: posizione, app, sfocatura, dimensione, spegni/riaccendi),
`test-bar-glass.ps1` (su entrambe le barre, esclusione dalle catture, adattamento, spegni/riaccendi), `test-menu-view.ps1`,
`test-winsettings.ps1`, `shot-dock.ps1` (foto del Dock), `measure-open.ps1`, `measure-hover.ps1`. Prove esistenti adattate
(conteggi delle impostazioni, misure dell'ingrandimento, il menu «Finestra» ora acceso dentro le prove che lo riguardano).

- **Due Dock a scale diverse provati** (`test-dock-multi-dpi.ps1`, scale vere cambiate e ripristinate a 100%/100%): primario 100→125% e secondario 100→150%, dal vivo e a freddo: ogni Dock dentro il proprio monitor, centrato in basso, con la scala del proprio monitor e la sua sfocatura.

**Non provato:** l'effetto a occhio su sfondi molto chiari, il clic reale sulle voci nuove del menu del Dock.

## Scorciatoie per le app del Dock (9 nov 2026)
Premendo **Ctrl+Alt+1…9** (di base) si apre la 1ª…9ª app del Dock, come un clic sulla sua icona (si avvia, si porta davanti o si
minimizza se è già davanti). Con la barra di Windows nascosta i tasti Windows+numero non funzionano più: questi li sostituiscono. Il
modificatore si sceglie in Impostazioni → Dock (**Ctrl+Alt**, **Ctrl+Maiusc**, **Alt+Maiusc** o disattivate) e vale subito.
- **Ctrl+Windows+numero non si può usare:** su questo PC Windows lo tiene riservato (nessun altro processo riesce a registrarlo, anche con EosOS spento): provato e scartato.
- Se una combinazione è già presa da un'altra app, quella sola non si registra (senza errori).
- **Prova** (`test-dock-keys.ps1`, nessun tasto premuto): nelle build di prova di base sono spente; accese risultano **registrate** (un altro processo riceve l'errore 1409 «già presa» su tutte e nove), cambiano e si spengono dalle Impostazioni, una scelta inventata è rifiutata; la pressione del numero 1 simulata dal comando di prova avvia il Blocco note messo come prima app (solo per la prova, ripristinato) e chiude solo quello nuovo.
- **Non provato:** la pressione vera dei tasti (la tastiera non si simula), e conflitti con app che usano le stesse combinazioni.

## Tema chiaro e leggibilità della barra (9 nov 2026)
Guardando il tema chiaro forzato (`scripts/shot-light.ps1`: foto delle pagine con `data-theme` chiaro e scuro) il testo della barra
risultava scuro su sfondo scuro. Ora **il colore del testo si sceglie dal contrasto vero**, calcolato sullo sfondo dietro la barra
mescolato con la tinta (luminanze lineari, come WCAG), non dal tema: chiaro o scuro, il testo resta leggibile su qualunque sfondo. Test
puro: **almeno 4,5:1 per ogni tema e per undici livelli di luminosità dello sfondo**. La tinta scura è un po' più forte sugli sfondi
chiari (fino a 0,70). Si ricalcola anche se il tema del sistema cambia a barra aperta. Impostazioni, Dock e pannelli in tema chiaro
guardati a occhio: a posto (il pannello sotto l'orologio è nascosto nelle foto, non confrontato).

## Menu EosOS e Launchpad (9 nov 2026) — il «menu mela» e la griglia delle app
macOS non ha un menu Start: ha il **menu mela** (a sinistra della barra) e il **Launchpad** (tutte le app in griglia). Fatti entrambi.
- **Menu EosOS** (pulsante a sinistra della barra, prima dei menu dell'app): Informazioni su EosOS (versione di EosOS e di Windows),
  Impostazioni di EosOS…, Impostazioni di Windows…, **Launchpad**, Mission Control, Blocca lo schermo, Sospendi, **Riavvia…**,
  **Spegni…** (chiedono comunque il «Sì» in una finestra di Windows) ed Esci da EosOS. Frecce su/giù, Invio, Esc. `test-eosmenu.ps1`.
- **Launchpad**: tutte le app in ordine alfabetico, a griglia, a tutta area di lavoro (è la finestra del Launcher in un'altra
  modalità, con lo stesso sfondo sfocato), con ricerca (senza badare ad accenti e maiuscole), frecce, Invio, Esc o un clic nello
  sfondo per chiudere. Si apre dall'**icona in testa al Dock** o dal menu EosOS. Le app arrivano a pagine da Rust (le prime subito).
  `test-launchpad.ps1` (apertura, copertura del monitor, ordine, ricerca, avvio di un Blocco note mio, chiusura, comandi vietati
  alle altre finestre).

## Il tasto Windows apre il Launcher (9 nov 2026)
Premere e rilasciare **il tasto Windows da solo** apre (o chiude) il Launcher al posto del menu Start. Impostazione
**«Tasto Windows apre il Launcher»** (sezione Launcher, **accesa** di base); spenta, il tasto torna a Windows subito.
- Come funziona: la macchina a stati dell'hook (`mckeys.rs`) segna Win come «tocco» solo se, tra la pressione e il rilascio, **non
  arriva nessun altro tasto** e nessun Ctrl/Maiusc/Alt era già premuto. Win+E, Win+D, Win+Maiusc+S, Win+Tab, Win+Ctrl+D… restano a
  Windows, intatti. Le ripetizioni del tasto tenuto non contano; dopo un'altra combinazione le ripetizioni non riarmano il tocco.
- Al rilascio vero di Win si inserisce il tasto neutro (lo stesso trucco già usato per Win+Tab) così Windows non apre il menu Start.
- Prove: 5 test puri sulla macchina a stati (tocco, tasto destro, ripetizioni, 7 altri tasti, Win+Tab, modificatori già premuti,
  impostazione spenta) e sul protocollo (`cfg a w t`). **Non provato:** con una tastiera vera (non posso simulare tasti premuti).
  Da provare a mano: tocco di Win → Launcher; Win+E/Win+D/Win+Tab come prima; spegnendo l'impostazione il menu Start torna.
- Resta ancora il menu Start con Ctrl+Esc (non toccato) e con la barra di Windows nascosta non cambia nulla.
- Rete di sicurezza: se Win risulta premuto ma la tastiera dice di no (rilascio perso, es. dopo Win+L), lo stato lo dimentica al tasto successivo, così il tocco seguente funziona. Win + clic o rotellina del mouse: l'hook del processo `--keyhook` osserva anche i pulsanti e la rotellina (senza mai mangiarli) e annulla il tocco, quindi non si apre il Launcher. Se l'hook del mouse non si installasse, resterebbe solo quel limite (non provato con mouse vero).

## Consumi a riposo con i due Dock (9 nov 2026, dopo le fasi nuove)
Misura (build di produzione, 30 s a riposo, due monitor): CPU totale **4,4-4,6 s** ogni 30 s (la passata finale era 1,38 s con meno funzioni:
il secondo Dock, il vetro della barra e le fasi nuove costano). Fatto: (1) l'elenco delle app del Dock si calcola **una volta** anche con due Dock
(condiviso 700 ms, invalidato da cambi certi); (2) la cattura dello sfondo **rallenta ancora** quando lo sfondo e' fermo da 8 s (Dock 350 ms,
barre 1,5 s; al primo cambiamento si torna subito al passo veloce). Risultato: **3,6-3,9 s** con due Dock, **2,1 s** con un solo Dock; processo
principale 1,3-1,6 s (era 1,75-2,4). Le misure oscillano molto da una prova all'altra (il PC fa anche altro). Chi vuole il minimo spegne
«Dock su tutti gli schermi».

**Causa vera trovata e corretta (9 nov 2026, sera).** Misurate le chiamate fra pagine e Rust per 35 s: ogni pagina Dock chiamava `dock_backdrop`
**~765 volte** (22 al secondo, anche con lo sfondo fermo), contro 30-60 di tutte le altre. Ogni chiamata costa lavoro nel runtime di Tauri, non nei
thread di EosOS (misurati per thread: i miei costano <=0,02 s ciascuno). Ora il sondaggio dello sfondo (Dock e pannelli) **rallenta da solo** quando
non arrivano fotogrammi nuovi (x1,5 fino a 250 ms) e torna a 40 ms al primo cambiamento (`nextPoll` in `lib/backdrop.ts`, testata). Risultato:
~120 chiamate per pagina e **CPU totale 1,5 s ogni 30 s con due Dock** (1,3 s con uno), come alla passata finale (1,38 s); processo principale 0,8-1,0 s.
