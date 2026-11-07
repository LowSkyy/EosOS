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
- **Vetro**: sfondo sfocato catturato e disegnato nella pagina (`backdrop.rs`), con sopra una tinta traslucida e il bordo luminoso.
- **Hit-testing**: la finestra copre una fascia più alta della pillola per far crescere le icone; le zone trasparenti devono lasciar passare i clic (region della finestra o `WS_EX_TRANSPARENT` dinamico), altrimenti blocca le app sotto.
- **App fissate**: lette all'avvio dai collegamenti di Windows in `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar` (`.lnk`), poi salvate in `%LOCALAPPDATA%\EosOS\dock.json` così l'elenco del Dock è indipendente dalla taskbar.
- **App aperte**: `EnumWindows` filtrando finestre di primo livello visibili, non `WS_EX_TOOLWINDOW`, con titolo; raggruppate per eseguibile. Aggiornamento tramite gli eventi già usati in `foreground.rs`.
- **Icone**: estratte dall'eseguibile (`SHGetFileInfo`/`IShellItemImageFactory`) in PNG, passate al frontend come data URL (la CSP ammette `img-src data:`).
- **Frontend**: componente React `Dock`, animazione di ingrandimento in CSS/JS con `transform: scale`, nessuna libreria.
- **Sicurezza**: il Dock non nasconde nulla del sistema, quindi non tocca la logica di ripristino taskbar. Un errore nel Dock non deve mai impedire il ripristino.

## Fasi

1. **D1 – Dati (Rust)** — FATTA (7 nov 2026): `src-tauri/src/dock.rs`, comando Tauri `dock_items`. Verificata sui dati reali (12 app fissate, app aperte, icone PNG 128 px con colori e trasparenza corretti). Note:
   - Le app che si aggiornano in una sottocartella versionata (Discord) sono abbinate per nome di file e cartella antenata, così non compaiono due volte.
   - Ordine reale delle fissate (registro `Taskband`): non letto, al primo avvio sono ordinate per nome (ora si possono riordinare trascinando).
   - `cargo test dump_dock -- --ignored --nocapture` stampa le voci reali e salva le icone in `%TEMP%\eosos-dock`.
2. **D2 – Finestra e vetro** — FATTA (7 nov 2026): `src-tauri/src/dockwin.rs` + `src/components/Dock.tsx`. Finestra "dock" (trasparente, sempre in primo piano, senza focus, fuori da Alt+Tab) centrata in basso sull'area di lavoro del monitor primario. Le misure le decide il CSS: il frontend misura la pillola e chiama `dock_fit`; la finestra nativa si adatta ed è ritagliata (`SetWindowRgn`) alla capsula, quindi i clic fuori dalla pillola passano alle app sotto. Verificata dal vivo con screenshot (icone, separatore, puntini) e con `test-taskbar` (4 scenari) e `test-workarea`. Note:
   - **Niente sfocatura dello sfondo.** Acrilico, blur (accent), `DwmEnableBlurBehindWindow` e lo sfondo di sistema di Windows 11 (`DWMSBT_TRANSIENTWINDOW`) riempiono tutti l'intero rettangolo della finestra e ignorano `SetWindowRgn`: gli angoli restavano quadrati (grigi o neri). Il vetro è quindi una tinta traslucida con bordo luminoso in una capsula con angoli perfetti. Risolto più tardi con la sfocatura catturata (vedi "Sfocatura vera" sotto).
   - Solo il monitor primario (un Dock per monitor non è previsto).
   - Il Dock oggi si sovrappone alle finestre massimizzate (non riserva spazio); l'auto-nascondi di D4 lo renderà meno invasivo.
