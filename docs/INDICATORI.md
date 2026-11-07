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
- **Verifica** (`scripts/test-indicators.ps1`, 22 controlli, dalla pagina via CDP): lo stato iniziale arriva con `indicators_get`; **nessun evento in 4 s senza cambi**; un cambio produce **un solo** evento con i dati giusti (batteria al 19% con tempo residuo, Wi-Fi senza Internet, audio disattivato); la scomparsa di batteria e audio dà campi nulli e un evento; un file illeggibile non cambia nulla; valori fuori scala (volume 250, batteria 150) sono limitati a 100. Lettura vera in sola lettura sul tuo PC: **nessuna batteria** (desktop: indicatore assente come previsto), rete **via cavo (Ethernet)**, volume **100%**. Test: 177 Rust, clippy a 0 avvisi.

Non provato: batteria vera (il PC non ne ha), Wi-Fi vero (c'è solo il cavo), cambio vero del dispositivo audio, ripresa dalla sospensione.

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
