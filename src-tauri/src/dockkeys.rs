//! Scorciatoie da tastiera per le app del Dock: premendo (di base) Ctrl+Alt+1…9 si apre la 1ª…9ª app del Dock, come un
//! clic sulla sua icona (si avvia se è chiusa, si porta davanti se è aperta, si minimizza se è già davanti). Con la barra
//! delle applicazioni di Windows nascosta i tasti Windows+numero di Windows non hanno più effetto: questi li sostituiscono.
//! (Ctrl+Windows+numero non si può usare: su questo sistema Windows non lo lascia registrare ad altri.)
//!
//! Funzionano con `RegisterHotKey` su un thread con il proprio ciclo di messaggi (come il Launcher). Il modificatore si sceglie
//! nelle Impostazioni (o si spengono). Se una combinazione è già presa da un'altra app non si registra, senza errori.
//!
//! Qui sta la parte pura (la scelta del modificatore e quale app) e il thread.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::OnceLock;

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, PostThreadMessageW, MSG, WM_APP, WM_HOTKEY};

/// Le scelte: (id, etichetta).
pub const MODES: &[(&str, &str)] = &[("off", "Disattivate"), ("ctrl_alt", "Ctrl+Alt+1…9"), ("ctrl_shift", "Ctrl+Maiusc+1…9"), ("alt_shift", "Alt+Maiusc+1…9")];

const BASE_ID: i32 = 0x0E20;
const RELOAD: u32 = WM_APP + 0x21;

/// La modalità effettiva dato il valore salvato (vuoto = quella di base). Nelle build di prova di base sono spente (così non
/// rubano tasti durante le prove) e `EOSOS_DOCK_KEYS=1` le accende; `EOSOS_DOCK_KEYS=0` le spegne sempre. Pura, testata.
pub fn effective_mode(saved: &str, env: Option<&str>, test_build: bool) -> &'static str {
    if env == Some("0") {
        return "off";
    }
    let want = if saved.is_empty() { None } else { MODES.iter().find(|m| m.0 == saved).map(|m| m.0) };
    match (want, test_build, env) {
        (Some(m), _, _) => m,
        (None, true, Some("1")) => "ctrl_alt",
        (None, true, _) => "off",
        (None, false, _) => "ctrl_alt",
    }
}

/// I modificatori della modalità (0 = spente). Pura, testata.
pub fn modifiers(mode: &str) -> u32 {
    match mode {
        "ctrl_alt" => MOD_CONTROL.0 | MOD_ALT.0,
        "ctrl_shift" => MOD_CONTROL.0 | MOD_SHIFT.0,
        "alt_shift" => MOD_ALT.0 | MOD_SHIFT.0,
        _ => 0,
    }
}

/// Il tasto del numero `n` (1…9): i tasti '1'…'9' della riga dei numeri. Pura, testata.
pub fn key_of(n: usize) -> Option<u32> {
    (1..=9).contains(&n).then(|| 0x30 + n as u32)
}

/// Quale voce del Dock apre il numero `n` (1 = la prima), se esiste. Pura, testata.
pub fn slot(n: usize, count: usize) -> Option<usize> {
    (n >= 1 && n <= count).then(|| n - 1)
}

static THREAD_ID: AtomicU32 = AtomicU32::new(0);
static REGISTERED: AtomicUsize = AtomicUsize::new(0);
static STARTED: OnceLock<()> = OnceLock::new();

/// Quante combinazioni sono registrate adesso (per le prove).
#[allow(dead_code)]
pub fn registered_count() -> usize {
    REGISTERED.load(Ordering::Relaxed)
}

fn current_mode() -> &'static str {
    effective_mode(&crate::mcstore::load().dock_keys, std::env::var("EOSOS_DOCK_KEYS").ok().as_deref(), cfg!(feature = "test-hooks"))
}

