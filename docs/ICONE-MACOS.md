# Pacchetto di icone in stile macOS per il Dock (12 nov 2026)

Di base le icone del Dock hanno la forma di macOS: tessera arrotondata («squircle» dove il motore lo permette), riflesso in alto, bordo sottile e ombra.
- **App di sistema con icona disegnata apposta** (`src/components/MacIcon.tsx`): Esplora file (faccina del Finder), Impostazioni (ingranaggio), Blocco note, Calcolatrice, Paint, Terminale/PowerShell/Prompt, Gestione attività (grafico), Strumento di cattura, Microsoft Store, Foto, Fotocamera, Orologio. Il riconoscimento (`src/lib/iconpack.ts`) guarda il nome dell'eseguibile e, per le app dello Store, il nome visibile.
- **Tutte le altre app**: la loro icona originale dentro la tessera chiara con un margine, come fa macOS con le app di terze parti.
- **Launchpad**: tessera grigia con i nove quadratini colorati.
- Le pile (cartelle) non cambiano.

Impostazione: Dock → «Icone in stile macOS» (accesa di base); spenta tornano le icone originali di Windows.
Per aggiungere un'icona: una voce in `ART` (MacIcon.tsx) e una riga in `BY_STEM`/`BY_NAME` (iconpack.ts).
Provato: anteprima delle 12 icone in un browser; test Rust (settings) e TypeScript (riconoscimento). Il Dock vero non è catturabile dagli screenshot (sfocatura), quindi l'aspetto nel Dock va guardato a occhio.

## Aggiornamento 12 nov: tema scuro, tutte le schermate e icone di Esplora file
- **Finder**: l'icona di Esplora file nel Dock e ovunque in EosOS e' il Finder di macOS (volto diviso azzurro/blu), ridisegnato piu' fedele. L'immagine ufficiale di Apple non si puo' copiare: e' un disegno nostro nello stesso stile.
- **Tema delle icone** (Impostazioni → Dock → «Tema delle icone»): Automatico (come le app di Windows), Chiaro o Scuro. Cambia sfondo e simboli di tutte le tessere.
- **Dove compaiono**: Dock, Launcher (risultati «app»), Launchpad e pannello delle app in background. Cartelle, file e icone dentro le app di altri restano di Windows.
- **Cartelle e file di Esplora file** (Impostazioni → Dock → «Icone di cartelle e file in stile macOS», spenta di base): cartella blu in stile macOS e icone per testo, immagini, audio, video, archivi, PDF, Word, Excel, PowerPoint e codice. Si scrive solo nel registro dell'UTENTE (`HKCU\Software\Classes\...\DefaultIcon`), le `.ico` stanno in `%LOCALAPPDATA%\EosOS\icons`, ogni chiave toccata e' ricordata in `iconpack.json`. Spegnendo l'opzione le chiavi tornano come erano (valore precedente rimesso, chiavi create da noi eliminate). Esplora file puo' aggiornare le icone solo dopo un attimo o riaprendo la finestra.
- Le `.ico` si rigenerano con `python scripts/build_icons.py` (Chrome + Pillow) dai disegni in `src/components/iconart.tsx`.
- Provato: anteprima chiara/scura delle tessere e delle icone dei file; prova sul registro vero (applica → controlla → ripristina, ignorata di base: `cargo test --lib real_registry_round_trip -- --ignored`).
