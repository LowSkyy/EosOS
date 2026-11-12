# Finder di EosOS (12 nov 2026)

L'interno di Esplora file (barra laterale, comandi, percorso, icone) lo disegna Windows e non si puo' ridisegnare da fuori. Per avere cartelle e file come su macOS EosOS ha una finestra propria: il **Finder**.

## Come si apre
- Clic sull'icona di Esplora file (il Finder) nel Dock. Se una finestra del Finder e' gia' aperta, la porta davanti.
- Impostazioni -> Aspetto -> «Finder di EosOS al posto di Esplora file» (accesa di base): spegnendola l'icona torna ad aprire Esplora file.
- Esplora file resta raggiungibile: clic destro -> «Mostra in Esplora file».

## Cosa c'e'
- Semafori a sinistra (veri pulsanti della finestra: nessuna finestrina sopra), barra laterale arrotondata con Preferiti (cartella utente, Scrivania, Documenti, Download, Immagini, Musica, Filmati) e Posizioni (unita').
- Barra degli strumenti: indietro/avanti, titolo, vista a icone o a elenco, nuova cartella, ricerca nella cartella.
- Icone in stile macOS: cartella blu con il simbolo impresso per le cartelle dell'utente, documenti per tipo di file; anteprima vera per immagini e video; icona di Windows per programmi e collegamenti. Tema chiaro/scuro come «Tema delle icone».
- Barra del percorso cliccabile e conteggio in basso.
- Operazioni: apri (doppio clic), rinomina (Invio o F2), nuova cartella (Ctrl+Maiusc+N), copia/taglia/incolla (Ctrl+C/X/V, fra finestre del Finder), sposta nel Cestino (Canc), seleziona tutto, elementi nascosti (Ctrl+Maiusc+.), nuova finestra (Ctrl+N), chiudi (Ctrl+W), frecce per muoversi, Backspace = indietro, Alt+Su = cartella superiore.
- Copia, spostamento e cestino passano per Windows: avanzamento, domande sui conflitti e «Annulla» sono quelli di sistema.

## Fase 2 (12 nov 2026, sera)
- **Stessa identita' per tutte le finestre**: i semafori di Finder, Impostazioni e Mission Control sono lo stesso componente (`src/components/Lights.tsx`) e seguono «Lato dei semafori», come le finestre delle app. A sinistra: chiudi, riduci, ingrandisci; a destra: riduci, ingrandisci, chiudi. Nelle Impostazioni riduci e ingrandisci sono grigi (come nei pannelli di macOS); in Mission Control il «x» e' diventato il cerchio rosso.
- **Appunti di Windows**: copia/taglia/incolla usano gli appunti veri, quindi funzionano anche con Esplora file, il desktop, le email e le chat.
- **Trascinamento**: si trascinano i file verso altre cartelle (anche nella barra laterale e nel percorso), altre finestre del Finder, Esplora file, il desktop e le altre app; si lasciano nel Finder file presi da fuori. Stessa unita' = sposta, altra unita' o Ctrl = copia (come Esplora file).
- **Ricerca nelle sottocartelle**: con del testo nella ricerca si sceglie «questa cartella» o «e sottocartelle» (fino a 500 risultati, 6 secondi).
- **Barra laterale**: OneDrive (se c'e'), Cestino, Etichette.
- **Cestino**: elenco con posizione originale e data, Ripristina, Elimina subito, Svuota (con conferma).
- **Etichette colorate** (7 colori di macOS) dal menu del clic destro; si vedono accanto al nome e nella barra laterale. Si salvano in `%LOCALAPPDATA%\EosOS	ags.json` (solo EosOS le vede).
- **Viste**: icone, elenco, colonne, galleria (Ctrl+1..4).
- **Anteprima rapida** con la barra spaziatrice (testo per testi e codice, immagine grande per il resto), **Ottieni informazioni** (Ctrl+I: dimensione anche delle cartelle, date, attributi), **Vai alla cartella** (Ctrl+Maiusc+G, anche percorsi di rete `\server\condivisione`).

## Limiti dichiarati
- Le etichette sono di EosOS: Esplora file non le vede.
- Le finestre «Apri/Salva» delle app e le cartelle aperte da altre app usano ancora Esplora file.
- Niente «Recenti», AirDrop, smart folder, tag nelle ricerche.
- Trascinando dentro il Finder il cursore mostra «copia» anche quando sposta (lo decide Windows per il bersaglio).

## Codice
`src-tauri/src/finder.rs` (finestre, elenco, operazioni; comandi `finder_*` permessi solo alle finestre `finder-N`), `src/components/Finder.tsx`, `src/lib/finder.ts`, `src/finder.css`, disegni in `src/components/iconart.tsx` (`folderArt`, `driveArt`).

## Prove
Rust (6 test: percorsi, nomi, elenco/rinomina/nuova cartella su una cartella temporanea) e TypeScript (6 test: tipi, ordinamento, selezione, percorso). Aspetto controllato in un browser con dati di prova (icone e elenco, chiaro e scuro). **Non ancora provato dentro EosOS con clic veri** (fase 1 e fase 2): apertura dal Dock, operazioni sui file, ridimensionamento della finestra.