fn register_all() {
    unsafe {
        for n in 1..=9 {
            let _ = UnregisterHotKey(None, BASE_ID + n);
        }
    }
    let mods = modifiers(current_mode());
    let mut ok = 0;
    if mods != 0 {
        for n in 1..=9usize {
            if let Some(vk) = key_of(n)
                && unsafe { RegisterHotKey(None, BASE_ID + n as i32, HOT_KEY_MODIFIERS(mods) | MOD_NOREPEAT, vk) }.is_ok()
            {
                ok += 1;
            }
        }
    }
    REGISTERED.store(ok, Ordering::Relaxed);
}

/// Apre l'app numero `n` del Dock, come un clic sulla sua icona. `false` se non c'è.
pub fn open_nth(n: usize) -> bool {
    let (items, windows) = crate::dock::items_and_windows();
    let Some(i) = slot(n, items.len()) else { return false };
    crate::dockact::activate(&items[i], &windows);
    true
}

/// Avvia il thread delle scorciatoie (una volta).
pub fn start() {
    if STARTED.set(()).is_err() {
        return;
    }
    std::thread::spawn(|| {
        THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
        register_all();
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            if msg.message == WM_HOTKEY {
                let n = (msg.wParam.0 as i32 - BASE_ID) as usize;
                // Fuori dal ciclo dei messaggi: l'attivazione enumera finestre e può avviare processi.
                std::thread::spawn(move || {
                    open_nth(n);
                });
            } else if msg.message == RELOAD {
                register_all();
            }
        }
    });
}

/// Le impostazioni sono cambiate: si ricontrollano e si registrano di nuovo le combinazioni.
pub fn reload() {
    let id = THREAD_ID.load(Ordering::Relaxed);
    if id != 0 {
        unsafe {
            let _ = PostThreadMessageW(id, RELOAD, WPARAM(0), LPARAM(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_ctrl_alt_except_in_test_builds() {
        assert_eq!(effective_mode("", None, false), "ctrl_alt");
        assert_eq!(effective_mode("", None, true), "off", "le build di prova non rubano tasti");
        assert_eq!(effective_mode("", Some("1"), true), "ctrl_alt");
        assert_eq!(effective_mode("ctrl_shift", None, true), "ctrl_shift", "la scelta dell'utente vale");
        assert_eq!(effective_mode("off", None, false), "off");
        assert_eq!(effective_mode("ctrl_alt", Some("0"), false), "off", "«0» le spegne sempre");
        assert_eq!(effective_mode("boh", None, false), "ctrl_alt", "un valore strano vale come non scelto");
        assert_eq!(effective_mode("ctrl_win", None, false), "ctrl_alt", "una scelta vecchia non piu ammessa vale come non scelta");
    }

    #[test]
    fn each_mode_has_its_modifiers() {
        assert_eq!(modifiers("off"), 0);
        assert_eq!(modifiers("nonesiste"), 0);
        assert_eq!(modifiers("ctrl_alt"), MOD_CONTROL.0 | MOD_ALT.0);
        assert_eq!(modifiers("ctrl_shift"), MOD_CONTROL.0 | MOD_SHIFT.0);
        assert_eq!(modifiers("alt_shift"), MOD_ALT.0 | MOD_SHIFT.0);
        for (id, _) in MODES.iter().filter(|m| m.0 != "off") {
            assert!(modifiers(id).count_ones() == 2, "{id}: due modificatori");
        }
    }

    #[test]
    fn digits_map_to_the_number_row() {
        assert_eq!(key_of(1), Some(0x31));
        assert_eq!(key_of(9), Some(0x39));
        assert_eq!(key_of(0), None);
        assert_eq!(key_of(10), None);
    }

    #[test]
    fn the_nth_app_is_the_nth_item_if_there_is_one() {
        assert_eq!(slot(1, 3), Some(0));
        assert_eq!(slot(3, 3), Some(2));
        assert_eq!(slot(4, 3), None);
        assert_eq!(slot(0, 3), None);
        assert_eq!(slot(1, 0), None);
    }
}
