//! Menu globale, fase G3: la riga di menu della barra per l'app in primo piano. Quando cambia la finestra in
//! primo piano (l'hook di `foreground.rs`) si emette SUBITO una riga di ripiego (nome dell'app + "Finestra"), poi
//! un thread legge il menu dell'app (con tempo massimo, in sola lettura) e, se la finestra è ancora quella,
//! emette la riga completa. Le letture si tengono in una cache a tempo; una nuova richiesta scavalca le vecchie.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::{CloseHandle, LPARAM, WPARAM};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, IsZoomed, PostMessageW, ShowWindow,
    SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WM_CLOSE,
};

use crate::menumodel::{filter_view, never_read, quits_only_this_window, is_excluded, toggle_excluded, compose, item_at, name_only, normalize, raw_items, raw_to_top, Action, Cmd, TopMenu, WinState};

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
static MENU_OFF: AtomicBool = AtomicBool::new(false);
/// Cosa mostrare nella riga: il nome dell'app e il menu «Finestra» (vedi `filter_view`).
static SHOW_NAME: AtomicBool = AtomicBool::new(true);
static SHOW_WINDOW: AtomicBool = AtomicBool::new(false);
static EXCLUDED: Mutex<Vec<String>> = Mutex::new(Vec::new());
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
    let row = if desktop { name_only(name) } else { compose(name, read, state) };
    // Le scelte dell'utente (nome dell'app e menu Finestra) valgono per ogni riga pubblicata.
    filter_view(row, SHOW_NAME.load(Ordering::Relaxed), SHOW_WINDOW.load(Ordering::Relaxed))
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

/// Rilegge le impostazioni (menu acceso o spento, app escluse) dal file.
pub fn reload_settings() {
    let s = crate::mcstore::load();
    MENU_OFF.store(s.menu_off, Ordering::Relaxed);
    SHOW_NAME.store(!s.menu_hide_name, Ordering::Relaxed);
    SHOW_WINDOW.store(s.menu_show_window, Ordering::Relaxed);
    *EXCLUDED.lock().unwrap_or_else(|e| e.into_inner()) = s.menu_excluded;
}

/// Il menu di quest'app non va letto né mostrato: menu globale spento o app esclusa. Pura nella decisione
/// (`off` e `excluded` arrivano da fuori), testata.
pub fn skip_menu(off: bool, excluded: &[String], name: &str) -> bool {
    off || is_excluded(excluded, name)
}

fn skipped(name: &str) -> bool {
    never_read(name) || skip_menu(MENU_OFF.load(Ordering::Relaxed), &EXCLUDED.lock().unwrap_or_else(|e| e.into_inner()), name)
}

/// Il nome del file eseguibile (senza estensione) che possiede la finestra, se lo si trova.
fn exe_stem_of(hwnd: HWND) -> Option<String> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let path = crate::util::image_path(handle);
    unsafe {
        let _ = CloseHandle(handle);
    }
    path?.file_stem().map(|s| s.to_string_lossy().to_string())
}

/// Azione del Launcher: accende o spegne il menu globale. Ridisegna la barra per la finestra in primo piano.
pub fn toggle_menu_off() {
    set_menu_off(!MENU_OFF.load(Ordering::Relaxed));
}

/// Accende (`off` falso) o spegne il menu globale, salva e ridisegna la barra.
pub fn set_menu_off(off: bool) {
    crate::mcstore::set_menu_off(off);
    reload_settings();
    refresh_current();
}

/// Sceglie se mostrare il nome dell'app e il menu «Finestra», salva e ridisegna la barra.
pub fn set_view(show_name: Option<bool>, show_window: Option<bool>) {
    crate::mcstore::set_menu_view(show_name, show_window);
    reload_settings();
    refresh_current();
}

/// Mette l'elenco delle app escluse dal menu globale, salva e ridisegna la barra.
pub fn set_excluded(list: Vec<String>) {
    crate::mcstore::set_menu_excluded(list);
    reload_settings();
    refresh_current();
}

