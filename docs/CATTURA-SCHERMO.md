# Cattura dello schermo

Tre modi, come in macOS. L'immagine si salva come PNG in **Immagini\Screenshots** («Cattura schermo 2026-11-09 alle 17.55.03.png»,
con « (2)» se due nello stesso secondo), si **copia negli appunti** e si accompagna con un breve suono.

| Cosa | Dal Centro di controllo | Scorciatoia (se accese) |
|---|---|---|
| **Schermo intero** (il monitor dove sta il cursore) | «Schermo intero» | modificatore + **3** |
| **Selezione**: si trascina un rettangolo | «Selezione…» | modificatore + **4** |
| **Finestra in primo piano** | – (dal pannello non c'è una finestra «in primo piano») | modificatore + **5** |

Le **scorciatoie sono spente di base**: Impostazioni → Mission Control → «Scorciatoie per la cattura dello schermo» → Ctrl+Maiusc,
Ctrl+Alt o Alt+Maiusc. Dal Centro di controllo (pulsante a due cursori nella barra) funzionano sempre, senza tastiera.

## Selezione
Un velo scuro copre il monitor del cursore; si trascina un rettangolo (la misura in pixel appare accanto), si rilascia e si scatta. **Esc**,
il tasto destro o un clic senza trascinare annullano. La selezione sta in un solo monitor. Il velo si nasconde prima dello scatto, quindi non
finisce nell'immagine.

## Che cosa c'è nell'immagine
Tutto ciò che si vede, **compresi il Dock, la barra dei menu e i pannelli di EosOS**: normalmente sono esclusi dalla cattura di sistema
(servono alla sfocatura), quindi per un attimo (~160 ms) si riabilitano e poi si rimettono com'erano. Una finestra coperta da un'altra si
cattura com'è vista sullo schermo (compreso ciò che la copre), perché si scatta l'area dello schermo, non il contenuto della finestra.

## Come funziona
- `src-tauri/src/capture.rs`: parte pura (nomi dei file, conversione della selezione da px CSS a fisici con la scala, DIB per gli appunti, tagli
  alla dimensione dello schermo; 9 test) + BitBlt dello schermo, PNG, appunti (`CF_DIB`, 32 bit, alfa opaco), scorciatoie `RegisterHotKey`.
- `src-tauri/src/capturewin.rs`: la finestra del velo (si crea al primo uso, poi resta nascosta); `src/components/Capture.tsx` e `lib/capture.ts`.
- La pagina manda a Rust solo un rettangolo e la scala; i comandi sono ristretti per finestra (`cmdpolicy.rs`): `capture_run` solo al pannello,
  `capture_region` e `capture_cancel` solo al velo.

## Prove
`scripts/test-capture.ps1` (36 controlli): impostazione spenta di base e 3 combinazioni scegliendo il modificatore; schermo intero (PNG della misura
esatta del monitor, nome, contenuto non uniforme, finestre di EosOS di nuovo escluse dalla cattura dopo lo scatto, due scatti = due file); finestra
di prova mia **magenta** (misura = parte visibile della finestra, centro 255,0,255); selezione con trascinamento (300×200 CSS → 300×200 px), Esc,
clic senza trascinare; tipo inventato e «finestra» dal pannello rifiutati; controllo accessi; appunti (solo se contengono testo o nulla: il testo
si salva e si rimette). Le immagini vanno in una cartella temporanea.
**Non provato:** la pressione vera dei tasti 3/4/5 (si prova la stessa azione senza tastiera), il trascinamento con il mouse vero (si provano gli
eventi del puntatore), la selezione su un monitor a scala diversa dal 100% (la conversione è testata a 150%), le app a schermo intero esclusivo
(giochi), il contenuto protetto (video DRM: Windows lo nasconde). Per la finestra in primo piano la prova usa l'handle della finestra, perché
Windows non lascia a una finestra di prova prendere il primo piano senza simulare tasti.
