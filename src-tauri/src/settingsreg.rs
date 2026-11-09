//! Impostazioni di EosOS, fase S1: l'elenco chiuso di tutte le preferenze, la loro validazione e la ricerca.
//! Tutto puro e testato: nessun file, nessuna finestra. Chi legge e scrive davvero i depositi
//! (`dockstore`, `indicatorstore`, `launcherstore`, `mcstore`) è un `Backend` (vedi `settingslive.rs`), così la
//! logica si prova con un deposito finto.
//!
//! Regola: la pagina manda solo una chiave e un valore; qui si decide se esistono e sono ammessi. Una chiave o
//! un valore sconosciuto viene rifiutato, mai passato ai depositi.

use serde::{Deserialize, Serialize};

/// Le sezioni della finestra, nell'ordine in cui compaiono.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    General,
    Appearance,
    Dock,
    MenuBar,
    Launcher,
    MissionControl,
    Notifications,
    Windows,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::General => "Generale",
            Section::Appearance => "Aspetto",
            Section::Dock => "Dock",
            Section::MenuBar => "Barra dei menu",
            Section::Launcher => "Launcher",
            Section::MissionControl => "Mission Control",
            Section::Notifications => "Notifiche e calendario",
            Section::Windows => "Impostazioni di Windows",
        }
    }
}

/// Che tipo di valore ha un'impostazione.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bool,
    /// Una scelta fra valori fissi: (id, etichetta).
    Choice(&'static [(&'static str, &'static str)]),
    /// Una scorciatoia di tastiera ("Ctrl+Alt+Up"), validata come le altre di EosOS.
    Hotkey,
    /// Un elenco di nomi di app.
    Names,
    /// Un elenco di cartelle (percorsi assoluti).
    Folders,
    /// Le app fissate nel Dock: si può togliere e riordinare, non aggiungere (si fissa dal Dock).
    Pinned,
    /// Un testo breve (al massimo tanti caratteri).
    Text(usize),
    /// Un pulsante che esegue un'azione (senza valore).
    Action,
}

/// Una voce dell'elenco chiuso.
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub key: &'static str,
    pub section: Section,
    pub label: &'static str,
    pub help: &'static str,
    pub kind: Kind,
    /// Parole in più per la ricerca.
    pub keywords: &'static [&'static str],
    /// Cambiarla ha effetto subito (altrimenti al prossimo avvio di EosOS).
    pub live: bool,
}

const WEEK_START: &[(&str, &str)] = &[("mon", "Lunedì"), ("sun", "Domenica")];
const DOCK_SIZE: &[(&str, &str)] = &[("small", "Piccolo"), ("medium", "Medio"), ("large", "Grande"), ("xl", "Molto grande")];
const DOCK_KEYS: &[(&str, &str)] = &[("ctrl_alt", "Ctrl+Alt+1…9"), ("ctrl_shift", "Ctrl+Maiusc+1…9"), ("alt_shift", "Alt+Maiusc+1…9"), ("off", "Disattivate")];
const LANGUAGES: &[(&str, &str)] = &[("it", "Italiano")];
const GLASS: &[(&str, &str)] = &[("clear", "Vetro trasparente"), ("standard", "Vetro standard"), ("solid", "Vetro opaco")];
const SNAP_KEYS: &[(&str, &str)] = &[("off", "Disattivate"), ("ctrl_alt", "Ctrl+Alt+…"), ("ctrl_shift", "Ctrl+Maiusc+…"), ("alt_shift", "Alt+Maiusc+…")];
const LAUNCHER_HOTKEYS: &[(&str, &str)] = &[("Alt+Space", "Alt+Spazio"), ("Ctrl+Alt+Space", "Ctrl+Alt+Spazio"), ("Ctrl+Shift+Space", "Ctrl+Maiusc+Spazio")];

