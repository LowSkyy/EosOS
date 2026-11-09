# Gestione delle finestre (snap)

Con una scorciatoia la finestra in primo piano si dispone in una metà, un terzo o un quarto dell'area di lavoro del suo monitor,
si massimizza, si centra, torna com'era o passa sull'altro monitor. **Spenta di base**: si accende da Impostazioni →
Mission Control → «Scorciatoie per disporre le finestre» scegliendo il modificatore (Ctrl+Alt, Ctrl+Maiusc o Alt+Maiusc).

| Tasto (con il modificatore) | Azione |
|---|---|
| ← / → | metà sinistra / destra |
| T / B | metà alta / bassa |
| U / I / J / K | quarto in alto a sinistra / destra, in basso a sinistra / destra |
| D / F / G | primo / secondo / terzo terzo (a larghezza) |
| Invio | massimizza (dentro l'area di lavoro) |
| C | centra, tenendo la misura |
| Backspace | torna com'era prima della disposizione |
| PagGiù / PagSu | sposta sul monitor successivo / precedente, in proporzione |

Niente freccia in su (è di Mission Control) e niente numeri (sono del Dock).

## Come funziona
- `src-tauri/src/winsnap.rs`: geometria pura (testata: 421 test Rust in tutto) + `RegisterHotKey` su un thread proprio, come il Dock.
- L'area di lavoro è quella di Windows: esclude la barra dei menu e il Dock quando riservano spazio. Ogni monitor usa la **sua**.
- I bordi invisibili delle finestre di Windows 10/11 sono compensati (cornice DWM), così la parte visibile finisce esattamente dove deve.
- Una finestra massimizzata o ridotta a icona si riporta prima alla misura normale. Si toccano solo finestre visibili, ridimensionabili,
  di altri processi (mai quelle di EosOS, né desktop e barra di Windows).
- «Ripristina» ricorda la posizione di prima della prima disposizione; se nel frattempo sposti la finestra a mano, la nuova posizione
  diventa quella da ricordare. Si tengono al massimo 64 finestre.
- Una combinazione già presa da un'altra app non si registra, senza errori.

## Prove
`scripts/test-winsnap.ps1` (31 controlli, su finestre di prova mie, un modulo ridimensionabile e uno a misura fissa): le 12 disposizioni
al pixel sull'area di lavoro vera, centra, ripristina, finestre non adatte rifiutate, due monitor (anche la metà sul secondo schermo),
da ridotta a icona; impostazione spenta di base, 16 combinazioni registrate scegliendo un modificatore e liberate rispegnendo.
**Non provato:** la pressione vera dei tasti (i comandi di prova eseguono le stesse azioni senza tastiera) e le app con bordi particolari
(UWP, app che si ridimensionano a scatti: possono fermarsi a una misura minima).