/// Azione del Launcher: esclude dal menu globale l'app in primo piano, o la reincluge se era esclusa.
pub fn toggle_current_excluded() -> bool {
    let Some(row) = current() else { return false };
    let list = toggle_excluded(&crate::mcstore::load().menu_excluded, &row.name);
    crate::mcstore::set_menu_excluded(list);
    reload_settings();
    refresh_current();
    true
}

/// Rimanda alla barra la finestra corrente (dopo un cambio di impostazioni).
fn refresh_current() {
    let id = CURRENT.load(Ordering::Relaxed);
    let name = current().map(|r| r.name);
    if let (true, Some(name), Some(tx)) = (id != 0, name, TX.get()) {
        *CACHE.lock().unwrap_or_else(|e| e.into_inner()) = None;
        let _ = tx.lock().unwrap_or_else(|e| e.into_inner()).send(Req { hwnd: id, name });
    }
}

/// Avvia il thread che legge i menu. Si chiama una volta all'avvio.
pub fn start(app: AppHandle) {
    reload_settings();
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
        // Menu globale spento o app esclusa: solo il nome, e nessuna lettura.
        if !desktop && skipped(&req.name) {
            publish(Row { hwnd: req.hwnd as u64, name: req.name.clone(), menus: build_row(&req.name, None, state, true), read: true });
            continue;
        }
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

// ------------------------------------------------------------------ tendina (G4)

/// Il menu `index` della riga corrente è «pigro» (voci da leggere)?
pub fn is_lazy(index: usize) -> bool {
    let g = LAST.lock().unwrap_or_else(|e| e.into_inner());
    g.as_ref().and_then(|r| r.menus.get(index)).is_some_and(|m| m.lazy)
}

/// Il menu `index` della riga corrente per la tendina. Se è «pigro» (voci non ancora lette) lo legge ora
/// (apre un istante il menu nell'app e lo richiude) e lo ricorda nella riga. `None` se l'indice non c'è o la
/// finestra è sparita.
pub fn menu(index: usize) -> Option<TopMenu> {
    let (hwnd, m) = {
        let g = LAST.lock().unwrap_or_else(|e| e.into_inner());
        let row = g.as_ref()?;
        (row.hwnd as usize, row.menus.get(index)?.clone())
    };
    if !unsafe { IsWindow(Some(HWND(hwnd as *mut _))) }.as_bool() {
        return None;
    }
    if !m.lazy {
        return Some(m);
    }
    let source = m.source?;
    let items = normalize(raw_items(source, &crate::menuread::read_lazy_items(hwnd, source)), 0);
    let filled = TopMenu { items, lazy: false, ..m };
    let mut g = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(row) = g.as_mut()
        && row.hwnd as usize == hwnd
        && let Some(slot) = row.menus.get_mut(index)
    {
        *slot = filled.clone();
    }
    Some(filled)
}

/// Toglie dalla cache la lettura di una finestra (il suo menu può essere cambiato).
fn forget_cache(hwnd: usize) {
    if let Some(c) = CACHE.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        c.remove(&hwnd);
    }
}