/// L'elenco chiuso di tutte le impostazioni. Le chiavi sono stabili: la pagina e i test le usano.
pub const SPECS: &[Spec] = &[
    Spec { key: "general.autostart", section: Section::General, label: "Apri EosOS all'accesso", help: "EosOS parte da solo quando accedi a Windows. Vale solo per il tuo utente e si spegne da qui o da Gestione attività.", kind: Kind::Bool, keywords: &["avvio automatico", "accesso", "login", "avvia con windows", "esecuzione automatica"], live: true },
    Spec { key: "general.hide_taskbar", section: Section::General, label: "Nascondi la barra delle applicazioni di Windows", help: "Il Dock prende il suo posto. Vale dal prossimo avvio di EosOS. In ogni momento Ctrl+Alt+Maiusc+Esc la rimette; se EosOS si chiude, Windows la ritrova da sola.", kind: Kind::Bool, keywords: &["taskbar", "barra delle applicazioni", "start", "nascondi", "windows"], live: false },
    Spec { key: "general.update_auto", section: Section::General, label: "Cerca aggiornamenti all'avvio", help: "All'avvio EosOS chiede al sito degli aggiornamenti se c'è una versione nuova (una richiesta di rete, senza inviare dati tuoi). Non installa mai da solo: lo decidi tu dal pulsante qui sotto. Spento di base.", kind: Kind::Bool, keywords: &["aggiornamenti", "versione", "update", "novità", "rete"], live: true },
    Spec { key: "general.language", section: Section::General, label: "Lingua", help: "Per ora EosOS parla solo italiano.", kind: Kind::Choice(LANGUAGES), keywords: &["italiano", "traduzione"], live: true },
    Spec { key: "general.report", section: Section::General, label: "Rapporto sui desktop virtuali", help: "Scrive e apre un file tecnico con la versione di Windows e l'esito dei controlli sui desktop virtuali. Non contiene dati personali.", kind: Kind::Action, keywords: &["diagnostica", "rapporto", "desktop virtuali", "spaces", "problemi"], live: true },
    Spec { key: "dock.blur", section: Section::Appearance, label: "Sfocatura dello sfondo", help: "Sfoca quello che sta dietro al Dock, alla barra e ai pannelli.", kind: Kind::Bool, keywords: &["vetro", "blur", "trasparenza"], live: true },
    Spec { key: "dock.glass", section: Section::Appearance, label: "Livello di vetro", help: "Quanto sono trasparenti Dock, barra e pannelli.", kind: Kind::Choice(GLASS), keywords: &["vetro", "trasparente", "opaco", "aspetto"], live: true },
    Spec { key: "dock.size", section: Section::Dock, label: "Dimensione del Dock", help: "Quanto sono grandi il Dock e le sue icone.", kind: Kind::Choice(DOCK_SIZE), keywords: &["dimensione", "grandezza", "icone", "ridimensiona", "scala"], live: true },
    Spec { key: "dock.all_screens", section: Section::Dock, label: "Dock su tutti gli schermi", help: "Un Dock su ogni monitor, ciascuno con la sua misura dello schermo. Spento: solo sul monitor principale.", kind: Kind::Bool, keywords: &["monitor", "schermi", "più schermi", "secondo schermo", "multi"], live: true },
    Spec { key: "dock.badges", section: Section::Dock, label: "Badge delle notifiche sulle icone", help: "Una pallina rossa con il numero di notifiche non lette sull'icona dell'app, come in macOS. Funziona solo con «Mostra le notifiche» acceso (e non con Non disturbare).", kind: Kind::Bool, keywords: &["badge", "notifiche", "pallino", "contatore", "icone"], live: true },
    Spec { key: "dock.keys", section: Section::Dock, label: "Scorciatoie per le app del Dock", help: "Premendo il tasto scelto più un numero da 1 a 9 si apre la 1ª…9ª app del Dock, come un clic sulla sua icona. Con la barra di Windows nascosta i tasti Windows+numero di Windows non funzionano più: questi li sostituiscono.", kind: Kind::Choice(DOCK_KEYS), keywords: &["scorciatoie", "tastiera", "numeri", "windows+1", "app del dock", "tasti"], live: true },
    Spec { key: "dock.autohide", section: Section::Dock, label: "Nascondi automaticamente", help: "Il Dock scompare quando non serve e riappare spostando il mouse sul bordo.", kind: Kind::Bool, keywords: &["auto", "nascondi", "scompare"], live: true },
    Spec { key: "dock.reserve", section: Section::Dock, label: "Riserva spazio alle finestre", help: "Le finestre ingrandite non finiscono sotto il Dock.", kind: Kind::Bool, keywords: &["area di lavoro", "spazio", "ingrandite"], live: true },
    Spec { key: "dock.pinned", section: Section::Dock, label: "App fissate", help: "Le app fissate nel Dock: riordinale o toglile (per fissarne una, trascinala nel Dock).", kind: Kind::Pinned, keywords: &["fissate", "pin", "icone", "ordine"], live: true },
    Spec { key: "dock.stacks", section: Section::Dock, label: "Pile", help: "Cartelle che stanno nel Dock a destra: un clic mostra i file più recenti sopra l'icona. Fino a cinque. Di base c'è Download.", kind: Kind::Folders, keywords: &["pile", "cartelle", "download", "documenti", "desktop", "stack"], live: true },
    Spec { key: "bar.battery", section: Section::MenuBar, label: "Batteria", help: "Mostra l'indicatore della batteria nella barra.", kind: Kind::Bool, keywords: &["indicatori", "energia"], live: true },
    Spec { key: "bar.percent", section: Section::MenuBar, label: "Percentuale della batteria", help: "Scrive la percentuale accanto all'icona della batteria.", kind: Kind::Bool, keywords: &["indicatori", "batteria", "per cento"], live: true },
    Spec { key: "bar.network", section: Section::MenuBar, label: "Rete", help: "Mostra l'indicatore di rete nella barra.", kind: Kind::Bool, keywords: &["indicatori", "wifi", "internet"], live: true },
    Spec { key: "bar.volume", section: Section::MenuBar, label: "Volume", help: "Mostra l'indicatore del volume nella barra.", kind: Kind::Bool, keywords: &["indicatori", "audio", "suono"], live: true },
    Spec { key: "menu.enabled", section: Section::MenuBar, label: "Menu globale", help: "Mostra nella barra i menu dell'app in primo piano.", kind: Kind::Bool, keywords: &["menu dell'app", "menu bar"], live: true },
    Spec { key: "menu.show_name", section: Section::MenuBar, label: "Mostra il nome dell'app", help: "Il nome dell'app in primo piano, in grassetto, a sinistra della barra (con le voci di EosOS: impostazioni, escludi l'app).", kind: Kind::Bool, keywords: &["nome", "app", "grassetto", "barra"], live: true },
    Spec { key: "menu.show_window", section: Section::MenuBar, label: "Mostra il menu Finestra", help: "Il menu Finestra (riduci, ingrandisci, chiudi). Di base è nascosto: ripete ciò che fa la barra del titolo.", kind: Kind::Bool, keywords: &["finestra", "riduci", "ingrandisci", "window"], live: true },
    Spec { key: "menu.excluded", section: Section::MenuBar, label: "App escluse dal menu globale", help: "Di queste app la barra mostra solo il nome.", kind: Kind::Names, keywords: &["eccezioni", "escludi", "menu globale"], live: true },
    Spec { key: "launcher.hotkey", section: Section::Launcher, label: "Scorciatoia del Launcher", help: "Il tasto che apre la ricerca.", kind: Kind::Choice(LAUNCHER_HOTKEYS), keywords: &["spotlight", "ricerca", "tastiera"], live: true },
    Spec { key: "launcher.winkey", section: Section::Launcher, label: "Tasto Windows apre il Launcher", help: "Premendo il tasto Windows da solo si apre il Launcher al posto del menu Start. Le altre combinazioni (Win+E, Win+D…) restano di Windows.", kind: Kind::Bool, keywords: &["start", "windows", "tasto", "menu start", "spotlight"], live: true },
    Spec { key: "launcher.clipboard", section: Section::Launcher, label: "Cronologia degli appunti", help: "Il Launcher ricorda il testo copiato (solo in memoria).", kind: Kind::Bool, keywords: &["appunti", "copia", "incolla", "clipboard"], live: true },
    Spec { key: "launcher.folders", section: Section::Launcher, label: "Cartelle di ricerca", help: "Cartelle in più dove il Launcher cerca i file.", kind: Kind::Folders, keywords: &["file", "cartelle", "percorsi"], live: true },
    Spec { key: "mc.hotkey", section: Section::MissionControl, label: "Scorciatoia di Mission Control", help: "Il tasto che apre e chiude la panoramica.", kind: Kind::Hotkey, keywords: &["tastiera", "panoramica"], live: true },
    Spec { key: "mc.corner", section: Section::MissionControl, label: "Angolo attivo", help: "Fermare il mouse nell'angolo in alto a sinistra apre la panoramica.", kind: Kind::Bool, keywords: &["angolo", "mouse"], live: true },
    Spec { key: "mc.alttab", section: Section::MissionControl, label: "Alt+Tab di EosOS", help: "Al posto del selettore di Windows. Spento: resta quello di Windows.", kind: Kind::Bool, keywords: &["selettore", "finestre", "tastiera"], live: true },
    Spec { key: "snap.keys", section: Section::MissionControl, label: "Scorciatoie per disporre le finestre", help: "Con il tasto scelto più ←/→ la finestra in primo piano va a metà schermo; T/B alto/basso; U/I/J/K i quarti; D/F/G i terzi; Invio massimizza; C centra; Backspace torna com'era; PagGiù/PagSu la sposta sull'altro monitor. Spente di base.", kind: Kind::Choice(SNAP_KEYS), keywords: &["finestre", "snap", "metà", "affianca", "dividi", "schermo", "monitor", "tastiera", "scorciatoie"], live: true },
    Spec { key: "mc.wintab", section: Section::MissionControl, label: "Win+Tab di EosOS", help: "Apre Mission Control. Spento: resta la Visualizzazione attività di Windows.", kind: Kind::Bool, keywords: &["visualizzazione attività", "task view", "tastiera"], live: true },
    Spec { key: "notif.read", section: Section::Notifications, label: "Mostra le notifiche", help: "EosOS legge le notifiche di Windows e le mostra nel pannello sotto l'orologio. Restano solo in memoria. Spento finché non lo accendi.", kind: Kind::Bool, keywords: &["centro notifiche", "avvisi", "toast", "privacy"], live: true },
    Spec { key: "notif.dnd", section: Section::Notifications, label: "Non disturbare", help: "Il pannello sotto l'orologio non legge né mostra le notifiche finché è acceso. Riguarda solo EosOS: le notifiche e le impostazioni di Windows non cambiano.", kind: Kind::Bool, keywords: &["silenzioso", "concentrazione", "avvisi", "notifiche"], live: true },
    Spec { key: "calendar.week_start", section: Section::Notifications, label: "La settimana comincia da", help: "Nel calendario sotto l'orologio.", kind: Kind::Choice(WEEK_START), keywords: &["calendario", "lunedì", "domenica", "settimana", "orologio"], live: true },
    Spec { key: "oggi.weather", section: Section::Notifications, label: "Meteo nel pannello Oggi", help: "Mostra il meteo di oggi nel pannello sotto l'orologio. Chiede a Open-Meteo (servizio gratuito) la posizione della città e le previsioni, al massimo ogni 30 minuti: all'esterno va solo il nome della città. Spento di base.", kind: Kind::Bool, keywords: &["meteo", "tempo", "temperatura", "previsioni", "oggi", "rete"], live: true },
    Spec { key: "oggi.city", section: Section::Notifications, label: "Città del meteo", help: "Il nome della città per cui mostrare il meteo (per esempio Roma o L'Aquila).", kind: Kind::Text(60), keywords: &["citta", "meteo", "luogo"], live: true },
    Spec { key: "windows.home", section: Section::Windows, label: "Impostazioni di Windows", help: "Apre le Impostazioni di Windows. Qui sotto trovi le pagine principali, già raggruppate: un clic apre quella giusta.", kind: Kind::Action, keywords: &["windows", "pannello di controllo", "schermo", "audio", "rete", "bluetooth", "aggiornamenti", "sistema"], live: true },
];

