//! Badge sulle icone del Dock: la pallina rossa con il numero di notifiche non lette di un'app, come in macOS.
//!
//! Le notifiche arrivano dal Centro notifiche di Windows (lo stesso lettore del pannello sotto l'orologio), quindi i badge compaiono
//! SOLO se «Mostra le notifiche» è acceso e Non disturbare è spento; con una delle due cose contrarie non si legge niente. Qui non
//! si tiene nessun testo: solo quante notifiche ha ogni icona. L'abbinamento fra una notifica (nome e identificatore dell'app) e
//! un'icona del Dock (nome ed eseguibile) è un'euristica prudente: meglio nessun badge che quello sull'icona sbagliata.
//!
//! La logica è pura (testata); il comando `dock_badges` la usa con una piccola cache, perché il Dock lo chiede ogni pochi secondi.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::notifmodel::RawNotif;

/// Quante icone al massimo si accettano dalla pagina.
pub const MAX_ITEMS: usize = 64;
/// Il tetto del numero mostrato: oltre, «99+».
pub const CAP: u32 = 99;
/// Per quanto tempo si riusa l'ultima lettura.
const CACHE: Duration = Duration::from_millis(2500);

/// Un'icona del Dock come la descrive la pagina: serve solo per l'abbinamento.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub path: String,
}

/// Il badge di un'icona.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Badge {
    pub id: String,
    pub count: u32,
    /// Il testo da mostrare: il numero, o «99+».
    pub label: String,
}

