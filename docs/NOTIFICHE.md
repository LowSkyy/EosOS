# Centro notifiche e calendario — specifica

Obiettivo: cliccando l'orologio nella barra si apre un pannello in stile Centro notifiche di macOS: in alto le **notifiche recenti** raggruppate per app, sotto un **calendario** del mese. Oggi l'orologio è solo un testo.

Stato: **solo specifica (8 nov 2026), niente codice.** Il passo l'ho scelto io, come l'ultima volta, fra quelli rimasti (l'altro candidato era la gestione delle finestre in stile Stage Manager). Se preferisci l'altro, dimmelo e riscrivo.

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
