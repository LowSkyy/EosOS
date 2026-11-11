# Schermata di blocco di EosOS (11 nov 2026)

Una schermata di blocco in stile macOS: sfondo del desktop sfocato, data e orologio grandi, avatar con la lettera del nome, campo password a pillola. Come su Windows, l'accesso compare **solo sul monitor principale**; gli altri monitor restano neri (in standby, senza contenuto) fino allo sblocco.

## Cosa NON è (da dire chiaro)
Non sostituisce la schermata di accesso di Windows. Quella (all'accensione, con `Win+L`, al risveglio) gira su un desktop protetto (`LogonUI`) a cui un programma normale non ha accesso: sostituirla vorrebbe dire modificare file di sistema o installare un credential provider, con il rischio di restare fuori dal PC, e comunque PIN e Windows Hello non si potrebbero conservare (li verifica solo Windows). EosOS parte dopo l'accesso, quindi non può comparire prima.
È un **blocco «da scrivania»**: tiene lontani gli sguardi e le mani mentre ti alzi per qualche minuto. Chi preme `Ctrl+Alt+Canc` (la combinazione la gestisce solo Windows), riavvia il PC o ha accesso fisico lo aggira; se EosOS si chiude, lo schermo risulta sbloccato. Per questo, dopo un tempo scelto (di base 5 minuti) EosOS chiama anche il **blocco vero di Windows**.

## Come si attiva
- **Blocca lo schermo** nel menu a sinistra (logo EosOS) e nel Launcher (azione «Blocca»).
- **`Ctrl+Alt+L`** (scorciatoia globale; se è già usata da un altro programma non si registra e resta il menu).
- **Dopo l'inattività** (Impostazioni → Generali → «Blocca dopo l'inattività»: Mai di base, 5, 10, 15, 30 minuti).
- Se spegni «Schermata di blocco EosOS», «Blocca lo schermo» usa il blocco di Windows (`LockWorkStation`).

## Come si sblocca (le credenziali le verifica sempre Windows)
- **Password** dell'account: `LogonUser` interattivo (verifica e basta, non apre nessuna sessione). EosOS non la salva né la scrive; la pagina la cancella dopo ogni tentativo.
- **Windows Hello / PIN**, dove ci sono: `UserConsentVerifier` (finestra di Windows). Il pulsante «Usa PIN o Windows Hello» compare solo se Windows dice che è disponibile. *Non provato sul PC di sviluppo: non ha Hello.*
- **Account senza password**: si controlla all'inizio del blocco con una password vuota; se Windows la accetta basta il pulsante «Entra».
- **Dopo 5 errori** si aspetta: 5 s, poi il doppio fino a 5 minuti. L'attesa la impone Rust (`lockscreen::backoff`): durante l'attesa anche la password giusta viene rifiutata, e la pagina non può aggirarla.

## Come resta bloccato
- Una finestra per monitor (`lock-0` = principale, con la pagina di accesso; `lock-1`… = nere; titolo «EosOS Blocco N»), sopra a tutto, copre anche barra e Dock. Un thread la rimette in primo piano ogni 250 ms.
- L'**hook di tastiera** (processo separato `--keyhook`, riga `lock 0|1`) scarta Win, Applicazioni, `Alt+Tab`, `Alt+Esc`, `Ctrl+Esc`, `Alt+F4`, `Alt+Spazio`, `Ctrl+Maiusc+Esc`. **Mai** `Ctrl+Alt+Maiusc+Esc`: è la via d'uscita di emergenza (chiude EosOS e ripristina la barra di Windows) e resta sempre libera.
- Launcher/Launchpad, Mission Control, Impostazioni, cattura dello schermo, i pannelli della barra e le icone del Dock non si aprono mentre è bloccato (`lockscreen::is_locked()`).
- Dopo il tempo scelto («Poi blocca anche Windows dopo»: Mai, 1, 5 di base, 15, 30 minuti) la schermata EosOS si toglie e parte il blocco di Windows: dopo l'accesso di Windows non ne trovi un'altra.

## Dove sta il codice
`src-tauri/src/lockscreen.rs` (stato, finestre, verifica, attese, scorciatoia, inattività), `mchook.rs` (`lock_blocks`, `set_locked`), `mcstore.rs` (`lockOff`, `lockIdle`, `lockWindows`), `settingsreg.rs`/`settingslive.rs` (`lock.eos`, `lock.idle`, `lock.windows`), `cmdpolicy.rs` (comandi solo per le finestre `lock-N`), `src/components/Lock.tsx` + `src/lib/lock.ts` (la pagina `?view=lock`), CSP: `img-src` ora ammette `blob:` per lo sfondo.

## Prove
- `scripts/test-lockscreen.ps1` (build `test-hooks`, circa 3 minuti, non toccare il mouse): finestre su ogni monitor e sopra a tutto, hook di tastiera informato, password giusta/sbagliata, attesa dopo 5 errori anche con la password giusta, altre finestre che non si aprono, schermata spenta → blocco di Windows, passaggio a Windows dopo 1 minuto, impostazioni non valide rifiutate, Hello finto, controllo accessi. La password è finta (`EOSOS_TEST_LOCK_PASSWORD`); il blocco vero di Windows in prova non parte mai (scrive solo la riga di prova).
- Rust: attese, soglie, nomi delle finestre, tasti scartati (compresa la via d'uscita), riga del protocollo, accessi. Vitest: orologio, testi, attese.
- Verificato sul PC di sviluppo: `LogonUser` vero rifiuta la password sbagliata e quella vuota (l'account ha una password); lo sblocco con la password VERA va provato a mano (vedi `docs/PROVE-A-MANO.md`, sezione 12).
