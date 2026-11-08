//! Centro notifiche, fase N3: il modello delle notifiche, tutto puro e testato (nessun Windows). Le notifiche come le
//! legge Windows (`RawNotif`) diventano gruppi per app già ripuliti e limitati, pronti per il pannello (N4): testi senza
//! caratteri di controllo né spazi in eccesso e tagliati a una lunghezza ragionevole, le notifiche vuote scartate, le più
//! recenti per prime, i gruppi ordinati dalla notifica più recente, un tetto al numero di gruppi e di notifiche per gruppo.
//! I testi delle notifiche sono privati: questo modulo non li scrive mai da nessuna parte.

use serde::Serialize;

pub const MAX_TITLE: usize = 100;
pub const MAX_BODY: usize = 240;
pub const MAX_GROUPS: usize = 30;
pub const MAX_PER_GROUP: usize = 20;
pub const MAX_APP_NAME: usize = 60;

/// Una notifica come arriva da Windows.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RawNotif {
    pub id: u32,
    /// Identificatore dell'app che l'ha mandata (AppUserModelId).
    pub app_id: String,
    /// Nome dell'app come lo mostra Windows (può mancare).
    pub app_name: String,
    /// Le righe di testo del toast, nell'ordine: la prima è il titolo.
    pub lines: Vec<String>,
    /// Quando è stata creata, in millisecondi dal 1970 (UTC).
    pub time_ms: i64,
}

/// Una notifica pronta da mostrare.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Notif {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub time_ms: i64,
    /// «adesso», «5 min fa», «ieri»…, calcolato al momento della costruzione.
    pub ago: String,
}

/// Le notifiche di un'app.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub app_id: String,
    pub app_name: String,
    pub items: Vec<Notif>,
    /// Quante notifiche in più ci sono oltre a quelle mostrate (tagliate dal tetto).
    pub hidden: usize,
}

/// Ripulisce un testo: via i caratteri di controllo (gli a capo diventano spazi), spazi multipli ridotti a uno, tagliato a
/// `max` caratteri con «…» senza spezzare un carattere. Pura, testata.
pub fn clean_text(s: &str, max: usize) -> String {
    let mapped: String = s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    let joined = mapped.split_whitespace().collect::<Vec<_>>().join(" ");
    if joined.chars().count() <= max {
        return joined;
    }
    let mut t: String = joined.chars().take(max.saturating_sub(1)).collect();
    t.push('…');
    t
}

/// Il nome da mostrare per un'app: quello di Windows se c'è, altrimenti l'ultimo pezzo dell'identificatore senza
/// estensione («…\\WindowsPowerShell\\v1.0\\powershell.exe» → «powershell»). Mai vuoto. Pura, testata.
pub fn app_label(app_name: &str, app_id: &str) -> String {
    let name = clean_text(app_name, MAX_APP_NAME);
    if !name.is_empty() {
        return name;
    }
    let last = app_id.rsplit(['\\', '/', '!']).next().unwrap_or(app_id);
    let stem = last.strip_suffix(".exe").unwrap_or(last);
    let stem = clean_text(stem, MAX_APP_NAME);
    if stem.is_empty() { "App".to_string() } else { stem }
}

/// «adesso», «3 min fa», «2 ore fa», «ieri», «5 giorni fa», o la data («8 nov») dopo una settimana. Un istante nel
/// futuro (orologi sfasati) vale «adesso». Pura, testata.
pub fn ago(now_ms: i64, then_ms: i64) -> String {
    let secs = (now_ms - then_ms).div_euclid(1000);
    if secs < 60 {
        return "adesso".into();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins} min fa");
    }
    let hours = mins / 60;
    if hours < 24 {
        return if hours == 1 { "1 ora fa".into() } else { format!("{hours} ore fa") };
    }
    let days = hours / 24;
    match days {
        1 => "ieri".into(),
        2..=6 => format!("{days} giorni fa"),
        _ => date_short(then_ms),
    }
}

/// «8 nov» da un istante (UTC: la data esatta locale la sceglie chi mostra, qui basta una data leggibile). Pura, testata.
pub fn date_short(ms: i64) -> String {
    const M: [&str; 12] = ["gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic"];
    // Giorni dal 1970 → data civile (algoritmo di Howard Hinnant).
    let z = ms.div_euclid(86_400_000) + 719_468;
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{d} {}", M[(m - 1) as usize])
}

