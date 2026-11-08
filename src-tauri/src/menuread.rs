//! Menu globale per app, fase G1: misura IN SOLA LETTURA di cosa si riesce a leggere dei menu di una finestra
//! altrui, in due modi: l'`HMENU` classico della finestra e la barra dei menu di UI Automation. Nessuna voce
//! viene invocata e il risultato contiene solo conteggi, tempi e la lunghezza massima dei titoli: mai i testi.
//! Serve a decidere (con numeri) quanto è praticabile il menu globale; la parte pura è testata.

use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use serde::Serialize;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, TreeScope_Children, TreeScope_Descendants, UIA_ControlTypePropertyId,
    UIA_MenuBarControlTypeId, UIA_MenuItemControlTypeId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetMenu, GetMenuItemCount, GetMenuItemInfoW, GetMenuStringW, IsWindow, HMENU, MENUITEMINFOW, MFS_CHECKED, MFS_DISABLED,
    MFS_GRAYED, MFT_SEPARATOR, MF_BYPOSITION, MIIM_FTYPE, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU,
};

use crate::menumodel::{RawItem, RawMenu};

/// Tempo massimo per leggere una finestra con UI Automation (una finestra bloccata non deve fermarci).
pub const UIA_TIMEOUT: Duration = Duration::from_millis(1500);

/// Esito della misura su una finestra.
#[derive(Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Probe {
    /// Voci di primo livello dell'`HMENU`, se la finestra ne ha uno.
    pub hmenu_items: Option<usize>,
    /// Quante di quelle voci hanno un titolo leggibile (non vuoto).
    pub hmenu_titled: usize,
    pub hmenu_ms: u64,
    /// UI Automation ha trovato una barra dei menu nella finestra.
    pub uia_menubar: bool,
    pub uia_items: usize,
    /// Identificatori di automazione delle barre dei menu trovate (nomi tecnici, non testi dell'utente).
    pub uia_bar_ids: Vec<String>,
    pub uia_ms: u64,
    /// UI Automation non ha risposto in tempo.
    pub uia_timed_out: bool,
}

/// Quanto è praticabile il menu di una finestra, in una parola.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Readability {
    /// Né `HMENU` né barra UIA: niente menu da portare in alto.
    None,
    /// Solo l'`HMENU` classico.
    HMenu,
    /// Solo UI Automation.
    Uia,
    /// Entrambi.
    Both,
    /// UI Automation non ha risposto e l'`HMENU` non c'è: non si sa.
    Unknown,
}

/// Una barra UIA con meno voci di così non è il menu di un'app: è la voce unica del menu di sistema della
/// finestra (la console classica la espone come una barra qualunque, senza l'identificatore "SystemMenuBar").
pub const MIN_UIA_MENU_ITEMS: usize = 2;

/// Classifica un esito. Pura, testata.
pub fn classify(p: &Probe) -> Readability {
    let h = p.hmenu_items.is_some_and(|n| n > 0);
    let u = p.uia_menubar && p.uia_items >= MIN_UIA_MENU_ITEMS;
    match (h, u) {
        (true, true) => Readability::Both,
        (true, false) => Readability::HMenu,
        (false, true) => Readability::Uia,
        (false, false) if p.uia_timed_out => Readability::Unknown,
        (false, false) => Readability::None,
    }
}

fn read_hmenu(hwnd: HWND, p: &mut Probe) {
    let t = Instant::now();
    let menu = unsafe { GetMenu(hwnd) };
    if !menu.is_invalid() {
        let n = unsafe { GetMenuItemCount(Some(menu)) };
        if n > 0 {
            p.hmenu_items = Some(n as usize);
            let mut buf = [0u16; 256];
            p.hmenu_titled = (0..n)
                .filter(|&i| unsafe { GetMenuStringW(menu, i as u32, Some(&mut buf), MF_BYPOSITION) } > 0)
                .count();
        }
    }
    p.hmenu_ms = t.elapsed().as_millis() as u64;
}

