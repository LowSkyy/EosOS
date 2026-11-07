//! Launcher (fase L3): azioni di sistema (blocca, sospendi, riavvia, spegni), Impostazioni e
//! cartelle note. La tabella è fissa e sta qui: la pagina manda solo l'`id` di un'azione che Rust
//! le ha mostrato, e comando e argomenti li ricava Rust dalla tabella. Riavvia e Spegni chiedono
//! conferma, e la conferma è gestita QUI (non dalla pagina): il primo invio memorizza l'azione, il
//! secondo, entro 10 secondi e sulla stessa azione, la esegue.
//!
//! Nelle build di prova (feature `test-hooks`) le azioni di alimentazione non vengono mai eseguite:
//! si scrive solo una riga in `%TEMP%\eosos-actions.log`.

use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use windows::core::{w, HSTRING};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::launcherrank::match_score_strict;

/// Entro quanto la seconda pressione conferma.
const CONFIRM_WINDOW: Duration = Duration::from_secs(10);

/// Cosa fa un'azione.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Run {
    /// Un programma con argomenti fissi.
    Exec(&'static str, &'static [&'static str]),
    /// Qualcosa che la shell sa aprire (`ms-settings:`, `shell:Downloads`).
    Open(&'static str),
    /// Un'operazione interna di EosOS, per nome (`execute` la conosce).
    Internal(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub id: &'static str,
    pub name: &'static str,
    /// Altri modi di cercarla (anche in inglese).
    pub keywords: &'static [&'static str],
    /// Chiede conferma prima di eseguire.
    pub confirm: bool,
    /// Spegne, riavvia, blocca o sospende: nelle build di prova (`test-hooks`) non si esegue davvero.
    #[cfg_attr(not(any(test, feature = "test-hooks")), allow(dead_code))]
    pub power: bool,
    pub run: Run,
}

pub const ACTIONS: &[Action] = &[
    Action {
        id: "action:lock",
        name: "Blocca lo schermo",
        keywords: &["lock", "blocca", "blocco"],
        confirm: false,
        power: true,
        run: Run::Exec("rundll32.exe", &["user32.dll,LockWorkStation"]),
    },
    Action {
        id: "action:sleep",
        name: "Sospendi",
        keywords: &["sleep", "sospensione", "standby"],
        confirm: false,
        power: true,
        run: Run::Exec("rundll32.exe", &["powrprof.dll,SetSuspendState", "0,1,0"]),
    },
    Action {
        id: "action:restart",
        name: "Riavvia il computer",
        keywords: &["restart", "reboot", "riavvia"],
        confirm: true,
        power: true,
        run: Run::Exec("shutdown.exe", &["/r", "/t", "0"]),
    },
    Action {
        id: "action:shutdown",
        name: "Spegni il computer",
        keywords: &["shutdown", "power off", "arresta", "spegni"],
        confirm: true,
        power: true,
        run: Run::Exec("shutdown.exe", &["/s", "/t", "0"]),
    },
    Action {
        id: "action:clip-clear",
        name: "Svuota la cronologia degli appunti",
        keywords: &["clipboard", "appunti", "cronologia", "clear"],
        confirm: false,
        power: false,
        run: Run::Internal("clip-clear"),
    },
    Action {
        id: "action:clip-off",
        name: "Disattiva la cronologia degli appunti",
        keywords: &["clipboard", "appunti", "cronologia", "disattiva"],
        confirm: false,
        power: false,
        run: Run::Internal("clip-off"),
    },
    Action {
        id: "action:hotkey-alt-space",
        name: "Scorciatoia del Launcher: Alt+Spazio",
        keywords: &["hotkey", "scorciatoia", "shortcut", "tasti"],
        confirm: false,
        power: false,
        run: Run::Internal("hotkey:Alt+Space"),
    },
    Action {
        id: "action:hotkey-ctrl-alt-space",
        name: "Scorciatoia del Launcher: Ctrl+Alt+Spazio",
        keywords: &["hotkey", "scorciatoia", "shortcut", "tasti"],
        confirm: false,
        power: false,
        run: Run::Internal("hotkey:Ctrl+Alt+Space"),
    },
    Action {
        id: "action:hotkey-ctrl-shift-space",
        name: "Scorciatoia del Launcher: Ctrl+Maiusc+Spazio",
        keywords: &["hotkey", "scorciatoia", "shortcut", "tasti"],
        confirm: false,
        power: false,
        run: Run::Internal("hotkey:Ctrl+Shift+Space"),
    },
    Action {
        id: "action:mission-control",
        name: "Mission Control",
        keywords: &["finestre", "windows", "panoramica", "expose", "overview"],
        confirm: false,
        power: false,
        run: Run::Internal("mc-open"),
    },
    Action {
        id: "action:settings",
        name: "Impostazioni",
        keywords: &["settings", "preferenze", "opzioni"],
        confirm: false,
        power: false,
        run: Run::Open("ms-settings:"),
    },
    Action {
        id: "action:downloads",
        name: "Cartella Download",
        keywords: &["downloads", "scaricati"],
        confirm: false,
        power: false,
        run: Run::Open("shell:Downloads"),
    },
    Action {
        id: "action:documents",
        name: "Cartella Documenti",
        keywords: &["documents", "documenti"],
        confirm: false,
        power: false,
        run: Run::Open("shell:Personal"),
    },
    Action {
        id: "action:desktop",
        name: "Cartella Scrivania",
        keywords: &["desktop", "scrivania"],
        confirm: false,
        power: false,
        run: Run::Open("shell:Desktop"),
    },
];

pub fn find(id: &str) -> Option<&'static Action> {
    ACTIONS.iter().find(|a| a.id == id)
}

/// Azioni che corrispondono a `query`, con punteggio (le parole chiave valgono un po' meno del
/// nome). Una query vuota non ne dà. Pura, testata.
pub fn search(query: &str) -> Vec<(&'static Action, i32)> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    ACTIONS
        .iter()
        .filter_map(|a| {
            let by_name = match_score_strict(query, a.name);
            let by_kw = a.keywords.iter().filter_map(|k| match_score_strict(query, k)).max().map(|s| s - 40);
            by_name.into_iter().chain(by_kw).max().map(|s| (a, s))
        })
        .collect()
}

/// Esito di una richiesta di esecuzione.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Eseguita.
    Done,
    /// Serve una seconda conferma (stessa azione, entro la finestra).
    Confirm,
    /// Id sconosciuto.
    Unknown,
    /// Eseguibile non partito.
    Failed,
}