/// Dalle notifiche di Windows ai gruppi da mostrare. Pura, testata.
pub fn build_groups(raw: &[RawNotif], now_ms: i64) -> Vec<Group> {
    use std::collections::HashMap;
    let mut by_app: HashMap<String, (String, String, Vec<Notif>)> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for r in raw {
        let mut lines = r.lines.iter().map(|l| clean_text(l, MAX_BODY)).filter(|l| !l.is_empty());
        let title = lines.next().map(|t| clean_text(&t, MAX_TITLE)).unwrap_or_default();
        let body = clean_text(&lines.collect::<Vec<_>>().join(" · "), MAX_BODY);
        if title.is_empty() && body.is_empty() {
            continue; // una notifica senza testo non si mostra
        }
        let key = r.app_id.trim().to_lowercase();
        let entry = by_app.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            (r.app_id.trim().to_string(), app_label(&r.app_name, &r.app_id), Vec::new())
        });
        // La stessa notifica non compare due volte.
        if entry.2.iter().any(|n| n.id == r.id) {
            continue;
        }
        entry.2.push(Notif { id: r.id, title, body, time_ms: r.time_ms, ago: ago(now_ms, r.time_ms) });
    }
    let mut groups: Vec<Group> = order
        .into_iter()
        .filter_map(|k| by_app.remove(&k))
        .map(|(app_id, app_name, mut items)| {
            items.sort_by(|a, b| b.time_ms.cmp(&a.time_ms).then(b.id.cmp(&a.id)));
            let hidden = items.len().saturating_sub(MAX_PER_GROUP);
            items.truncate(MAX_PER_GROUP);
            Group { app_id, app_name, items, hidden }
        })
        .collect();
    groups.sort_by(|a, b| b.items[0].time_ms.cmp(&a.items[0].time_ms).then(a.app_name.cmp(&b.app_name)));
    groups.truncate(MAX_GROUPS);
    groups
}

/// L'`id` esiste fra i gruppi mostrati? Serve a rifiutare la cancellazione di un id inventato. Pura, testata.
pub fn contains_id(groups: &[Group], id: u32) -> bool {
    groups.iter().any(|g| g.items.iter().any(|n| n.id == id))
}

