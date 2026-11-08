//! Menu globale, fase G2: il modello dei menu, tutto puro e testato (nessun Windows, nessuna finestra).
//! Un menu letto da un'app (G1) o fatto da EosOS diventa un albero di voci già ripulito e limitato, pronto per
//! la barra (G3) e la tendina (G4). L'ibrido scelto dall'utente sta in `compose`: il menu dell'app, se leggibile,
//! altrimenti solo i menu fatti da EosOS (nome dell'app e "Finestra"), così la barra non resta mai vuota.

// Il modello è pronto prima di chi lo usa (la barra e la tendina arrivano in G3 e G4): fino ad allora solo i test lo chiamano.
#![allow(dead_code)]

use serde::Serialize;

/// Massimo di voci per livello, di livelli e di caratteri di un titolo: una app con un menu enorme o
/// profondissimo non deve poter appesantire la barra.
pub const MAX_ITEMS: usize = 40;
pub const MAX_DEPTH: usize = 3;
pub const MAX_TITLE: usize = 60;
/// Massimo di menu di primo livello nella barra (oltre, la barra uscirebbe dallo schermo).
pub const MAX_TOP: usize = 12;

/// Comandi fatti da EosOS (menu dell'app e "Finestra").
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Cmd {
    Quit,
    Minimize,
    Maximize,
    Restore,
    Close,
    ShowInDock,
}

/// Cosa succede scegliendo una voce.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Action {
    /// Un comando di EosOS.
    Eos(Cmd),
    /// La voce dell'app: posizione nel suo menu (percorso di indici), da invocare in G5.
    App(Vec<usize>),
}

/// Una voce di menu.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub title: String,
    /// Scorciatoia da mostrare a destra (solo testo: la gestisce l'app).
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub checked: bool,
    pub separator: bool,
    pub action: Option<Action>,
    pub children: Vec<Item>,
}

impl Item {
    pub fn separator() -> Item {
        Item { title: String::new(), shortcut: None, enabled: false, checked: false, separator: true, action: None, children: Vec::new() }
    }
    pub fn leaf(title: &str, action: Action) -> Item {
        Item { title: title.into(), shortcut: None, enabled: true, checked: false, separator: false, action: Some(action), children: Vec::new() }
    }
    pub fn is_submenu(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Un menu di primo livello della barra.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TopMenu {
    pub title: String,
    pub items: Vec<Item>,
    /// Il menu col nome dell'app (in grassetto nella barra).
    pub bold: bool,
    /// Le voci non sono ancora state lette (UI Automation le espone solo ad app aperta): si leggono quando
    /// l'utente apre il menu (G4). Un menu "pigro" resta nella barra anche con l'elenco vuoto.
    pub lazy: bool,
}

/// Divide un titolo grezzo ("&Apri...\tCtrl+O") in etichetta e scorciatoia, tolti gli acceleratori `&` (`&&`
/// diventa `&`) e gli spazi. Pura, testata.
pub fn split_label(raw: &str) -> (String, Option<String>) {
    let (label, short) = match raw.split_once('\t') {
        Some((l, s)) => (l, Some(s.trim().to_string()).filter(|s| !s.is_empty())),
        None => (raw, None),
    };
    let mut out = String::with_capacity(label.len());
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            // `&&` e' una e commerciale vera; un `&` solo e' l'acceleratore: si toglie il segno, resta la lettera.
            if chars.peek() == Some(&'&') {
                out.push('&');
                chars.next();
            }
        } else if !c.is_control() {
            out.push(c);
        }
    }
    (limit_title(out.trim()), short.map(|s| limit_title(&s)))
}

/// Accorcia un titolo a `MAX_TITLE` caratteri (con "…"), senza spezzare un carattere. Pura, testata.
pub fn limit_title(s: &str) -> String {
    if s.chars().count() <= MAX_TITLE {
        return s.to_string();
    }
    let mut t: String = s.chars().take(MAX_TITLE - 1).collect();
    t.push('…');
    t
}

/// Ripulisce un livello di voci: tolte quelle senza titolo (che non siano separatori), tolti i separatori
/// iniziali, finali e ripetuti, tagliato a `MAX_ITEMS`, e sotto `MAX_DEPTH` livelli i sottomenu diventano
/// voci semplici. I percorsi delle azioni non cambiano (indicano la posizione nel menu originale). Pura, testata.
pub fn normalize(items: Vec<Item>, depth: usize) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::new();
    for mut it in items {
        if !it.separator && it.title.trim().is_empty() {
            continue;
        }
        if it.separator {
            if out.last().is_none_or(|l| l.separator) {
                continue; // iniziale o ripetuto
            }
        } else if depth + 1 >= MAX_DEPTH {
            it.children.clear();
        } else {
            it.children = normalize(std::mem::take(&mut it.children), depth + 1);
        }
        out.push(it);
        if out.len() >= MAX_ITEMS {
            break;
        }
    }
    while out.last().is_some_and(|l| l.separator) {
        out.pop();
    }
    out
}