/// Azione in attesa di conferma: (id, istante in ms).
pub type Pending = Option<(String, u64)>;

/// Decide se eseguire ora o chiedere conferma. `pending` si aggiorna: una richiesta con conferma
/// la memorizza; la stessa azione entro 10 s la esegue e svuota; un'altra azione (o una conferma
/// scaduta) riparte da capo. Pura, testata.
pub fn step(pending: &mut Pending, action: &Action, now_ms: u64) -> Outcome {
    if !action.confirm {
        *pending = None;
        return Outcome::Done;
    }
    let confirmed = matches!(pending, Some((id, at))
        if id == action.id && now_ms >= *at && now_ms - *at <= CONFIRM_WINDOW.as_millis() as u64);
    if confirmed {
        *pending = None;
        Outcome::Done
    } else {
        *pending = Some((action.id.to_string(), now_ms));
        Outcome::Confirm
    }
}

/// Qualunque altra cosa che l'utente fa nel Launcher annulla una conferma in sospeso.
pub fn cancel_pending() {
    *PENDING.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

static PENDING: Mutex<Pending> = Mutex::new(None);

/// Funzione che apre Mission Control, registrata all'avvio dell'app.
static MC_OPEN: std::sync::OnceLock<fn()> = std::sync::OnceLock::new();

/// Registra la funzione che apre Mission Control (la chiama l'avvio dell'app).
pub fn set_mc_open(f: fn()) {
    let _ = MC_OPEN.set(f);
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

fn execute(action: &Action) -> bool {
    #[cfg(feature = "test-hooks")]
    if action.power {
        let line = format!("{} {}\n", now_ms(), action.id);
        let path = std::env::temp_dir().join("eosos-actions.log");
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        });
        return true;
    }
    match action.run {
        Run::Internal("mc-open") => {
            // Chi apre la panoramica la registra all'avvio dell'app (`set_mc_open`): questo modulo non
            // nomina il codice delle finestre Tauri, altrimenti gli eseguibili di test, che non hanno il
            // manifest di comctl32 v6, non partirebbero (STATUS_ENTRYPOINT_NOT_FOUND).
            MC_OPEN.get().map(|f| f()).is_some()
        }
        Run::Internal("clip-clear") => {
            crate::launcherclip::clear();
            true
        }
        Run::Internal("clip-off") => {
            crate::launcher::set_clipboard(false);
            true
        }
        Run::Internal(name) if name.starts_with("hotkey:") => {
            // Cambia subito la scorciatoia; si salva solo se la nuova combinazione è libera.
            let text = &name["hotkey:".len()..];
            match crate::launcherwin::parse_hotkey(text) {
                Some(h) if crate::launcherwin::change_hotkey(h) => {
                    let default = h == crate::launcherwin::DEFAULT_HOTKEY;
                    crate::launcherstore::set_hotkey((!default).then(|| crate::launcherwin::format_hotkey(h)));
                    true
                }
                _ => false,
            }
        }
        Run::Internal(_) => false,
        Run::Exec(prog, args) => {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            std::process::Command::new(prog).args(args).creation_flags(CREATE_NO_WINDOW).spawn().is_ok()
        }
        Run::Open(target) => {
            let t = HSTRING::from(target);
            let r = unsafe { ShellExecuteW(None, w!("open"), &t, None, None, SW_SHOWNORMAL) };
            r.0 as usize > 32
        }
    }
}