/// Chiude con WM_CLOSE tutte le finestre visibili di primo livello del processo di `hwnd` (esci dall'app nel
/// modo gentile: ogni finestra può chiedere di salvare). Mai si termina un processo.
fn quit_app(hwnd: HWND) {
    // Un programma che ospita finestre di molte app (o la shell): si chiude solo la finestra scelta.
    if exe_stem_of(hwnd).as_deref().is_none_or(quits_only_this_window) {
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        return;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return;
    }
    struct Ctx {
        pid: u32,
        wins: Vec<usize>,
    }
    unsafe extern "system" fn each(h: HWND, l: LPARAM) -> windows::core::BOOL {
        let ctx = unsafe { &mut *(l.0 as *mut Ctx) };
        let mut p = 0u32;
        unsafe { GetWindowThreadProcessId(h, Some(&mut p)) };
        if p == ctx.pid && unsafe { IsWindowVisible(h) }.as_bool() {
            ctx.wins.push(h.0 as usize);
        }
        true.into()
    }
    let mut ctx = Ctx { pid, wins: Vec::new() };
    unsafe {
        let _ = EnumWindows(Some(each), LPARAM(&mut ctx as *mut Ctx as isize));
    }
    for w in ctx.wins {
        unsafe {
            let _ = PostMessageW(Some(HWND(w as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

/// Esegue un comando di EosOS sulla finestra `hwnd`. `false` se la finestra non c'è più.
fn run_cmd(hwnd: usize, cmd: Cmd) -> bool {
    // Le impostazioni non riguardano nessuna finestra (nemmeno il desktop): si aprono e basta.
    if cmd == Cmd::OpenSettings {
        crate::settingswin::open_soon(None);
        return true;
    }
    let h = HWND(hwnd as *mut _);
    if !unsafe { IsWindow(Some(h)) }.as_bool() {
        return false;
    }
    unsafe {
        match cmd {
            Cmd::Minimize => {
                let _ = ShowWindow(h, SW_MINIMIZE);
            }
            Cmd::Maximize => {
                let _ = ShowWindow(h, SW_MAXIMIZE);
            }
            Cmd::Restore => {
                let _ = ShowWindow(h, SW_RESTORE);
            }
            Cmd::Close => {
                let _ = PostMessageW(Some(h), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            Cmd::Quit => quit_app(h),
            Cmd::OpenSettings => {}
        }
    }
    true
}

/// Sceglie la voce `path` del menu `index` della riga corrente. Si agisce solo su una voce che esiste, è attiva
/// e non è un sottomenu né un separatore; i comandi di EosOS si eseguono qui, quelli dell'app arrivano in G5.
/// Restituisce se è stato eseguito qualcosa.
pub fn choose(index: usize, path: &[usize]) -> bool {
    let (hwnd, action, title) = {
        let g = LAST.lock().unwrap_or_else(|e| e.into_inner());
        let Some(row) = g.as_ref() else { return false };
        let Some(menu) = row.menus.get(index) else { return false };
        let Some(item) = item_at(menu, path) else { return false };
        if !item.enabled || item.separator || item.is_submenu() {
            return false;
        }
        (row.hwnd as usize, item.action.clone(), item.title.clone())
    };
    match action {
        Some(Action::Eos(cmd)) => run_cmd(hwnd, cmd),
        Some(Action::App(path)) => {
            // Prima l'app in primo piano (una finestra di dialogo che si apre deve comparire davanti), poi la voce.
            let h = HWND(hwnd as *mut _);
            crate::dockact::focus(h);
            match crate::menuread::invoke_item(hwnd, &path, &title) {
                Ok(()) => {
                    // La riga letta resta valida, ma l'app può aver cambiato il suo menu (elenchi recenti, spunte):
                    // si dimentica la cache così al prossimo cambio di finestra si rilegge.
                    forget_cache(hwnd);
                    true
                }
                Err(e) => {
                    if e == crate::menuread::InvokeErr::Changed {
                        forget_cache(hwnd);
                    }
                    false
                }
            }
        }
        None => false,
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
    fn off_or_excluded_apps_are_not_read() {
        let l = vec!["Chrome".to_string()];
        assert!(skip_menu(true, &[], "Qualsiasi"));
        assert!(skip_menu(false, &l, "chrome"));
        assert!(!skip_menu(false, &l, "Blocco note"));
        assert!(!skip_menu(false, &[], ""));
    }

    #[test]
    fn the_desktop_shows_only_its_name() {
        let r = build_row("Desktop", None, state(), true);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].items.len(), 1, "solo la voce delle impostazioni");
    }

    #[test]
    fn an_app_gets_the_hybrid_row() {
        let read = vec![TopMenu {
            title: "File".into(),
            items: vec![Item::leaf("Nuovo", Action::App(vec![0, 0]))],
            bold: false,
            lazy: false,
            source: None,
        }];
        let r = build_row("Blocco note", Some(read), state(), false);
        // Di base il menu «Finestra» è nascosto (vedi `filter_view`): restano il nome dell'app e i menu letti.
        assert_eq!(r.iter().map(|m| m.title.as_str()).collect::<Vec<_>>(), vec!["Blocco note", "File"]);
        let fallback = build_row("Chrome", None, state(), false);
        assert_eq!(fallback.len(), 1, "senza lettura: solo il nome dell'app");
    }
}