/// Una voce come l'ha letta il lettore (G1/G3), prima di ogni pulizia.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawItem {
    pub text: String,
    pub separator: bool,
    pub grayed: bool,
    pub checked: bool,
    pub children: Vec<RawItem>,
}

/// Un menu di primo livello come l'ha letto il lettore.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawMenu {
    pub text: String,
    pub items: Vec<RawItem>,
    /// Le voci non sono state lette (si leggono all'apertura).
    pub lazy: bool,
}

fn raw_item(r: &RawItem, path: Vec<usize>) -> Item {
    let (title, shortcut) = split_label(&r.text);
    if r.separator {
        return Item::separator();
    }
    let children: Vec<Item> =
        r.children.iter().enumerate().map(|(i, c)| raw_item(c, [path.clone(), vec![i]].concat())).collect();
    // Un sottomenu si apre, non si esegue: l'azione ce l'hanno solo le foglie.
    let action = if children.is_empty() { Some(Action::App(path)) } else { None };
    Item { title, shortcut, enabled: !r.grayed, checked: r.checked, separator: false, action, children }
}

/// Dai menu letti ai menu del modello: titoli ripuliti e percorsi che indicano la POSIZIONE ORIGINALE di ogni
/// voce (primo indice = il menu di primo livello), prima di qualunque taglio, così G5 trova sempre quella giusta.
/// Pura, testata.
pub fn raw_to_top(raw: &[RawMenu]) -> Vec<TopMenu> {
    raw.iter()
        .enumerate()
        .map(|(i, m)| TopMenu {
            title: split_label(&m.text).0,
            bold: false,
            lazy: m.lazy,
            items: m.items.iter().enumerate().map(|(j, it)| raw_item(it, vec![i, j])).collect(),
        })
        .collect()
}

/// Lo stato della finestra che serve ai menu di EosOS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinState {
    pub maximized: bool,
    pub minimized: bool,
}

/// Il menu col nome dell'app: solo "Nascondi nel Dock" e "Esci da ...". Pura, testata.
pub fn app_menu(app_name: &str) -> TopMenu {
    let name = limit_title(app_name.trim());
    let name = if name.is_empty() { "App".to_string() } else { name };
    TopMenu {
        title: name.clone(),
        bold: true,
        lazy: false,
        items: vec![
            Item::leaf("Mostra nel Dock", Action::Eos(Cmd::ShowInDock)),
            Item::separator(),
            Item::leaf(&format!("Esci da {name}"), Action::Eos(Cmd::Quit)),
        ],
    }
}

/// La riga della barra quando non c'è un'app (il desktop): solo il nome, in grassetto, senza voci. Pura, testata.
pub fn name_only(name: &str) -> Vec<TopMenu> {
    vec![TopMenu { title: limit_title(name.trim()), items: Vec::new(), bold: true, lazy: false }]
}

/// Il menu "Finestra" fatto da EosOS. Pura, testata.
pub fn window_menu(w: WinState) -> TopMenu {
    let size = if w.maximized {
        Item::leaf("Ripristina", Action::Eos(Cmd::Restore))
    } else {
        Item::leaf("Ingrandisci", Action::Eos(Cmd::Maximize))
    };
    let mut minimize = Item::leaf("Riduci a icona", Action::Eos(Cmd::Minimize));
    minimize.enabled = !w.minimized;
    TopMenu {
        title: "Finestra".into(),
        bold: false,
        lazy: false,
        items: vec![minimize, size, Item::separator(), Item::leaf("Chiudi", Action::Eos(Cmd::Close))],
    }
}

/// È già un menu "Finestra" (in italiano o in inglese)? Pura, testata.
pub fn is_window_menu(title: &str) -> bool {
    matches!(title.trim().to_lowercase().as_str(), "finestra" | "window" | "windows" | "fenêtre" | "fenster")
}

