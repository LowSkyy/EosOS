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
