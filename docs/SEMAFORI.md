# Semafori in stile macOS al posto dei tasti di Windows (11 nov 2026)

Su tutte le finestre con barra del titolo, chiudi / riduci a icona / schermo intero diventano i tre cerchi rosso, giallo e verde di macOS. Il simbolo (×, −, frecce) compare passando il mouse; le finestre non attive hanno i cerchi grigi.

## Come funziona (e perché non tocca le altre app)
Windows non permette a un programma di cambiare i tasti della barra del titolo di un'altra app. EosOS quindi disegna, per ogni finestra, una finestrina «a strati» senza focus (`src-tauri/src/lights.rs`) **sopra** i tasti di Windows: ha il colore della barra (letto dallo schermo), copre i tasti veri e disegna i cerchi. Il clic manda alla finestra lo stesso comando dei tasti veri (`WM_SYSCOMMAND`: chiudi, riduci, ingrandisci/ripristina). Le finestrine seguono la finestra (spostamento, ordine, attivazione, riduzione) e spariscono con lei.

Se EosOS si chiude o si blocca, le finestrine spariscono e **i tasti di Windows tornano da soli**: non è stato modificato nulla nelle altre app.

## Impostazioni (Aspetto)
- «Semafori al posto dei tasti di Windows» (acceso di base).
- «Lato dei semafori»: a sinistra come macOS (di base) o a destra come Windows (ordine: riduci, schermo intero, chiudi).
- «Stile Finder per Esplora file»: toglie il bordo sottile alle finestre di Esplora file (restano arrotondate); si ripristina spegnendo l'opzione.

## Limiti dichiarati
- **A sinistra i cerchi coprono l'icona e l'inizio del titolo** (o la prima scheda in Esplora file/Blocco note): EosOS non può spostare il contenuto delle altre app. A destra non copre nulla di più dei tasti di Windows.
- «Schermo intero» (verde) = ingrandisci/ripristina la finestra, come il tasto di Windows: lo schermo intero vero lo gestiscono le singole app (F11).
- Il colore della barra si legge dallo schermo: con barre sfumate o con immagini la finestrina può differire di poco.
- Niente semafori su finestre senza barra del titolo di sistema (app «senza cornice»), finestre strumento, finestre a schermo intero (video, giochi) e mentre è attiva la schermata di blocco.
- **Esplora file non è ridisegnato**: barra laterale, barra dei comandi e percorso li decide Windows. Cambia solo il bordo e, con i semafori, la barra del titolo.

## Prove
Rust: ordine e disposizione dei cerchi, clic, colori, simboli, angolo arrotondato, comandi (10 test). Dal vivo, con una finestra di prova e clic veri (build `test-hooks`): a sinistra e a destra i cerchi compaiono al posto dei tasti, verde ingrandisce e ripristina, giallo riduce a icona, rosso chiude.
