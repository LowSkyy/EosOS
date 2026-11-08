# Centro notifiche e calendario — specifica

Obiettivo: cliccando l'orologio nella barra si apre un pannello in stile Centro notifiche di macOS: in alto le **notifiche recenti** raggruppate per app, sotto un **calendario** del mese. Oggi l'orologio è solo un testo.

Stato: specifica (8 nov 2026) e **N1-N4 e N6 fatte (N5 saltata)**. **Decisioni dell'utente (8 nov 2026): sì a tutto**, cioè è questo il passo successivo, il calendario mostra **solo il mese e il giorno** (niente eventi), e **niente banner temporanei** per le notifiche nuove nella prima versione.

## Cosa fa macOS (da replicare)

Non ho fatto una ricerca nuova per questa parte: è il comportamento noto del Centro notifiche di macOS, da confermare contro le fonti prima di costruire i dettagli.

- Un clic sull'orologio della barra apre un pannello a destra, sotto la barra, che si chiude con Esc, con un clic fuori o ricliccando l'orologio.
- In alto le **notifiche**, raggruppate per app, le più recenti per prime; ognuna si può cancellare con una «×» (o tutto il gruppo); un clic su una notifica apre l'app che l'ha mandata.
- Sotto, il **calendario del mese** con il giorno di oggi evidenziato e i pulsanti per cambiare mese.
- Le notifiche nuove compaiono anche come **avvisi temporanei** in alto a destra (banner) e scompaiono da soli; la modalità **Non disturbare** (Concentrazione) li silenzia.

## Cosa permette Windows (e dove sta il rischio)

