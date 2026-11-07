# Indicatori di sistema — specifica

Obiettivo: nella barra in alto di EosOS, a destra e prima dell'orologio, gli indicatori di sistema che la barra di macOS 27 "Golden Gate" mostra: **batteria**, **rete (Wi-Fi/cavo)** e **volume**, con lo stato sempre aggiornato e, al clic, un piccolo pannello per agire.

## Cosa fa macOS (da replicare) e da dove viene

Fonti: [TWiT, "Take full control of your macOS Tahoe menu bar"](https://twit.tv/posts/tech/take-full-control-your-macos-tahoe-menu-bar), [9to5Mac, novità del Control Center di Tahoe](https://9to5mac.com/2025/07/09/macos-tahoe-has-big-control-center-upgrades-heres-everything-new/), [allthings.how, Control Center in macOS 26](https://allthings.how/use-the-new-control-center-features-in-macos-26-tahoe), [MacRumors, funzioni di Tahoe](https://www.macrumors.com/2025/09/16/10-macos-tahoe-features).

**Limite della ricerca**: per macOS 27 non ho trovato novità verificabili sulla barra dei menu; le fonti descrivono macOS 26 (Tahoe). La specifica replica quel comportamento, non novità non verificate; la maggior parte delle fonti sono blog e non la documentazione Apple, e non ho controllato le etichette esatte dei menu.

- Gli indicatori sono icone piccole a destra della barra; ognuno si può mostrare o nascondere (Impostazioni di Sistema > Barra dei menu).
- **Batteria**: icona con livello, simbolo di ricarica, e opzionalmente la percentuale accanto.
- **Wi-Fi**: icona a onde (con l'intensità del segnale); senza rete un'icona barrata.
- **Suono**: icona altoparlante, barrata se silenziato.
- Un clic apre un pannello di vetro (Liquid Glass) con i dettagli e i comandi: reti disponibili, cursore del volume, stato della batteria.
- Barra dei menu trasparente per impostazione predefinita (si può dare un fondo); EosOS ha già la sua tinta, quindi qui basta restare coerenti con la barra attuale.

## Scelte tecniche (proposte)

- **Dove**: nel componente della barra (`App.tsx`), tra il nome dell'app attiva e l'orologio, a destra. Una barra per monitor: gli indicatori compaiono su tutte (come macOS), ognuna con la stessa informazione.
- **Dati (Rust)**: un modulo per indicatore, con una parte **pura e testata** (modelli di stato, scelta dell'icona, testi) e una parte che parla con Windows; stato inviato alle barre con un evento Tauri solo quando cambia (niente polling dalla pagina):
  - *Batteria*: `GetSystemPowerStatus` (livello, rete elettrica, in ricarica, tempo residuo) riletto sugli eventi di alimentazione (`WM_POWERBROADCAST` / `RegisterPowerSettingNotification`) con un controllo lento di sicurezza. Senza batteria (desktop) l'indicatore non compare.
  - *Rete*: WinRT `NetworkInformation` (`GetInternetConnectionProfile`: livello di connettività, tipo cavo/Wi-Fi, nome della rete, `GetSignalBars` per le barre) con l'evento `NetworkStatusChanged`. Stati: connesso, senza Internet, disconnesso; cavo, Wi-Fi.
  - *Volume*: Core Audio (`IMMDeviceEnumerator`, `IAudioEndpointVolume`) sul dispositivo di uscita predefinito, con notifiche di cambio volume/silenzio e di cambio dispositivo.
- **Icone**: SVG disegnati nella pagina (come la lente del Launcher), che seguono i colori della barra e il tema chiaro/scuro; nessuna immagine esterna.
- **Clic**: nella prima versione un clic apre la pagina giusta di Impostazioni (`ms-settings:batterysaver`, `ms-settings:network`, `ms-settings:sound`), con id fisso scelto da Rust. Il pannello vero (cursore del volume, elenco delle reti) viene dopo.
- **Pannello (fase successiva)**: una finestra di vetro sotto l'indicatore, con la stessa pipeline di sfocatura del Dock e del Launcher; il cursore del volume scrive su `IAudioEndpointVolume`, il silenzio è un interruttore.
- **Impostazioni** in `%LOCALAPPDATA%\EosOS` con la stessa robustezza di `dock.json`/`launcher.json` (BOM, `.bad`, scrittura atomica, nessuna riscrittura dopo una lettura fallita): quali indicatori mostrare e se la batteria mostra la percentuale.
- **Sicurezza**: la pagina non manda mai comandi o percorsi; chiede azioni per **id** fisso (`open:battery`, `volume:set`, `volume:mute`) e Rust controlla i valori (volume 0-100, NaN rifiutato).

## Fasi

1. **I1 — Dati (Rust)**: modelli e funzioni pure (percentuale → livello icona, testo del tempo residuo, stato di rete → icona, volume → icona) con test; lettura vera di batteria, rete e volume; evento Tauri `indicators-changed` solo sui cambi; nessuna interfaccia ancora.
2. **I2 — Barra**: componente degli indicatori, icone SVG, percentuale della batteria opzionale, clic che apre le Impostazioni giuste; una barra per monitor, DPI, temi.
3. **I3 — Pannello**: finestra di vetro sotto l'indicatore con cursore del volume e silenzio, stato della rete e della batteria; chiusura al clic fuori e con Esc.
4. **I4 — Rifiniture**: impostazioni (quali indicatori, percentuale), cambio dispositivo audio, ibernazione/riattivazione, riavvio di Explorer, prove su più monitor e scale diverse, revisione del codice.

## I1 — Dati: FATTA (7 nov 2026)

Solo Rust, nessuna interfaccia ancora. Modulo `indicators.rs` (14 test sulla parte pura + una lettura vera).

- **Stato già pronto per la pagina**: ogni indicatore porta con sé icona e testi calcolati in Rust (la pagina non decide nulla).
  - *Batteria* (`Battery`): `percent`, `charging`, `plugged`, `minutes_left`, `icon` (`battery-critical/low/half/high/full/charging/plugged`), `label` (**sempre la percentuale**, "85%") e `title` ("Batteria 85% · 2 h 15 min rimanenti", "in carica", "collegata alla rete"). Dai campi di `GetSystemPowerStatus` (`battery_from_status`): nessuna batteria (flag 128) o dati sconosciuti (255) → l'indicatore non c'è; il tempo residuo si dà solo a batteria.
  - *Rete* (`Network`): tipo (`wifi`/`ethernet`/`cellular`/`none`), raggiungibilità (`internet`/`limited`/`none`), nome, barre 0-5, `icon` (`wifi-1/2/3`, `wifi-alert`, `ethernet`, `ethernet-alert`, `cellular`, `net-off`) e `title` ("Wi-Fi: Casa", "Wi-Fi: Ospiti · senza Internet", "Cavo di rete (Ethernet)", "Nessuna connessione"). Dai profili di connessione WinRT (`pick_network` sceglie il livello più alto; a pari livello cavo, poi Wi-Fi, poi rete mobile).
  - *Volume* (`Volume`): `percent` (NaN e valori fuori scala limitati a 0-100), `muted`, `icon` (`volume-0/1/2/3`, `volume-muted`), `title` ("Volume 40%", "Audio disattivato"). Dal dispositivo di uscita predefinito di Core Audio, riacquisito a ogni giro (un cambio di cuffie si vede subito).
- **Servizio**: un thread controlla ogni 1,5 s e confronta con lo stato di prima (`update_state`); **solo se cambia** salva lo stato e manda l'evento Tauri `indicators-changed` a tutte le finestre. Il comando `indicators_get` dà lo stato corrente alle pagine che si caricano dopo. Un panic nella lettura non ferma il servizio.
- **Scelta rispetto alla specifica**: la specifica pensava a eventi di sistema più un controllo lento. In I1 c'è solo il controllo ogni 1,5 s con confronto: una lettura costa circa 3,5 ms (misurato a regime, build non ottimizzata), cioè circa lo 0,2% di un core, regge gli eventi mancati (sospensione, cavo staccato, cuffie) e non richiede tre meccanismi diversi. Gli eventi (`NetworkStatusChanged`, notifiche di volume, `WM_POWERBROADCAST`) restano un'ottimizzazione possibile per I4 se la latenza di 1,5 s dà fastidio.
- **Difetto trovato e corretto**: 20 letture di fila in un thread davano `ACCESS_VIOLATION`. Causa: COM inizializzato e chiuso a ogni lettura mentre le fabbriche WinRT restano in cache e puntano a un apartment smontato. Ora il thread del servizio inizializza COM (MTA) una volta e lo tiene per tutta la sua vita (e lo stesso fa la prova); la regola è scritta nel commento di `read_real`. Tiene vivo l'apartment anche per le ricerche di file del Launcher, che usano WinRT con thread brevi.
- **Sorgente simulata (solo build di prova)**: con `EOSOS_FAKE_INDICATORS=<file.json>` lo stato viene da un file riletto a ogni giro (file illeggibile: si tiene lo stato di prima). Così si prova senza scaricare la batteria, staccare la rete o cambiare il volume dell'utente.
- **Verifica** (`scripts/test-indicators.ps1`, 20 controlli, dalla pagina via CDP): lo stato iniziale arriva con `indicators_get`; **nessun evento in 4 s senza cambi**; un cambio produce **un solo** evento con i dati giusti (batteria al 19% con tempo residuo, Wi-Fi senza Internet, audio disattivato); la scomparsa di batteria e audio dà campi nulli e un evento; un file illeggibile non cambia nulla; valori fuori scala (volume 250, batteria 150) sono limitati a 100. Lettura vera in sola lettura sul tuo PC: **nessuna batteria** (desktop: indicatore assente come previsto), rete **via cavo (Ethernet)**, volume **100%**. Test: 176 Rust, clippy a 0 avvisi.

Non provato: batteria vera (il PC non ne ha), Wi-Fi vero (c'è solo il cavo), cambio vero del dispositivo audio, ripresa dalla sospensione.

## I2 — Barra: FATTA (7 nov 2026)

- **Componente** (`Indicators.tsx`, logica pura in `lib/indicators.ts`, 9 test): a destra della barra, prima dell'orologio, nell'ordine di macOS: **batteria, rete, audio**. Chiede lo stato a Rust all'avvio (`indicators_get`) e poi ascolta `indicators-changed` (che parte solo sui cambi). Un indicatore assente non compare, e se non c'è nulla sparisce l'intero blocco. Una barra per monitor: tutte mostrano gli stessi indicatori (provato con le due barre del tuo PC).
- **Batteria con la percentuale sempre visibile** ("85%" accanto all'icona, cifre a larghezza fissa), come richiesto. Il riempimento dell'icona segue la percentuale vera; in carica ha il fulmine, collegata e piena la spina.
- **Icone SVG disegnate nella pagina** (nessuna immagine), con `currentColor`, quindi seguono tema chiaro e scuro. Rete: Wi-Fi a tre archi (gli archi oltre il livello sono sbiaditi), Wi-Fi con punto esclamativo se senza Internet, cavo, rete mobile, scollegato (Wi-Fi sbiadito barrato). Audio: altoparlante con 0-3 onde, croce se silenziato.
- **Allarme discreto**: batteria quasi scarica (≤ 10% e non in carica) e rete assente o senza Internet diventano rosse; l'audio disattivato no (è una scelta, non un guasto).
- **Clic**: apre la pagina giusta di Impostazioni (`ms-settings:batterysaver`, `ms-settings:network`, `ms-settings:sound`). La pagina manda solo un id fisso (`battery`, `network`, `sound`; il volume usa `sound`) e **Rust sceglie l'indirizzo** da una tabella (`settings_uri`): un id sconosciuto, un indirizzo di Impostazioni diverso o un percorso non aprono nulla (test).
- **Accessibilità**: ogni indicatore è un pulsante con nome per i lettori di schermo e suggerimento al passaggio del mouse (il testo calcolato da Rust: "Batteria 85% · in carica", "Wi-Fi: Casa"); stato di focus visibile; la barra resta alta 28 px CSS.
- **Controllo incrociato delle icone**: un test Rust verifica che ogni icona che Rust può emettere (tutte le combinazioni di livello, stato, tipo di rete) sia tra le 20 che la pagina disegna, e un test della pagina controlla la stessa lista: se uno dei due lati cambia, si rompe un test.
- **Verifica dal vivo** (`scripts/test-indicators.ps1`, 37 controlli, barra vera con sorgente simulata): ordine e testi giusti; percentuale sempre visibile; altezza della barra invariata; la barra del secondo monitor mostra gli stessi tre indicatori; batteria al 5% e Wi-Fi senza Internet in allarme, audio silenziato no; senza batteria (come questo PC) compaiono solo rete e audio; senza nulla il blocco sparisce e ricompare quando tornano; **clic vero** con il mouse sull'indicatore di rete, dopo aver verificato che sotto il cursore ci fosse la barra di EosOS: si apre la finestra Impostazioni, poi chiusa. Schermate controllate nei temi scuro e chiaro e in allarme. Test: 178 Rust, 80 pagina, clippy a 0 avvisi.

Non provato: scala diversa dal 100% e secondo monitor a scala diversa per gli indicatori (previsto in I4, la barra è già per monitor e le misure sono in px CSS), clic con il mouse sulla batteria e sull'audio (stessa funzione di quello di rete, coperta dal test della tabella), Wi-Fi e batteria veri.

## I3 — Pannello: FATTA (7 nov 2026)

- **Finestra** (`panelwin.rs`, 8 test sulle parti pure): una finestra Tauri `panel` creata nascosta all'avvio (come il Launcher): senza bordi, trasparente, sempre in primo piano, 300 px CSS di larghezza, riceve mouse e tastiera, non compare in Alt+Tab. Si chiude con Esc, cliccando fuori (perdita del focus, con 350 ms di tolleranza all'apertura), con Alt+F4 (senza chiudere EosOS) o ricliccando lo stesso indicatore. Se il primo piano non arriva si richiude da sola.
- **Posizione**: la pagina della barra manda il rettangolo dell'indicatore in px CSS; **Rust lo converte in pixel dello schermo** con posizione e scala della finestra che ha chiamato (`anchor_to_physical`, rifiuta NaN, valori enormi, misure nulle) e posiziona il pannello **centrato sotto l'indicatore a 6 px**, mai fuori dal monitor di almeno 8 px (a destra e a sinistra si ferma al bordo; se non c'è spazio sotto, sale sopra). Funziona con coordinate negative (monitor a sinistra) e con qualsiasi scala.
- **Clic** (`decide`, pura): chiuso → apre; aperto sullo stesso indicatore → chiude; aperto su un altro → cambia contenuto e posizione. Se il clic ha già tolto il focus al pannello (che si è chiuso un attimo prima) **non lo riapre** (finestra di 400 ms): senza questo un secondo clic lo chiuderebbe e riaprirebbe subito.
- **Contenuto** (pagina `Panel.tsx`, logica pura in `lib/indicators.ts`):
  - *Audio*: pulsante del silenzio, **cursore 0-100 con la tastiera** (frecce, Home, Fine) e il mouse, percentuale. Muovere il cursore con l'audio spento lo riaccende, come su macOS. Il valore si manda a Rust al massimo ogni 40 ms mentre si trascina, ma **l'ultimo valore arriva sempre** (`makeLatestSender`, testata con timer finti). Subito dopo un tuo gesto vale il tuo valore per 2,5 s: lo stato del sistema arriva con 1,5 s di ritardo e farebbe tornare indietro il cursore (`reconcileVolume`); poi si segue il sistema, così un cambio fatto altrove si vede.
  - *Rete*: nome, tipo, stato ("Connesso a Internet" / "senza Internet" / "Non connesso") e segnale ("4 su 5").
  - *Batteria*: percentuale grande, stato (a batteria / in carica / collegata) e tempo rimanente.
  - In ogni pannello un pulsante apre la pagina giusta delle Impostazioni (stessa tabella fissa di I2).
- **Sfondo sfocato**: terzo canale di cattura (`Channel::Panel`) accanto a Dock e Launcher; la finestra è esclusa dalla cattura; l'altezza della finestra segue il contenuto (`panel_fit`).
- **Scrittura del volume** (`indicators.rs`): comandi `volume_set(percent)` e `volume_mute(on)`. Il valore è validato in Rust (`sanitize_volume`: NaN e infiniti rifiutati, 0-100 arrotondato e limitato). Scrive sul dispositivo di uscita predefinito con Core Audio (`SetMasterVolumeLevelScalar`, `SetMute`).
- **Prova simulata del volume, come deciso**: nelle build di prova con `EOSOS_FAKE_INDICATORS` le scritture vanno **solo nel file finto** (modifica atomica che conserva tutti gli altri campi) e **mai sull'audio vero**. Il percorso che scrive sull'audio vero non è mai stato eseguito dalle prove: esiste un test `real_volume_roundtrip` (`#[ignore]`, cambia di un punto e ripristina) da lanciare solo con il tuo ok esplicito: `cargo test real_volume_roundtrip -- --ignored --nocapture`.
- **Difetto trovato e corretto**: dopo "riattiva l'audio" muovendo il cursore, il volume non cambiava. Volume e silenzio partono come due comandi paralleli e nel file simulato la lettura-modifica-scrittura di uno cancellava quella dell'altro. Ora le scritture simulate sono serializzate con un blocco (sull'audio vero le due chiamate sono indipendenti).
- **Verifica dal vivo** (`scripts/test-panel.ps1`, 42 controlli, ripetuti due volte con esito identico): clic vero sull'indicatore audio (dopo aver controllato che sotto il cursore ci fosse la barra) apre il pannello, largo 300 px, a 6 px sotto l'indicatore, tutto dentro il monitor, in primo piano con il cursore del volume a fuoco e il valore 40%; finestra esclusa dalla cattura e sfondo sfocato attivi; altezza della finestra pari al contenuto. Tasti veri: 5 frecce → 45 nel file simulato (gli altri campi del file restano), Home → 0, Fine → 100, 30 passi di fila → arriva il valore finale; la barra segue entro il controllo periodico. Clic vero sul pulsante del silenzio (con controllo del bersaglio): il file ha il silenzio, il pannello e la barra lo mostrano; muovere il cursore lo toglie. Esc chiude; secondo clic sullo stesso indicatore chiude **senza riaprire**; cliccare un altro indicatore cambia contenuto (rete: Casa, Wi-Fi, Connesso a Internet, 4 su 5; batteria: 85%, a batteria, 2 h 15 min); aprire il Launcher chiude il pannello da solo; con un indicatore assente non c'è nulla da cliccare; sul secondo monitor il pannello si apre su quel monitor. **Il volume vero prima e dopo è identico (100%, non silenziato)**. Schermate controllate. Test: 188 Rust (3 a richiesta), 88 pagina, clippy a 0 avvisi.

Non provato: la scrittura sull'audio vero (volutamente, vedi sopra); scala diversa dal 100% (previsto in I4); tastiere con tasti multimediali che cambiano il volume mentre il pannello è aperto (il pannello segue il sistema dopo 2,5 s, ma non l'ho provato con un cambio vero).

## Fuori scopo

Bluetooth, Non disturbare/Focus, luminosità, tastiera/monitor esterni, Live Activities, estensioni di terze parti per la barra, Centro di controllo completo con pagine e controlli ridimensionabili.

## Rischi aperti

- **Test sulla macchina vera**: la batteria non si può scaricare né collegare a comando e la rete non va staccata. Si prova con una **sorgente simulata** (build di prova, come `test-hooks` nel Launcher) che immette stati finti, e con le funzioni pure; sulla macchina vera si legge soltanto.
- **Volume e audio**: le prove non devono cambiare davvero il volume dell'utente. Il cursore del volume si prova con un dispositivo/valore simulato; se serve una prova reale, solo con l'ok esplicito, un piccolo cambio e ripristino garantito.
- **Desktop senza batteria / senza Wi-Fi / senza uscita audio**: l'indicatore non compare, nessun errore.
- **Dispositivi che cambiano** (cuffie collegate, cavo di rete) e **ripresa dalla sospensione**: gli eventi possono mancare; serve un controllo lento di sicurezza.
- **WinRT e COM**: i thread che li usano devono avere l'apartment giusto, come per la ricerca file del Launcher.
- **Costo**: niente polling frequente; stato inviato solo se cambia e solo alle barre visibili.
- **Tutti i test dal vivo sul PC reale** seguono la regola già nota: nessun clic su icone vere dell'utente, bersaglio controllato prima di ogni clic.
