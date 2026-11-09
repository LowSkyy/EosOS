//! Guardia delle cornici: nessuna finestra di EosOS deve mai avere la barra del titolo di Windows (`WS_CAPTION`) né i pulsanti di sistema.
//!
//! Tauri crea le finestre «senza decorazioni», ma in certe operazioni (mostrare, ridimensionare, cambiare monitor) rimette gli stili con cornice, e per
//! pochi istanti Windows disegna la barra del titolo in cima alla finestra. Il Dock ha già un controllo suo a ogni passo (`dockwin::enforce_styles`);
//! questo lo estende a TUTTE le altre finestre di EosOS (barre dei menu, pannelli, Launcher, Impostazioni, Mission Control, selezione della cattura).
//! Ogni 250 ms si guardano le finestre di questo processo e, solo se hanno una cornice, la si toglie. Costo trascurabile (nessuna scrittura se è tutto a posto).

use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongPtrW, GetWindowThreadProcessId, SetWindowLongPtrW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU,
};

/// Gli stili di una finestra con cornice.
pub const CHROME: isize = (WS_CAPTION.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0) as isize;

/// Lo stile senza cornice, oppure `None` se non ne ha (niente da scrivere). Pura, testata.
pub fn without_chrome(style: isize) -> Option<isize> {
    (style & CHROME != 0).then_some(style & !CHROME)
}

unsafe extern "system" fn strip(h: HWND, _l: LPARAM) -> windows::core::BOOL {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(h, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            let style = GetWindowLongPtrW(h, GWL_STYLE);
            if without_chrome(style).is_some() {
                // Si rilegge subito prima di scrivere: `show()` (altro thread) può aver aggiunto WS_VISIBLE dopo la prima lettura.
                let fresh = GetWindowLongPtrW(h, GWL_STYLE);
                SetWindowLongPtrW(h, GWL_STYLE, fresh & !CHROME);
                let _ = SetWindowPos(h, None, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED);
            }
        }
    }
    windows::core::BOOL(1)
}

/// Un giro: toglie la cornice a ogni finestra di EosOS che ce l'ha. Pubblica per le prove.
pub fn sweep() {
    unsafe {
        let _ = EnumWindows(Some(strip), LPARAM(0));
    }
}

/// Avvia il thread di guardia.
pub fn start() {
    std::thread::spawn(|| loop {
        std::thread::sleep(Duration::from_millis(250));
        sweep();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_frame_is_removed_and_other_bits_stay() {
        let popup_visible = 0x1400_0000_isize; // WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS
        assert_eq!(without_chrome(popup_visible), None, "senza cornice: niente da scrivere");
        let with_frame = popup_visible | CHROME;
        assert_eq!(without_chrome(with_frame), Some(popup_visible), "si tolgono solo i bit della cornice");
        // Anche uno solo dei bit (ad esempio il menu di sistema) conta come cornice.
        assert_eq!(without_chrome(popup_visible | WS_SYSMENU.0 as isize), Some(popup_visible));
        // Lo stile visto nel registro dell'osservatore per le barre e i pannelli.
        assert_eq!(without_chrome(0x14CB_0000), Some(0x1400_0000));
    }
}