/// Esegue l'azione `id` (con la conferma, se serve). Il frontend può chiedere solo ciò che sta
/// nella tabella.
pub fn run(id: &str) -> Outcome {
    let Some(action) = find(id) else { return Outcome::Unknown };
    let mut pending = PENDING.lock().unwrap_or_else(|e| e.into_inner());
    match step(&mut pending, action, now_ms()) {
        Outcome::Done => {
            drop(pending);
            if execute(action) { Outcome::Done } else { Outcome::Failed }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn act(id: &str) -> &'static Action {
        find(id).unwrap()
    }

    #[test]
    fn table_is_consistent() {
        let mut ids: Vec<_> = ACTIONS.iter().map(|a| a.id).collect();
        assert!(ids.iter().all(|i| i.starts_with("action:")));
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ACTIONS.len(), "id unici");
        // Riavvia e spegni chiedono conferma; le altre no.
        for a in ACTIONS {
            let must = matches!(a.id, "action:restart" | "action:shutdown");
            assert_eq!(a.confirm, must, "{}", a.id);
        }
        // Tutto ciò che spegne, riavvia, blocca o sospende e' marcato `power`.
        for a in ACTIONS {
            let looks_power = matches!(a.run, Run::Exec(p, _) if p == "shutdown.exe" || p == "rundll32.exe");
            assert_eq!(a.power, looks_power, "{}", a.id);
        }
    }

    #[test]
    fn hotkey_presets_are_valid_and_distinct() {
        let mut seen = Vec::new();
        for a in ACTIONS {
            if let Run::Internal(n) = a.run
                && let Some(text) = n.strip_prefix("hotkey:")
            {
                let h = crate::launcherwin::parse_hotkey(text).unwrap_or_else(|| panic!("{text} non valida"));
                assert!(!seen.contains(&h), "preset duplicato");
                seen.push(h);
            }
        }
        assert_eq!(seen.len(), 3);
        assert!(seen.contains(&crate::launcherwin::DEFAULT_HOTKEY), "c'e' il ritorno alla predefinita");
    }

    #[test]
    fn commands_are_fixed_and_exact() {
        assert_eq!(act("action:restart").run, Run::Exec("shutdown.exe", &["/r", "/t", "0"]));
        assert_eq!(act("action:shutdown").run, Run::Exec("shutdown.exe", &["/s", "/t", "0"]));
        assert_eq!(act("action:lock").run, Run::Exec("rundll32.exe", &["user32.dll,LockWorkStation"]));
        assert_eq!(act("action:settings").run, Run::Open("ms-settings:"));
    }

    #[test]
    fn unknown_ids_do_not_resolve() {
        assert!(find("action:format-c").is_none());
        assert!(find("").is_none());
        assert!(find("shutdown.exe").is_none());
        assert_eq!(run("action:nope"), Outcome::Unknown);
        assert_eq!(run(r"C:\Windows\System32\cmd.exe"), Outcome::Unknown);
    }

    fn names(q: &str) -> Vec<&'static str> {
        let mut v = search(q);
        v.sort_by_key(|(_, s)| -s);
        v.into_iter().map(|(a, _)| a.id).collect()
    }

    #[test]
    fn search_by_name_and_keywords_in_both_languages() {
        assert_eq!(names("riavv")[0], "action:restart");
        assert_eq!(names("reboot")[0], "action:restart");
        assert_eq!(names("spegn")[0], "action:shutdown");
        assert_eq!(names("shutdown")[0], "action:shutdown");
        assert_eq!(names("blocca")[0], "action:lock");
        assert_eq!(names("sleep")[0], "action:sleep");
        assert_eq!(names("impost")[0], "action:settings");
        assert_eq!(names("settings")[0], "action:settings");
        assert!(names("download").contains(&"action:downloads"));
        assert!(names("scriv").contains(&"action:desktop"));
    }

    #[test]
    fn scattered_letters_do_not_surface_actions() {
        // "cal" non e' una parola di nessuna azione: prima compariva "Scorciatoia ... Ctrl+Alt+Spazio".
        assert!(search("cal").is_empty(), "{:?}", names("cal"));
        assert!(search("clt").is_empty());
    }

    #[test]
    fn search_ignores_empty_and_unrelated_queries() {
        assert!(search("").is_empty());
        assert!(search("   ").is_empty());
        assert!(search("zzqq").is_empty());
        assert!(search("chrome").is_empty());
    }

    #[test]
    fn plain_actions_run_immediately() {
        let mut p: Pending = None;
        assert_eq!(step(&mut p, act("action:settings"), 1000), Outcome::Done);
        assert!(p.is_none());
    }

    #[test]
    fn dangerous_actions_need_a_second_press_on_the_same_action() {
        let r = act("action:restart");
        let mut p: Pending = None;
        assert_eq!(step(&mut p, r, 1000), Outcome::Confirm);
        assert!(p.is_some());
        assert_eq!(step(&mut p, r, 4000), Outcome::Done, "seconda pressione entro 10 s");
        assert!(p.is_none(), "la conferma non resta valida");
        assert_eq!(step(&mut p, r, 4100), Outcome::Confirm, "una nuova esecuzione chiede di nuovo");
    }

    #[test]
    fn confirmation_expires_and_does_not_transfer_between_actions() {
        let (r, s) = (act("action:restart"), act("action:shutdown"));
        let mut p: Pending = None;
        assert_eq!(step(&mut p, r, 1000), Outcome::Confirm);
        assert_eq!(step(&mut p, r, 11_001), Outcome::Confirm, "scaduta: serve riconfermare");
        assert_eq!(step(&mut p, s, 11_500), Outcome::Confirm, "un'altra azione non eredita la conferma");
        assert_eq!(step(&mut p, r, 12_000), Outcome::Confirm, "tornare alla prima riparte da capo");
        assert_eq!(step(&mut p, r, 12_500), Outcome::Done);
    }

    #[test]
    fn a_plain_action_in_between_cancels_the_pending_confirmation() {
        let (r, n) = (act("action:restart"), act("action:settings"));
        let mut p: Pending = None;
        assert_eq!(step(&mut p, r, 1000), Outcome::Confirm);
        assert_eq!(step(&mut p, n, 1500), Outcome::Done);
        assert_eq!(step(&mut p, r, 2000), Outcome::Confirm, "la conferma precedente e' stata annullata");
    }

    #[test]
    fn a_clock_going_backwards_never_confirms() {
        let r = act("action:shutdown");
        let mut p: Pending = None;
        assert_eq!(step(&mut p, r, 50_000), Outcome::Confirm);
        assert_eq!(step(&mut p, r, 10_000), Outcome::Confirm);
    }
}
