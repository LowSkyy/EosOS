# Centro di controllo

Un pulsante nella barra dei menu (i due cursori, a destra, accanto all'orologio) apre un pannello con gli interruttori rapidi, come il
Centro di controllo di macOS.

| Controllo | Che cosa fa |
|---|---|
| **Wi-Fi** | accende/spegne la radio Wi-Fi di Windows (c'è solo se il PC ne ha una) |
| **Bluetooth** | accende/spegne la radio Bluetooth (idem) |
| **Non disturbare** | è l'impostazione `notif.dnd` (nasconde le notifiche nel pannello sotto l'orologio) |
| **Mantieni sveglio** | finché è acceso il PC e lo schermo non si sospendono per inattività; **non si salva**: a ogni avvio di EosOS è spento |
| **Volume** | il cursore del volume (come nel pannello dell'indicatore audio) |
| Impostazioni di EosOS… / di Windows… | scorciatoie |

Frecce per muoversi fra i controlli, Esc o un clic fuori per chiudere.

## Come funziona
- `src-tauri/src/control.rs`: radio con `Windows.Devices.Radios` (serve il permesso di Windows: se lo nega, il pannello lo dice e
  l'interruttore torna com'era); «Mantieni sveglio» con `SetThreadExecutionState` su un thread proprio.
- I comandi `control_state` e `control_set` sono **solo del pannello** (controllo accessi, `cmdpolicy.rs`); `control_set` accetta solo
  gli id `wifi`, `bluetooth`, `awake`.
- Frontend: `ControlPanel.tsx`, `ControlButton.tsx`, logica pura in `lib/control.ts` (testata).
- Il pulsante è sempre nella barra (non c'è un'impostazione per nasconderlo: si può aggiungere se serve).

## Prove
- `scripts/test-control.ps1` (30 controlli): pulsante e pannello, Wi-Fi e Bluetooth **finti** (`EOSOS_TEST_RADIOS=fake`: spegnere il
  Wi-Fi vero taglierebbe la rete), Non disturbare (salvato e ripristinato), Mantieni sveglio acceso e spento, tastiera, controllo
  accessi (barra e Launcher non possono), lettura delle radio vere in sola lettura.
- Rust 425 test, vitest 181, clippy 0.
- **Non provato:** l'accensione/spegnimento delle radio VERE (taglierebbe rete o dispositivi), la luminosità (non fatta: serve un portatile
  o un monitor con controllo DDC, e non posso provarla), e se «Mantieni sveglio» impedisce davvero la sospensione (`powercfg /requests`
  vuole l'amministratore).
- **Non fatto:** luminosità, tema chiaro/scuro (toccherebbe il registro dell'utente).

## Aspetto (rifatto il 10 nov 2026)
Il pannello era sbilanciato: titolo piccolo e addossato alle tessere, tessere larghe con solo testo, contorno bianco di focus che sembrava una selezione, volume come riga sottile scollegata. Ora:
- **Titolo** a 14 px con 12 px di aria sotto; tutte le sezioni distano 12 px fra loro e hanno lo stesso raggio (12-14 px) e lo stesso fondo.
- **Tessere** 2x2 uguali: icona rotonda in alto (Wi-Fi, Bluetooth, luna per Non disturbare, sole per Mantieni sveglio), sotto nome e stato su una riga. L'icona diventa blu quando l'interruttore e' acceso (non piu' tutta la tessera). Con 2 tessere (senza Wi-Fi/Bluetooth) restano due tessere uguali.
- **Volume** in un blocco con etichetta e valore («100%», «Silenzioso» se muto) sopra il cursore a tutta larghezza.
- **Cattura schermo** con il suo titolo di sezione e due pulsanti uguali. I link alle impostazioni di EosOS e di Windows sono stati tolti l'11 nov (stanno nel menu a sinistra, il logo EosOS).
- Il **focus iniziale** resta sul primo controllo (tastiera), con un contorno blu sottile all'interno della tessera.
- Prove: `test-control`, `test-panel`, `test-notif-panel`, `test-settings-s6` TUTTO OK. La larghezza (300 px) e' la stessa di tutti i pannelli.
