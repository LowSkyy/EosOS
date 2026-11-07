//! Launcher (fase L1): corrispondenza approssimata e classifica. Solo funzioni pure, testate:
//! niente Windows, niente file, niente orologio (l'ora arriva dal chiamante).
//!
//! Punteggi per corrispondenza (più alto = meglio): nome identico 1000, inizio del nome 900,
//! inizio di una parola 800, sigla delle iniziali 750, sottostringa 600, sottosequenza
//! 100-560. A questo si somma un piccolo bonus di "frecency" (uso frequente e recente), che
//! può spostare un'app tra livelli vicini ma non scavalcare una corrispondenza molto migliore.

use std::collections::HashMap;

/// Quante volte e quando (secondi da epoca Unix) si è avviata un'app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Usage {
    pub count: u32,
    pub last: u64,
}

/// Dopo quanti giorni il peso di un uso si dimezza.
const HALF_LIFE_DAYS: f64 = 14.0;
/// Tetto del bonus di frecency, nell'unità dei punteggi di corrispondenza.
const MAX_BONUS: i32 = 120;

/// Minuscolo e senza accenti comuni (italiano e lingue vicine), per confrontare "caffè" con "caffe".
/// Pura, testata.
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

/// Iniziali delle parole di un nome, anche dentro le maiuscole interne: "Visual Studio Code"
/// -> "vsc", "PowerToys" -> "pt", "7-Zip File Manager" -> "7zfm". Pura, testata.
pub fn initials(name: &str) -> String {
    let mut out = String::new();
    let mut prev: Option<char> = None;
    for c in name.chars() {
        let starts_word = match prev {
            None => c.is_alphanumeric(),
            Some(p) => c.is_alphanumeric() && (!p.is_alphanumeric() || (p.is_lowercase() && c.is_uppercase())),
        };
        if starts_word {
            out.push_str(&fold(&c.to_string()));
        }
        prev = Some(c);
    }
    out
}

fn words(folded: &str) -> Vec<&str> {
    folded.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect()
}

/// Punteggio di una sottosequenza: tutte le lettere di `q` in ordine dentro `n`. Per evitare rumore
/// ("paint" dentro "Pannello di controllo", "calc" dentro "Cartella Scrivania") la prima lettera
/// deve cadere a inizio parola e ogni lettera seguente deve essere consecutiva alla precedente,
/// oppure cadere a inizio parola ("vscode" in "Visual Studio Code"), oppure essere l'unico salto
/// di una sola lettera della query (un refuso: "clcolatrice"). Si cerca la scelta migliore tra
/// le posizioni possibili, non solo la prima. `None` se non c'è una corrispondenza così.
fn subsequence_score(q: &str, n: &str) -> Option<i32> {
    let nc: Vec<char> = n.chars().collect();
    let qc: Vec<char> = q.chars().collect();
    fn go(qc: &[char], nc: &[char], qi: usize, prev: Option<usize>, jump_used: bool) -> Option<i32> {
        if qi == qc.len() {
            return Some(0);
        }
        let from = prev.map_or(0, |p| p + 1);
        let mut best: Option<i32> = None;
        for hit in from..nc.len() {
            if nc[hit] != qc[qi] {
                continue;
            }
            let word_start = hit == 0 || !nc[hit - 1].is_alphanumeric();
            let consecutive = prev.is_some_and(|p| p + 1 == hit);
            let gap1 = prev.is_some_and(|p| p + 2 == hit);
            let (bonus, jump) = if prev.is_none() {
                if !word_start {
                    continue;
                }
                (15, jump_used)
            } else if consecutive {
                (12, jump_used)
            } else if word_start {
                (15, jump_used)
            } else if gap1 && !jump_used {
                (-4, true)
            } else {
                continue;
            };
            if let Some(rest) = go(qc, nc, qi + 1, Some(hit), jump) {
                let total = bonus + rest;
                best = Some(best.map_or(total, |b| b.max(total)));
            }
        }
        best
    }
    go(&qc, &nc, 0, None, false).map(|b| (300 + b).clamp(100, 560))
}

