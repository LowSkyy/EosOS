# EosOS: Piano di lavoro

EosOS è una shell desktop per Windows 11 con interfaccia ispirata a macOS. Non è un sistema operativo: è un'app che gira sopra Windows, non modifica file di sistema e può essere chiusa in qualsiasi momento tornando al desktop normale.

Stack: Tauri 2, Rust (crate `windows`), React + TypeScript + Vite, `window-vibrancy`. Font Inter e icone open source (nessun asset Apple).

Stato: **FASE 1, in attesa di approvazione.**

## Principi di sicurezza (valgono per tutti i passaggi)

1. Il ripristino della taskbar viene implementato e testato (P4) **prima** di qualsiasi esecuzione che la nasconda. Fino ad allora, in dev la parte "nascondi taskbar" è disattivata da un flag (`EOSOS_HIDE_TASKBAR=1` per attivarla).
2. Ripristino su più livelli indipendenti: uscita normale, panic hook, handler Ctrl+C / chiusura console / logoff, watchdog in un processo separato, scorciatoia globale di emergenza.
3. Nessuna modifica persistente a registro o file di sistema. Il peggio possibile è riavviare `explorer.exe`.
4. Le API vengono verificate con Context7 prima di usarle.

## LEGENDA dei passaggi

| ID | Titolo | Obiettivo | File coinvolti | Rischio | Criterio di completamento | Stato |
|----|--------|-----------|----------------|---------|---------------------------|-------|
| P1 | Verifica prerequisiti | Controllare Rust (rustup, toolchain MSVC), Node + npm, WebView2 Runtime, VS Build Tools con workload C++, Git; installare quelli mancanti (winget) | `scripts/check-prereqs.ps1`, `PIANO.md` | Basso (installazioni di sistema, chiedo conferma prima) | Lo script stampa tutto OK; `rustc -V`, `node -v`, `cargo --version` rispondono | ☑ |
| P2 | Skill e strumenti | Fase 2: `/reload-plugins` e skill agent-skills, MCP Context7, Graphify (`graphifyy`), proposte extra | config Claude Code, `graphify-out/` | Basso | Tabella installato/verificato; `/mcp` mostra context7 connesso; grafo generato | ☑ |
| P3 | Scaffold Tauri 2 + React/TS | Creare il progetto, finestra base, build che parte, Inter in locale | `src-tauri/*`, `src/*`, `package.json`, `vite.config.ts`, `.gitignore` | Basso | `npm run tauri dev` apre una finestra normale; `cargo check` e `tsc` senza errori | ☑ |
| P4 | Modulo taskbar + ripristino di emergenza | Nascondere `Shell_TrayWnd` e `Shell_SecondaryTrayWnd` (`ShowWindow`), con ripristino garantito: (a) `Drop` guard + uscita normale, (b) panic hook, (c) handler Ctrl+C/console close/logoff (`SetConsoleCtrlHandler`), (d) **watchdog** in processo separato che ripristina se EosOS muore (anche con kill), (e) scorciatoia globale di emergenza (es. `Ctrl+Alt+Shift+Esc`) | `src-tauri/src/taskbar.rs`, `src-tauri/src/watchdog.rs`, `src-tauri/src/emergency.rs`, `src-tauri/src/main.rs`, `src-tauri/src/bin/eosos-watchdog.rs` | **Alto** | Test manuali documentati: uscita normale, panic forzato, kill del processo, scorciatoia, tutti riportano la taskbar. Test unitari sulla logica di stato. Nasconde solo con flag fino a fine test | ☑ |
| P5 | Barra superiore appbar | Finestra senza bordi, always on top, registrata con `SHAppBarMessage` (`ABM_NEW`, `ABM_QUERYPOS`, `ABM_SETPOS`, `ABM_REMOVE`) così le finestre massimizzate non ci finiscono sotto; rimozione appbar anche nei percorsi di ripristino di P4 | `src-tauri/src/appbar.rs`, `src-tauri/tauri.conf.json`, `src/App.tsx` | Medio (lo spazio riservato resta se non si fa `ABM_REMOVE`) | Una finestra massimizzata si ferma sotto la barra; all'uscita lo spazio di lavoro torna intero | ☑ |
| P6 | App attiva e orologio | Nome dell'app in primo piano a sinistra (`SetWinEventHook` con `EVENT_SYSTEM_FOREGROUND`, nome da exe/titolo, evento Tauri verso il frontend); orologio a destra (locale it-IT) | `src-tauri/src/foreground.rs`, `src/components/ActiveApp.tsx`, `src/components/Clock.tsx` | Medio (hook su thread con message loop, filtro finestre proprie) | Cambiando finestra il nome si aggiorna entro ~100 ms; l'orologio scatta al minuto | ☑ |
| P7 | Multi-monitor e DPI | Una barra per monitor (enumerazione monitor, reazione a `WM_DISPLAYCHANGE`/cambio DPI), per-monitor DPI v2, dimensioni in pixel fisici corrette | `src-tauri/src/monitors.rs`, manifest/config DPI, `appbar.rs` | Medio | Con 2 monitor (o simulato) e scala 100%/150% la barra ha altezza e posizione corrette; hot-plug non rompe nulla | ☑ |
| P8 | Vetro e design system | Mica/Acrylic con `window-vibrancy` (fallback pulito su Windows senza supporto), token CSS, tema chiaro/scuro che segue il sistema | `src/styles/tokens.css`, `src/styles/theme.ts`, `src-tauri/src/vibrancy.rs` | Basso | Effetto visibile, tema che cambia con quello di Windows, contrasto leggibile | ☑ |
| P9 | Test, README, prova sicura | Test automatici dove sensato, checklist manuale, README con avvio dev, build, uscita/ripristino (anche manuale: riavvio di `explorer.exe`), istruzioni per provarlo in VM Hyper-V o secondo utente Windows | `README.md`, `docs/TEST-MANUALI.md`, test Rust/TS | Basso | README completo; checklist eseguita; tutti i criteri del traguardo verificati | ☑ |