/// La fila di menu per la barra (l'ibrido): menu col nome dell'app, poi i menu dell'app se li abbiamo letti
/// (puliti e limitati), poi "Finestra" fatto da EosOS se l'app non ne ha già uno. Senza menu letti restano solo
/// il nome dell'app e "Finestra". Pura, testata.
pub fn compose(app_name: &str, read: Option<Vec<TopMenu>>, w: WinState) -> Vec<TopMenu> {
    let mut row = vec![app_menu(app_name)];
    let mut has_window = false;
    for m in read.unwrap_or_default() {
        let title = limit_title(m.title.trim());
        let items = normalize(m.items, 0);
        if title.is_empty() || (items.is_empty() && !m.lazy) {
            continue; // un menu vuoto non si mostra
        }
        has_window |= is_window_menu(&title);
        row.push(TopMenu { title, items, bold: false, lazy: m.lazy });
        if row.len() >= MAX_TOP {
            break;
        }
    }
    if !has_window && row.len() < MAX_TOP {
        row.push(window_menu(w));
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    fn it(title: &str) -> Item {
        Item::leaf(title, Action::App(vec![0]))
    }
    fn w() -> WinState {
        WinState { maximized: false, minimized: false }
    }

    fn rawi(text: &str) -> RawItem {
        RawItem { text: text.into(), ..RawItem::default() }
    }

    #[test]
    fn raw_menus_become_model_menus_with_original_paths() {
        let mut sub = rawi("Recenti");
        sub.children = vec![rawi("uno"), RawItem { separator: true, ..RawItem::default() }, rawi("due")];
        let mut gray = rawi("&Salva	Ctrl+S");
        gray.grayed = true;
        gray.checked = true;
        let raw = vec![
            RawMenu { text: "&File".into(), items: vec![rawi("&Nuovo"), gray, sub], lazy: false },
            RawMenu { text: "Aiuto".into(), items: vec![], lazy: true },
        ];
        let top = raw_to_top(&raw);
        assert_eq!(top[0].title, "File");
        assert_eq!(top[0].items[0].action, Some(Action::App(vec![0, 0])));
        let s = &top[0].items[1];
        assert!(!s.enabled && s.checked && s.shortcut.as_deref() == Some("Ctrl+S"));
        assert_eq!(s.action, Some(Action::App(vec![0, 1])));
        let r = &top[0].items[2];
        assert!(r.is_submenu() && r.action.is_none(), "un sottomenu non ha azione");
        assert_eq!(r.children[2].action, Some(Action::App(vec![0, 2, 2])), "il percorso conta anche i separatori");
        assert!(top[0].items[2].children[1].separator);
        assert!(top[1].lazy && top[1].items.is_empty());
    }

    #[test]
    fn paths_keep_pointing_at_the_original_position_after_cleaning() {
        let raw = vec![RawMenu { text: "File".into(), items: vec![rawi(""), rawi("Apri")], lazy: false }];
        let row = compose("App", Some(raw_to_top(&raw)), WinState { maximized: false, minimized: false });
        // La voce senza titolo è sparita, ma "Apri" indica ancora la posizione 1 del menu originale.
        assert_eq!(row[1].items[0].title, "Apri");
        assert_eq!(row[1].items[0].action, Some(Action::App(vec![0, 1])));
    }

    #[test]
    fn lazy_menus_stay_in_the_bar_even_when_empty() {
        let read = vec![TopMenu { title: "File".into(), items: vec![], bold: false, lazy: true }];
        let row = compose("App", Some(read), WinState { maximized: false, minimized: false });
        assert_eq!(row.iter().map(|m| m.title.as_str()).collect::<Vec<_>>(), vec!["App", "File", "Finestra"]);
        assert!(row[1].lazy);
    }

    #[test]
    fn labels_lose_accelerators_and_split_the_shortcut() {
        assert_eq!(split_label("&File"), ("File".into(), None));
        assert_eq!(split_label("&Apri...\tCtrl+O"), ("Apri...".into(), Some("Ctrl+O".into())));
        assert_eq!(split_label("Salva &con nome\tCtrl+Maiusc+S"), ("Salva con nome".into(), Some("Ctrl+Maiusc+S".into())));
        assert_eq!(split_label("Tom && Jerry"), ("Tom & Jerry".into(), None), "&& e' una e commerciale vera");
        assert_eq!(split_label("  Modifica \t "), ("Modifica".into(), None), "scorciatoia vuota = nessuna");
        assert_eq!(split_label("a\u{7}b"), ("ab".into(), None), "i caratteri di controllo si tolgono");
        assert_eq!(split_label("Finale&"), ("Finale".into(), None));
    }

    #[test]
    fn long_titles_are_cut_without_breaking_characters() {
        let long = "è".repeat(200);
        let t = limit_title(&long);
        assert_eq!(t.chars().count(), MAX_TITLE);
        assert!(t.ends_with('…'));
        assert_eq!(limit_title("corto"), "corto");
        assert_eq!(limit_title(&"x".repeat(MAX_TITLE)).chars().count(), MAX_TITLE, "esattamente al limite resta com'e'");
    }

    #[test]
    fn separators_are_cleaned_and_empty_items_dropped() {
        let v = vec![Item::separator(), it("A"), Item::separator(), Item::separator(), it(""), it("   "), it("B"), Item::separator()];
        let out = normalize(v, 0);
        let shape: Vec<&str> = out.iter().map(|i| if i.separator { "-" } else { i.title.as_str() }).collect();
        assert_eq!(shape, vec!["A", "-", "B"]);
        assert!(normalize(vec![Item::separator()], 0).is_empty());
    }

    #[test]
    fn levels_and_counts_are_limited() {
        let many: Vec<Item> = (0..100).map(|i| it(&format!("v{i}"))).collect();
        assert_eq!(normalize(many, 0).len(), MAX_ITEMS);
        // Profondità: un sottomenu oltre il limite diventa una voce semplice.
        let mut deep = it("l3");
        deep.children = vec![it("l4")];
        let mut l2 = it("l2");
        l2.children = vec![deep];
        let mut l1 = it("l1");
        l1.children = vec![l2];
        let out = normalize(vec![l1], 0);
        assert_eq!(out[0].children[0].children[0].title, "l3");
        assert!(out[0].children[0].children[0].children.is_empty(), "il quarto livello non c'e'");
    }

    #[test]
    fn action_paths_and_flags_survive_normalization() {
        let mut a = Item::leaf("Apri", Action::App(vec![0, 2]));
        a.checked = true;
        a.enabled = false;
        a.shortcut = Some("Ctrl+O".into());
        let out = normalize(vec![a.clone()], 0);
        assert_eq!(out, vec![a]);
    }

    #[test]
    fn app_menu_names_the_app_and_never_is_blank() {
        let m = app_menu("Blocco note");
        assert!(m.bold && m.title == "Blocco note");
        assert_eq!(m.items.last().unwrap().title, "Esci da Blocco note");
        assert_eq!(m.items.last().unwrap().action, Some(Action::Eos(Cmd::Quit)));
        assert_eq!(app_menu("   ").title, "App");
        assert!(app_menu(&"x".repeat(500)).title.chars().count() <= MAX_TITLE);
    }

    #[test]
    fn the_desktop_row_is_only_a_name() {
        let r = name_only(" Desktop ");
        assert_eq!(r.len(), 1);
        assert!(r[0].bold && r[0].items.is_empty() && r[0].title == "Desktop");
    }

    #[test]
    fn window_menu_follows_the_window_state() {
        let m = window_menu(w());
        assert_eq!(m.items[1].title, "Ingrandisci");
        assert!(m.items[0].enabled);
        let max = window_menu(WinState { maximized: true, minimized: false });
        assert_eq!(max.items[1].title, "Ripristina");
        let mini = window_menu(WinState { maximized: false, minimized: true });
        assert!(!mini.items[0].enabled, "una finestra gia' ridotta non si riduce di nuovo");
    }

    #[test]
    fn without_a_readable_menu_the_bar_has_only_eos_menus() {
        let row = compose("Chrome", None, w());
        let titles: Vec<&str> = row.iter().map(|m| m.title.as_str()).collect();
        assert_eq!(titles, vec!["Chrome", "Finestra"]);
        assert_eq!(compose("Chrome", Some(Vec::new()), w()), row, "un elenco vuoto vale 'non leggibile'");
    }

    #[test]
    fn with_a_readable_menu_the_app_menus_come_between_name_and_window() {
        let read = vec![
            TopMenu { title: "File".into(), items: vec![it("Nuovo"), it("Apri")], bold: false, lazy: false },
            TopMenu { title: "Vuoto".into(), items: vec![Item::separator()], bold: false, lazy: false },
            TopMenu { title: "".into(), items: vec![it("x")], bold: false, lazy: false },
            TopMenu { title: "Modifica".into(), items: vec![it("Copia")], bold: true, lazy: false },
        ];
        let row = compose("Blocco note", Some(read), w());
        let titles: Vec<&str> = row.iter().map(|m| m.title.as_str()).collect();
        assert_eq!(titles, vec!["Blocco note", "File", "Modifica", "Finestra"]);
        assert!(row[0].bold && !row[2].bold, "solo il nome dell'app e' in grassetto");
    }

    #[test]
    fn an_app_that_has_its_own_window_menu_does_not_get_a_second_one() {
        for name in ["Finestra", "Window", " window "] {
            let read = vec![TopMenu { title: name.into(), items: vec![it("Riduci")], bold: false, lazy: false }];
            let row = compose("App", Some(read), w());
            assert_eq!(row.len(), 2, "{name}");
        }
        assert!(is_window_menu("Fenster") && !is_window_menu("Finestre di dialogo"));
    }

    #[test]
    fn the_bar_never_gets_too_many_menus() {
        let read: Vec<TopMenu> = (0..40).map(|i| TopMenu { title: format!("M{i}"), items: vec![it("x")], bold: false, lazy: false }).collect();
        let row = compose("App", Some(read), w());
        assert!(row.len() <= MAX_TOP);
        assert_eq!(row[0].title, "App");
    }
}
