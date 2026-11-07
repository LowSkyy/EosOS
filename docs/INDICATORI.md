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