/// Quanto `name` corrisponde a `query`, o `None`. Una query vuota corrisponde a tutto con 0. Con
/// una sola lettera contano solo l'inizio del nome e delle parole. Pura, testata.
pub fn match_score(query: &str, name: &str) -> Option<i32> {
    let q = fold(query.trim());
    if q.is_empty() {
        return Some(0);
    }
    let n = fold(name);
    if n == q {
        return Some(1000);
    }
    if n.starts_with(&q) {
        return Some(900 - (n.len() - q.len()).min(100) as i32);
    }
    if let Some(i) = words(&n).iter().position(|w| w.starts_with(&q)) {
        return Some(800 - (i as i32 * 15).min(100));
    }
    if q.chars().count() < 2 {
        return None;
    }
    let ini = initials(name);
    if ini.starts_with(&q) {
        return Some(750 - (ini.len() - q.len()).min(50) as i32);
    }
    if let Some(pos) = n.find(&q) {
        return Some(600 - pos.min(100) as i32);
    }
    subsequence_score(&q, &n)
}

/// Peso di un'app in base a quanto e quando è stata usata: cresce piano con il numero di usi
/// (logaritmo) e si dimezza ogni `HALF_LIFE_DAYS`. Pura, testata.
pub fn frecency(u: Usage, now: u64) -> f64 {
    if u.count == 0 {
        return 0.0;
    }
    let age_days = now.saturating_sub(u.last) as f64 / 86_400.0;
    (1.0 + f64::from(u.count)).ln() * 0.5f64.powf(age_days / HALF_LIFE_DAYS)
}

/// Bonus di frecency in punti di corrispondenza, con tetto. Pura, testata.
pub fn bonus(u: Usage, now: u64) -> i32 {
    ((frecency(u, now) * 40.0) as i32).min(MAX_BONUS)
}

/// Un risultato: posizione nell'elenco passato e punteggio finale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ranked {
    pub index: usize,
    pub score: i32,
}