Fonti trovate: [Microsoft, notifiche toast da app desktop](https://learn.microsoft.com/en-us/windows/uwp/design/shell/tiles-and-notifications/toast-desktop-apps), [Microsoft, dare un'identità a un'app non impacchettata](https://learn.microsoft.com/cs-cz/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps), [Microsoft, scegliere il modello di pacchetto](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/choose-packaging-model).

**Limite della ricerca:** le fonti dicono che molte funzioni di notifica di Windows richiedono che l'app abbia una «identità di pacchetto» (cioè sia impacchettata, anche in modo leggero) e che le app non impacchettate le chiamano senza successo con errori tipo `APPMODEL_ERROR_NO_PACKAGE`; esiste una via con «pacchetto con posizione esterna» (Windows 10 2004 e successivi) che dà l'identità senza trasformare l'app in MSIX. **Non ho trovato nessuna fonte su `UserNotificationListener`, l'interfaccia che serve a LEGGERE le notifiche di altre app**: se richiede l'identità di pacchetto, se EosOS (oggi un exe normale) può usarla, e se serve il consenso dell'utente. È **la domanda decisiva di tutto il passo**, e non so rispondere senza provare.

Le notifiche di Windows, inoltre, **restano comunque quelle del sistema**: EosOS non può «spostarle» dal Centro notifiche di Windows al suo; al massimo le legge e le mostra, mentre Windows continua a mostrare le sue (a meno di nasconderle con altre impostazioni di sistema, fuori scopo).

## Decisioni che servono da te

1. **È questo il passo giusto?** Il Centro notifiche vero dipende da un'API che potrebbe non essere usabile da EosOS; il **calendario** invece è completamente fattibile. La proposta (consigliata) è fare **prima il calendario** (utile subito, nessun rischio) e **in parallelo una prova di fattibilità** sulle notifiche; se la prova fallisce, il pannello resta calendario + un messaggio onesto, senza finte notifiche.
2. **Cosa deve mostrare il calendario?** Solo il mese e il giorno (consigliato per la prima versione), oppure anche gli **eventi** dal tuo calendario di Windows (richiede un permesso e un'altra API, un'altra incognita).
3. **Banner temporanei per le notifiche nuove:** sì o no (se sì, EosOS dovrebbe far convivere i suoi banner con quelli di Windows: due avvisi per ogni notifica). Consiglio di no nella prima versione.

## Fasi

1. **N1 — Prova di fattibilità, in sola lettura.** Un piccolo strumento che prova a ottenere `UserNotificationListener` dall'exe di EosOS e riferisce: accesso consentito, rifiutato, richiede identità di pacchetto o consenso. Esito: una tabella nella specifica e la decisione (continuare con le notifiche o fermarsi al calendario). Se serve l'identità di pacchetto, la prova dice anche se il «pacchetto con posizione esterna» basta, **senza ancora installare niente**: l'installazione la decidi tu.
2. **N2 — Il calendario (funziona comunque).** Il pannello sotto l'orologio (stessa finestra dei pannelli degli indicatori): mese con giorno di oggi, cambio mese con i pulsanti e con le frecce, ritorno a oggi, settimana che comincia dal lunedì (italiano), numeri di settimana; logica pura e testata (giorni del mese, bisestili, cambio d'anno), aggiornamento a mezzanotte, clic sull'orologio apre/chiude. Nessuna dipendenza da permessi.
3. **N3 — Modello delle notifiche** (solo se N1 dice sì): elenco raggruppato per app, ordine, cancellazione, limiti di numero e di lunghezza; funzioni pure testate.
4. **N4 — Elenco nel pannello:** lettura (con consenso esplicito dell'utente), gruppi, «×», apertura dell'app; nessun testo di notifica nei log.
5. **N5 — Non disturbare** (solo se N1 dice sì e se Windows offre un modo documentato): un interruttore che spegne i banner di EosOS (non quelli di Windows).
6. **N6 — Rifiniture:** tastiera e accessibilità, scale diverse, impostazioni (sezione nelle Impostazioni di EosOS), revisione.

## N1 — Prova di fattibilità: FATTA (8 nov 2026) — **le notifiche si possono leggere**

`notifprobe.rs` (sola lettura, 5 test) chiede a Windows tre cose, senza chiedere consensi né cambiare niente: l'identità di pacchetto del processo (`GetCurrentPackageFullName`), lo stato di accesso di `UserNotificationListener` (`GetAccessStatus`, **mai** `RequestAccessAsync`) e, se è consentito, **quante** notifiche toast ci sono (solo il numero). Il risultato (comando `notif_probe`) contiene codici e parole fisse, mai testi dell'utente.

**Esito su questo PC (Windows build 26300, EosOS come exe normale non impacchettato):**

| Domanda | Risposta |
|---|---|
| EosOS ha un'identità di pacchetto? | **No** (codice 15700, nessun pacchetto) |
| `UserNotificationListener::Current()` | **funziona** (nessun errore) |
| Stato di accesso | **`allowed`**, senza che EosOS abbia chiesto o ricevuto alcun consenso |
| Contare le notifiche toast | **funziona** (3 al momento della prova) |
| Verifica fino in fondo con una notifica MIA | Una finestra di PowerShell manda un toast di prova (titolo `EosOS-N1-prova`); il conteggio delle notifiche di quell'app passa da 0 a 1; tolto il toast torna a 0 |

**Quindi: sì, un exe normale di EosOS può leggere le notifiche su questo PC, senza identità di pacchetto e senza consenso.** La preoccupazione principale della specifica (l'identità di pacchetto) non si è verificata: **non serve cambiare il modo in cui EosOS si installa**.

**Cose da non dare per scontate:**
- È la risposta di **una** build di Windows (26300, 26H2). Su altre versioni lo stato potrebbe essere `unspecified` o `denied`: in quel caso il pannello deve dirlo e mandare alle Impostazioni di Windows, senza fingere. Il codice della prova già distingue i casi (`Usable`, `NeedsConsent`, `NeedsPackageIdentity`, `Unknown`).
- Il fatto che Windows risponda «consentito» a un'app senza consenso è una **scelta di Windows su questa build**: va considerata una caratteristica di sistema da rispettare con attenzione (per questo N4 prevede un **interruttore di EosOS** per spegnere la lettura, spento finché non lo accende l'utente, e nessun testo di notifica nei log né su disco).
- Si è provato il **conteggio** e il riconoscimento di notifiche **di una sola app (la mia)**; leggere titolo e testo delle notifiche e riceverle in tempo reale (evento di cambiamento) sono N3/N4 e non sono ancora stati provati.
- `notif_count_for` (conta le notifiche di una app per identificatore) esiste solo nelle build con `test-hooks`: nelle build normali risponde sempre «niente».

**Verifica** (`scripts/test-notif-probe.ps1`, tutti OK): EosOS senza identità; il listener risponde; contare funziona; il conteggio per l'app di prova sale di uno con la mia notifica e torna a zero quando la tolgo. Durante la prova compare per un attimo un avviso a schermo («EosOS-N1-prova»). Rust 337 test (3 a richiesta), clippy a 0 avvisi (anche con la feature di prova).

## N2 — Il calendario: FATTO (8 nov 2026)

**Cosa fa.** L'orologio nella barra ora è un pulsante («Calendario»): un clic apre sotto di lui il pannello con il **calendario del mese**; un secondo clic, Esc o un clic fuori lo chiudono. È la **stessa finestra dei pannelli degli indicatori** (stessa sfocatura e stessi comportamenti di focus, chiave `calendar`): niente finestra nuova.

- **Il mese:** titolo («Novembre 2026»), sei righe da sette giorni con i giorni dei mesi vicini in grigio, **settimana che comincia da lunedì**, sabato e domenica attenuati, **numeri di settimana ISO 8601** nella prima colonna, il **giorno di oggi** evidenziato in blu.
- **Navigazione:** pulsanti «‹» e «›» (e freccia sinistra/destra, PagSu/PagGiù) cambiano mese attraversando gli anni; **«Oggi»** (o Home) torna al mese corrente (disattivato quando ci si è già); a ogni apertura si riparte dal mese di oggi.
- **Mezzanotte:** il giorno di oggi si riallinea da solo al cambio di data (timer fino alla prossima mezzanotte locale, mai sotto il secondo).
- **Accessibilità:** gruppo «Calendario», tabella con intestazioni di colonna e di riga, ogni giorno con la sua descrizione completa («domenica 8 novembre 2026», con `aria-current="date"` per oggi), settimane con «Settimana 45», titolo del mese con `aria-live`.

**Logica pura e testata** (`lib/calendar.ts`, 18 test, indipendente dal fuso e dalla lingua del sistema: date come terne e calcoli con `Date.UTC`, così un cambio d'ora legale non sposta mai un giorno): anni bisestili (regola dei secoli), giorni del mese, giorno della settimana (con date prima del 1970), somma di giorni, settimana ISO (anni a cavallo come 29 dicembre 2025 = settimana 1, 1 gennaio 2021 = settimana 53, 31 dicembre 2026 = settimana 53), cambio di mese attraverso l'anno, griglia di sei settimane (febbraio 2027 che comincia di lunedì: la sesta riga è tutta di marzo), un solo «oggi», millisecondi alla mezzanotte.

**Verifica dal vivo** (`scripts/test-calendar.ps1`, **23 controlli**): l'orologio è un pulsante con il titolo giusto; un clic apre il pannello e mostra il mese di oggi **confrontato con il calendario di .NET** (nome del mese e anno, colonna del primo del mese, giorni del mese, e **la settimana ISO della prima riga calcolata da .NET Framework**, indipendente dal codice della pagina); un solo giorno evidenziato (oggi, con `aria-current`); pulsanti e tastiera: avanti, indietro due volte, «Oggi», freccia destra, PagSu, Home, **14 mesi avanti attraversando l'anno**; secondo clic chiude, riaperto riparte da oggi, **Esc con la tastiera vera lo chiude** (solo dopo aver verificato che il pannello fosse in primo piano); il pannello degli indicatori (volume) funziona ancora e **non** mostra il calendario. Screenshot reale controllato: la tabella entra nei 300 px del pannello (276/276, nessun scorrimento orizzontale), tema scuro leggibile. Rust 337 test (3 a richiesta), pagina 142, clippy a 0 avvisi. Regressioni: pannello degli indicatori, barra, impostazioni: OK.

**Un test vecchio rimesso a posto:** la sezione G di `test-indicators.ps1` («un clic vero sull'indicatore di rete apre le Impostazioni di Windows») era **rimasta indietro dalla fase I3**: da allora il clic apre il pannello e le Impostazioni di Windows si aprono dal link nel pannello. Falliva già prima di questa fase; ora controlla che il clic apra il pannello.

**Non provato:** il calendario a scale diverse dal 100% (stessa finestra del pannello, già provata nella fase degli indicatori, ma non con il calendario dentro); il cambio di data a mezzanotte dal vivo (la funzione è testata, non ho aspettato la mezzanotte); la settimana che comincia da domenica (non c'è l'impostazione: sarà in N6 se la vuoi).

## N3 — Il modello delle notifiche: FATTO (8 nov 2026)

**Interruttore di lettura, spento di default.** Nuova impostazione «**Mostra le notifiche**» (`notif.read`, sezione **Notifiche** delle Impostazioni di EosOS, che diventa la settima): finché non la accendi tu, `notif_list` risponde «off» **senza leggere niente** e `notif_dismiss` rifiuta tutto. Windows consente la lettura anche senza consenso (vedi N1), quindi il consenso è di EosOS: è una scelta tua e si spegne con un clic.

**Modello puro** (`notifmodel.rs`, 11 test, nessun Windows): dalle notifiche come le legge Windows (`RawNotif`: id, app, righe di testo, istante) ai **gruppi per app** già pronti per il pannello:
- testi ripuliti (caratteri di controllo e a capo → spazi, spazi multipli ridotti, tagliati con «…» senza spezzare un carattere): **titolo 100, corpo 240 caratteri**; le righe dopo il titolo si uniscono con «·»;
- una notifica **senza testo si scarta**; la stessa notifica due volte compare una sola; **app riconosciute senza badare alle maiuscole**;
- **più recenti per prime**, e i gruppi ordinati dalla loro notifica più recente; **tetti**: 30 gruppi, 20 notifiche per gruppo (le altre si contano in `hidden` e nel totale);
- nome dell'app: quello di Windows, altrimenti l'ultimo pezzo dell'identificatore, mai vuoto;
- tempo relativo in italiano («adesso», «3 min fa», «2 ore fa», «ieri», «5 giorni fa», poi la data breve «8 nov»; un istante nel futuro vale «adesso»);
- `contains_id`: si può togliere solo una notifica che c'è.

**Lettura e rimozione** (`notifread.rs`, tocca testi privati, nessun test automatico: la verifica è dal vivo): legge titolo e righe di ogni toast, app e istante dal listener di Windows; i testi restano in memoria, mai nei log né su disco. **Togliere** una notifica la rimuove dal Centro notifiche di Windows (`RemoveNotification`), solo se la lettura è accesa e **l'id è fra quelli che ci sono adesso** (un id inventato si rifiuta).

**Verifica dal vivo** (`scripts/test-notif-model.ps1`, 22 controlli; **solo con notifiche mie**: tre toast mandati da una finestra di PowerShell con titoli noti; la pagina filtra il solo gruppo dell'app PowerShell, quelle di altre app non escono dalla pagina e non si stampano; `mission.json` ripristinato):
- **lettura spenta:** risposta «off», nessun gruppo, nessun totale, e non si toglie niente; l'impostazione risulta spenta;
- **accesa:** le tre notifiche di prova compaiono nel gruppo della loro app (nome «Windows PowerShell»); ordine **tre, due, uno**; il titolo con spazi e tabulazioni diventa «EosOS-N3-due con spazi»; due righe di corpo → «riga 1 · riga 2»; il titolo di 300 caratteri è tagliato **esattamente a 100 con «…»**, il corpo di 400 a **240**; «adesso» per la più recente; identificatori tutti diversi; il totale le conta;
- **togliere:** un id inventato rifiutato; togliere la notifica «due» riesce, sparisce dall'elenco e le altre due restano; spento di nuovo, non si legge né si toglie più niente.

Controlli di lunghezza e di carattere fatti **nella pagina**: la console di PowerShell 5 storpia gli accenti e i puntini nell'output (un «…» si vede come tre caratteri), per cui leggere lì le lunghezze dava risultati falsi. Rust 348 test (3 a richiesta), pagina 142, clippy a 0 avvisi. Regressioni: impostazioni (S1, finestra, S6, con la settima sezione), prova di fattibilità N1: OK.

**Non provato:** l'**evento in tempo reale** quando arriva una notifica (qui si legge a richiesta: N4 rilegge quando il pannello si apre e a intervalli); notifiche con immagini, pulsanti o testi molto strani; notifiche di app dello Store con più associazioni di testo; il comportamento con **Non disturbare / Assistente notifiche** di Windows attivo; la rimozione di una notifica che è sparita nel frattempo (restituisce «falso»).

## N4 — L'elenco nel pannello: FATTO (8 nov 2026)

Cliccando l'orologio il pannello ora ha **le notifiche in alto e il calendario sotto** (`NotifList.tsx` dentro il pannello del calendario; altezza massima del pannello portata da 400 a 640 px, l'elenco scorre dopo 230 px).

- **Lettura spenta (predefinito):** il pannello dice «La lettura delle notifiche è spenta. Accendendola, EosOS mostra qui le notifiche di Windows (restano solo in memoria)» con il pulsante **«Mostra le notifiche»**, che accende l'impostazione e rilegge. Nessuna notifica viene letta finché non lo fai.
- **Acceso:** intestazione «Notifiche (n)»; **un gruppo per app** (nome dell'app in maiuscoletto) con le sue notifiche, ognuna con **titolo, corpo (al massimo due righe) e «quanto fa»**, le più recenti per prime; «+n altre» sotto un gruppo che supera il tetto; «Nessuna notifica» se non ce ne sono; un messaggio se Windows non le dà.
- **«×» su una notifica:** la toglie **subito** dal pannello e dal Centro notifiche di Windows (la rilettura che segue conferma lo stato vero). **«×» sul gruppo:** toglie tutte le notifiche dell'app, **anche quelle oltre il tetto di 20** (`ids_of_app`, testata), solo se l'app ha notifiche adesso.
- **Un clic su una notifica apre l'app** che l'ha mandata (come un clic sul avviso di Windows) e **toglie la notifica**; il pannello si chiude. Si apre con `shell:AppsFolder\<identificatore>`: funziona per le app dello Store e per quelle che Windows conosce con quell'identificatore; per le altre risponde «no» e non succede niente. L'identificatore si controlla prima (`valid_app_id`: non vuoto, al massimo 300 caratteri, senza controlli né virgolette, barre verticali, `<` `>`).
- **Aggiornamento:** si rilegge all'apertura e **ogni 4 secondi** mentre il pannello è aperto; una risposta più vecchia non sovrascrive una più nuova (`isLatest`); se lo spegni dalle Impostazioni mentre è aperto, entro la rilettura torna a dire che è spento.
- **Accessibilità:** sezione «Notifiche», gruppi con il nome dell'app, ogni notifica apribile con un nome («Apri Messaggi: Anna») e ogni «×» con il suo («Rimuovi la notifica «Anna» di Messaggi», «Rimuovi tutte le notifiche di Messaggi»), errori con `role="alert"`.

**Verifica dal vivo** (`scripts/test-notif-panel.ps1`, **20 controlli**, solo con notifiche mie — tre toast «EosOS-N4-…» da PowerShell — leggendo dal pannello **solo il gruppo dell'app PowerShell**; `mission.json` ripristinato):
- **A.** lettura spenta: si dice, nessuna notifica mostrata, il calendario c'è ancora; «Mostra le notifiche» accende la lettura (nelle Impostazioni risulta accesa) e lo stato cambia;
- **B.** le tre notifiche compaiono nel gruppo «Windows PowerShell» entro la rilettura periodica, **dalla più recente: tre, due, uno**, con corpo e «adesso», e il totale nell'intestazione;
- **C.** «×» su «due»: sparisce subito e **resta sparita dopo la rilettura** (tolta davvero da Windows);
- **D.** un clic su «uno»: Rust chiede di aprire **l'app giusta** (l'identificatore di PowerShell; nelle prove `EOSOS_TEST_NO_OPEN` registra la richiesta senza lanciare niente), il pannello si chiude e la notifica non c'è più;
- **E.** «×» del gruppo (provato **solo se nel gruppo ci sono soltanto le mie**): toglie tutto e non ricompare dopo la rilettura;
- **F.** spenta mentre il pannello è aperto: entro la rilettura torna «spenta», senza notifiche.
Controllato anche con uno screenshot della struttura dell'elenco con dati finti (non si sono mai visualizzate notifiche vere): gruppi, titoli, corpo a due righe, «quanto fa», «+1 altra», scorrimento sottile. Rust 350 test (3 a richiesta), pagina 148, clippy a 0 avvisi. Regressioni: calendario, pannello degli indicatori, N3, impostazioni: OK.

**Non provato:** **aprire davvero un'app** da una notifica (nelle prove non si lancia niente; si è provata solo la richiesta con l'identificatore giusto e il rifiuto di identificatori sospetti nei test puri); notifiche con immagini o pulsanti; il pannello con molte notifiche vere insieme (si è provato il tetto solo nei test puri); le notifiche che arrivano mentre il pannello è chiuso (si vedono alla prossima apertura); un secondo monitor.

## N5 — Non disturbare: SALTATA (8 nov 2026)

Non fatta: l'utente ha chiesto di procedere con N6. Si puo' riprendere in futuro dopo aver verificato che Windows offra un modo documentato per leggere/impostare l'Assistente notifiche.

## N6 — Rifiniture: FATTE (8 nov 2026)

- **«La settimana comincia da» (Impostazioni, sezione «Notifiche e calendario»):** lunedi' (predefinito) o domenica. Vale subito (`calendar.week_start`, salvato in `mission.json` come `calendarSunday`). Intestazioni, prima colonna e weekend seguono la scelta; il numero di settimana ISO si legge sempre dal giovedi' della riga. Valori strani vengono rifiutati.
- **Accessibilita':** ogni pulsante ha un nome, la tabella ha titolo e intestazioni di colonna/riga, ogni giorno ha la data completa, il titolo del mese e' annunciato (`aria-live`), il focus da tastiera ha un contorno.
- **Impostazioni:** ora 23, la sezione si chiama «Notifiche e calendario».

**Verifica dal vivo:** `scripts/test-calendar-n6.ps1` (20 controlli: lunedi'/domenica confrontati con .NET, accessibilita'); `scripts/test-calendar-dpi.ps1` (scala vera del primario a 100/125/150/175%, **su entrambe le barre** il pannello si apre sotto l'orologio, dentro il monitor, di 300 px CSS, senza tagli; scala ripristinata a 100%). Regressioni: impostazioni (3 script), calendario, N3, N4, pannello: OK. Vitest 153, Rust 350, clippy 0.

**Non provato:** tema chiaro dal vivo (i colori del pannello usano solo i token del tema, nessun colore fisso); passaggio di mezzanotte dal vivo; aprire davvero un'app da una notifica; notifiche con immagini/pulsanti.

## Principi (gli stessi di sempre)

- Rust decide, la pagina mostra; **le notifiche contengono testi privati**: mai nei log, mai nei test stampati, mai su disco (solo in memoria), con un interruttore per spegnere la lettura.
- Prove dal vivo solo con notifiche **create da me** (una piccola app di prova che manda toast con un titolo noto); mai lette o toccate quelle dell'utente.
- Funzioni pure testate; clippy a 0; prove a scale diverse.

## Fuori scopo

Sostituire il Centro notifiche di Windows o nascondere le sue notifiche; rispondere alle notifiche dal pannello; widget; sincronizzare con altri dispositivi.

## Rischi aperti

- L'API per leggere le notifiche potrebbe non essere usabile da un exe non impacchettato: in quel caso il passo si riduce al calendario (N2).
- Il consenso dell'utente per leggere le notifiche è un permesso di sistema che si dà dalle Impostazioni di Windows: la finestra delle impostazioni di EosOS può solo aprirle.
- Dare l'identità di pacchetto cambia il modo in cui EosOS si installa e si avvia: non lo faccio senza il tuo ok esplicito.
