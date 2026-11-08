//! Menu globale per app, fase G1: misura IN SOLA LETTURA di cosa si riesce a leggere dei menu di una finestra
//! altrui, in due modi: l'`HMENU` classico della finestra e la barra dei menu di UI Automation. Nessuna voce
//! viene invocata e il risultato contiene solo conteggi, tempi e la lunghezza massima dei titoli: mai i testi.
//! Serve a decidere (con numeri) quanto è praticabile il menu globale; la parte pura è testata.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use serde::Serialize;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationExpandCollapsePattern, IUIAutomationInvokePattern, TreeScope_Children, TreeScope_Descendants,
    UIA_ControlTypePropertyId, UIA_ExpandCollapsePatternId, UIA_InvokePatternId, UIA_MenuBarControlTypeId, UIA_MenuItemControlTypeId,
};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetMenu, GetMenuItemCount, GetMenuItemInfoW, GetMenuStringW, IsWindow, PostMessageW, HMENU, MENUITEMINFOW, MFS_CHECKED,
    MFS_DISABLED, MFS_GRAYED, MFT_SEPARATOR, MF_BYPOSITION, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU,
    WM_COMMAND,
};

use crate::menumodel::{RawItem, RawMenu};

/// Letture UI Automation ancora in corso (anche quelle abbandonate dopo il tempo massimo): se un'app non risponde
/// il suo thread resta bloccato, e non se ne accumulano all'infinito.
static INFLIGHT: AtomicUsize = AtomicUsize::new(0);
/// Oltre questo numero di letture in corso non se ne avviano altre (si considera "nessun menu").
pub const MAX_INFLIGHT: usize = 4;

/// Si può avviare un'altra lettura? Pura, testata.
pub fn may_start_read(inflight: usize) -> bool {
    inflight < MAX_INFLIGHT
}

/// Conta una lettura in corso finché il guard vive.
struct Inflight;
impl Inflight {
    fn enter() -> Option<Inflight> {
        let n = INFLIGHT.fetch_add(1, Ordering::SeqCst);
        if may_start_read(n) {
            Some(Inflight)
        } else {
            INFLIGHT.fetch_sub(1, Ordering::SeqCst);
            None
        }
    }
}
impl Drop for Inflight {
    fn drop(&mut self) {
        INFLIGHT.fetch_sub(1, Ordering::SeqCst);
    }
}

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
    let guard = Inflight::enter()?;
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _guard = guard; // si libera quando la lettura finisce davvero, anche se nessuno la aspetta più
        let _ = tx.send(read_uia(hwnd_id));
    });
    match rx.recv_timeout(UIA_TIMEOUT) {
        Ok((true, n, _, names)) if n >= MIN_UIA_MENU_ITEMS => {
            Some(names.into_iter().map(|text| RawMenu { text, items: Vec::new(), lazy: true }).collect())
        }
        _ => None,
    }
}

/// La barra dei menu "dell'app" fra quelle di UI Automation di una finestra: non quella di sistema, la più
/// ricca (almeno `MIN_UIA_MENU_ITEMS` voci).
fn best_bar(
    auto: &IUIAutomation,
    el: &windows::Win32::UI::Accessibility::IUIAutomationElement,
) -> windows::core::Result<Option<windows::Win32::UI::Accessibility::IUIAutomationElement>> {
    let (cond, item) = unsafe {
        (
            auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_MenuBarControlTypeId.0))?,
            auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_MenuItemControlTypeId.0))?,
        )
    };
    let bars = unsafe { el.FindAll(TreeScope_Descendants, &cond)? };
    let mut best: Option<(windows::Win32::UI::Accessibility::IUIAutomationElement, i32)> = None;
    for i in 0..unsafe { bars.Length()? } {
        let bar = unsafe { bars.GetElement(i)? };
        let id = unsafe { bar.CurrentAutomationId() }.map(|s| s.to_string()).unwrap_or_default();
        if is_system_bar(&id) {
            continue;
        }
        let n = unsafe { bar.FindAll(TreeScope_Children, &item).and_then(|a| a.Length()) }.unwrap_or(0);
        if n as usize >= MIN_UIA_MENU_ITEMS && best.as_ref().is_none_or(|(_, b)| n > *b) {
            best = Some((bar, n));
        }
    }
    Ok(best.map(|(b, _)| b))
}

