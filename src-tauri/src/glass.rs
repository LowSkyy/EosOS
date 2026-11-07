//! Effetto vetro sulle barre: Acrylic -> blur -> nessun effetto (fondo solido
//! gestito dal CSS). Toglie anche angoli arrotondati e bordo DWM, che su una
//! barra a tutta larghezza sarebbero fuori luogo.
//!
//! Il frontend legge lo stato con il comando `glass_state` e imposta
//! `data-glass` sul documento: con "none" i token CSS usano colori opachi.

use std::sync::Mutex;

use tauri::WebviewWindow;
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_DONOTROUND,
};
use window_vibrancy::{apply_acrylic, apply_blur};

/// Effetto effettivamente applicato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glass {
    Acrylic,
    Blur,
    None,
}

impl Glass {
    pub fn as_str(self) -> &'static str {
        match self {
            Glass::Acrylic => "acrylic",
            Glass::Blur => "blur",
            Glass::None => "none",
        }
    }
}

/// Stato dell'ultimo tentativo (uguale per tutte le barre: stesso sistema).
static STATE: Mutex<Glass> = Mutex::new(Glass::None);

pub fn state() -> Glass {
    *STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Sceglie il primo effetto che riesce. `try_acrylic`/`try_blur` sono parametri
/// per poter testare la catena di ripiego senza finestre reali.
pub fn choose<A, B>(try_acrylic: A, try_blur: B) -> Glass
where
    A: FnOnce() -> bool,
    B: FnOnce() -> bool,
{
    if try_acrylic() {
        Glass::Acrylic
    } else if try_blur() {
        Glass::Blur
    } else {
        Glass::None
    }
}

/// Applica angoli squadrati e nessun bordo.
pub(crate) fn square_borderless(window: &WebviewWindow) {
    let Ok(hwnd) = window.hwnd() else { return };
    unsafe {
        let corner = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner as *const _ as *const _,
            size_of_val(&corner) as u32,
        );
        let none = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &none as *const _ as *const _,
            size_of_val(&none) as u32,
        );
    }
}

/// Applica l'effetto vetro alla finestra e registra lo stato risultante.
pub fn apply(window: &WebviewWindow) -> Glass {
    square_borderless(window);

    // Tinta di ripiego per il blur (la tinta principale la dà il CSS).
    let g = choose(
        || apply_acrylic(window, None).is_ok(),
        || apply_blur(window, Some((24, 24, 26, 120))).is_ok(),
    );
    if std::env::var("EOSOS_DEBUG_APPBAR").is_ok() {
        eprintln!("glass[{}]: {}", window.label(), g.as_str());
    }
    *STATE.lock().unwrap_or_else(|e| e.into_inner()) = g;
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acrylic_wins_when_available() {
        assert_eq!(choose(|| true, || panic!("non deve provare il blur")), Glass::Acrylic);
    }

    #[test]
    fn falls_back_to_blur() {
        assert_eq!(choose(|| false, || true), Glass::Blur);
    }

    #[test]
    fn falls_back_to_none() {
        assert_eq!(choose(|| false, || false), Glass::None);
        assert_eq!(Glass::None.as_str(), "none");
    }
}
