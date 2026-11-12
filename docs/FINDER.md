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

## Limiti dichiarati
- Copia/incolla solo fra finestre del Finder (non con gli appunti di Esplora file); niente trascinamento dei file.
- La ricerca filtra la cartella corrente, non le sottocartelle.
- Niente Cestino, rete, OneDrive come voci dedicate, ne' colonne/galleria/tag/anteprima rapida.
- Le finestre «Apri/Salva» delle app e le cartelle aperte da altre app usano ancora Esplora file.

## Codice
`src-tauri/src/finder.rs` (finestre, elenco, operazioni; comandi `finder_*` permessi solo alle finestre `finder-N`), `src/components/Finder.tsx`, `src/lib/finder.ts`, `src/finder.css`, disegni in `src/components/iconart.tsx` (`folderArt`, `driveArt`).

## Prove
Rust (6 test: percorsi, nomi, elenco/rinomina/nuova cartella su una cartella temporanea) e TypeScript (6 test: tipi, ordinamento, selezione, percorso). Aspetto controllato in un browser con dati di prova (icone e elenco, chiaro e scuro). **Non ancora provato dentro EosOS con clic veri**: apertura dal Dock, operazioni sui file, ridimensionamento della finestra.