/// Le voci del menu `source` di una finestra con barra UI Automation. Per leggerle il menu si APRE un istante
/// nell'app (ExpandCollapse) e si richiude: è l'unico modo che l'accessibilità offre per le app che non
/// espongono le voci chiuse, e si fa solo quando l'utente apre quel menu nella barra.
fn lazy_uia(hwnd_id: usize, source: usize) -> Vec<RawItem> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let out = (|| -> windows::core::Result<Vec<RawItem>> {
            let auto: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            let el = auto.ElementFromHandle(HWND(hwnd_id as *mut _))?;
            let Some(bar) = best_bar(&auto, &el)? else { return Ok(Vec::new()) };
            let item = auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_MenuItemControlTypeId.0))?;
            let tops = bar.FindAll(TreeScope_Children, &item)?;
            let top = tops.GetElement(source as i32)?;
            let pattern: windows::core::Result<IUIAutomationExpandCollapsePattern> = top.GetCurrentPatternAs(UIA_ExpandCollapsePatternId);
            if let Ok(p) = &pattern {
                let _ = p.Expand();
                std::thread::sleep(Duration::from_millis(120));
            }
            let kids = top.FindAll(TreeScope_Descendants, &item)?;
            let mut out = Vec::new();
            for i in 0..kids.Length()?.min(READ_ITEMS as i32) {
                let k = kids.GetElement(i)?;
                out.push(RawItem {
                    text: k.CurrentName().map(|s| s.to_string()).unwrap_or_default(),
                    separator: false,
                    grayed: !k.CurrentIsEnabled().map(|b| b.as_bool()).unwrap_or(true),
                    checked: false,
                    children: Vec::new(),
                });
            }
            if let Ok(p) = &pattern {
                let _ = p.Collapse();
            }
            Ok(out)
        })();
        if init {
            CoUninitialize();
        }
        out.unwrap_or_default()
    }
}

/// Legge le voci del menu pigro `source` (con tempo massimo). Vuoto se non si riesce.
pub fn read_lazy_items(hwnd_id: usize, source: usize) -> Vec<RawItem> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(lazy_uia(hwnd_id, source));
    });
    rx.recv_timeout(LAZY_TIMEOUT).unwrap_or_default()
}

/// Tempo massimo per aprire e leggere un menu pigro.
pub const LAZY_TIMEOUT: Duration = Duration::from_millis(2500);

// ------------------------------------------------------------------ invocazione (G5)

/// Perché una voce non è stata eseguita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvokeErr {
    /// La finestra non c'è più o non ha più il menu.
    Gone,
    /// Il menu è cambiato dopo che l'abbiamo letto (la voce a quel posto ha un altro titolo): non si indovina.
    Changed,
    /// La voce è disattivata, è un separatore o un sottomenu.
    NotInvokable,
    /// L'app non ha risposto in tempo o ha rifiutato.
    Failed,
}

/// Il titolo mostrato coincide con quello della voce letta ora? Pura, testata.
pub fn titles_match(shown: &str, raw_now: &str) -> bool {
    crate::menumodel::split_label(raw_now).0 == shown.trim()
}

/// Il valore di `WPARAM` di un `WM_COMMAND` da menu per l'identificatore `id`: la parola bassa è l'id, quella alta
/// 0 (menu). `None` se l'id è 0 (voci senza comando) o non sta in 16 bit. Pura, testata.
pub fn wm_command_wparam(id: u32) -> Option<usize> {
    (id != 0 && id <= 0xFFFF).then_some(id as usize)
}

/// Esegue la voce `path` del menu `HMENU` della finestra: si rilegge la voce (posizione e titolo devono
/// coincidere con quello mostrato) e si manda alla finestra il `WM_COMMAND` del suo identificatore, come se
/// l'utente l'avesse scelta nel menu.
fn invoke_hmenu(hwnd: HWND, path: &[usize], expect_title: &str) -> Result<(), InvokeErr> {
    let mut menu = unsafe { GetMenu(hwnd) };
    if menu.is_invalid() {
        return Err(InvokeErr::Gone);
    }
    for (k, &pos) in path.iter().enumerate() {
        let mut buf = [0u16; 256];
        let mut mii = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_FTYPE | MIIM_STATE | MIIM_SUBMENU | MIIM_STRING | MIIM_ID,
            dwTypeData: windows::core::PWSTR(buf.as_mut_ptr()),
            cch: buf.len() as u32 - 1,
            ..Default::default()
        };
        if unsafe { GetMenuItemInfoW(menu, pos as u32, true, &mut mii) }.is_err() {
            return Err(InvokeErr::Changed);
        }
        if k + 1 < path.len() {
            // Un livello intermedio: deve essere un sottomenu.
            if mii.hSubMenu.is_invalid() {
                return Err(InvokeErr::Changed);
            }
            menu = mii.hSubMenu;
            continue;
        }
        let text = String::from_utf16_lossy(&buf[..(mii.cch as usize).min(buf.len() - 1)]);
        if mii.fType.0 & MFT_SEPARATOR.0 != 0 || !mii.hSubMenu.is_invalid() || mii.fState.0 & (MFS_GRAYED.0 | MFS_DISABLED.0) != 0 {
            return Err(InvokeErr::NotInvokable);
        }
        if !titles_match(expect_title, &text) {
            return Err(InvokeErr::Changed);
        }
        let Some(wp) = wm_command_wparam(mii.wID) else { return Err(InvokeErr::NotInvokable) };
        return unsafe { PostMessageW(Some(hwnd), WM_COMMAND, WPARAM(wp), LPARAM(0)) }.map_err(|_| InvokeErr::Failed);
    }
    Err(InvokeErr::Changed)
}

