//! Azioni del Dock (fase D3): avviare, portare in primo piano, minimizzare,
//! chiudere e fissare/togliere un'app.
//!
//! Ogni azione parte da una voce già presente nel Dock (`dock::find`): il
//! frontend manda solo l'`id`, mai un percorso.

use std::path::{Path, PathBuf};

use windows::core::{w, HSTRING};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYEVENTF_KEYUP, VK_MENU};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    IsIconic, PostMessageW, SetForegroundWindow, ShowWindow, SW_MINIMIZE,
    SW_RESTORE, SW_SHOWNORMAL, WM_CLOSE,
};

use crate::dock::{self, same_app, DockItem, Pinned};
use crate::dockstore;

/// Cosa si può chiedere dal menu contestuale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Open,
    Quit,
    Pin,
    Unpin,
    ToggleAutoHide,
    /// L'id è il livello di vetro (`clear`, `standard`, `solid`).
    SetGlass,
    ToggleBlur,
    ToggleReserve,
}

const PREFIX: &str = "dock";

/// Identificatore di una voce di menu: `dock:<azione>:<id dell'app>`. Pura, testata.
pub fn menu_id(action: MenuAction, id: &str) -> String {
    let a = match action {
        MenuAction::Open => "open",
        MenuAction::Quit => "quit",
        MenuAction::Pin => "pin",
        MenuAction::Unpin => "unpin",
        MenuAction::ToggleAutoHide => "autohide",
        MenuAction::SetGlass => "glass",
        MenuAction::ToggleBlur => "blur",
        MenuAction::ToggleReserve => "reserve",
    };
    format!("{PREFIX}:{a}:{id}")
}

/// Inverso di `menu_id`. L'`id` può contenere `:` (es. `c:\\...`). Pura, testata.
pub fn parse_menu_id(raw: &str) -> Option<(MenuAction, String)> {
    let mut parts = raw.splitn(3, ':');
    if parts.next()? != PREFIX {
        return None;
    }
    let action = match parts.next()? {
        "open" => MenuAction::Open,
        "quit" => MenuAction::Quit,
        "pin" => MenuAction::Pin,
        "unpin" => MenuAction::Unpin,
        "autohide" => MenuAction::ToggleAutoHide,
        "glass" => MenuAction::SetGlass,
        "blur" => MenuAction::ToggleBlur,
        "reserve" => MenuAction::ToggleReserve,
        _ => return None,
    };
    let id = parts.next()?;
    (!id.is_empty()).then(|| (action, id.to_string()))
}

/// Finestre aperte dell'app, dall'alto verso il basso nell'ordine Z.
fn windows_of(item: &DockItem, windows: &[(HWND, PathBuf)]) -> Vec<HWND> {
    windows
        .iter()
        .filter(|(_, p)| same_app(p, Path::new(&item.path)))
        .map(|(h, _)| *h)
        .collect()
}
/// Avvia l'eseguibile della voce.
fn launch(item: &DockItem) {
    // App dello Store: la shell sa avviare `shell:AppsFolder\<identificatore>`.
    if dock::parse_aumid(&item.path).is_some() {
        let target = HSTRING::from(item.path.as_str());
        unsafe {
            let _ = ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL);
        }
        return;
    }
    let path = Path::new(&item.path);
    if !path.is_file() {
        return;
    }
    let dir = path.parent().map(|d| HSTRING::from(d.to_string_lossy().as_ref()));
    let file = HSTRING::from(item.path.as_str());
    unsafe {
        let _ = match &dir {
            Some(d) => ShellExecuteW(None, w!("open"), &file, None, d, SW_SHOWNORMAL),
            None => ShellExecuteW(None, w!("open"), &file, None, None, SW_SHOWNORMAL),
        };
    }
}

/// Alt o Windows sono premuti adesso (sinistro, destro o generico).
pub(crate) fn alt_or_win_down() -> bool {
    [0x12, 0xA4, 0xA5, 0x5B, 0x5C].iter().any(|&vk| unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vk) } < 0)
}

/// Porta `hwnd` in primo piano, ripristinandola se è minimizzata.
pub(crate) fn focus(hwnd: HWND) {
    let mut ok;
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        // Il trucco dell'Alt solo se Windows NEGA il primo piano. Non basta che il primo piano
        // finisca altrove: se l'app ha una finestra di dialogo aperta il primo piano va a lei
        // anche quando la richiesta riesce, e un Alt finto attiverebbe la barra dei menu.
        ok = SetForegroundWindow(hwnd).as_bool();
        // Con Alt GIA' premuto (Alt+Tab di Mission Control) il trucco no: il suo rilascio finto
        // azzererebbe l'Alt dell'utente nello stato di Windows e il Tab successivo non avrebbe piu' Alt.
        if !ok && !alt_or_win_down() {
            // Windows nega il primo piano a chi non ha ricevuto input: premere
            // Alt un istante lo sblocca (trucco classico).
            keybd_event(VK_MENU.0 as u8, 0, Default::default(), 0);
            ok = SetForegroundWindow(hwnd).as_bool();
            keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }
    // Si ricorda la finestra solo se il primo piano è stato ottenuto davvero. Se Windows lo
    // ha negato (l'icona lampeggia nella taskbar) ricordarla farebbe credere al clic
    // successivo che l'app sia già davanti, e la minimizzerebbe invece di portarla avanti.
    if ok {
        crate::foreground::note_activated(hwnd);
    }
}