/// Classifica `entries` (id, nome) per `query`. Con la query vuota restano solo le app già usate,
/// dalla più frequente e recente; altrimenti si tengono quelle che corrispondono, ordinate per
/// punteggio (corrispondenza + frecency), poi per nome più corto e infine alfabetico.
/// Al massimo `limit` risultati. Pura, testata.
pub fn rank(
    entries: &[(String, String)],
    usage: &HashMap<String, Usage>,
    query: &str,
    now: u64,
    limit: usize,
) -> Vec<Ranked> {
    let empty = query.trim().is_empty();
    let mut out: Vec<Ranked> = entries
        .iter()
        .enumerate()
        .filter_map(|(index, (id, name))| {
            let u = usage.get(id).copied().unwrap_or_default();
            if empty {
                (u.count > 0).then(|| Ranked { index, score: bonus(u, now) })
            } else {
                match_score(query, name).map(|m| Ranked { index, score: m + bonus(u, now) })
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| entries[a.index].1.chars().count().cmp(&entries[b.index].1.chars().count()))
            .then_with(|| fold(&entries[a.index].1).cmp(&fold(&entries[b.index].1)))
    });
    out.truncate(limit);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_800_000_000;
    const DAY: u64 = 86_400;

    fn entries(names: &[&str]) -> Vec<(String, String)> {
        names.iter().map(|n| (n.to_lowercase(), n.to_string())).collect()
    }
    fn names(e: &[(String, String)], r: &[Ranked]) -> Vec<String> {
        r.iter().map(|x| e[x.index].1.clone()).collect()
    }

    #[test]
    fn fold_lowercases_and_strips_common_accents() {
        assert_eq!(fold("Caffè Perché"), "caffe perche");
        assert_eq!(fold("ÀÈÌÒÙ Ünïcode"), "aeiou unicode");
        assert_eq!(fold("Niño Ça"), "nino ca");
        assert_eq!(fold(""), "");
    }

    #[test]
    fn initials_cover_words_and_inner_capitals() {
        assert_eq!(initials("Visual Studio Code"), "vsc");
        assert_eq!(initials("PowerToys"), "pt");
        assert_eq!(initials("7-Zip File Manager"), "7zfm");
        assert_eq!(initials("google chrome"), "gc");
        assert_eq!(initials("  --  "), "");
    }

    #[test]
    fn match_tiers_are_ordered() {
        let exact = match_score("calcolatrice", "Calcolatrice").unwrap();
        let prefix = match_score("calc", "Calcolatrice").unwrap();
        let word = match_score("studio", "Visual Studio Code").unwrap();
        let acronym = match_score("vsc", "Visual Studio Code").unwrap();
        let contains = match_score("latr", "Calcolatrice").unwrap();
        let subseq = match_score("visal", "Visual Studio Code").unwrap();
        assert!(exact > prefix && prefix > word && word > acronym && acronym > contains && contains > subseq);
        assert!(subseq >= 100);
    }

    #[test]
    fn accents_and_case_do_not_matter() {
        assert_eq!(match_score("CAFFE", "Caffè").unwrap(), 1000);
        assert!(match_score("perche", "Perché no").is_some());
    }

    #[test]
    fn non_matching_and_degenerate_queries() {
        assert_eq!(match_score("xyz", "Calcolatrice"), None);
        assert_eq!(match_score("calcz", "Calcolatrice"), None, "una lettera mancante non basta");
        assert_eq!(match_score("", "Qualsiasi"), Some(0));
        assert_eq!(match_score("   ", "Qualsiasi"), Some(0));
        // Una lettera sola: solo inizi, non sottostringhe o sottosequenze.
        assert!(match_score("c", "Calcolatrice").is_some());
        assert_eq!(match_score("t", "Calcolatrice"), None);
        assert_eq!(match_score("x", ""), None);
    }

    #[test]
    fn scattered_letters_are_not_a_match() {
        // Lettere sparse su parole diverse: rumore, non una corrispondenza.
        assert_eq!(match_score("paint", "Pannello di controllo"), None);
        assert_eq!(match_score("calc", "PC Health Check"), None);
        assert_eq!(match_score("calc", "Criteri di sicurezza locali"), None);
        // La prima lettera deve stare a inizio parola.
        assert_eq!(match_score("chr", "Tastiera su schermo"), None);
        // Ma una sottosequenza vicina e ben ancorata si', anche per refusi comuni.
        assert!(match_score("visal", "Visual Studio Code").is_some());
        assert!(match_score("calclatrice", "Calcolatrice").is_some(), "un refuso (manca una o) si perdona");
        assert!(match_score("calcxlatrice", "Calcolatrice").is_none(), "una lettera estranea no");
        assert!(match_score("clcolatrice", "Calcolatrice").is_some(), "una lettera saltata si perdona");
    }

    #[test]
    fn subsequence_prefers_consecutive_letters_and_word_starts() {
        let a = subsequence_score("abc", "abc xyz").unwrap();
        let b = subsequence_score("abc", "abxc").unwrap();
        assert!(a > b, "consecutive battono un salto ({a} vs {b})");
        assert_eq!(subsequence_score("abc", "axbxc"), None, "un solo salto e' perdonato");
    }

    #[test]
    fn frecency_grows_slowly_with_use_and_decays_with_age() {
        let one = frecency(Usage { count: 1, last: NOW }, NOW);
        let ten = frecency(Usage { count: 10, last: NOW }, NOW);
        let hundred = frecency(Usage { count: 100, last: NOW }, NOW);
        assert!(one < ten && ten < hundred);
        assert!(hundred < ten * 3.0, "crescita logaritmica, non lineare");
        let old = frecency(Usage { count: 10, last: NOW - 14 * DAY }, NOW);
        assert!((old - ten / 2.0).abs() < 1e-9, "dopo 14 giorni il peso si dimezza");
        assert_eq!(frecency(Usage::default(), NOW), 0.0);
        // Un'ora nel futuro (orologio) non deve dare un peso maggiore del massimo.
        assert!(frecency(Usage { count: 10, last: NOW + 3600 }, NOW) <= ten + 1e-9);
    }

    #[test]
    fn bonus_is_capped() {
        assert_eq!(bonus(Usage::default(), NOW), 0);
        assert_eq!(bonus(Usage { count: u32::MAX, last: NOW }, NOW), MAX_BONUS);
        assert!(bonus(Usage { count: 3, last: NOW }, NOW) < MAX_BONUS);
    }

    #[test]
    fn rank_orders_by_match_then_frecency_then_length() {
        let e = entries(&["Calcolatrice", "Calendario", "Calc Plus", "Google Calendar"]);
        let r = rank(&e, &HashMap::new(), "calc", NOW, 10);
        // "Calc Plus" e "Calcolatrice" iniziano con calc; Calendario no ("cale"); Google Calendar no.
        assert_eq!(names(&e, &r)[0], "Calc Plus", "a pari livello vince il nome piu' corto");
        assert!(names(&e, &r).contains(&"Calcolatrice".to_string()));
        assert!(!names(&e, &r).contains(&"Calendario".to_string()));
    }

    #[test]
    fn frecency_breaks_ties_but_cannot_beat_a_much_better_match() {
        let e = entries(&["Chrome Remote Desktop", "Google Chrome", "Chrome"]);
        let mut usage = HashMap::new();
        // "Google Chrome" molto usato, "Chrome" mai usato.
        usage.insert("google chrome".to_string(), Usage { count: 200, last: NOW });
        let r = rank(&e, &usage, "chrome", NOW, 10);
        let n = names(&e, &r);
        // "Chrome" e' identico (1000): resta primo anche se l'altro e' molto usato.
        assert_eq!(n[0], "Chrome");
        // Tra due corrispondenze di parola (indice 0 contro 1) l'uso alto fa salire "Google Chrome".
        let pos = |t: &str| n.iter().position(|x| x == t).unwrap();
        assert!(pos("Google Chrome") < pos("Chrome Remote Desktop"));
    }

    #[test]
    fn heavy_use_lifts_an_app_within_the_same_tier() {
        let e = entries(&["Notepad Aaaa", "Notepad Bbbb"]);
        let mut usage = HashMap::new();
        usage.insert("notepad bbbb".to_string(), Usage { count: 30, last: NOW });
        let r = rank(&e, &usage, "notepad", NOW, 10);
        assert_eq!(names(&e, &r)[0], "Notepad Bbbb");
        // Senza uso: stessa lunghezza, ordine alfabetico.
        let r2 = rank(&e, &HashMap::new(), "notepad", NOW, 10);
        assert_eq!(names(&e, &r2)[0], "Notepad Aaaa");
    }

    #[test]
    fn empty_query_lists_only_used_apps_by_frecency() {
        let e = entries(&["Alpha", "Beta", "Gamma", "Delta"]);
        let mut usage = HashMap::new();
        usage.insert("gamma".to_string(), Usage { count: 20, last: NOW });
        usage.insert("beta".to_string(), Usage { count: 2, last: NOW });
        usage.insert("delta".to_string(), Usage { count: 20, last: NOW - 60 * DAY }); // vecchia
        let r = rank(&e, &usage, "", NOW, 10);
        assert_eq!(names(&e, &r), ["Gamma", "Beta", "Delta"]);
        assert!(!names(&e, &r).contains(&"Alpha".to_string()), "mai usata: non compare");
        assert!(rank(&e, &HashMap::new(), "  ", NOW, 10).is_empty());
    }

    #[test]
    fn limit_and_empty_input_are_respected() {
        let e = entries(&["A1", "A2", "A3", "A4"]);
        assert_eq!(rank(&e, &HashMap::new(), "a", NOW, 2).len(), 2);
        assert!(rank(&e, &HashMap::new(), "a", NOW, 0).is_empty());
        assert!(rank(&[], &HashMap::new(), "a", NOW, 5).is_empty());
    }

    #[test]
    fn ranking_is_deterministic() {
        let e = entries(&["Zeta App", "Alpha App", "Mid App"]);
        let a = rank(&e, &HashMap::new(), "app", NOW, 10);
        let b = rank(&e, &HashMap::new(), "app", NOW, 10);
        assert_eq!(a, b);
        // A pari punteggio vince il nome piu' corto: Mid (7), Zeta (8), Alpha (9).
        assert_eq!(names(&e, &a), vec!["Mid App", "Zeta App", "Alpha App"]);
    }
}
