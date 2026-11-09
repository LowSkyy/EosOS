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
