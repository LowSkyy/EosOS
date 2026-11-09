# Badge delle notifiche sul Dock

Sull'icona di un'app nel Dock compare una **pallina rossa col numero di notifiche non lette** (99+ oltre le 99), come in macOS.

## Quando compare
- Solo con **«Mostra le notifiche» acceso** (Impostazioni → Notifiche) e **Non disturbare spento**: con una delle due cose contrarie
  EosOS non legge niente e non c'è nessun badge.
- Si spegne a parte da Impostazioni → Dock → «Badge delle notifiche sulle icone».
- Il Dock chiede i conteggi ogni 4 secondi (Rust riusa l'ultima lettura per 2,5 s). Nessun testo di notifica arriva al Dock: solo il numero.

## Come si abbina una notifica a un'icona (prudente: meglio nessun badge che quello sull'icona sbagliata)
Una notifica va a **una sola icona** (la prima che combacia) se:
- il nome dell'app nella notifica è quello dell'icona o del suo eseguibile (senza badare a maiuscole, spazi e simboli; almeno 3 caratteri), oppure
- uno dei «pezzi» dell'identificatore dell'app (`com.squirrel.Discord.Discord` → com, squirrel, discord, discord) è proprio il nome
  dell'eseguibile (almeno 4 lettere).
Le app di Microsoft Store con identificatori molto diversi dal nome dell'eseguibile possono non avere il badge (i test puri documentano i casi).

## Dove sta il codice
`src-tauri/src/dockbadge.rs` (abbinamento e conteggio puri, 8 test; comando `dock_badges`, solo per le finestre del Dock — `cmdpolicy.rs`),
`src/lib/badge.ts` (mappa e testo accessibile, testati), `Dock.tsx` (sondaggio e pallina) e `styles.css` (`.dock-badge`).
Il testo accessibile dell'icona dice anche quante sono: «Discord, 3 notifiche».

## Prove
`scripts/test-dock-badges.ps1`: Dock di prova con la sola icona di PowerShell e **toast veri ma solo miei** («EosOS-B-…», tolti alla fine):
nessun badge con la lettura spenta, «2» poi «3» con la lettura accesa, sparisce con Non disturbare e con l'impostazione spenta (e torna),
sparisce quando le notifiche vengono tolte, il testo accessibile, e la barra/il Launcher non ricevono i badge. `dock.json` e `mission.json` ripristinati.
**Non provato:** abbinamento con le tue app vere (Discord, Chrome, app dello Store) e icone di app nel Dock dopo molte ore di uso.