/// L'impostazione con quella chiave, se esiste.
pub fn spec(key: &str) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.key == key)
}

/// Un valore. Dal JSON: `true`, `"standard"`, `["a", "b"]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Text(String),
    List(Vec<String>),
}

/// Perché un valore non è stato accettato. I messaggi sono pensati per essere mostrati all'utente.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct SettingsError(pub String);

impl SettingsError {
    fn new(m: impl Into<String>) -> Self {
        SettingsError(m.into())
    }
}

/// Massimo di elementi in un elenco e di caratteri per elemento.
pub const MAX_NAMES: usize = crate::menumodel::MAX_EXCLUDED;
pub const MAX_FOLDERS: usize = 20;
pub const MAX_NAME_LEN: usize = 60;
pub const MAX_PINNED: usize = 100;
pub const MAX_PINNED_NAME: usize = 200;
pub const MAX_PATH_LEN: usize = 260;

/// Un percorso di cartella accettabile: assoluto (lettera di unità o `\\\\server`), senza `..`, senza caratteri
/// vietati né di controllo, non più lungo di `MAX_PATH_LEN`. Controlla solo la forma: se esiste lo vede chi scrive.
/// Pura, testata.
pub fn valid_folder(p: &str) -> bool {
    let p = p.trim();
    if p.is_empty() || p.chars().count() > MAX_PATH_LEN || p.chars().any(|c| c.is_control() || "<>\"|?*".contains(c)) {
        return false;
    }
    let b = p.as_bytes();
    let drive = b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
    // I percorsi di rete (`\\server\...`) non si accettano: enumerarli manda le credenziali a quel server.
    drive && !p.split(['\\', '/']).any(|c| c == "..")
}

