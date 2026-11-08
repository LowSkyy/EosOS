//! Widget «Oggi» (fase O1): i promemoria. Una lista semplice di cose da fare, con un'ora facoltativa, salvata solo sul PC
//! (dentro `mission.json`). Niente allarmi né notifiche: l'ora serve a ordinare e a segnare in rosso ciò che è passato.
//! Tutta la logica è pura e testata; i comandi stanno in `lib.rs` e il deposito in `mcstore.rs`.

use serde::{Deserialize, Serialize};

pub const MAX_REMINDERS: usize = 50;
pub const MAX_TEXT: usize = 120;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Reminder {
    pub id: u32,
    pub text: String,
    /// "HH:MM" (24 ore) oppure niente.
    pub time: Option<String>,
    pub done: bool,
}

/// Testo ripulito: spazi in eccesso e caratteri di controllo tolti, al massimo `MAX_TEXT` caratteri, mai vuoto. Pura.
pub fn clean_text(t: &str) -> Option<String> {
    let one: String = t.chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
    if one.is_empty() {
        return None;
    }
    Some(one.chars().take(MAX_TEXT).collect())
}

/// «HH:MM» valido (ore 00-23, minuti 00-59) → lo stesso testo; vuoto o assente → nessuna ora; altro → errore. Pura.
pub fn clean_time(t: Option<&str>) -> Result<Option<String>, String> {
    let Some(t) = t.map(str::trim).filter(|t| !t.is_empty()) else { return Ok(None) };
    let b = t.as_bytes();
    let ok = b.len() == 5 && b[2] == b':' && b[0].is_ascii_digit() && b[1].is_ascii_digit() && b[3].is_ascii_digit() && b[4].is_ascii_digit();
    let (h, m) = if ok { (t[0..2].parse::<u32>().unwrap_or(99), t[3..5].parse::<u32>().unwrap_or(99)) } else { (99, 99) };
    if h < 24 && m < 60 { Ok(Some(t.to_string())) } else { Err("L'ora deve essere nel formato 09:30 (da 00:00 a 23:59).".to_string()) }
}

/// Aggiunge un promemoria (id nuovo = massimo + 1). Errore leggibile se il testo è vuoto, l'ora è strana o la lista è piena.
pub fn add(list: &mut Vec<Reminder>, text: &str, time: Option<&str>) -> Result<(), String> {
    let text = clean_text(text).ok_or_else(|| "Scrivi qualcosa da ricordare.".to_string())?;
    let time = clean_time(time)?;
    if list.len() >= MAX_REMINDERS {
        return Err(format!("Ci sono già {MAX_REMINDERS} promemoria: togline qualcuno."));
    }
    let id = list.iter().map(|r| r.id).max().map_or(1, |m| m.saturating_add(1));
    list.push(Reminder { id, text, time, done: false });
    Ok(())
}

/// Segna come fatto (o ancora da fare) il promemoria `id`. `false` se non c'è.
pub fn toggle(list: &mut [Reminder], id: u32) -> bool {
    list.iter_mut().find(|r| r.id == id).map(|r| r.done = !r.done).is_some()
}

/// Toglie il promemoria `id`. `false` se non c'è.
pub fn remove(list: &mut Vec<Reminder>, id: u32) -> bool {
    let n = list.len();
    list.retain(|r| r.id != id);
    list.len() != n
}

/// Toglie tutti quelli fatti; dice quanti. Pura.
pub fn clear_done(list: &mut Vec<Reminder>) -> usize {
    let n = list.len();
    list.retain(|r| !r.done);
    n - list.len()
}

/// Come si mostrano: prima quelli da fare con un'ora (dall'ora più vicina), poi quelli da fare senza ora (nell'ordine in
/// cui sono stati scritti), in fondo quelli fatti. Pura.
pub fn sorted(list: &[Reminder]) -> Vec<Reminder> {
    let mut v = list.to_vec();
    v.sort_by(|a, b| {
        let key = |r: &Reminder| (r.done, r.time.is_none(), r.time.clone().unwrap_or_default(), r.id);
        key(a).cmp(&key(b))
    });
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(id: u32, text: &str, time: Option<&str>, done: bool) -> Reminder {
        Reminder { id, text: text.into(), time: time.map(String::from), done }
    }

    #[test]
    fn text_is_trimmed_collapsed_and_bounded() {
        assert_eq!(clean_text("  comprare   il pane \n\t"), Some("comprare il pane".into()));
        assert_eq!(clean_text("a\u{7}b"), Some("a b".into()));
        assert_eq!(clean_text("   \n "), None);
        assert_eq!(clean_text(&"x".repeat(500)).unwrap().chars().count(), MAX_TEXT);
        assert_eq!(clean_text("è già così"), Some("è già così".into()));
    }

    #[test]
    fn time_is_strict() {
        assert_eq!(clean_time(None), Ok(None));
        assert_eq!(clean_time(Some("")), Ok(None));
        assert_eq!(clean_time(Some("  ")), Ok(None));
        assert_eq!(clean_time(Some("09:30")), Ok(Some("09:30".into())));
        assert_eq!(clean_time(Some("00:00")), Ok(Some("00:00".into())));
        assert_eq!(clean_time(Some("23:59")), Ok(Some("23:59".into())));
        for bad in ["24:00", "9:30", "12:60", "1230", "ab:cd", "12:3", "12:300", "-1:00", "12.30"] {
            assert!(clean_time(Some(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn add_assigns_new_ids_and_enforces_limits() {
        let mut l = vec![];
        assert!(add(&mut l, "uno", None).is_ok() && add(&mut l, "due", Some("08:00")).is_ok());
        assert_eq!(l.iter().map(|x| x.id).collect::<Vec<_>>(), [1, 2]);
        assert!(remove(&mut l, 1));
        assert!(add(&mut l, "tre", None).is_ok());
        assert_eq!(l.last().unwrap().id, 3, "gli id non si riusano");
        assert!(add(&mut l, "  ", None).is_err());
        assert!(add(&mut l, "x", Some("99:99")).is_err());
        assert_eq!(l.len(), 2, "gli errori non aggiungono niente");
        let mut full: Vec<Reminder> = (1..=MAX_REMINDERS as u32).map(|i| r(i, "x", None, false)).collect();
        assert!(add(&mut full, "ancora", None).is_err());
        assert_eq!(full.len(), MAX_REMINDERS);
    }

    #[test]
    fn toggle_remove_and_clear() {
        let mut l = vec![r(1, "a", None, false), r(2, "b", None, true), r(3, "c", None, true)];
        assert!(toggle(&mut l, 1) && l[0].done);
        assert!(toggle(&mut l, 1) && !l[0].done);
        assert!(!toggle(&mut l, 99));
        assert!(!remove(&mut l, 99) && l.len() == 3);
        assert_eq!(clear_done(&mut l), 2);
        assert_eq!(l, vec![r(1, "a", None, false)]);
        assert_eq!(clear_done(&mut l), 0);
    }

    #[test]
    fn display_order() {
        let l = vec![r(1, "senza ora", None, false), r(2, "tardi", Some("18:00"), false), r(3, "fatto", Some("07:00"), true), r(4, "presto", Some("08:15"), false), r(5, "altro senza", None, false)];
        let ids: Vec<u32> = sorted(&l).iter().map(|x| x.id).collect();
        assert_eq!(ids, [4, 2, 1, 5, 3]);
    }
}