/// La barra dei menu "di sistema" (quella della barra del titolo, con la sola voce del menu di sistema) ha
/// questo identificatore di automazione: non è il menu dell'app. Pura, testata.
pub fn is_system_bar(automation_id: &str) -> bool {
    automation_id.eq_ignore_ascii_case("SystemMenuBar")
}

/// UI Automation: cerca una barra dei menu fra i discendenti della finestra e conta le sue voci.
/// Dentro un thread dedicato con il proprio COM, per poterlo abbandonare se non risponde.
fn read_uia(hwnd_id: usize) -> (bool, usize, Vec<String>, Vec<String>) {
    unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let r = (|| -> windows::core::Result<(bool, usize, Vec<String>, Vec<String>)> {
            let auto: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            let el = auto.ElementFromHandle(HWND(hwnd_id as *mut _))?;
            let cond = auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_MenuBarControlTypeId.0))?;
            let bars = el.FindAll(TreeScope_Descendants, &cond)?;
            let item = auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_MenuItemControlTypeId.0))?;
            let mut ids = Vec::new();
            let mut best: Option<usize> = None;
            let mut names: Vec<String> = Vec::new();
            for i in 0..bars.Length()? {
                let bar = bars.GetElement(i)?;
                let id = bar.CurrentAutomationId().map(|s| s.to_string()).unwrap_or_default();
                ids.push(id.clone());
                // La barra "di sistema" (menu della barra del titolo) non è il menu dell'app.
                if is_system_bar(&id) {
                    continue;
                }
                let Ok(all) = bar.FindAll(TreeScope_Children, &item) else { continue };
                let n = all.Length().unwrap_or(0).max(0) as usize;
                if best.is_none_or(|b| n > b) {
                    best = Some(n);
                    names = (0..all.Length().unwrap_or(0))
                        .filter_map(|i| all.GetElement(i).ok())
                        .map(|e| e.CurrentName().map(|s| s.to_string()).unwrap_or_default())
                        .collect();
                }
            }
            Ok((best.is_some(), best.unwrap_or(0), ids, names))
        })();
        if init {
            CoUninitialize();
        }
        r.unwrap_or((false, 0, Vec::new(), Vec::new()))
    }
}

/// Legge l'`HMENU` di una finestra con tutte le sue voci (sottomenu compresi, fino a `READ_DEPTH` livelli e
/// `READ_ITEMS` voci per livello). Non invoca nulla.
fn read_hmenu_tree(menu: HMENU, depth: usize) -> Vec<RawItem> {
    let n = unsafe { GetMenuItemCount(Some(menu)) }.clamp(0, READ_ITEMS as i32);
    let mut out = Vec::new();
    for pos in 0..n as u32 {
        let mut buf = [0u16; 256];
        let mut mii = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_FTYPE | MIIM_STATE | MIIM_SUBMENU | MIIM_STRING,
            dwTypeData: windows::core::PWSTR(buf.as_mut_ptr()),
            cch: buf.len() as u32 - 1,
            ..Default::default()
        };
        if unsafe { GetMenuItemInfoW(menu, pos, true, &mut mii) }.is_err() {
            out.push(RawItem::default());
            continue;
        }
        let text = String::from_utf16_lossy(&buf[..(mii.cch as usize).min(buf.len() - 1)]);
        let separator = mii.fType.0 & MFT_SEPARATOR.0 != 0;
        let children = if !mii.hSubMenu.is_invalid() && depth + 1 < READ_DEPTH {
            read_hmenu_tree(mii.hSubMenu, depth + 1)
        } else {
            Vec::new()
        };
        out.push(RawItem {
            text,
            separator,
            grayed: mii.fState.0 & (MFS_GRAYED.0 | MFS_DISABLED.0) != 0,
            checked: mii.fState.0 & MFS_CHECKED.0 != 0,
            children,
        });
    }
    out
}

