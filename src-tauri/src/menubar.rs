//! Menu globale, fase G3: la riga di menu della barra per l'app in primo piano. Quando cambia la finestra in
//! primo piano (l'hook di `foreground.rs`) si emette SUBITO una riga di ripiego (nome dell'app + "Finestra"), poi
//! un thread legge il menu dell'app (con tempo massimo, in sola lettura) e, se la finestra è ancora quella,
//! emette la riga completa. Le letture si tengono in una cache a tempo; una nuova richiesta scavalca le vecchie.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{IsIconic, IsZoomed};

use crate::menumodel::{compose, name_only, raw_to_top, TopMenu, WinState};

pub const EVENT_NAME: &str = "menubar-row";

/// La riga di menu per la barra.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Row {
    /// La finestra a cui si riferisce (per riconoscere le righe vecchie).
    pub hwnd: u64,
    pub name: String,
    pub menus: Vec<TopMenu>,
    /// `false` = riga di ripiego in attesa della lettura.
    pub read: bool,
}

struct Req {
    hwnd: usize,
    name: String,
}


static APP: OnceLock<AppHandle> = OnceLock::new();
static TX: OnceLock<Mutex<Sender<Req>>> = OnceLock::new();
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static LAST: Mutex<Option<Row>> = Mutex::new(None);
type Cache = HashMap<usize, (Instant, Option<Vec<TopMenu>>)>;
static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

/// Quanto vale una lettura in cache: un menu trovato 5 s, un "nessun menu" 3 s (le app lente a esporlo si
/// riprovano prima). Pura, testata.
pub fn cache_fresh(age: Duration, found: bool) -> bool {
    age < Duration::from_secs(if found { 5 } else { 3 })
}

/// La riga per un'app: sul desktop solo il nome, altrimenti l'ibrido. Pura, testata.
pub fn build_row(name: &str, read: Option<Vec<TopMenu>>, state: WinState, desktop: bool) -> Vec<TopMenu> {
    if desktop {
        name_only(name)
    } else {
        compose(name, read, state)
    }
}

fn win_state(hwnd: usize) -> WinState {
    let h = HWND(hwnd as *mut _);
    WinState { maximized: unsafe { IsZoomed(h) }.as_bool(), minimized: unsafe { IsIconic(h) }.as_bool() }
}

fn publish(row: Row) {
    if CURRENT.load(Ordering::Relaxed) != row.hwnd as usize {
        return; // nel frattempo è cambiata la finestra
    }
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(row.clone());
    if let Some(app) = APP.get() {
        let _ = app.emit(EVENT_NAME, row);
    }
}

/// L'ultima riga emessa (per la pagina che parte dopo l'evento).
pub fn current() -> Option<Row> {
    LAST.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Avvia il thread che legge i menu. Si chiama una volta all'avvio.
pub fn start(app: AppHandle) {
    let _ = APP.set(app);
    let (tx, rx) = channel::<Req>();
    let _ = TX.set(Mutex::new(tx));
    std::thread::spawn(move || worker(rx));
}

/// La finestra in primo piano è cambiata (o è cambiata l'app). Qui si fa SOLO l'annotazione e l'invio al
/// thread dei menu: la chiamata arriva dall'hook di primo piano e non deve mai aspettare niente (emettere
/// eventi o leggere finestre altrui da lì può bloccare l'hook e con lui il nome dell'app).
pub fn on_window(hwnd: HWND, name: &str) {
    let id = hwnd.0 as usize;
    if id == 0 {
        return;
    }
    CURRENT.store(id, Ordering::Relaxed);
    if let Some(tx) = TX.get() {
        let _ = tx.lock().unwrap_or_else(|e| e.into_inner()).send(Req { hwnd: id, name: name.into() });
    }
}

fn worker(rx: Receiver<Req>) {
    while let Ok(mut req) = rx.recv() {
        // Vince l'ultima richiesta: se l'utente cambia finestra in fretta non si leggono quelle di mezzo.
        while let Ok(newer) = rx.try_recv() {
            req = newer;
        }
        if CURRENT.load(Ordering::Relaxed) != req.hwnd {
            continue;
        }
        let desktop = req.name == "Desktop";
        let state = win_state(req.hwnd);
        // Dalla cache, se fresca: nessun lampo di ripiego.
        let cached = {
            let g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
            g.as_ref().and_then(|c| c.get(&req.hwnd)).filter(|(at, v)| cache_fresh(at.elapsed(), v.is_some())).map(|(_, v)| v.clone())
        };
        if let Some(v) = cached {
            let menus = build_row(&req.name, v, state, desktop);
            publish(Row { hwnd: req.hwnd as u64, name: req.name, menus, read: true });
            continue;
        }
        // Subito la riga di ripiego, poi la lettura vera.
        publish(Row { hwnd: req.hwnd as u64, name: req.name.clone(), menus: build_row(&req.name, None, state, desktop), read: false });
        if desktop {
            continue;
        }
        let top: Option<Vec<TopMenu>> = crate::menuread::read_menus(req.hwnd).map(|raw| raw_to_top(&raw));
        {
            let mut g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
            let c = g.get_or_insert_with(HashMap::new);
            c.retain(|_, (at, v)| cache_fresh(at.elapsed(), v.is_some()));
            c.insert(req.hwnd, (Instant::now(), top.clone()));
        }
        let menus = build_row(&req.name, top, win_state(req.hwnd), false);
        publish(Row { hwnd: req.hwnd as u64, name: req.name, menus, read: true });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menumodel::{Action, Item};

    fn state() -> WinState {
        WinState { maximized: false, minimized: false }
    }

    #[test]
    fn the_cache_forgets_quickly_when_nothing_was_found() {
        assert!(cache_fresh(Duration::from_secs(4), true));
        assert!(!cache_fresh(Duration::from_secs(6), true));
        assert!(cache_fresh(Duration::from_secs(2), false));
        assert!(!cache_fresh(Duration::from_secs(4), false));
    }

    #[test]
    fn the_desktop_shows_only_its_name() {
        let r = build_row("Desktop", None, state(), true);
        assert_eq!(r.len(), 1);
        assert!(r[0].items.is_empty());
    }

    #[test]
    fn an_app_gets_the_hybrid_row() {
        let read = vec![TopMenu {
            title: "File".into(),
            items: vec![Item::leaf("Nuovo", Action::App(vec![0, 0]))],
            bold: false,
            lazy: false,
        }];
        let r = build_row("Blocco note", Some(read), state(), false);
        assert_eq!(r.iter().map(|m| m.title.as_str()).collect::<Vec<_>>(), vec!["Blocco note", "File", "Finestra"]);
        let fallback = build_row("Chrome", None, state(), false);
        assert_eq!(fallback.len(), 2);
    }
}