/// Il numero totale di notifiche (per un'eventuale pallina sull'orologio). Pura, testata.
pub fn total(groups: &[Group]) -> usize {
    groups.iter().map(|g| g.items.len() + g.hidden).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_794_000_000_000; // un istante qualunque di fine 2026

    fn raw(id: u32, app: &str, name: &str, lines: &[&str], ago_s: i64) -> RawNotif {
        RawNotif { id, app_id: app.into(), app_name: name.into(), lines: lines.iter().map(|s| s.to_string()).collect(), time_ms: NOW - ago_s * 1000 }
    }

    #[test]
    fn text_is_cleaned_and_cut() {
        assert_eq!(clean_text("  ciao \n\t mondo\u{7} ", 50), "ciao mondo");
        assert_eq!(clean_text("", 10), "");
        assert_eq!(clean_text("\u{0}\u{1}  ", 10), "");
        let long = "è".repeat(300);
        let t = clean_text(&long, 100);
        assert_eq!(t.chars().count(), 100);
        assert!(t.ends_with('…'));
        assert_eq!(clean_text(&"x".repeat(100), 100).chars().count(), 100, "esattamente al limite resta com'e'");
    }

    #[test]
    fn the_app_label_never_is_empty() {
        assert_eq!(app_label("Messaggi", "x"), "Messaggi");
        assert_eq!(app_label("", r"{1AC1}\WindowsPowerShell\v1.0\powershell.exe"), "powershell");
        assert_eq!(app_label("  ", "Microsoft.WindowsStore_8wekyb3d8bbwe!App"), "App");
        assert_eq!(app_label("", ""), "App");
        assert!(app_label(&"n".repeat(500), "x").chars().count() <= MAX_APP_NAME);
    }

    #[test]
    fn relative_times_read_naturally() {
        let n = NOW;
        assert_eq!(ago(n, n), "adesso");
        assert_eq!(ago(n, n + 5000), "adesso", "un istante nel futuro vale adesso");
        assert_eq!(ago(n, n - 59_000), "adesso");
        assert_eq!(ago(n, n - 60_000), "1 min fa");
        assert_eq!(ago(n, n - 59 * 60_000), "59 min fa");
        assert_eq!(ago(n, n - 3_600_000), "1 ora fa");
        assert_eq!(ago(n, n - 5 * 3_600_000), "5 ore fa");
        assert_eq!(ago(n, n - 24 * 3_600_000), "ieri");
        assert_eq!(ago(n, n - 3 * 86_400_000), "3 giorni fa");
        let old = ago(n, n - 10 * 86_400_000);
        let parts: Vec<&str> = old.split(' ').collect();
        assert!(parts.len() == 2 && parts[0].parse::<u32>().is_ok() && parts[1].len() == 3, "dopo una settimana e' una data breve: {old}");
    }

    #[test]
    fn short_dates_are_right() {
        assert_eq!(date_short(0), "1 gen"); // 1970-01-01
        assert_eq!(date_short(1_794_096_000_000), "8 nov"); // 8 novembre 2026, 00:00 UTC (20765 giorni dal 1970)
        assert_eq!(date_short(951_782_400_000), "29 feb"); // 2000-02-29, bisestile
        assert_eq!(date_short(-86_400_000), "31 dic"); // prima del 1970
    }

    #[test]
    fn groups_are_by_app_newest_first_and_empty_ones_dropped() {
        let v = vec![
            raw(1, "chat", "Chat", &["Marco", "ciao!"], 600),
            raw(2, "mail", "Posta", &["Fattura", "in allegato", "grazie"], 30),
            raw(3, "chat", "Chat", &["Anna", "ci sei?"], 60),
            raw(4, "CHAT", "Chat", &["Luca"], 3000), // stessa app, maiuscole diverse
            raw(5, "chat", "Chat", &["", "  "], 10), // senza testo: scartata
        ];
        let g = build_groups(&v, NOW);
        assert_eq!(g.iter().map(|x| x.app_name.as_str()).collect::<Vec<_>>(), vec!["Posta", "Chat"], "il gruppo con la notifica piu' recente per primo");
        let chat = &g[1];
        assert_eq!(chat.items.iter().map(|n| n.id).collect::<Vec<_>>(), vec![3, 1, 4], "dalla piu' recente");
        assert_eq!(chat.items[0].title, "Anna");
        assert_eq!(chat.items[0].body, "ci sei?");
        assert_eq!(chat.items[0].ago, "1 min fa");
        assert_eq!(g[0].items[0].body, "in allegato · grazie", "le righe dopo il titolo si uniscono");
        assert_eq!(chat.items[2].body, "", "senza corpo");
        assert!(!contains_id(&g, 5));
        assert_eq!(total(&g), 4);
    }

    #[test]
    fn a_notification_with_only_a_body_or_only_a_title_is_kept() {
        let g = build_groups(&[raw(1, "a", "A", &["solo titolo"], 5), raw(2, "a", "A", &["", "solo corpo"], 6)], NOW);
        assert_eq!(g[0].items.len(), 2);
        assert_eq!(g[0].items[1].title, "solo corpo", "la prima riga non vuota fa da titolo");
        assert_eq!(g[0].items[1].body, "");
    }

    #[test]
    fn the_same_notification_does_not_appear_twice() {
        let g = build_groups(&[raw(7, "a", "A", &["x"], 5), raw(7, "a", "A", &["x"], 5)], NOW);
        assert_eq!(g[0].items.len(), 1);
    }

    #[test]
    fn limits_hold_for_many_groups_and_many_items() {
        let many: Vec<RawNotif> = (0..50).map(|i| raw(i, "solo", "Solo", &["t"], i64::from(i))).collect();
        let g = build_groups(&many, NOW);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].items.len(), MAX_PER_GROUP);
        assert_eq!(g[0].hidden, 30);
        assert_eq!(total(&g), 50, "il totale conta anche quelle nascoste");
        assert_eq!(g[0].items[0].id, 0, "le piu' recenti restano");
        let apps: Vec<RawNotif> = (0..100).map(|i| raw(i, &format!("app{i}"), &format!("App{i}"), &["t"], i64::from(i))).collect();
        assert_eq!(build_groups(&apps, NOW).len(), MAX_GROUPS);
        assert!(build_groups(&[], NOW).is_empty());
    }

    #[test]
    fn long_texts_are_cut_and_nothing_with_control_characters_survives() {
        let g = build_groups(&[raw(1, "a", "A", &[&"T".repeat(500), &format!("corpo\n{}\u{7}", "c".repeat(500))], 5)], NOW);
        assert!(g[0].items[0].title.chars().count() <= MAX_TITLE);
        assert!(g[0].items[0].body.chars().count() <= MAX_BODY);
        assert!(!g[0].items[0].body.chars().any(|c| c.is_control()));
    }

    #[test]
    fn only_listed_ids_can_be_dismissed() {
        let g = build_groups(&[raw(10, "a", "A", &["x"], 5), raw(11, "b", "B", &["y"], 6)], NOW);
        assert!(contains_id(&g, 10) && contains_id(&g, 11));
        assert!(!contains_id(&g, 12) && !contains_id(&g, 0));
    }

    #[test]
    fn groups_serialize_for_the_page() {
        let g = build_groups(&[raw(1, "a", "A", &["x", "y"], 5)], NOW);
        let j = serde_json::to_string(&g).unwrap();
        assert!(j.contains("\"app_name\":\"A\"") && j.contains("\"ago\":\"adesso\"") && j.contains("\"hidden\":0"), "{j}");
    }
}