/// Quale finestra va minimizzata al clic su un'app? Pura, testata.
///
/// Si minimizza solo se la finestra attivata per ultima dall'utente (`front`)
/// appartiene a questa app ed è visibile: allora è l'app "davanti". Si preferisce
/// la finestra `front` stessa se è tra quelle note, altrimenti la prima visibile.
/// `wins` sono le finestre dell'app (hwnd, minimizzata?).
///
/// Non si usano `GetForegroundWindow` (cliccando sul Dock il primo piano passa a
/// EosOS) né l'ordine Z (le finestre "sempre in primo piano" di Explorer stanno
/// davanti a tutto): si usa l'ultima attivazione vista dall'hook di sistema.
pub fn window_to_minimize(
    front: Option<isize>,
    front_belongs: bool,
    front_minimized: bool,
    wins: &[(isize, bool)],
) -> Option<isize> {
    let front = front?;
    if !front_belongs || front_minimized {
        return None;
    }
    if wins.iter().any(|(h, min)| *h == front && !min) {
        return Some(front);
    }
    wins.iter().find(|(_, min)| !min).map(|(h, _)| *h)
}

/// Clic sull'icona: avvia se chiusa; se aperta la porta in primo piano, e se è
/// già quella davanti la minimizza.
pub fn activate(item: &DockItem, windows: &[(HWND, PathBuf)]) {
    let wins = windows_of(item, windows);
    let Some(&top) = wins.first() else {
        launch(item);
        return;
    };
    let front = crate::foreground::last_window();
    let front_belongs = front
        .and_then(dock::window_identity)
        .is_some_and(|p| same_app(&p, Path::new(&item.path)));
    let front_minimized = front.is_some_and(|h| unsafe { IsIconic(h).as_bool() });
    let state: Vec<(isize, bool)> =
        wins.iter().map(|h| (h.0 as isize, unsafe { IsIconic(*h).as_bool() })).collect();
    match window_to_minimize(front.map(|h| h.0 as isize), front_belongs, front_minimized, &state) {
        Some(target) => unsafe {
            let _ = ShowWindow(HWND(target as *mut _), SW_MINIMIZE);
        },
        None => focus(top),
    }
}

/// "Esci": chiede la chiusura (WM_CLOSE) di tutte le finestre dell'app, come
/// farebbe il pulsante X: l'app può ancora chiedere di salvare.
pub fn quit(item: &DockItem, windows: &[(HWND, PathBuf)]) {
    for hwnd in windows_of(item, windows) {
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

/// Esegue un'azione del menu su una voce identificata dall'`id`.
pub fn run_menu_action(action: MenuAction, id: &str) {
    match action {
        MenuAction::ToggleAutoHide => return crate::dockwin::toggle_autohide(),
        MenuAction::ToggleBlur => return crate::dockwin::toggle_blur(),
        MenuAction::ToggleReserve => return crate::dockwin::toggle_reserve(),
        MenuAction::SetGlass => {
            if let Some(v) = dockstore::glass_value(id) {
                dockstore::set_glass(v);
            }
            return;
        }
        _ => {}
    }
    let Some((item, windows)) = dock::find_with_windows(id) else { return };
    match action {
        MenuAction::Open => activate_or_launch(&item, &windows),
        MenuAction::Quit => quit(&item, &windows),
        MenuAction::Pin => dockstore::pin(Pinned { name: item.name.clone(), path: item.path.clone().into() }),
        MenuAction::Unpin => dockstore::unpin(Path::new(&item.path)),
        MenuAction::ToggleAutoHide
        | MenuAction::SetGlass
        | MenuAction::ToggleBlur
        | MenuAction::ToggleReserve => {} // gestite sopra: non riguardano una voce
    }
}

/// "Apri" dal menu: porta in primo piano senza mai minimizzare.
fn activate_or_launch(item: &DockItem, windows: &[(HWND, PathBuf)]) {
    match windows_of(item, windows).first() {
        Some(&top) => focus(top),
        None => launch(item),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_id_roundtrip_with_colons_in_the_id() {
        for a in [MenuAction::Open, MenuAction::Quit, MenuAction::Pin, MenuAction::Unpin, MenuAction::ToggleAutoHide, MenuAction::SetGlass, MenuAction::ToggleBlur, MenuAction::ToggleReserve] {
            let raw = menu_id(a, r"c:\program files\app\app.exe");
            assert_eq!(parse_menu_id(&raw), Some((a, r"c:\program files\app\app.exe".to_string())));
        }
    }

    #[test]
    fn minimizes_only_when_the_app_is_already_in_front() {
        let wins = [(1, false), (2, true)];
        // Davanti c'è una finestra dell'app: si minimizza proprio quella.
        assert_eq!(window_to_minimize(Some(1), true, false, &wins), Some(1));
        // L'evento di primo piano riporta una finestra non in elenco ma dell'app:
        // si minimizza la prima visibile.
        assert_eq!(window_to_minimize(Some(77), true, false, &wins), Some(1));
        // Davanti c'è un'altra app: si porta in primo piano, non si minimizza.
        assert_eq!(window_to_minimize(Some(9), false, false, &wins), None);
        // La finestra davanti è minimizzata: va ripristinata.
        assert_eq!(window_to_minimize(Some(1), true, true, &wins), None);
        // Nessuna finestra attivata finora, o tutte minimizzate.
        assert_eq!(window_to_minimize(None, true, false, &wins), None);
        assert_eq!(window_to_minimize(Some(5), true, false, &[(2, true)]), None);
    }

    #[test]
    fn foreign_or_malformed_menu_ids_are_ignored() {
        assert_eq!(parse_menu_id("altro:open:x"), None);
        assert_eq!(parse_menu_id("dock:boom:x"), None);
        assert_eq!(parse_menu_id("dock:open:"), None);
        assert_eq!(parse_menu_id("dock:open"), None);
        assert_eq!(parse_menu_id(""), None);
    }
}
