//! Centro di controllo: gli interruttori rapidi del pannello che si apre dalla barra (Wi-Fi, Bluetooth, Non disturbare, Mantieni sveglio).
//!
//! - **Wi-Fi e Bluetooth**: le radio di Windows (`Windows.Devices.Radios`). Se il PC non ne ha una (un desktop senza Wi-Fi) l'interruttore
//!   non c'è. Spegnere il Wi-Fi taglia la rete: lo fa solo un clic dell'utente sul pannello (mai le prove: usano una radio finta).
//! - **Non disturbare** è l'impostazione `notif.dnd` (la gestisce la pagina con i comandi già esistenti).
//! - **Mantieni sveglio**: finché è acceso il PC e lo schermo non vanno in sospensione per inattività (`SetThreadExecutionState`).
//!   Non si salva: a ogni avvio di EosOS è spento, così non resta acceso per dimenticanza.
//!
//! Qui sta la parte pura (identificatori, stato) e quella che parla con Windows.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;

/// Gli interruttori che questo modulo gestisce: (id, etichetta). `dnd` è dell'impostazione, qui non c'è.
pub const TOGGLES: &[(&str, &str)] = &[("wifi", "Wi-Fi"), ("bluetooth", "Bluetooth"), ("awake", "Mantieni sveglio")];

/// L'id è uno di quelli che si possono cambiare da qui. Pura, testata.
pub fn is_toggle(id: &str) -> bool {
    TOGGLES.iter().any(|t| t.0 == id)
}

/// Lo stato che la pagina mostra: `None` = non disponibile su questo PC (l'interruttore non si vede).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ControlState {
    pub wifi: Option<bool>,
    pub bluetooth: Option<bool>,
    pub awake: bool,
}

// ------------------------------------------------------------------ radio

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Radio {
    Wifi,
    Bluetooth,
}

fn radio_of(id: &str) -> Option<Radio> {
    match id {
        "wifi" => Some(Radio::Wifi),
        "bluetooth" => Some(Radio::Bluetooth),
        _ => None,
    }
}

/// Radio finte per le prove: `EOSOS_TEST_RADIOS=fake` (solo build con test-hooks). Stato in memoria, mai le radio vere.
static FAKE: Mutex<(Option<bool>, Option<bool>)> = Mutex::new((Some(true), Some(false)));

fn fake_on() -> bool {
    cfg!(feature = "test-hooks") && std::env::var("EOSOS_TEST_RADIOS").is_ok_and(|v| v == "fake")
}

fn read_radio(r: Radio) -> Option<bool> {
    if fake_on() {
        let f = FAKE.lock().unwrap_or_else(|e| e.into_inner());
        return if r == Radio::Wifi { f.0 } else { f.1 };
    }
    use windows::Devices::Radios::{Radio as WRadio, RadioKind, RadioState};
    let kind = if r == Radio::Wifi { RadioKind::WiFi } else { RadioKind::Bluetooth };
    let radios = WRadio::GetRadiosAsync().ok()?.join().ok()?;
    for radio in radios {
        if radio.Kind().ok()? == kind {
            return Some(radio.State().ok()? == RadioState::On);
        }
    }
    None
}

fn write_radio(r: Radio, on: bool) -> bool {
    if fake_on() {
        let mut f = FAKE.lock().unwrap_or_else(|e| e.into_inner());
        let slot = if r == Radio::Wifi { &mut f.0 } else { &mut f.1 };
        if slot.is_none() {
            return false;
        }
        *slot = Some(on);
        return true;
    }
    use windows::Devices::Radios::{Radio as WRadio, RadioAccessStatus, RadioKind, RadioState};
    let kind = if r == Radio::Wifi { RadioKind::WiFi } else { RadioKind::Bluetooth };
    let Some(radios) = WRadio::GetRadiosAsync().ok().and_then(|o| o.join().ok()) else { return false };
    for radio in radios {
        if radio.Kind().ok() != Some(kind) {
            continue;
        }
        // Senza il permesso di Windows non si cambia nulla.
        if WRadio::RequestAccessAsync().ok().and_then(|o| o.join().ok()) != Some(RadioAccessStatus::Allowed) {
            return false;
        }
        let state = if on { RadioState::On } else { RadioState::Off };
        return radio.SetStateAsync(state).ok().and_then(|o| o.join().ok()) == Some(RadioAccessStatus::Allowed);
    }
    false
}

// ------------------------------------------------------------------ mantieni sveglio

static AWAKE: AtomicBool = AtomicBool::new(false);

/// Il thread che tiene acceso lo stato di «sveglio»: `SetThreadExecutionState` vale per il thread che lo chiama, quindi uno proprio.
fn awake_thread() {
    use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED};
    unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED) };
    while AWAKE.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
}

fn set_awake(on: bool) {
    // Spento e riacceso in fretta: il vecchio thread esce da solo, il nuovo parte pulito.
    let was = AWAKE.swap(on, Ordering::Relaxed);
    if on && !was {
        std::thread::spawn(awake_thread);
    }
}

/// Lo stato di tutto ciò che sta qui.
pub fn state() -> ControlState {
    ControlState { wifi: read_radio(Radio::Wifi), bluetooth: read_radio(Radio::Bluetooth), awake: AWAKE.load(Ordering::Relaxed) }
}

/// Cambia l'interruttore `id`. `false` se l'id non è uno di questi o se Windows non lo permette.
pub fn set(id: &str, on: bool) -> bool {
    if !is_toggle(id) {
        return false;
    }
    if id == "awake" {
        set_awake(on);
        return true;
    }
    match radio_of(id) {
        Some(r) => write_radio(r, on),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_toggles_are_accepted() {
        for (id, _) in TOGGLES {
            assert!(is_toggle(id));
        }
        assert!(!is_toggle("dnd"), "Non disturbare e' un'impostazione, non passa di qui");
        assert!(!is_toggle("") && !is_toggle("Wifi") && !is_toggle("wifi "));
        assert!(!set("boh", true), "un id inventato non fa niente");
        assert_eq!(radio_of("wifi"), Some(Radio::Wifi));
        assert_eq!(radio_of("awake"), None);
    }

    #[test]
    fn keep_awake_starts_off_and_toggles() {
        // Stato globale: un solo test lo tocca. Acceso e spento di fila, senza lasciare il PC sveglio.
        assert!(!state().awake);
        assert!(set("awake", true) && state().awake);
        assert!(set("awake", true) && state().awake, "acceso due volte: resta acceso, un solo thread");
        assert!(set("awake", false) && !state().awake);
    }

    #[test]
    fn real_radios_can_be_read_without_changing_anything() {
        // Sola lettura (mai scrittura): su un PC senza Wi-Fi o Bluetooth viene None, ed è giusto.
        let s = state();
        println!("wifi {:?}, bluetooth {:?}", s.wifi, s.bluetooth);
    }
}
