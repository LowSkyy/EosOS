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
| `scripts\test-workarea.ps1` | niente fascia vuota con taskbar nascosta; auto-nascondi ripristinato dopo uscita, kill e kill totale |
| `scripts\test-explorer-restart.ps1` | riavvia Explorer con EosOS attivo: la nuova taskbar resta nascosta e la barra si ri-registra |
| `scripts\test-dpi.ps1` | come test-multimonitor, ma segnala la scala corrente e salva un report per confrontare scale diverse (la scala la imposti tu) |
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

- Con la taskbar nascosta EosOS attiva temporaneamente l'**auto-nascondi** di Windows (altrimenti resterebbe una fascia vuota in basso) e lo disattiva all'uscita; se era già attivo non lo tocca. Se vengono terminati **contemporaneamente** EosOS e il watchdog, l'impostazione resta attiva finché non rilanci EosOS (la annulla da solo all'avvio; marcatore in `%LOCALAPPDATA%\EosOS\autohide-pending`) e la taskbar può restare nascosta: usa il ripristino manuale sopra.
- Scale DPI: verificate dal vivo a 100% e 125% (anche miste: primario 125%, secondo 100%): altezza barra 35 px e 28 px, area di lavoro corretta, testo nitido e non tagliato (screenshot sul primario). Non provate 150% o più, né il cambio di scala con EosOS già aperto.
- Lo scollegamento reale di un monitor è coperto solo da test unitari e dall'hook di debug, non verificato dal vivo.
- Per le finestre di shell di Windows il nome è mappato solo per i componenti più comuni (Start, Cerca, Windows, Tastiera, Blocco schermo); altri possono mostrare la descrizione tecnica del file.

## Fuori scopo (roadmap)

Dock, launcher stile Spotlight, indicatori di sistema, Mission Control, menu globale per app.

Vedi `PIANO.md` per il piano e lo stato dei passi.

## Icona

L'icona è originale (un sole che sorge, da Eos = alba), generata da `scripts/make-icon.py` (Pillow) in `assets/icon-source.png` e convertita con `npm run tauri icon assets/icon-source.png`. Nessun asset di terzi.