/// Ripulisce un elenco: elementi tagliati, vuoti scartati, doppi (senza badare alle maiuscole) tolti, ordine
/// mantenuto. Pura, testata.
pub fn clean_list(items: &[String], max_items: usize, max_len: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for it in items {
        let t = it.trim();
        if t.is_empty() || t.chars().count() > max_len {
            continue;
        }
        if !out.iter().any(|x| x.to_lowercase() == t.to_lowercase()) {
            out.push(t.to_string());
        }
        if out.len() >= max_items {
            break;
        }
    }
    out
}

/// Controlla `raw` per l'impostazione `s` e lo restituisce normalizzato. Pura, testata.
pub fn validate(s: &Spec, raw: &Value) -> Result<Value, SettingsError> {
    match (s.kind, raw) {
        (Kind::Bool, Value::Bool(b)) => Ok(Value::Bool(*b)),
        (Kind::Bool, _) => Err(SettingsError::new("Serve un sì o un no.")),
        (Kind::Choice(opts), Value::Text(t)) => opts
            .iter()
            .find(|(id, _)| id == t)
            .map(|(id, _)| Value::Text((*id).to_string()))
            .ok_or_else(|| SettingsError::new("Valore non ammesso per questa impostazione.")),
        (Kind::Choice(_), _) => Err(SettingsError::new("Serve una delle scelte indicate.")),
        (Kind::Hotkey, Value::Text(t)) => crate::launcherwin::parse_hotkey(t)
            .map(|h| Value::Text(crate::launcherwin::format_hotkey(h)))
            .ok_or_else(|| SettingsError::new("Scorciatoia non valida: serve almeno un modificatore (Ctrl, Alt, Win) e un tasto ammesso.")),
        (Kind::Hotkey, _) => Err(SettingsError::new("Serve una scorciatoia scritta, per esempio Ctrl+Alt+Up.")),
        (Kind::Names, Value::List(l)) => Ok(Value::List(clean_list(l, MAX_NAMES, MAX_NAME_LEN))),
        (Kind::Names, _) => Err(SettingsError::new("Serve un elenco di nomi.")),
        (Kind::Folders, Value::List(l)) => {
            if let Some(bad) = l.iter().find(|p| !p.trim().is_empty() && !valid_folder(p)) {
                return Err(SettingsError::new(format!("Cartella non valida: {}", bad.chars().take(60).collect::<String>())));
            }
            Ok(Value::List(clean_list(l, MAX_FOLDERS, MAX_PATH_LEN)))
        }
        (Kind::Folders, _) => Err(SettingsError::new("Serve un elenco di cartelle.")),
        (Kind::Pinned, Value::List(l)) => Ok(Value::List(clean_list(l, MAX_PINNED, MAX_PINNED_NAME))),
        (Kind::Pinned, _) => Err(SettingsError::new("Serve un elenco di app.")),
        (Kind::Text(max), Value::Text(t)) => {
            // Per ora l'unico testo libero è la città del meteo: vuoto = nessuna città.
            let t = t.trim();
            if t.is_empty() {
                Ok(Value::Text(String::new()))
            } else {
                crate::weather::valid_city(t).filter(|c| c.chars().count() <= max).map(Value::Text).ok_or_else(|| SettingsError::new("Scrivi il nome di una città (lettere, spazi, apostrofo o trattino)."))
            }
        }
        (Kind::Text(_), _) => Err(SettingsError::new("Serve un testo.")),
        (Kind::Action, _) => Err(SettingsError::new("Questa voce è un pulsante: non ha un valore da cambiare.")),
    }
}

