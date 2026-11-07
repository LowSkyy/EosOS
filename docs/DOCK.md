# Dock — specifica

Obiettivo: un Dock in stile macOS 27 "Golden Gate" (Liquid Glass) per EosOS su Windows 11.

Fonti sul comportamento di macOS 27: [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/), [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/). Novità rilevanti rispetto a macOS 26: puntino grigio chiaro per le app che restano attive dopo la chiusura delle finestre, e cursore unico "Liquid Glass" che regola la trasparenza del Dock (dallo sfondo ben visibile a un materiale più opaco).

## Comportamento (da replicare)

- Pillola di vetro flottante, centrata in basso, con angoli molto arrotondati e un piccolo spazio dal bordo dello schermo.
- Icone grandi, con ingrandimento al passaggio del mouse (effetto onda: l'icona sotto il cursore cresce, le vicine in modo graduale).
- Sinistra: app fissate. Destra, dopo un separatore sottile: app aperte non fissate.
- Puntino sotto le app in esecuzione. Puntino grigio chiaro per le app attive senza finestre (nuovo in macOS 27).
- Clic: se l'app non è aperta la avvia (con rimbalzo dell'icona), se è aperta la porta in primo piano, se è già in primo piano ne ripristina/minimizza la finestra.
- Clic destro: menu con "Apri", "Esci", "Fissa nel Dock / Rimuovi dal Dock".
- Auto-nascondi: scompare scivolando verso il basso e ricompare toccando il bordo inferiore (come l'impostazione di macOS).
- Cursore "Liquid Glass" (trasparenza) regolabile, in una prossima fase.

## Scelte tecniche

- **Finestra**: nuova finestra Tauri "dock", trasparente, sempre in primo piano, senza decorazioni, `skipTaskbar`. Non è un appbar (non riserva spazio): in macOS il Dock fisso riserva spazio, ma qui con l'auto-nascondi è flottante; la riserva di spazio sarà un'opzione successiva.
- **Vetro**: stesso materiale della barra in alto (`glass.rs`: Acrylic con ripiego opaco).
- **Hit-testing**: la finestra copre una fascia più alta della pillola per far crescere le icone; le zone trasparenti devono lasciar passare i clic (region della finestra o `WS_EX_TRANSPARENT` dinamico), altrimenti blocca le app sotto.
- **App fissate**: lette all'avvio dai collegamenti di Windows in `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar` (`.lnk`), poi salvate in `%LOCALAPPDATA%\EosOS\dock.json` così l'elenco del Dock è indipendente dalla taskbar.
- **App aperte**: `EnumWindows` filtrando finestre di primo livello visibili, non `WS_EX_TOOLWINDOW`, con titolo; raggruppate per eseguibile. Aggiornamento tramite gli eventi già usati in `foreground.rs`.
- **Icone**: estratte dall'eseguibile (`SHGetFileInfo`/`IShellItemImageFactory`) in PNG, passate al frontend come data URL (la CSP ammette `img-src data:`).
- **Frontend**: componente React `Dock`, animazione di ingrandimento in CSS/JS con `transform: scale`, nessuna libreria.
- **Sicurezza**: il Dock non nasconde nulla del sistema, quindi non tocca la logica di ripristino taskbar. Un errore nel Dock non deve mai impedire il ripristino.

## Fasi

1. **D1 – Dati (Rust)** — FATTA (7 nov 2026): `src-tauri/src/dock.rs`, comando Tauri `dock_items`. Verificata sui dati reali (12 app fissate, app aperte, icone PNG 128 px con colori e trasparenza corretti). Note:
   - Le app che si aggiornano in una sottocartella versionata (Discord) sono abbinate per nome di file e cartella antenata, così non compaiono due volte.
   - Non ancora gestite: app dello Store/UWP (fissate senza percorso, e finestre dentro `ApplicationFrameHost`) e ordine reale delle fissate (registro `Taskband`): ora sono ordinate per nome.
   - `cargo test dump_dock -- --ignored --nocapture` stampa le voci reali e salva le icone in `%TEMP%\eosos-dock`.
2. **D2 – Finestra e vetro**: finestra Dock in basso al centro, pillola di vetro con icone statiche, hit-testing corretto, più monitor (Dock sul monitor primario).
3. **D3 – Interazione**: clic per avviare/portare in primo piano/minimizzare, puntini di stato, menu contestuale, fissa/rimuovi.
4. **D4 – Effetti**: ingrandimento a onda, rimbalzo all'avvio, auto-nascondi con scorrimento.
5. **D5 – Rifiniture**: puntino grigio per app senza finestre, DPI misti, riavvio di Explorer, cursore Liquid Glass, riserva di spazio opzionale.

## Fuori scopo

Cestino, cartelle (Download), stack, trascinamento per riordinare (possibile dopo D5).

## Rischi aperti

- Hit-testing di una finestra trasparente più grande della pillola (D2).
- Icone di app UWP/Store, che non hanno un `.exe` semplice (D1): prima versione con ripiego a icona generica.
- Una finestra sempre in primo piano in basso può confliggere con la taskbar visibile se EosOS gira senza nasconderla.