/// Livelli e voci per livello che si leggono al massimo (il modello poi taglia ancora).
pub const READ_DEPTH: usize = 4;
pub const READ_ITEMS: usize = 100;

/// Legge il menu di una finestra per la barra: prima l'`HMENU` classico (con tutte le voci), altrimenti la
/// barra di UI Automation (solo i titoli di primo livello, le voci si leggono all'apertura). `None` se la
/// finestra non ha un menu leggibile o UI Automation non risponde in tempo. Solo lettura. Dentro un thread
/// con tempo massimo, così una finestra bloccata non ferma nessuno.
pub fn read_menus(hwnd_id: usize) -> Option<Vec<RawMenu>> {
    let hwnd = HWND(hwnd_id as *mut _);
    if !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        return None;
    }
    let menu = unsafe { GetMenu(hwnd) };
    if !menu.is_invalid() {
        let items = read_hmenu_tree(menu, 0);
        // Il primo livello dell'HMENU sono i menu della barra.
        let top: Vec<RawMenu> =
            items.into_iter().map(|i| RawMenu { text: i.text, items: i.children, lazy: false }).collect();
        if top.iter().any(|m| !m.text.trim().is_empty()) {
            return Some(top);
        }
    }
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(read_uia(hwnd_id));
    });
    match rx.recv_timeout(UIA_TIMEOUT) {
        Ok((true, n, _, names)) if n >= MIN_UIA_MENU_ITEMS => {
            Some(names.into_iter().map(|text| RawMenu { text, items: Vec::new(), lazy: true }).collect())
        }
        _ => None,
    }
}

/// Misura la finestra `hwnd_id` (solo lettura). `None` se non è più una finestra.
pub fn probe(hwnd_id: usize) -> Option<Probe> {
    let hwnd = HWND(hwnd_id as *mut _);
    if !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        return None;
    }
    let mut p = Probe::default();
    read_hmenu(hwnd, &mut p);
    let t = Instant::now();
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(read_uia(hwnd_id));
    });
    match rx.recv_timeout(UIA_TIMEOUT) {
        Ok((bar, n, ids, _)) => {
            p.uia_menubar = bar;
            p.uia_items = n;
            p.uia_bar_ids = ids;
        }
        Err(_) => p.uia_timed_out = true,
    }
    p.uia_ms = t.elapsed().as_millis() as u64;
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(h: Option<usize>, bar: bool, items: usize, timeout: bool) -> Probe {
        Probe { hmenu_items: h, uia_menubar: bar, uia_items: items, uia_timed_out: timeout, ..Probe::default() }
    }

    #[test]
    fn the_title_bar_system_menu_is_not_the_app_menu() {
        assert!(is_system_bar("SystemMenuBar") && is_system_bar("systemmenubar"));
        assert!(!is_system_bar("") && !is_system_bar("MenuBar") && !is_system_bar("Menu"));
    }

    #[test]
    fn classification_covers_every_case() {
        assert_eq!(classify(&p(Some(5), true, 5, false)), Readability::Both);
        assert_eq!(classify(&p(Some(5), false, 0, false)), Readability::HMenu);
        assert_eq!(classify(&p(None, true, 7, false)), Readability::Uia);
        assert_eq!(classify(&p(None, false, 0, false)), Readability::None);
        assert_eq!(classify(&p(None, false, 0, true)), Readability::Unknown, "senza risposta non si dice 'nessun menu'");
        assert_eq!(classify(&p(Some(5), false, 0, true)), Readability::HMenu, "l'HMENU basta anche se UIA tace");
        assert_eq!(classify(&p(None, true, 0, false)), Readability::None, "barra vuota = niente da mostrare");
        assert_eq!(classify(&p(None, true, 1, false)), Readability::None, "una sola voce = menu di sistema, non dell'app");
        assert_eq!(classify(&p(None, true, 2, false)), Readability::Uia);
        assert_eq!(classify(&p(Some(0), false, 0, false)), Readability::None);
    }
}
