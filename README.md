# EosOS

Shell desktop ispirata a macOS che gira **sopra Windows 11**. Non è un sistema operativo, non modifica file di sistema e si chiude in qualsiasi momento.

Cosa fa oggi: una barra superiore (appbar) su ogni monitor, con effetto vetro (Acrylic), nome dell'app attiva e orologio. Opzionalmente nasconde la taskbar di Windows e la ripristina in modo affidabile.

Stack: Tauri 2, backend Rust (crate `windows`), frontend React + TypeScript + Vite. Font Inter e icone open source.

## Requisiti

Windows 10/11 con WebView2, Rust (MSVC), Visual Studio Build Tools con Windows SDK, Node.js. Controllo automatico:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\check-prereqs.ps1
```

## Avvio in sviluppo

```powershell
npm install
npm run tauri dev
```

Per impostazione predefinita la **taskbar NON viene toccata**: compare solo la barra in alto (e riserva 28 px di area di lavoro, che Windows libera da solo alla chiusura).

Per nascondere anche la taskbar (opt-in):

```powershell
$env:EOSOS_HIDE_TASKBAR = "1"
npm run tauri dev
```

Prima di usare questa modalità leggi la sezione "Uscire e ripristinare la taskbar".

## Build

```powershell
npm run tauri build
```

L'eseguibile è in `src-tauri\target\release\eosos.exe` (più gli installer in `...\release\bundle\`). La build non avvia nulla.

## Uscire e ripristinare la taskbar

Dalla più semplice alla più drastica:

1. **Chiusura normale**: chiudi una barra (Alt+F4 sulla barra) o termina il processo dal Task Manager con "Termina attività". L'app esce, rimuove le appbar e rimostra la taskbar.
2. **Scorciatoia di emergenza**: `Ctrl+Alt+Shift+Esc`. La gestisce un processo separato (watchdog), quindi funziona anche se l'interfaccia è bloccata: termina EosOS e ripristina la taskbar.
3. **Crash o kill di EosOS**: il watchdog se ne accorge e ripristina da solo. Windows rimuove comunque le appbar quando il processo muore.
4. **A mano** (se tutto il resto fallisce, es. sono stati terminati entrambi i processi `eosos.exe`):

   ```powershell
   Stop-Process -Name explorer -Force
   Start-Process explorer
   ```

   Oppure: `Ctrl+Shift+Esc` → Task Manager → "Esegui nuova attività" → `explorer.exe`. Riavviare `explorer.exe` rimostra sempre la taskbar.

Il watchdog scrive un log in `%LOCALAPPDATA%\EosOS\logs\watchdog.log` (rotazione a 1 MB).

> Come funziona la sicurezza: la taskbar viene nascosta solo dopo che il watchdog ha confermato di essere pronto (handshake). Se il watchdog non parte, la taskbar resta visibile.

## Provarlo in sicurezza

Per la prima prova con la taskbar nascosta, meglio un ambiente isolato:

- **VM Hyper-V** (Windows 11 Pro): crea una VM con "Creazione rapida" → Windows 11, installa i prerequisiti o copia `eosos.exe` costruito, esegui lì. Fai un checkpoint prima. Nota: in VM l'effetto Acrylic può non essere disponibile; il ripiego (fondo opaco) è previsto.
- **Secondo utente Windows**: crea un utente locale di prova, accedi con "Cambia utente" ed esegui lì. Se qualcosa va storto, esci dalla sessione: `explorer.exe` dell'altro utente non è toccato.

Poi segui `docs/TEST-MANUALI.md`.

## Test automatici

```powershell
cd src-tauri; cargo test --lib; cd ..   # 21 test Rust (geometria, diff monitor, taskbar con finestre finte, watchdog, nomi app, vetro)
npm test                                # 18 test vitest (orologio, tema, contrasto WCAG dei token)
npm run build                           # tsc + vite
```

Script end-to-end (usano l'app reale; **nascondono la vera taskbar** e la ripristinano):

| Script | Cosa verifica |
|---|---|
| `scripts\test-taskbar.ps1` | 4 scenari sulla taskbar: uscita normale, kill, panic, scorciatoia di emergenza |
| `scripts\test-appbar.ps1` | area di lavoro riservata all'avvio e liberata a uscita/kill |
| `scripts\test-multimonitor.ps1 [-Hotplug]` | una barra per monitor, altezza = 28 × scala DPI, simulazione di scollegamento |

## Variabili di debug

| Variabile | Effetto |
|---|---|
| `EOSOS_HIDE_TASKBAR=1` | nasconde la taskbar (con watchdog) |
| `EOSOS_DEBUG_APPBAR=1` | log su stderr di appbar e vetro |
| `EOSOS_DEBUG_FOREGROUND=1` | log su stderr dell'app in primo piano |
| `EOSOS_TEST_PANIC=1` | solo build debug: panic dopo 9 s (test del ripristino) |
| file `%TEMP%\eosos-hide-monitor` | solo build debug: simula lo scollegamento del monitor il cui nome è nel file |

Il tema si forza con `?theme=light|dark` nell'URL o con `localStorage["eosos.theme"]`.

## Limiti noti

- Con la taskbar nascosta, sul monitor primario resta una fascia vuota in basso (Windows continua a riservare lo spazio della taskbar).
- Se vengono terminati **contemporaneamente** EosOS e il watchdog, la taskbar resta nascosta: usa il ripristino manuale sopra.
- Se `explorer.exe` si riavvia mentre EosOS è attivo, la taskbar nuova non viene nascosta di nuovo.
- Scollegamento reale di un monitor e scale DPI diverse da 100% sono coperti da test unitari, non verificati dal vivo.
- Il nome dell'app per finestre di sistema (es. menu Start) può mostrare una descrizione tecnica.
- Le icone sono quelle predefinite di Tauri, da sostituire con un set open source.

## Fuori scopo (roadmap)

Dock, launcher stile Spotlight, indicatori di sistema, Mission Control, menu globale per app.

Vedi `PIANO.md` per il piano e lo stato dei passi.