/// Com'è andata una scrittura: applicata subito o dopo il riavvio.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applied {
    pub live: bool,
}

/// Dove stanno davvero i valori. Il vero (`settingslive`) tocca i depositi; i test usano uno finto.
pub trait Backend {
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&mut self, key: &str, value: &Value) -> Result<(), SettingsError>;
    /// Esegue l'azione di un pulsante (`Kind::Action`).
    fn action(&mut self, key: &str) -> Result<(), SettingsError>;
}

/// Preme il pulsante `key`: deve esistere ed essere di tipo azione. Pura nel senso che dipende solo dal
/// `Backend`; testata.
pub fn run_action(b: &mut dyn Backend, key: &str) -> Result<(), SettingsError> {
    let s = spec(key).ok_or_else(|| SettingsError::new("Impostazione sconosciuta."))?;
    if s.kind != Kind::Action {
        return Err(SettingsError::new("Questa impostazione non è un pulsante."));
    }
    b.action(s.key)
}

/// Cambia un'impostazione: la chiave deve esistere, il valore essere ammesso; poi lo scrive il deposito. Pura nel
/// senso che dipende solo dal `Backend` passato; testata.
pub fn set_value(b: &mut dyn Backend, key: &str, raw: &Value) -> Result<Applied, SettingsError> {
    let s = spec(key).ok_or_else(|| SettingsError::new("Impostazione sconosciuta."))?;
    let v = validate(s, raw)?;
    b.set(s.key, &v)?;
    Ok(Applied { live: s.live })
}

/// Una riga dello stato di tutte le impostazioni, per la pagina.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: &'static str,
    pub section: Section,
    pub section_label: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub kind: KindOut,
    pub live: bool,
    pub value: Option<Value>,
}

/// Il tipo come lo vede la pagina.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KindOut {
    Bool,
    Choice { options: Vec<(String, String)> },
    Hotkey,
    Names { max: usize },
    Folders { max: usize },
    Pinned,
    Text { max: usize },
    Action,
}

fn kind_out(k: Kind) -> KindOut {
    match k {
        Kind::Bool => KindOut::Bool,
        Kind::Choice(o) => KindOut::Choice { options: o.iter().map(|(a, b)| ((*a).to_string(), (*b).to_string())).collect() },
        Kind::Hotkey => KindOut::Hotkey,
        Kind::Names => KindOut::Names { max: MAX_NAMES },
        Kind::Folders => KindOut::Folders { max: MAX_FOLDERS },
        Kind::Pinned => KindOut::Pinned,
        Kind::Text(max) => KindOut::Text { max },
        Kind::Action => KindOut::Action,
    }
}

/// Lo stato di tutte le impostazioni, nell'ordine dell'elenco.
pub fn snapshot(b: &dyn Backend) -> Vec<Entry> {
    SPECS
        .iter()
        .map(|s| Entry {
            key: s.key,
            section: s.section,
            section_label: s.section.label(),
            label: s.label,
            help: s.help,
            kind: kind_out(s.kind),
            live: s.live,
            value: b.get(s.key),
        })
        .collect()
}

