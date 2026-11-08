//! Aggiornamenti di EosOS. La pagina Impostazioni può chiedere «c'è una versione nuova?» e, se sì, «installala». Il
//! lavoro lo fa il plugin ufficiale di Tauri: scarica dal feed (un `latest.json` e il programma di installazione) e
//! **verifica la firma** con la chiave pubblica scritta in `tauri.conf.json`; un file non firmato con la nostra chiave
//! privata viene rifiutato, qualunque sia il server. Il feed è fisso nella configurazione: la pagina non può indicarne un
//! altro. Niente parte da solo: la ricerca all'avvio è un'impostazione spenta di default (`general.update_auto`), e
//! l'installazione parte solo dal pulsante.
//!
//! Nelle build di prova (`test-hooks`) il feed si può sostituire con `EOSOS_TEST_UPDATE_URL` (per provare con un server
//! locale); la firma si controlla comunque.

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri_plugin_updater::{Update, UpdaterExt};

/// Quanto aspettare il server per sapere se c'è una versione nuova.
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);
/// La ricerca all'avvio parte dopo questo tempo, per non pesare sull'avvio.
pub const AUTO_DELAY: Duration = Duration::from_secs(20);

/// Lo stato che vede la pagina.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Status {
    /// "idle" (mai cercato), "none" (aggiornato), "available", "installing", "error".
    pub state: &'static str,
    /// La versione in uso.
    pub current: String,
    /// La versione disponibile (se `state` è "available").
    pub version: Option<String>,
    /// Le note della versione, accorciate.
    pub notes: Option<String>,
    pub message: Option<String>,
    /// Il testo da mostrare, già in italiano (`describe`).
    pub text: String,
}

static STATUS: Mutex<Option<Status>> = Mutex::new(None);
static PENDING: Mutex<Option<Update>> = Mutex::new(None);

/// Taglia le note a una lunghezza ragionevole, senza controlli. Pura, testata.
pub fn clean_notes(n: &str) -> Option<String> {
    let t: String = n.chars().filter(|c| !c.is_control() || *c == '\n').collect();
    let t = t.trim();
    if t.is_empty() {
        return None;
    }
    Some(if t.chars().count() > 600 { format!("{}…", t.chars().take(600).collect::<String>()) } else { t.to_string() })
}

/// Il testo da mostrare per uno stato, in italiano. Pura, testata.
pub fn describe(s: &Status) -> String {
    match s.state {
        "none" => format!("EosOS {} è aggiornato.", s.current),
        "available" => format!("È disponibile la versione {} (hai la {}).", s.version.as_deref().unwrap_or("?"), s.current),
        "installing" => "Installazione in corso: EosOS si chiuderà e si aggiornerà.".to_string(),
        "error" => s.message.clone().unwrap_or_else(|| "Non riesco a cercare aggiornamenti.".to_string()),
        _ => format!("Versione {}. Non ho ancora cercato aggiornamenti.", s.current),
    }
}