3. **D3 – Interazione** — FATTA (7 nov 2026): `dockact.rs`, `dockstore.rs`, comandi `dock_activate` e `dock_menu`. Verificata dal vivo con clic reali del mouse (Blocco note): minimizza/ripristina in alternanza per 6 clic su 6; menu con Apri / Esci / Fissa nel Dock / Rimuovi dal Dock (fissa, esci, clic su icona fissata che avvia l'app, rimuovi: tutto OK). Note:
   - **Sicurezza**: il frontend manda solo l'`id` di una voce; il percorso da avviare lo ricava Rust da `dock::find`, quindi la pagina non può far eseguire percorsi arbitrari.
   - **Elenco fissate** in `%LOCALAPPDATA%\EosOS\dock.json`: al primo avvio parte dalla taskbar (senza doppioni, per esempio più collegamenti a Chrome), poi è indipendente. Scrittura atomica; un file illeggibile fa ripartire dalla taskbar.
   - **Quale app è "davanti"**: non si usa né `GetForegroundWindow` (cliccando sul Dock il primo piano passa a EosOS) né l'ordine Z (Explorer ha finestre sempre in primo piano che stanno davanti a tutto): si usa l'ultima finestra attivata dall'utente, vista dall'hook di sistema in `foreground.rs`, più quelle attivate dal Dock stesso.
   - **Ordine stabile**: le app aperte non fissate restano nell'ordine di comparsa (`stable_order`), non si riordinano a ogni cambio di finestra.
   - **Icone**: l'estrazione dalla shell è serializzata (chiamate simultanee fallivano a caso).
   - Non ancora: trascinare per riordinare, app dello Store/UWP, rimbalzo all'avvio (D4).
4. **D4 – Effetti** — FATTA (7 nov 2026): `dockgeom.rs` (geometria e animazione, funzioni pure testate), `dockwin.rs` (ciclo a ~60 Hz), `Dock.tsx`. Verificata dal vivo: ingrandimento a onda (screenshot con il mouse sul Dock), rimbalzo (8 fotogrammi consecutivi, l'icona sale e scende), auto-nascondi attivato dal menu e misurato (scompare dopo ~0,5 s di cursore lontano, riappare toccando il bordo inferiore), `test-taskbar` (4 scenari) e `test-workarea` ancora OK. Note:
   - **Finestra fissa e più alta della capsula** (margine `--dock-head` sopra e `--dock-side` ai lati, in `tokens.css`): non cambia mai dimensione durante l'uso, quindi niente sfarfallio. Il frontend misura una copia invisibile della capsula a riposo (`Ghost`) e comunica a Rust zona e capsula con `dock_fit`.
   - **Zona cliccabile dinamica** (`SetWindowRgn`): finché il cursore è fuori è solo la capsula a riposo (i clic sopra e ai lati passano alle app); toccando la capsula diventa tutta la finestra, perché le icone ingrandite devono vedersi e poter essere cliccate. Isteresi: si entra dalla capsula, si resta nella finestra intera. Verificato misurando quale finestra riceve il clic.
   - **Ingrandimento**: campana a coseno (1,7× al massimo, raggio 2,4 icone), calcolata dalla distanza dal centro attuale di ogni icona; si scrive direttamente nel DOM (una variabile CSS per icona), senza rendering React a ogni movimento. Rispetta `prefers-reduced-motion`.
   - **Auto-nascondi** (impostazione `autohide` in `dock.json`, menu con il clic destro sullo sfondo della capsula): la finestra scorre sotto il bordo con uscita morbida; la parte fuori schermo è ritagliata, per non comparire su un monitor posto sotto. Il Dock non si nasconde mentre un menu è aperto.
   - **Rimbalzo**: parte al clic su un'app chiusa e continua finché non compare (massimo 8 s).
   - Menu nativo: la posizione dipende dal cursore; per provarlo senza tastiera si possono usare la porta di debug di WebView2 e UI Automation per trovare la voce, ma serve un clic vero del mouse sul suo centro.
5. **D5 – Rifiniture** — IN PARTE (7 nov 2026). Fatte:
   - **Puntino grigio (macOS 27)**: un'app fissata senza finestre ma con un processo attivo (es. Discord ridotto nell'area di notifica) mostra un puntino grigio chiaro (`DockItem.background`, `dock::processes_named`, `mark_background`). Si apre un handle solo per i processi che hanno lo stesso nome di file di un'app fissata, non per tutti. Misurato dal vivo: luminosità del puntino normale 222, di quello in background 122.
   - **Livello del vetro** (la versione a menu del cursore "Liquid Glass"): clic destro sullo sfondo della capsula, "Vetro trasparente / standard / opaco" (0,15 / 0,7 / 1,0). Salvato in `dock.json`, aggiornato subito nel frontend con l'evento `dock-settings-changed`. Verificato leggendo il colore calcolato: opacità 0,8 → 0,525 → 0,95 → 0,8.
   - **`dock.json` robusto**: tollera il BOM UTF-8 (Blocco note e PowerShell 5 lo aggiungono) e, se il file esiste ma non si legge, lo conserva come `dock.json.bad` invece di sovrascriverlo con l'elenco della taskbar. Trovato durante la prova: una mia modifica con PowerShell 5 cancellava le impostazioni.
   Non fatte (decisione presa insieme alla specifica, con il motivo):
   - ~~Sfocatura vera dello sfondo~~: fatta, vedi "Sfocatura vera" sotto.
   - ~~App dello Store (UWP)~~: fatte, vedi "App dello Store" sotto.
   - **Ordine reale delle fissate**: è nel registro (`Taskband`) in un formato binario non documentato. Ora sono in ordine alfabetico al primo avvio; trascinare per riordinare sarebbe la strada più sensata.
   - ~~Riavvio di Explorer dal vivo~~: fatto, vedi "Riavvio di Explorer" sotto.
   - **DPI misti / scala diversa**: la geometria è testata a 100%, 125%, 150% e con origine negativa, ma non ho cambiato la scala del tuo schermo.
   - ~~Riserva di spazio~~: fatta, vedi "Spazio riservato" sotto.

6. **Riordino con il trascinamento** — FATTO (7 nov 2026): `Dock.tsx` (eventi del puntatore), `dockstore::place`, comando `dock_reorder`. Si trascina un'icona fissata e la si rilascia in un'altra posizione tra le fissate; un'app aperta non fissata, rilasciata tra le fissate, viene fissata lì. Mentre si trascina le altre icone si scostano per aprire il varco, l'ingrandimento è sospeso e il clic che segue il rilascio non avvia nulla. L'ordine è subito visibile (aggiornamento ottimistico) e viene salvato in `dock.json`. Verificato dal vivo con il mouse vero e un'app di prova (Blocco note): spostata in avanti e indietro, ordine salvato, nessun avvio al rilascio. Risolve anche l'ordine delle fissate (non più solo alfabetico). Non incluso: trascinare fuori dal Dock per toglierla (si usa il menu). Durante la prova sono emersi e sono stati corretti tre difetti più grandi del trascinamento stesso:
   - **Stili della finestra**: Tauri riapplica dopo la creazione gli stili di una finestra normale, quindi il Dock era attivabile (cliccandolo il primo piano passava a EosOS: per questo in D3 serviva la scorciatoia dell'"ultima finestra attivata") e portava cornice, titolo e pulsanti, che Windows disegnava in cima al Dock appena veniva attivato (trascinamento, menu). Ora `enforce_styles` toglie la cornice e imposta `NOACTIVATE`/`TOOLWINDOW` a ogni ciclo (due letture): il primo piano resta all'app che si sta usando.
   - **App con processi di supporto** (Steam: le finestre sono di `steamwebhelper.exe`, figlio di `Steam.exe`): comparivano come un'app diversa e quella fissata risultava chiusa. Ora una finestra il cui eseguibile non è fissato è attribuita al suo antenato fissato (`find_ancestor`, fino a 4 livelli).
   - Un clic fermo (premere e rilasciare senza muovere) è un clic: nelle prove si preme solo sull'icona dell'app di prova.

7. **Sfocatura vera** — FATTA (7 nov 2026): `backdrop.rs`, `Dock.tsx` (canvas `.dock-blur`), `dockgeom::capture_rect`. Nessun effetto di Windows (acrilico, blur, sfondo di sistema, `DwmEnableBlurBehind`) rispetta una forma arrotondata, e la Composition API richiederebbe componenti COM scritti a mano o il Windows App SDK. Si fa quindi il contrario: si cattura lo schermo dietro il Dock, si manda alla pagina e lo sfoca il CSS (`filter: blur() saturate()`), ritagliato dalla capsula (`clip-path`): forma perfetta con bordi lisci.
   - **Cattura**: GDI (`BitBlt`) della finestra del Dock allargata di 48 px per lato (la sfocatura ha bisogno di contorno), ridotta di 2 (media a blocchi), a ~25 Hz, solo quando il Dock è visibile e la sfocatura è attiva; un fotogramma identico al precedente non si ripubblica. Il frontend lo chiede con `dock_backdrop(since)`, risposta binaria (intestazione di 28 byte + RGBA).
   - **Esclusione dalla cattura**: la finestra del Dock usa `WDA_EXCLUDEFROMCAPTURE`, altrimenti il Dock sfocherebbe se stesso. **Effetto collaterale: con la sfocatura attiva gli screenshot e le registrazioni schermo non includono il Dock.** Si può spegnere dal menu ("Sfocatura dello sfondo", clic destro sullo sfondo della capsula; impostazione `blur` in `dock.json`, attiva di default). Su un Windows più vecchio della versione 2004, dove l'esclusione non esiste, la sfocatura si disattiva da sola.
   - **Tinta**: con la sfocatura la tinta è molto più leggera (`--dock-blur-alpha-lo/hi`).
   - **Verificato dal vivo** con una finestra di prova a strisce e testo dietro il Dock: la capsula mostra lo sfondo sfocato con gli angoli perfetti. Allineamento misurato pixel per pixel contro lo schermo vero dietro di lei (sfocatura e tinta azzerate): 95% di concordanza, contro 68%, 44% e 4% spostando di 10, 20 e 40 px. Costo: 0,3% della CPU totale (circa l'8% di un core) con la sfocatura, 0,03% senza.
   - **Per provarla** senza vedere il Dock nelle catture: lo screenshot della pagina via porta di debug di WebView2 (`Page.captureScreenshot`) mostra il contenuto del Dock.

8. **App dello Store** — FATTE (7 nov 2026): `dock.rs` (`aumid_path`, `parse_aumid`, `identity`, `window_aumid`, `shell_display_name`, `window_identity`), `dockact.rs`. Le app dello Store (UWP) non hanno un eseguibile proprio: le finestre sono gusci `ApplicationFrameHost` e i collegamenti fissati puntano a un identificatore (AppUserModelID). Ora un'app dello Store è identificata dal "percorso" `shell:AppsFolder\<AUMID>`, che la shell sa avviare e da cui ricava nome e icona.
   - **Finestre**: dell'identificatore si leggono con `SHGetPropertyStoreForWindow`, solo per i gusci UWP e per gli eseguibili in `\WindowsApps\` (le app normali, anche se impostano un identificatore, restano per eseguibile). Un guscio senza identificatore non si attribuisce a nessuna app, come prima.
   - **Fissate**: se un collegamento della taskbar non ha percorso si legge l'identificatore dal suo archivio di proprietà. Anche per un'app impacchettata con percorso si preferisce l'identificatore (il percorso cambia a ogni versione).
   - **Avvio, nome, icona**: `ShellExecute` su `shell:AppsFolder\<AUMID>`; nome da `IShellItem::GetDisplayName`; icona con lo stesso `IShellItemImageFactory` delle app normali.
   - **Clic**: la finestra "davanti" segnalata dall'hook di primo piano può essere la finestra interna (`CoreWindow`, figlia del guscio): `window_identity` risale alla finestra principale prima di calcolare l'identità. Senza questo un'app dello Store non si minimizzava mai.
   - **Verificato dal vivo** con la Calcolatrice (`Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`): nome "Calcolatrice" e icona corretti, nessun doppione tra fissata e aperta, clic con il mouse vero che minimizza e ripristina per 4 volte su 4, "Esci" che la chiude (resta il processo sospeso, normale per le app UWP), clic su un'icona fissata e chiusa che la riavvia.
   - **Non coperto dal vivo**: la lettura di un collegamento fissato della taskbar che punta a un'app dello Store (sul PC di prova non ce ne sono: le 12 fissate sono tutte programmi normali). Il codice c'è ma l'ho provato solo con una voce scritta a mano in `dock.json`.
   - Il puntino grigio (app in background) non vale per le app dello Store.

9. **Spazio riservato** — FATTO (7 nov 2026): `appbar::reserve_bottom` / `release_bottom`, `dockgeom::reserve_height`, impostazione `reserve` in `dock.json` (attiva di default) e voce di menu "Riserva spazio alle finestre". Le finestre massimizzate si fermano sopra la capsula; le icone ingrandite e il rimbalzo salgono sopra le finestre, come in macOS.
   - **Come**: il Dock si registra come appbar in basso (stesso meccanismo e stessa registrazione della barra in alto, quindi anche la stessa rimozione a ogni uscita, a un panic e a Ctrl+C). La fascia è alta capsula + margine dal bordo + 6 px di respiro (92 px a scala 100%). Se in basso c'è la taskbar, la fascia si ferma sopra di lei (`ABM_QUERYPOS`).
   - **Posizione del Dock**: si appoggia al bordo *disponibile* restituito da `ABM_QUERYPOS`, non all'area di lavoro, che esclude già la riserva: altrimenti il Dock salirebbe a ogni aggiornamento. Il sistema viene avvisato solo quando il rettangolo cambia (ogni secondo si controlla, ma si scrive solo se serve).
   - **Con l'auto-nascondi** non si riserva nulla, e passando da un'impostazione all'altra la fascia si riserva o si rilascia entro un paio di secondi (verificato dal vivo dal menu: area di lavoro 956 → 1048 → 956).
   - **`EOSOS_DOCK_RESERVE=0`** spegne la riserva qualunque sia l'impostazione. Serve agli script di prova che misurano l'area di lavoro della sola barra in alto (`test-appbar`, `test-dpi`, `test-explorer-restart`, `test-multimonitor`, `test-workarea`: ora lo impostano).
   - **Verifica**: `scripts/test-dock-reserve.ps1` (nessun clic): area di lavoro −92 px; Blocco note massimizzato che finisce sopra la capsula (fondo 964 = 956 + 8 px di cornice invisibile, la capsula parte a 962); `EOSOS_DOCK_RESERVE=0` e auto-nascondi senza riduzione; uscita normale e kill che ripristinano l'area di lavoro.
   - **Difetto trovato e corretto durante la prova**: il Dock restava a volte nascosto all'avvio (4 esecuzioni su 4 in una serie, poi 6 su 6 corrette). Causa: competizione tra thread. `enforce_styles` leggeva lo stile della finestra e lo riscriveva senza la cornice; se nel frattempo `show()` (altro thread) aveva aggiunto `WS_VISIBLE`, la scrittura basata sulla lettura vecchia lo cancellava. Ora si rilegge lo stile subito prima di scrivere e, come rete di sicurezza, se il Dock mostrato risulta nascosto lo si rimostra.
   - **Limite**: solo il monitor primario, come il Dock.

10. **Riavvio di Explorer** — FATTO (7 nov 2026): `scripts/test-explorer-restart.ps1 -Exe ... -Dock`, correzione in `appbar::reserve_bottom`. Con il consenso dell'utente si è riavviato davvero Explorer (taskbar e desktop spariscono per qualche secondo e le finestre di Esplora risorse si chiudono).
   - **Difetto trovato prima della correzione** (primo riavvio): Explorer perde tutte le appbar registrate. La barra in alto si ri-registra (`handle_explorer_restart`), ma il Dock "ricordava" di aver già applicato la fascia: dopo il riavvio si registrava di nuovo (`ABM_NEW`) e poi saltava `ABM_SETPOS` perché il rettangolo coincideva con quello memorizzato. Risultato misurato: area di lavoro in basso da 988 a 1080, cioè **riserva persa** fino al riavvio di EosOS. Il Dock stesso restava visibile e allo stesso punto.
   - **Correzione**: una registrazione nuova azzera la memoria dell'ultima fascia applicata, così la posizione si comunica sempre.
   - **Dopo la correzione** (secondo riavvio): area di lavoro in basso 988 prima e dopo; Dock visibile, allo stesso livello e centrato; taskbar ri-nascosta; barra in alto ri-registrata; uscita normale che ripristina taskbar e marcatore.
   - Il controllo "stesso punto" confronta bordo inferiore e centro, non la larghezza: il riavvio chiude le finestre di Esplora risorse e la sua icona sparisce, quindi la capsula si restringe.

## Fuori scopo

Cestino, cartelle (Download), stack, trascinamento per riordinare (possibile dopo D5).

## Rischi aperti

- Hit-testing di una finestra trasparente più grande della pillola (D2).
- Icone di app UWP/Store, che non hanno un `.exe` semplice (D1): prima versione con ripiego a icona generica.
- Una finestra sempre in primo piano in basso può confliggere con la taskbar visibile se EosOS gira senza nasconderla.