## Dettagli sul ripristino (P4)

- **Stato salvato**: prima di nascondere, si legge la visibilità attuale di ogni taskbar. Il ripristino riporta lo stato originale, ed è idempotente (si può chiamare più volte).
- **Watchdog**: processo separato e minimale, avviato da EosOS con il PID del genitore. Attende il termine del processo (`WaitForSingleObject`) e poi chiama il ripristino della taskbar e `ABM_REMOVE`. Copre kill dal Task Manager e crash nativi.
- **Scorciatoia di emergenza**: registrata sia dal processo principale sia dal watchdog (così funziona anche se la UI si blocca).
- **Ultima spiaggia manuale**: `Stop-Process explorer -Force; Start-Process explorer` oppure rilanciare EosOS senza `EOSOS_HIDE_TASKBAR` (annulla un auto-nascondi rimasto attivo). Lo script `restore-taskbar.ps1` previsto in origine non è stato creato: la procedura manuale è nel README.

## Traguardo della sessione

EosOS avvia una barra superiore appbar con nome dell'app attiva e orologio, nasconde la taskbar e la ripristina in modo affidabile all'uscita normale, al crash e con la scorciatoia di emergenza.

## Stato finale (7 nov 2026)

P1–P9 completati. Il traguardo è raggiunto e verificato dal vivo su questa macchina (2 monitor), sia con la build di debug sia con quella release (tranne il panic, vedi sotto).