/// Solo lettere e cifre, in minuscolo («Visual Studio Code» → «visualstudiocode»). Pura.
fn fold(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Il nome del file senza cartella né estensione, ripulito («C:\\App\\Discord.exe» → «discord»). Pura.
fn stem(path: &str) -> String {
    let last = path.rsplit(['\\', '/']).next().unwrap_or(path);
    let no_ext = last.rsplit_once('.').map_or(last, |(a, _)| a);
    fold(no_ext)
}

/// I pezzi dell'identificatore di un'app, in minuscolo, separati da tutto ciò che non è lettera o cifra
/// («com.squirrel.Discord.Discord» → [com, squirrel, discord, discord]). Pura.
fn tokens(app_id: &str) -> Vec<String> {
    app_id.split(|c: char| !c.is_alphanumeric()).filter(|t| !t.is_empty()).map(|t| t.to_lowercase()).collect()
}

/// Questa notifica è di questa icona? Sì se il nome dell'app nella notifica è quello dell'icona (o del suo eseguibile), oppure se
/// uno dei pezzi dell'identificatore è proprio il nome dell'eseguibile (almeno 4 lettere: «code» da solo si cerca per nome). Pura, testata.
pub fn matches(item: &Item, n: &RawNotif) -> bool {
    let (name, st) = (fold(&item.name), stem(&item.path));
    let nn = fold(&n.app_name);
    if nn.chars().count() >= 3 && (nn == name || nn == st) {
        return true;
    }
    st.chars().count() >= 4 && tokens(&n.app_id).contains(&st)
}

/// Quante notifiche ha ogni icona. Ogni notifica va a UNA sola icona (la prima che combacia), così due icone dello stesso programma
/// non la contano doppia. Le icone senza notifiche non compaiono. Nell'ordine delle icone. Pura, testata.
pub fn counts(items: &[Item], raw: &[RawNotif]) -> Vec<Badge> {
    let items = &items[..items.len().min(MAX_ITEMS)];
    let mut per: HashMap<usize, u32> = HashMap::new();
    for n in raw {
        if let Some(i) = items.iter().position(|it| matches(it, n)) {
            *per.entry(i).or_insert(0) += 1;
        }
    }
    items.iter().enumerate().filter_map(|(i, it)| per.get(&i).map(|&c| Badge { id: it.id.clone(), count: c, label: label(c) })).collect()
}

/// Il testo del badge: il numero, o «99+» oltre il tetto. Pura, testata.
pub fn label(count: u32) -> String {
    if count > CAP { format!("{CAP}+") } else { count.to_string() }
}

/// Le notifiche vanno lette? Solo se la lettura è accesa, Non disturbare è spento e i badge non sono stati spenti. Pura, testata.
pub fn should_read(read_on: bool, dnd: bool, badges_off: bool) -> bool {
    read_on && !dnd && !badges_off
}

static CACHE_SLOT: Mutex<Option<(Instant, Vec<RawNotif>)>> = Mutex::new(None);

/// I badge adesso per le icone date. Vuoto se non si può o non si deve leggere (senza errori: il Dock non mostra nulla).
pub fn badges(items: &[Item]) -> Vec<Badge> {
    let s = crate::mcstore::load();
    if !should_read(s.notif_read, s.notif_dnd, s.dock_badges_off) || items.is_empty() {
        *CACHE_SLOT.lock().unwrap_or_else(|e| e.into_inner()) = None;
        return Vec::new();
    }
    let raw = {
        let mut slot = CACHE_SLOT.lock().unwrap_or_else(|e| e.into_inner());
        match &*slot {
            Some((t, r)) if t.elapsed() < CACHE => r.clone(),
            _ => {
                let r = crate::notifread::read_all().unwrap_or_default();
                *slot = Some((Instant::now(), r.clone()));
                r
            }
        }
    };
    counts(items, &raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, name: &str, path: &str) -> Item {
        Item { id: id.into(), name: name.into(), path: path.into() }
    }
    fn notif(id: u32, app_id: &str, app_name: &str) -> RawNotif {
        RawNotif { id, app_id: app_id.into(), app_name: app_name.into(), ..Default::default() }
    }

    #[test]
    fn a_notification_matches_by_app_name_or_by_a_token_of_its_identifier() {
        let discord = item("d", "Discord", r"C:\Users\x\AppData\Local\Discord\app-1.0\Discord.exe");
        assert!(matches(&discord, &notif(1, "com.squirrel.Discord.Discord", "Discord")), "stesso nome");
        assert!(matches(&discord, &notif(2, "com.squirrel.Discord.Discord", "")), "senza nome: dall'identificatore");
        assert!(matches(&discord, &notif(3, "x", "DISCORD")), "senza badare alle maiuscole");
        let code = item("c", "Visual Studio Code", r"C:\Program Files\Microsoft VS Code\Code.exe");
        assert!(matches(&code, &notif(4, "Microsoft.VisualStudioCode", "Visual Studio Code")), "nome con spazi");
    }

    #[test]
    fn similar_but_different_apps_do_not_match() {
        let chrome = item("c", "Google Chrome", r"C:\Program Files\Google\Chrome\Application\chrome.exe");
        assert!(!matches(&chrome, &notif(1, "Microsoft.Windows.Explorer", "Esplora file")));
        assert!(!matches(&chrome, &notif(2, "chromium.thing", "")), "«chromium» non e «chrome»: il pezzo dev'essere uguale");
        // Un nome corto o un eseguibile corto non bastano da soli (troppi falsi positivi).
        let a = item("a", "A", r"C:\a.exe");
        assert!(!matches(&a, &notif(3, "a", "a")), "nome di una sola lettera");
        let ps = item("p", "Prompt", r"C:\Tools\ps.exe");
        assert!(!matches(&ps, &notif(4, "ps.thing", "")), "eseguibile di 2 lettere");
        assert!(!matches(&chrome, &notif(5, "", "")), "notifica senza dati");
    }

    #[test]
    fn counts_go_to_one_icon_each_and_only_icons_with_notifications_appear() {
        let items = vec![item("d", "Discord", r"C:\D\Discord.exe"), item("c", "Chrome", r"C:\G\chrome.exe"), item("n", "Notepad", r"C:\W\notepad.exe")];
        let raw = vec![notif(1, "x.Discord", "Discord"), notif(2, "x.Discord", "Discord"), notif(3, "Chrome", "Chrome"), notif(4, "boh", "Altro")];
        assert_eq!(counts(&items, &raw), vec![Badge { id: "d".into(), count: 2, label: "2".into() }, Badge { id: "c".into(), count: 1, label: "1".into() }]);
        assert!(counts(&items, &[]).is_empty());
        assert!(counts(&[], &raw).is_empty());
    }

    #[test]
    fn two_icons_of_the_same_program_do_not_count_a_notification_twice() {
        let items = vec![item("a", "Discord", r"C:\D\Discord.exe"), item("b", "Discord (2)", r"C:\D\Discord.exe")];
        let raw = vec![notif(1, "x.Discord", "Discord")];
        let b = counts(&items, &raw);
        assert_eq!(b.iter().map(|x| x.count).sum::<u32>(), 1);
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn the_page_cannot_make_us_scan_a_huge_list() {
        let items: Vec<Item> = (0..500).map(|i| item(&format!("i{i}"), &format!("App{i}"), &format!(r"C:\a\app{i}.exe"))).collect();
        let raw = vec![notif(1, "x", "App499")];
        assert!(counts(&items, &raw).is_empty(), "oltre il tetto di icone non si guarda");
        let raw = vec![notif(1, "x", "App7")];
        assert_eq!(counts(&items, &raw).len(), 1);
    }

    #[test]
    fn the_label_caps_at_99() {
        assert_eq!(label(1), "1");
        assert_eq!(label(99), "99");
        assert_eq!(label(100), "99+");
        assert_eq!(label(5000), "99+");
    }

    #[test]
    fn nothing_is_read_unless_reading_is_on_and_do_not_disturb_is_off() {
        assert!(should_read(true, false, false));
        assert!(!should_read(false, false, false), "lettura spenta");
        assert!(!should_read(true, true, false), "Non disturbare");
        assert!(!should_read(true, false, true), "badge spenti dall'utente");
    }

    #[test]
    fn stems_and_tokens_are_normalized() {
        assert_eq!(stem(r"C:\Program Files\Foo Bar\Foo-Bar.EXE"), "foobar");
        assert_eq!(stem("noext"), "noext");
        assert_eq!(fold("Visual Studio Code"), "visualstudiocode");
        assert_eq!(tokens("com.squirrel.Discord!App"), vec!["com", "squirrel", "discord", "app"]);
    }
}
