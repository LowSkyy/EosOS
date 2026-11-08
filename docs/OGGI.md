# Widget «Oggi»

Stato: **fatto** (9 nov 2026). Decisioni dell'utente: **meteo con Open-Meteo, a scelta (spento di default)**; **promemoria miei, dentro EosOS**.

## Cosa fa
In cima al pannello sotto l'orologio (sopra le notifiche e il calendario):
- **La data per esteso** («lunedì 9 novembre 2026»).
- **Meteo** (solo se acceso): simbolo, temperatura attuale, città, condizione e minima/massima di oggi («☁️ 10° · Roma · Nuvoloso · 10° / 17°»). Si accende in Impostazioni → *Notifiche e calendario* → **Meteo nel pannello Oggi**, e si sceglie la **Città del meteo** (campo di testo; vuoto = nessuna; si salva con Invio o uscendo dal campo). Acceso senza città: avviso «scegli la città nelle Impostazioni». Errori leggibili («Non trovo la città «X»», «Meteo non disponibile: …»); il pannello funziona comunque.
- **Promemoria**: testo (fino a 120 caratteri) e ora facoltativa (HH:MM); un cerchio per segnarli fatti (barrati, in fondo), «×» per toglierli, «Svuota completati». Ordine: da fare con ora (dalla più vicina), da fare senza ora, fatti. Quelli con un'ora già passata e non fatti hanno l'ora in rosso. Fino a **50**. **Nessun allarme né notifica**: l'ora serve a ordinare e segnalare.

## Privacy e rete
- **Spento di default: nessuna richiesta di rete.** Acceso e con una città, EosOS chiede a `geocoding-api.open-meteo.com` (dove sta la città) e a `api.open-meteo.com` (le previsioni di oggi), **al massimo ogni 30 minuti** (copia in memoria per la stessa città) e solo quando apri il pannello. All'esterno va **solo il nome della città** (più l'IP, come in ogni richiesta). Indirizzi fissi nel codice; risposta limitata a 100 KB, letta con `serde_json` e scartata se strana (coordinate o temperature fuori scala); tempo massimo 8 s. La richiesta usa il client HTTP di Windows (stesso proxy del sistema).
- I **promemoria** stanno solo nel file locale `%LOCALAPPDATA%\EosOS\mission.json`; non escono dal PC.
- I comandi `oggi_*` si accettano solo dal pannello; la città e l'interruttore del meteo si cambiano solo dalle Impostazioni (`cmdpolicy.rs`).

## Come è fatto
Rust: `reminders.rs` (puro: pulizia testo e ora, aggiungi/segna/togli/svuota, ordine, limite), `weather.rs` (puro: città valida, codifica degli indirizzi, lettura delle risposte, descrizione dei codici WMO; rete e cache), nuovo tipo di impostazione *testo* (`Kind::Text`), `mcstore` (campi `reminders`, `weatherOn`, `weatherCity`). Pagina: `OggiBox.tsx`, `lib/oggi.ts`. L'altezza massima del pannello passa da 640 a 760 px.

## Verifica
- `scripts/test-oggi.ps1` (**36 controlli**; il meteo viene da un server **locale** che imita Open-Meteo, `scripts/weather-mock-server.py`; nessuna richiesta esce dal PC): data; meteo spento = zero richieste; promemoria (aggiungi, ordine, in ritardo, fatto, riapertura, file locale, svuota, togli, testo vuoto e ora impossibile rifiutati da Rust, tetto di 50); meteo (avviso senza città, città non valida rifiutata, dati corretti e nome per i lettori di schermo, una ricerca e una previsione, **nessuna nuova richiesta entro 30 minuti**, città inesistente, server spento, spegnimento); comandi rifiutati da barra, Launcher e per la città dal pannello.
- Il formato delle risposte è stato confrontato con **una vera risposta di Open-Meteo** (città e previsioni di Roma) prima di scrivere la lettura.
- Regressioni: impostazioni (4 script), calendario, calendario a 100/125/150/175% di scala (scala ripristinata), notifiche, Non disturbare, pila, aggiornamenti, comandi per finestra: OK. Rust 383 test, pagina 162, clippy 0.
- **Non provato:** il meteo con il servizio vero dentro EosOS (le prove usano il server locale; il parsing è sulla risposta vera); città omonime (si prende il primo risultato di Open-Meteo); il pannello su schermi bassi con tutti i blocchi pieni (l'altezza si limita al monitor e il resto scorre); tema chiaro.