**Oltre il piano originale**
- Hardening dalla review di sicurezza (handle unico al genitore, handshake "watchdog pronto", log con rotazione, CSP restrittivo).
- Fascia vuota con taskbar nascosta: auto-nascondi temporaneo con marcatore (`%LOCALAPPDATA%\EosOS\autohide-pending`), ripristino a ogni uscita e recupero all'avvio dopo un kill totale; un thread tiene la taskbar nascosta.
- Riavvio di Explorer: taskbar ri-nascosta e appbar ri-registrate (`detect_restart`).
- B3/B4: callback `EnumWindows` a prova di panic; si nasconde solo una taskbar di `explorer.exe`.
- Nomi comprensibili per i componenti di shell di Windows; icona originale; Clippy a zero avvisi.
- Scale DPI: verificate a 100% e 125% (anche miste e con cambio di scala a EosOS aperto).

**Verifiche**: 24 test Rust, 18 vitest, `tsc`/Vite, build release (MSI + NSIS), script dal vivo `test-taskbar` (4 scenari), `test-appbar`, `test-workarea`, `test-multimonitor -Hotplug`, `test-explorer-restart`, `test-dpi`.

**Prova in Windows Sandbox (7 nov 2026)**: build release su Windows 11 Enterprise 10.0.26100, utente `WDAGUtilityAccount`, via `scripts/prepare-sandbox.ps1`. Prova automatica tutta OK:
- `test-workarea`: area di lavoro 0..775 → 28..823 con taskbar nascosta, nessuna fascia vuota; uscita normale e kill del processo principale ripristinano taskbar, area di lavoro e marcatore; dopo il kill simultaneo di EosOS e watchdog (limite noto: taskbar ancora nascosta) il riavvio recupera tutto.
- Panic sulla build release verificato con la feature Cargo `test-hooks` (`cargo build --release --features test-hooks`, build separata, mai distribuita): `test-taskbar` completo, 4 scenari su 4 OK. Nota: il release usa `panic = "abort"`, ma l'hook di panic gira prima dell'abort e il watchdog copre comunque la morte del processo.
- `test-taskbar -SkipPanic`: uscita normale, kill brutale e scorciatoia `Ctrl+Alt+Shift+Esc` ripristinano la taskbar (confermato dal log del watchdog).
- Prova manuale (`Ctrl+Alt+Shift+Esc`, chiusura, kill dal Task Manager, ripristino): tutto OK.

**Non verificato / aperto**
- Scale oltre il 125% e ritorno al 100% con EosOS aperto (rinviati su richiesta).
- Scollegamento reale di un monitor (solo hook di debug).
- Prova in VM Hyper-V o con un secondo utente: sostituita dalla Windows Sandbox per la parte automatica.
- Un'esecuzione isolata di `test-multimonitor -Hotplug` è risultata KO senza righe di errore; 5 esecuzioni consecutive successive sono tutte OK, quindi non riproducibile (probabile interferenza esterna).
- Dopo il kill simultaneo di EosOS e watchdog l'auto-nascondi resta attivo fino al prossimo avvio di EosOS (limite accettato: non resta nessun processo vivo che possa ripristinare; il recupero avviene all'avvio successivo).

## Roadmap (FUORI SCOPE per ora)

- Dock: FATTO (7 nov 2026), vedi `docs/DOCK.md`
- Launcher stile Spotlight: specifica scritta (7 nov 2026), vedi `docs/LAUNCHER.md`; L1-L6 fatte (vedi `docs/LAUNCHER.md`); manca solo la prova del riavvio di Explorer con il Launcher attivo (`scripts\test-launcher.ps1 -WithExplorerRestart`)
- Indicatori di sistema (batteria, rete, volume): I1-I4 fatte (7 nov 2026), vedi `docs/INDICATORI.md`; non provati: riavvio di Explorer, ripresa dalla sospensione, scrittura sull'audio vero
- Mission Control: specifica scritta (7 nov 2026), vedi `docs/MISSION-CONTROL.md`; M1-M4 fatte (dati, panoramica, animazione/scorciatoia/angolo, scale e fascia dei desktop), restano M5 (Spaces completo) e M6 (Win+Tab e Alt+Tab)
- Menu globale per app