/// Esegue la voce `j` del menu `source` di una finestra con barra UI Automation: si apre il menu un istante,
/// si ritrova la voce (stesso titolo di quello mostrato) e la si invoca; il menu si chiude da solo.
fn invoke_uia(hwnd_id: usize, source: usize, j: usize, expect_title: &str) -> Result<(), InvokeErr> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let out = (|| -> windows::core::Result<Result<(), InvokeErr>> {
            let auto: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            let el = auto.ElementFromHandle(HWND(hwnd_id as *mut _))?;
            let Some(bar) = best_bar(&auto, &el)? else { return Ok(Err(InvokeErr::Gone)) };
            let item = auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_MenuItemControlTypeId.0))?;
            let tops = bar.FindAll(TreeScope_Children, &item)?;
            let Ok(top) = tops.GetElement(source as i32) else { return Ok(Err(InvokeErr::Changed)) };
            let expand: windows::core::Result<IUIAutomationExpandCollapsePattern> = top.GetCurrentPatternAs(UIA_ExpandCollapsePatternId);
            if let Ok(p) = &expand {
                let _ = p.Expand();
                std::thread::sleep(Duration::from_millis(120));
            }
            let kids = top.FindAll(TreeScope_Descendants, &item)?;
            let Ok(k) = kids.GetElement(j as i32) else { return Ok(Err(InvokeErr::Changed)) };
            let name = k.CurrentName().map(|s| s.to_string()).unwrap_or_default();
            let result = if !titles_match(expect_title, &name) {
                Err(InvokeErr::Changed)
            } else if !k.CurrentIsEnabled().map(|b| b.as_bool()).unwrap_or(false) {
                Err(InvokeErr::NotInvokable)
            } else {
                let inv: windows::core::Result<IUIAutomationInvokePattern> = k.GetCurrentPatternAs(UIA_InvokePatternId);
                match inv {
                    Ok(p) => p.Invoke().map_err(|_| InvokeErr::Failed),
                    Err(_) => Err(InvokeErr::NotInvokable),
                }
            };
            if result.is_err()
                && let Ok(p) = &expand
            {
                let _ = p.Collapse();
            }
            Ok(result)
        })();
        if init {
            CoUninitialize();
        }
        out.unwrap_or(Err(InvokeErr::Failed))
    }
}

/// Esegue la voce `path` (percorso nel menu originale dell'app: primo indice = menu di primo livello) della
/// finestra `hwnd_id`, solo se la voce è ancora quella mostrata (`expect_title`). `HMENU` classico se la
/// finestra ne ha uno, altrimenti UI Automation (con tempo massimo).
pub fn invoke_item(hwnd_id: usize, path: &[usize], expect_title: &str) -> Result<(), InvokeErr> {
    let hwnd = HWND(hwnd_id as *mut _);
    if path.len() < 2 || !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        return Err(InvokeErr::Gone);
    }
    if !unsafe { GetMenu(hwnd) }.is_invalid() {
        return invoke_hmenu(hwnd, path, expect_title);
    }
    if path.len() != 2 {
        return Err(InvokeErr::Changed); // di UI Automation si leggono solo le voci del primo livello
    }
    let (source, j) = (path[0], path[1]);
    let title = expect_title.to_string();
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(invoke_uia(hwnd_id, source, j, &title));
    });
    rx.recv_timeout(LAZY_TIMEOUT).unwrap_or(Err(InvokeErr::Failed))
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
    fn abandoned_reads_are_capped() {
        assert!(may_start_read(0) && may_start_read(MAX_INFLIGHT - 1));
        assert!(!may_start_read(MAX_INFLIGHT));
    }

    #[test]
    fn a_shown_title_must_still_be_the_one_in_the_menu() {
        assert!(titles_match("Apri", "&Apri"));
        assert!(titles_match("Apri...", "&Apri...\tCtrl+O"));
        assert!(titles_match(" Apri ", "Apri"));
        assert!(!titles_match("Apri", "Aprire"), "il menu e' cambiato: non si indovina");
        assert!(!titles_match("Apri", ""));
    }

    #[test]
    fn menu_commands_need_a_usable_id() {
        assert_eq!(wm_command_wparam(1), Some(1));
        assert_eq!(wm_command_wparam(0xFFFF), Some(0xFFFF));
        assert_eq!(wm_command_wparam(0), None, "voce senza comando");
        assert_eq!(wm_command_wparam(0x1_0000), None, "non sta in 16 bit");
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