fn current_version(app: &tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

fn set(mut s: Status) -> Status {
    s.text = describe(&s);
    *STATUS.lock().unwrap_or_else(|e| e.into_inner()) = Some(s.clone());
    s
}

/// Lo stato attuale (se non si è mai cercato, «idle»).
pub fn status(app: &tauri::AppHandle) -> Status {
    // Il lock si rilascia PRIMA di `set`, che lo riprende: tenerlo bloccherebbe il thread principale per sempre.
    let known = STATUS.lock().unwrap_or_else(|e| e.into_inner()).clone();
    known.unwrap_or_else(|| set(idle(current_version(app))))
}

/// Lo stato di chi non ha mai cercato. Pura, testata.
pub fn idle(current: String) -> Status {
    Status { state: "idle", current, version: None, notes: None, message: None, text: String::new() }
}

fn updater(app: &tauri::AppHandle) -> tauri_plugin_updater::Result<tauri_plugin_updater::Updater> {
    #[cfg(feature = "test-hooks")]
    if let Ok(url) = std::env::var("EOSOS_TEST_UPDATE_URL") {
        if let Ok(u) = url.parse() {
            return app.updater_builder().endpoints(vec![u])?.timeout(CHECK_TIMEOUT).build();
        }
    }
    app.updater_builder().timeout(CHECK_TIMEOUT).build()
}

/// Cerca una versione nuova e ricorda l'esito. Non scarica niente.
pub async fn check(app: &tauri::AppHandle) -> Status {
    let current = current_version(app);
    let fail = |m: String| Status { state: "error", current: current.clone(), version: None, notes: None, message: Some(m), text: String::new() };
    let result = match updater(app) {
        Ok(u) => u.check().await,
        Err(e) => return set(fail(format!("Aggiornamenti non disponibili: {e}"))),
    };
    match result {
        Ok(Some(update)) => {
            let st = Status { state: "available", current: current.clone(), version: Some(update.version.clone()), notes: update.body.as_deref().and_then(clean_notes), message: None, text: String::new() };
            *PENDING.lock().unwrap_or_else(|e| e.into_inner()) = Some(update);
            set(st)
        }
        Ok(None) => {
            *PENDING.lock().unwrap_or_else(|e| e.into_inner()) = None;
            set(Status { state: "none", current, version: None, notes: None, message: None, text: String::new() })
        }
        Err(e) => set(fail(format!("Non riesco a cercare aggiornamenti: {e}"))),
    }
}

/// Scarica e installa la versione trovata da `check`. Il plugin controlla la firma prima di installare; su Windows
/// l'installatore chiude EosOS e lo aggiorna. `Err` con il motivo se non si può.
pub async fn install(app: &tauri::AppHandle) -> Result<(), String> {
    let update = PENDING.lock().unwrap_or_else(|e| e.into_inner()).take().ok_or_else(|| "Prima cerca gli aggiornamenti.".to_string())?;
    let current = current_version(app);
    set(Status { state: "installing", current: current.clone(), version: Some(update.version.clone()), notes: None, message: None, text: String::new() });
    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => Ok(()),
        Err(e) => {
            let m = format!("Aggiornamento non riuscito: {e}");
            set(Status { state: "error", current, version: None, notes: None, message: Some(m.clone()), text: String::new() });
            Err(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(state: &'static str) -> Status {
        Status { state, current: "0.1.0".into(), version: Some("0.2.0".into()), notes: None, message: None, text: String::new() }
    }

    #[test]
    fn setting_the_status_twice_in_a_row_never_blocks() {
        // `set` prende il lock dello stato: chiamarlo due volte di fila (come fa `status` al primo uso) non deve bloccarsi.
        let a = set(idle("0.1.0".into()));
        let b = set(idle("0.1.0".into()));
        assert_eq!(a, b);
        assert!(a.text.contains("Non ho ancora cercato"));
    }

    #[test]
    fn notes_are_clean_and_short() {
        assert_eq!(clean_notes("  \n  "), None);
        assert_eq!(clean_notes("Novità\u{7}\nUna riga").as_deref(), Some("Novità\nUna riga"));
        let long = "a".repeat(2000);
        let n = clean_notes(&long).unwrap();
        assert_eq!(n.chars().count(), 601);
        assert!(n.ends_with('…'));
    }

    #[test]
    fn each_state_has_an_italian_message() {
        assert!(describe(&st("none")).contains("aggiornato"));
        assert!(describe(&st("available")).contains("0.2.0") && describe(&st("available")).contains("0.1.0"));
        assert!(describe(&st("installing")).contains("Installazione"));
        assert!(describe(&st("idle")).contains("Non ho ancora cercato"));
        let mut e = st("error");
        assert_eq!(describe(&e), "Non riesco a cercare aggiornamenti.");
        e.message = Some("Rete assente".into());
        assert_eq!(describe(&e), "Rete assente");
    }
}