/// Come riordinare/ridurre le app fissate: `wanted` è l'elenco di nomi che si vuole (nell'ordine voluto), `current`
/// quelli di adesso. Restituisce le posizioni delle app attuali da tenere, nell'ordine voluto. Ogni nome deve
/// esistere fra quelli attuali (senza badare alle maiuscole) e ognuno si usa una volta sola (due app con lo
/// stesso nome si distinguono per ordine); non si aggiunge niente. Pura, testata.
pub fn plan_pins(current: &[String], wanted: &[String]) -> Result<Vec<usize>, SettingsError> {
    let mut used = vec![false; current.len()];
    let mut order = Vec::new();
    for w in wanted {
        let pos = current.iter().enumerate().position(|(i, c)| !used[i] && c.to_lowercase() == w.to_lowercase());
        let Some(i) = pos else {
            return Err(SettingsError::new(format!("Nel Dock non c'e' un'app fissata chiamata {}.", w.chars().take(40).collect::<String>())));
        };
        used[i] = true;
        order.push(i);
    }
    Ok(order)
}

/// Toglie accenti e maiuscole per la ricerca. Pura, testata.
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ä' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            _ => c,
        })
        .collect()
}

/// Le chiavi delle impostazioni che corrispondono a `query`: ogni parola deve comparire nell'etichetta, nella
/// sezione, nell'aiuto o nelle parole chiave (accenti e maiuscole ignorati). Prima quelle con la parola
/// nell'etichetta, poi le altre, a parità l'ordine dell'elenco. Query vuota = nessuna. Pura, testata.
pub fn search(query: &str) -> Vec<&'static str> {
    let q = fold(query);
    let words: Vec<&str> = q.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(u8, usize, &'static str)> = Vec::new();
    for (i, s) in SPECS.iter().enumerate() {
        let label = fold(s.label);
        let hay = format!("{} {} {} {}", label, fold(s.section.label()), fold(s.help), s.keywords.iter().map(|k| fold(k)).collect::<Vec<_>>().join(" "));
        if !words.iter().all(|w| hay.contains(w)) {
            continue;
        }
        let rank = if words.iter().all(|w| label.contains(w)) { 0 } else { 1 };
        hits.push((rank, i, s.key));
    }
    hits.sort();
    hits.into_iter().map(|(_, _, k)| k).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Un deposito finto in memoria.
    #[derive(Default)]
    struct Fake {
        data: HashMap<String, Value>,
        fail: bool,
    }
    impl Backend for Fake {
        fn get(&self, key: &str) -> Option<Value> {
            self.data.get(key).cloned()
        }
        fn action(&mut self, key: &str) -> Result<(), SettingsError> {
            self.data.insert(format!("ran:{key}"), Value::Bool(true));
            Ok(())
        }
        fn set(&mut self, key: &str, v: &Value) -> Result<(), SettingsError> {
            if self.fail {
                return Err(SettingsError("deposito non scrivibile".into()));
            }
            self.data.insert(key.to_string(), v.clone());
            Ok(())
        }
    }
    fn t(s: &str) -> Value {
        Value::Text(s.into())
    }
    fn l(items: &[&str]) -> Value {
        Value::List(items.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn the_list_is_well_formed() {
        let mut keys: Vec<&str> = SPECS.iter().map(|s| s.key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n, "chiavi doppie");
        for s in SPECS {
            assert!(!s.label.trim().is_empty() && !s.help.trim().is_empty(), "{}", s.key);
            assert!(s.key.contains('.') && s.key == s.key.to_lowercase(), "{}", s.key);
            if let Kind::Choice(o) = s.kind {
                assert!(!o.is_empty(), "{}", s.key);
            }
        }
        // Ogni sezione ha almeno un'impostazione, e l'ordine dell'elenco segue quello delle sezioni.
        let order = [Section::General, Section::Appearance, Section::Dock, Section::MenuBar, Section::Launcher, Section::MissionControl, Section::Notifications, Section::Windows];
        let mut last = 0;
        for s in SPECS {
            let pos = order.iter().position(|o| *o == s.section).unwrap();
            assert!(pos >= last, "{} fuori ordine", s.key);
            last = pos;
        }
        for o in order {
            assert!(SPECS.iter().any(|s| s.section == o));
        }
    }

    #[test]
    fn unknown_keys_and_wrong_types_are_refused() {
        let mut f = Fake::default();
        assert!(set_value(&mut f, "dock.nope", &Value::Bool(true)).is_err());
        assert!(set_value(&mut f, "dock.autohide", &t("si")).is_err(), "un testo non e' un booleano");
        assert!(set_value(&mut f, "dock.autohide", &l(&["a"])).is_err());
        assert!(set_value(&mut f, "dock.glass", &Value::Bool(true)).is_err());
        assert!(set_value(&mut f, "menu.excluded", &t("Chrome")).is_err(), "un elenco non e' un testo");
        assert!(f.data.is_empty(), "nulla e' stato scritto");
    }

    #[test]
    fn choices_accept_only_their_options() {
        let mut f = Fake::default();
        for ok in ["clear", "standard", "solid"] {
            assert_eq!(set_value(&mut f, "dock.glass", &t(ok)), Ok(Applied { live: true }));
            assert_eq!(f.get("dock.glass"), Some(t(ok)));
        }
        for bad in ["", "Clear", "mezzo", "standard "] {
            assert!(set_value(&mut f, "dock.glass", &t(bad)).is_err(), "{bad:?}");
        }
        assert_eq!(f.get("dock.glass"), Some(t("solid")), "un valore rifiutato non cambia lo stato");
        assert!(set_value(&mut f, "launcher.hotkey", &t("Alt+F4")).is_err(), "solo le tre scorciatoie sicure");
        assert!(set_value(&mut f, "launcher.hotkey", &t("Ctrl+Alt+Space")).is_ok());
    }

    #[test]
    fn hotkeys_are_validated_and_written_in_a_canonical_form() {
        let mut f = Fake::default();
        assert!(set_value(&mut f, "mc.hotkey", &t("ctrl+alt+up")).is_ok());
        assert_eq!(f.get("mc.hotkey"), Some(t("Ctrl+Alt+Up")), "forma canonica");
        for bad in ["Up", "Shift+A", "Ctrl+Ctrl+A", "Alt+F4", "", "Ctrl+Alt+", "Ctrl+Alt+Su+Giu"] {
            assert!(set_value(&mut f, "mc.hotkey", &t(bad)).is_err(), "{bad:?}");
        }
        assert_eq!(f.get("mc.hotkey"), Some(t("Ctrl+Alt+Up")));
        // La scorciatoia di Mission Control si cambia a caldo.
        assert_eq!(set_value(&mut f, "mc.hotkey", &t("Ctrl+Alt+K")), Ok(Applied { live: true }));
    }

    #[test]
    fn name_lists_are_cleaned() {
        let mut f = Fake::default();
        let r = set_value(&mut f, "menu.excluded", &l(&[" Chrome ", "chrome", "", "  ", "Discord", &"x".repeat(200)]));
        assert!(r.is_ok());
        assert_eq!(f.get("menu.excluded"), Some(l(&["Chrome", "Discord"])), "trim, doppi, vuoti e troppo lunghi tolti");
        let many: Vec<String> = (0..300).map(|i| format!("app{i}")).collect();
        set_value(&mut f, "menu.excluded", &Value::List(many)).unwrap();
        match f.get("menu.excluded") {
            Some(Value::List(v)) => assert_eq!(v.len(), MAX_NAMES),
            other => panic!("{other:?}"),
        }
        set_value(&mut f, "menu.excluded", &l(&[])).unwrap();
        assert_eq!(f.get("menu.excluded"), Some(l(&[])), "svuotare e' ammesso");
    }

    #[test]
    fn folders_must_be_absolute_and_safe() {
        for ok in [r"D:\Progetti", r"C:\Users\x\Documents", "E:/Dati", r"c:\a b\c"] {
            assert!(valid_folder(ok), "{ok}");
        }
        for bad in ["", "   ", "Documenti", r"..\x", r"C:\a\..\b", r"C:", r"C:\a|b", r"C:\a?b", "C:\\a\u{7}b", r"\\", r"\\server\share\cartella", "relative\\path", &format!("C:\\{}", "a".repeat(300))] {
            assert!(!valid_folder(bad), "{bad:?}");
        }
        let mut f = Fake::default();
        assert!(set_value(&mut f, "launcher.folders", &l(&[r"D:\Progetti", r"D:\progetti", r"E:\Dati"])).is_ok());
        assert_eq!(f.get("launcher.folders"), Some(l(&[r"D:\Progetti", r"E:\Dati"])), "doppi tolti senza badare alle maiuscole");
        assert!(set_value(&mut f, "launcher.folders", &l(&[r"D:\Ok", "non assoluta"])).is_err(), "una sola cartella non valida rifiuta tutto");
        assert_eq!(f.get("launcher.folders"), Some(l(&[r"D:\Progetti", r"E:\Dati"])), "e lo stato resta com'era");
        let many: Vec<String> = (0..50).map(|i| format!(r"D:\c{i}")).collect();
        set_value(&mut f, "launcher.folders", &Value::List(many)).unwrap();
        match f.get("launcher.folders") {
            Some(Value::List(v)) => assert_eq!(v.len(), MAX_FOLDERS),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn pinned_apps_can_be_reordered_and_removed_but_not_invented() {
        let cur: Vec<String> = ["Discord", "FiveM", "Chrome", "FiveM"].iter().map(|s| s.to_string()).collect();
        let w = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(plan_pins(&cur, &w(&["Chrome", "Discord"])).unwrap(), vec![2, 0], "riordinate e una tolta");
        assert_eq!(plan_pins(&cur, &w(&["fivem", "FIVEM"])).unwrap(), vec![1, 3], "due app con lo stesso nome: ognuna una volta");
        assert!(plan_pins(&cur, &w(&["FiveM", "FiveM", "FiveM"])).is_err(), "non si puo' usare la stessa app tre volte");
        assert!(plan_pins(&cur, &w(&["Inventata"])).is_err());
        assert_eq!(plan_pins(&cur, &[]).unwrap(), Vec::<usize>::new(), "svuotare e' ammesso");
        let mut f = Fake::default();
        assert!(set_value(&mut f, "dock.pinned", &l(&["Discord", "", "discord", "Chrome"])).is_ok());
        assert_eq!(f.get("dock.pinned"), Some(l(&["Discord", "Chrome"])), "ripulito come gli altri elenchi");
        assert!(set_value(&mut f, "dock.pinned", &Value::Bool(true)).is_err());
    }

    #[test]
    fn buttons_run_actions_but_have_no_value() {
        let mut f = Fake::default();
        assert!(run_action(&mut f, "general.report").is_ok());
        assert_eq!(f.get("ran:general.report"), Some(Value::Bool(true)));
        assert!(run_action(&mut f, "dock.autohide").is_err(), "un interruttore non e' un pulsante");
        assert!(run_action(&mut f, "nope").is_err());
        assert!(set_value(&mut f, "general.report", &Value::Bool(true)).is_err(), "un pulsante non si imposta");
    }

    #[test]
    fn the_language_has_only_italian_and_autostart_is_a_plain_switch() {
        let mut f = Fake::default();
        assert!(set_value(&mut f, "general.language", &t("it")).is_ok());
        assert!(set_value(&mut f, "general.language", &t("en")).is_err());
        assert!(set_value(&mut f, "general.autostart", &Value::Bool(true)).is_ok());
        assert!(set_value(&mut f, "general.autostart", &t("true")).is_err());
    }

    #[test]
    fn a_failing_backend_reports_the_error_and_changes_nothing() {
        let mut f = Fake { fail: true, ..Fake::default() };
        let r = set_value(&mut f, "dock.autohide", &Value::Bool(true));
        assert_eq!(r, Err(SettingsError("deposito non scrivibile".into())));
        assert!(f.data.is_empty());
    }

    #[test]
    fn the_snapshot_lists_every_setting_in_order() {
        let mut f = Fake::default();
        f.set("dock.autohide", &Value::Bool(true)).unwrap();
        let snap = snapshot(&f);
        assert_eq!(snap.len(), SPECS.len());
        assert_eq!(snap[0].key, "general.autostart");
        assert_eq!(snap.iter().find(|e| e.key == "general.report").unwrap().value, None, "un pulsante non ha valore");
        let auto = snap.iter().find(|e| e.key == "dock.autohide").unwrap();
        assert_eq!(auto.value, Some(Value::Bool(true)));
        assert_eq!(snap[0].value, None, "non ancora scritta nel deposito finto");
        let glass = snap.iter().find(|e| e.key == "dock.glass").unwrap();
        assert!(matches!(&glass.kind, KindOut::Choice { options } if options.len() == 3));
        // Serializza senza errori e con i nomi che la pagina si aspetta.
        let json = serde_json::to_string(&snap[0]).unwrap();
        assert!(json.contains("\"section_label\":\"Generale\"") && json.contains("\"type\":\"bool\""), "{json}");
    }

    #[test]
    fn values_parse_from_plain_json() {
        assert_eq!(serde_json::from_str::<Value>("true").unwrap(), Value::Bool(true));
        assert_eq!(serde_json::from_str::<Value>("\"solid\"").unwrap(), t("solid"));
        assert_eq!(serde_json::from_str::<Value>("[\"a\",\"b\"]").unwrap(), l(&["a", "b"]));
        for bad in ["1", "null", "{}", "[1]", "[true]"] {
            assert!(serde_json::from_str::<Value>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn search_finds_by_label_help_and_keywords_ignoring_accents() {
        assert!(search("").is_empty() && search("   ").is_empty());
        assert_eq!(search("autohide"), Vec::<&str>::new());
        let nascondi = search("nascondi");
        assert!(nascondi.contains(&"dock.autohide") && nascondi.contains(&"general.hide_taskbar"), "le due voci con «Nascondi» nell etichetta");
        assert!(search("vetro").contains(&"dock.glass"));
        // Accenti e maiuscole ignorati.
        assert_eq!(search("VISUALIZZAZIONE ATTIVITA"), vec!["mc.wintab"]);
        assert!(search("appunti").contains(&"launcher.clipboard"));
        // Piu' parole: devono esserci tutte.
        assert_eq!(search("alt tab")[0], "mc.alttab");
        assert!(search("zzzz").is_empty());
        // Chi ha la parola nell'etichetta viene prima di chi ce l'ha solo nell'aiuto.
        let r = search("batteria");
        assert_eq!(r[0], "bar.battery");
        assert!(r.contains(&"bar.percent"));
    }

    #[test]
    fn folding_removes_accents_and_case() {
        assert_eq!(fold("Visualizzazione Attività"), "visualizzazione attivita");
        assert_eq!(fold("ÈÉ"), "ee");
        assert_eq!(fold(""), "");
    }
}
