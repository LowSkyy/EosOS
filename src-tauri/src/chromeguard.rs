//! Guardia delle cornici: nessuna finestra di EosOS deve mai avere la barra del titolo di Windows (`WS_CAPTION`) né i pulsanti di sistema.
//!
//! Tauri crea le finestre «senza decorazioni», ma in certe operazioni (mostrare, ridimensionare, cambiare monitor) rimette gli stili con cornice, e per
//! pochi istanti Windows disegna la barra del titolo in cima alla finestra. Il Dock ha già un controllo suo a ogni passo (`dockwin::enforce_styles`);
//! questo lo estende a TUTTE le altre finestre di EosOS (barre dei menu, pannelli, Launcher, Impostazioni, Mission Control, selezione della cattura).
//! Ogni 250 ms si guardano le finestre di questo processo e, solo se hanno una cornice, la si toglie. Costo trascurabile (nessuna scrittura se è tutto a posto).

use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMNCRP_ENABLED, DWMWA_NCRENDERING_POLICY};
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongPtrW, GetWindowThreadProcessId, SetWindowLongPtrW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, STYLESTRUCT, WM_NCACTIVATE, WM_NCCALCSIZE, WM_NCPAINT, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU,
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

/// Messaggio «lo stile della finestra sta per cambiare».
const WM_STYLECHANGING: u32 = 0x007C;
/// Identificatore della sottoclasse (uno a caso, fisso).
const SUBCLASS_ID: usize = 0xE05C;

/// Gli stili che si possono davvero scrivere dopo una richiesta di cambio: senza i bit della cornice. Pura, testata.
pub fn style_allowed(requested: u32) -> u32 {
    requested & !(CHROME as u32)
}

/// La procedura delle finestre protette: **impedisce che lo stile con la cornice venga mai impostato** (così non c'è nemmeno un fotogramma con la
/// barra del titolo, cosa che il controllo ogni 250 ms non può garantire) e non lascia mai un'area non client.
unsafe extern "system" fn guard_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _id: usize, _data: usize) -> LRESULT {
    match msg {
        // wParam = GWL_STYLE (-16): si tolgono i bit della cornice dal nuovo stile prima che Windows lo scriva.
        WM_STYLECHANGING if wparam.0 as i32 == GWL_STYLE.0 && lparam.0 != 0 => {
            // SAFETY: per WM_STYLECHANGING lParam punta a una STYLESTRUCT valida per la durata del messaggio.
            let ss = unsafe { &mut *(lparam.0 as *mut STYLESTRUCT) };
            ss.styleNew = style_allowed(ss.styleNew);
        }
        // Area client = tutta la finestra, comunque (con wParam = 0 lParam e' un RECT che resta com'e': nessuna area non client).
        WM_NCCALCSIZE => return LRESULT(0),
        // Non c'e' niente da disegnare fuori dall'area client: Windows non deve mai disegnare (nemmeno una volta) una cornice, la barra del titolo
        // «EosOS Dock» che compariva sopra il Dock ad ogni clic su un'app era un disegno del genere, rimasto nella finestra.
        WM_NCPAINT => return LRESULT(0),
        // L'attivazione/disattivazione non deve ridisegnare la cornice: lParam = -1 dice a Windows di non toccarla.
        WM_NCACTIVATE => return unsafe { DefSubclassProc(hwnd, msg, wparam, LPARAM(-1)) },
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// Protegge una finestra appena creata (da chiamare subito dopo `build()`, prima che si veda).
pub fn protect_hwnd(h: HWND) {
    unsafe {
        let _ = SetWindowSubclass(h, Some(guard_proc), SUBCLASS_ID, 0);
        // Il rendering non client di DWM acceso: con quello spento (come lo lasciava la creazione della finestra trasparente) Windows disegna
        // la cornice in stile classico, azzurra, con il titolo.
        let policy = DWMNCRP_ENABLED;
        let _ = DwmSetWindowAttribute(h, DWMWA_NCRENDERING_POLICY, &policy as *const _ as *const _, size_of_val(&policy) as u32);
        // E toglie subito una cornice eventualmente gia' presente.
        let style = GetWindowLongPtrW(h, GWL_STYLE);
        if without_chrome(style).is_some() {
            SetWindowLongPtrW(h, GWL_STYLE, style & !CHROME);
        }
    }
}

/// Come `protect_hwnd`, per una finestra di Tauri, **completata prima di tornare** (cosi' la finestra non puo' essere mostrata senza protezione).
/// `SetWindowSubclass` funziona solo dal thread che possiede la finestra (il principale di Tauri): se si e' su un altro thread (le barre in alto
/// si creano cosi') si passa dal thread principale e si aspetta, al massimo 2 secondi.
pub fn protect(win: &tauri::WebviewWindow) {
    let Ok(h) = win.hwnd() else { return };
    let same_thread = unsafe { GetWindowThreadProcessId(h, None) == GetCurrentThreadId() };
    if same_thread {
        protect_hwnd(h);
        return;
    }
    let raw = h.0 as isize;
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    if win
        .run_on_main_thread(move || {
            protect_hwnd(HWND(raw as *mut _));
            let _ = tx.send(());
        })
        .is_ok()
    {
        let _ = rx.recv_timeout(Duration::from_secs(2));
    }
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
    fn a_requested_style_never_keeps_the_frame_bits() {
        assert_eq!(style_allowed(0x14CB_0000), 0x1400_0000);
        assert_eq!(style_allowed(0x1400_0000), 0x1400_0000);
        assert_eq!(style_allowed(WS_CAPTION.0), 0);
        assert_eq!(style_allowed(0x1400_0000 | WS_SYSMENU.0), 0x1400_0000);
    }

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
