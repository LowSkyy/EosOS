# Launcher — specifica

Obiettivo: un Launcher in stile Spotlight di macOS 27 "Golden Gate" per EosOS su Windows 11: una scorciatoia da tastiera apre un campo di ricerca al centro dello schermo e da lì si avviano app, si trovano file, si eseguono azioni e si ritrova ciò che si è copiato.

## Cosa fa Spotlight (da replicare) e da dove viene

Fonti: [Apple Support: cronologia degli appunti](https://support.apple.com/guide/mac-help/search-your-clipboard-history-mchl40d5b86b/mac), [Paste](https://pasteapp.io/blog/macos-tahoe-clipboard-history), [MacMost](https://macmost.com/how-to-use-the-spotlight-clipboard-history-in-macos-tahoe.html).

**Limite della ricerca**: per macOS 27 non ho trovato novità di Spotlight verificabili; le fonti descrivono Spotlight di macOS 26 (Tahoe) e dicono che in macOS 27 la cronologia degli appunti resta invariata. Quindi la specifica replica quel comportamento, non novità non verificate.

- Un pannello di vetro, in alto rispetto al centro dello schermo, con un campo di ricerca grande. Compare con una scorciatoia e sparisce con Esc o cliccando fuori.
- Risultati a elenco sotto il campo, con icona, nome e tipo; si naviga con le frecce e si apre con Invio.
- Quattro schede: **App**, **File**, **Azioni**, **Appunti**, richiamabili da tastiera (Cmd+1…4 su Mac; qui Ctrl+1…4).
- Calcoli e conversioni digitando nel campo (`12*7`, `20% di 340`).
- Cronologia degli appunti breve (ore, non archivio), con richiesta di permesso al primo uso.
- Le app usate più spesso salgono in cima ("frecency": frequenza e recenza).

## Scelte tecniche (proposte)

- **Finestra**: nuova finestra Tauri "launcher", senza bordi, trasparente, sempre in primo piano, centrata a un terzo dall'alto del monitor dove sta il cursore. A differenza del Dock **deve poter ricevere la tastiera**: niente `WS_EX_NOACTIVATE`; all'apertura si porta in primo piano (permesso, perché parte da una scorciatoia premuta dall'utente) e quando perde il focus si nasconde.
- **Vetro**: la stessa pipeline del Dock (`backdrop.rs`: cattura dello schermo dietro, sfocatura e ritaglio nel CSS, finestra esclusa dalla cattura). Per un pannello rettangolare arrotondato si può riusare così com'è.
- **Scorciatoia globale**: `RegisterHotKey`. Predefinita **Alt+Spazio** (come PowerToys Run; sostituisce il menu di sistema della finestra, che quasi nessuno usa). Va resa configurabile e, se la registrazione fallisce (combinazione già presa), il Launcher deve dirlo invece di non aprirsi in silenzio. `Win+Spazio` e `Ctrl+Spazio` sono occupate (lingua di input).
- **App**: si elenca `shell:AppsFolder` (programmi classici e app dello Store insieme, con nome, icona e identificatore), la stessa fonte che il Dock usa già per l'avvio e le icone (`SHCreateItemFromParsingName`, `IShellItemImageFactory`, `ShellExecute` su `shell:AppsFolder\...`). L'indice si ricostruisce in un thread, non all'apertura.
- **Sicurezza come nel Dock**: il frontend manda solo l'`id` di un risultato che il Launcher gli ha mostrato; il comando o il percorso da eseguire li ricava Rust. La pagina non può far eseguire stringhe arbitrarie.
- **Classifica**: corrispondenza approssimata (inizio di parola, sigle: `vsc` → Visual Studio Code, sottosequenza) più frecency salvata in `%LOCALAPPDATA%\EosOS\launcher.json`. Funzioni pure e testate, come `merge` nel Dock.
- **File**: partire da ciò che Windows già indicizza (Windows Search tramite `ISearchQueryHelper`/OLE DB `SystemIndex`) invece di riscansionare il disco; se l'indice non c'è si cade su file recenti e cartelle note. Mai scansioni ricorsive sul thread dell'interfaccia.
- **Azioni**: calcolatrice (parser di espressioni in Rust, senza `eval`), comandi di sistema (blocca, sospendi, riavvia, spegni: questi ultimi con conferma), apertura di Impostazioni e cartelle note.
- **Appunti**: `AddClipboardFormatListener` su una finestra nascosta; solo testo nella prima versione; cronologia **in memoria**, non su disco, con scadenza (ore) e limite di voci. Si ignorano le copie che i gestori di password marcano con `ExcludeClipboardContentFromMonitorProcessing` / `CanIncludeInClipboardHistory = 0`. Nessun contenuto negli log.
- **Impostazioni** in `launcher.json` (scorciatoia, schede attive, durata della cronologia), con la stessa robustezza di `dock.json`: tolleranza del BOM, file illeggibile conservato come `.bad`, scrittura atomica sotto lock, nessuna riscrittura dopo una lettura fallita.

## Fasi

1. **L1 – Dati e classifica (Rust)**: indice delle app da `shell:AppsFolder` (nome, id, icona), corrispondenza approssimata, frecency, comando `launcher_search`. Test unitari su classifica e punteggi.
2. **L2 – Finestra e scorciatoia**: finestra del Launcher con vetro, campo di ricerca e risultati con le frecce, scorciatoia globale con gestione del conflitto, apertura/chiusura (Esc, perdita del focus), avvio di un'app con Invio.
3. **L3 – Azioni e calcolatrice**: parser di espressioni, comandi di sistema con conferma, schede da tastiera.
4. **L4 – File**: ricerca nell'indice di Windows Search con ripiego.
5. **L5 – Appunti**: cronologia in memoria con esclusioni, scheda dedicata, richiesta di consenso al primo uso.
6. **L6 – Rifiniture**: scorciatoia configurabile, animazioni, più monitor (si apre dove sta il cursore), DPI, riavvio di Explorer, revisione di codice.

## Fuori scopo

Anteprime dei file (Quick Look), Siri/IA, ricerca nel contenuto dei file, sincronizzazione degli appunti tra dispositivi, plugin di terze parti.

## Rischi aperti

- **Focus**: portare una finestra in primo piano da una scorciatoia globale funziona di norma, ma su alcune configurazioni Windows lo nega; serve il ripiego già usato dal Dock (`dockact::focus`), senza l'Alt finto se non serve.
- **IME e tastiera**: il campo di ricerca deve funzionare con le tastiere internazionali e gli editor di metodo di input (accenti, cinese/giapponese).
- **Indice dei file**: Windows Search può essere disattivato o incompleto (OLE DB `SystemIndex` non sempre disponibile).
- **Appunti**: privacy; cronologia solo in memoria e sempre svuotabile. Non deve mai registrare password.
- **Scorciatoia**: `Alt+Spazio` può essere già presa da altri programmi (PowerToys Run, launcher di terzi).
- **Tutti i test dal vivo sul PC reale** seguono la regola già nota: app di prova, nessun clic su icone vere dell'utente, bersaglio controllato prima di ogni clic.
