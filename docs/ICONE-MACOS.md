# Pacchetto di icone in stile macOS per il Dock (12 nov 2026)

Di base le icone del Dock hanno la forma di macOS: tessera arrotondata («squircle» dove il motore lo permette), riflesso in alto, bordo sottile e ombra.
- **App di sistema con icona disegnata apposta** (`src/components/MacIcon.tsx`): Esplora file (faccina del Finder), Impostazioni (ingranaggio), Blocco note, Calcolatrice, Paint, Terminale/PowerShell/Prompt, Gestione attività (grafico), Strumento di cattura, Microsoft Store, Foto, Fotocamera, Orologio. Il riconoscimento (`src/lib/iconpack.ts`) guarda il nome dell'eseguibile e, per le app dello Store, il nome visibile.
- **Tutte le altre app**: la loro icona originale dentro la tessera chiara con un margine, come fa macOS con le app di terze parti.
- **Launchpad**: tessera grigia con i nove quadratini colorati.
- Le pile (cartelle) non cambiano.

Impostazione: Dock → «Icone in stile macOS» (accesa di base); spenta tornano le icone originali di Windows.
Per aggiungere un'icona: una voce in `ART` (MacIcon.tsx) e una riga in `BY_STEM`/`BY_NAME` (iconpack.ts).
Provato: anteprima delle 12 icone in un browser; test Rust (settings) e TypeScript (riconoscimento). Il Dock vero non è catturabile dagli screenshot (sfocatura), quindi l'aspetto nel Dock va guardato a occhio.
