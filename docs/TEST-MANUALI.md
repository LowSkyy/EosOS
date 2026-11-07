# Checklist di test manuali

Da eseguire in una VM Hyper-V o con un secondo utente Windows (vedi README). Segna ogni voce.

## A. Senza nascondere la taskbar (`npm run tauri dev`)

- [ ] Compare una barra in alto, a tutta larghezza, alta ~28 px, senza bordo né angoli arrotondati.
- [ ] Le finestre normali (es. Blocco note massimizzato) non coprono la barra.
- [ ] A sinistra c'è il nome dell'app attiva e cambia quando cambi finestra; sul desktop mostra "Desktop".
- [ ] A destra c'è l'orologio, aggiornato al cambio del minuto.
- [ ] Effetto vetro visibile (se supportato); altrimenti fondo opaco leggibile.
- [ ] Con tema Windows chiaro e scuro il testo è leggibile.
- [ ] Con due monitor: una barra per monitor; con scala diversa da 100% l'altezza resta proporzionata.
- [ ] Chiusura (Alt+F4 sulla barra): la barra sparisce e l'area di lavoro torna intera.

## B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`)

Prima: snapshot della VM o utente di prova.

- [ ] La taskbar scompare solo dopo l'avvio; nel Task Manager esistono due `eosos.exe`.
- [ ] **Uscita normale** (Alt+F4 sulla barra): la taskbar torna subito.
- [ ] **Kill** di `eosos.exe` principale dal Task Manager: la taskbar torna entro qualche secondo.
- [ ] **Scorciatoia** `Ctrl+Alt+Shift+Esc`: EosOS si chiude e la taskbar torna.
- [ ] **Riavvio ripristino manuale**: termina entrambi gli `eosos.exe` insieme; la taskbar resta nascosta; `Stop-Process explorer -Force; Start-Process explorer` la riporta.
- [ ] Il log `%LOCALAPPDATA%\EosOS\logs\watchdog.log` riporta gli eventi.

## C. Script automatici (sulla macchina di prova)

- [ ] `scripts\test-taskbar.ps1` → 4/4 scenari OK
- [ ] `scripts\test-appbar.ps1` → OK
- [ ] `scripts\test-multimonitor.ps1 -Hotplug` → OK (con 2 monitor)

## D. Traguardo finale

- [ ] Barra superiore appbar + nome app attiva + orologio.
- [ ] Taskbar nascosta e ripristinata a uscita normale, crash e scorciatoia di emergenza.

## E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

La scala la imposti tu in Impostazioni > Sistema > Schermo; gli script non la cambiano. Per ogni scala (125%, 150%, 175%, e scale diverse tra i due monitor):

1. Imposta la scala, poi esci e rientra nella sessione se Windows lo chiede.
2. `powershell -File scripts\test-dpi.ps1` (richiede `cargo build` in `src-tauri`): controlla per monitor altezza barra = round(28 × scala), larghezza e area di lavoro, e aggiunge una riga a `%TEMP%\eosos-dpi-report.txt`.
3. A occhio, con `npm run tauri dev`:
   - [ ] il testo della barra non è tagliato né sfocato;
   - [ ] nome app e orologio hanno dimensione proporzionata alla scala;
   - [ ] spostando una finestra tra i due monitor la barra resta a posto;
   - [ ] cambiando la scala mentre EosOS è aperto la barra si riadatta entro ~1 s (se no, annotalo).
