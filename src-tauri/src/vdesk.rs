//! Desktop virtuali, backend `Documented` (Mission Control, fase M4): usa solo ciò che è stabile.
//!
//! - **Elenco e desktop corrente**: dal registro (`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\
//!   VirtualDesktops`: `VirtualDesktopIDs` = GUID da 16 byte uno dopo l'altro, `CurrentVirtualDesktop` = il
//!   GUID corrente; i nomi scelti dall'utente stanno in `...\Desktops\{GUID}`, valore `Name`). Formato non
//!   documentato ma stabile da anni e in sola lettura: se manca (Windows con un solo desktop mai
//!   toccato) c'è un unico desktop.
//! - **Desktop di una finestra**: `IVirtualDesktopManager::GetWindowDesktopId`, l'interfaccia documentata.
//! - **Passare a un desktop e crearne uno**: si simulano le scorciatoie di Windows (Win+Ctrl+Freccia,
//!   Win+Ctrl+D) con `SendInput`: nessuna interfaccia non documentata.
//!
//! Spostare finestre fra desktop, rimuovere e rinominare richiedono le interfacce non documentate e sono
//! in M5, con profili per build e degradazione. Questo modulo è il ripiego sempre disponibile.
//!
//! Le parti pure (lettura dei GUID, costruzione dell'elenco, piano dei tasti da premere) sono testate;
//! il registro e `SendInput` sono sottili.

use std::collections::HashMap;

use serde::Serialize;
use windows::core::HSTRING;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYBD_EVENT_FLAGS,
    VIRTUAL_KEY, VK_CONTROL, VK_D, VK_LEFT, VK_LWIN, VK_RIGHT,
};

const ROOT: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops";

/// Un desktop virtuale come lo vede la panoramica.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Desktop {
    /// GUID del desktop, in maiuscolo e fra graffe.
    pub id: String,
    pub name: String,
    pub current: bool,
}

// ------------------------------------------------------------------ pure

/// Un GUID da 16 byte come li scrive Windows (`Data1` a 32 bit, `Data2` e `Data3` a 16 bit, poi 8 byte,
/// tutti little endian) → "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}". `None` se non sono 16 byte. Pura, testata.
pub fn guid_from_bytes(b: &[u8]) -> Option<String> {
    let b: &[u8; 16] = b.try_into().ok()?;
    let d1 = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    let d2 = u16::from_le_bytes([b[4], b[5]]);
    let d3 = u16::from_le_bytes([b[6], b[7]]);
    Some(format!(
        "{{{d1:08X}-{d2:04X}-{d3:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    ))
}

/// L'elenco dei GUID da `VirtualDesktopIDs` (blocchi da 16 byte; un resto incompleto si ignora). Pura, testata.
pub fn parse_ids(blob: &[u8]) -> Vec<String> {
    blob.as_chunks::<16>().0.iter().filter_map(|c| guid_from_bytes(c)).collect()
}

/// Testo UTF-16 little endian (con o senza NUL finale) → stringa. Pura, testata.
pub fn utf16_to_string(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).take_while(|&u| u != 0).collect();
    String::from_utf16_lossy(&units)
}

/// Costruisce l'elenco: nome scelto dall'utente se c'è, altrimenti "Desktop N". Se `current` non è tra gli
/// id (registro incoerente) o manca, il corrente è il primo: meglio un desktop evidenziato che nessuno.
/// Senza id c'è un solo desktop. Pura, testata.
pub fn desktops_from(ids: &[String], current: Option<&str>, names: &HashMap<String, String>) -> Vec<Desktop> {
    if ids.is_empty() {
        return vec![Desktop { id: String::new(), name: "Desktop 1".into(), current: true }];
    }
    let cur = current.and_then(|c| ids.iter().position(|i| i.eq_ignore_ascii_case(c))).unwrap_or(0);
    ids.iter()
        .enumerate()
        .map(|(i, id)| Desktop {
            id: id.clone(),
            name: names.get(id).map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| format!("Desktop {}", i + 1)),
            current: i == cur,
        })
        .collect()
}

/// Verso di un passaggio di desktop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Left,
    Right,
}

/// Quante volte e in che verso premere Win+Ctrl+Freccia per andare dal desktop `from` al desktop `to`
/// (indici nell'elenco di `count` desktop). `None` se sono uguali o fuori elenco. Windows non gira in
/// tondo: da 1 a 3 sono due passi a destra. Pura, testata.
pub fn plan_switch(from: usize, to: usize, count: usize) -> Option<(Dir, usize)> {
    if from >= count || to >= count || from == to {
        return None;
    }
    Some(if to > from { (Dir::Right, to - from) } else { (Dir::Left, from - to) })
}

/// L'indice del desktop corrente nell'elenco. Pura, testata.
pub fn current_index(list: &[Desktop]) -> usize {
    list.iter().position(|d| d.current).unwrap_or(0)
}

// ------------------------------------------------------------------ registro (sola lettura)

/// Legge un valore del registro (HKCU) come byte. `None` se la chiave o il valore non esistono.
fn read_value(path: &str, name: &str) -> Option<Vec<u8>> {
    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyExW(HKEY_CURRENT_USER, &HSTRING::from(path), None, KEY_READ, &mut key).ok().ok()?;
        let name = HSTRING::from(name);
        let mut ty = REG_VALUE_TYPE(0);
        let mut len = 0u32;
        let sized = RegQueryValueExW(key, &name, None, Some(&mut ty), None, Some(&mut len)).ok().is_ok();
        let out = if sized && len > 0 && len < 1 << 20 {
            let mut buf = vec![0u8; len as usize];
            RegQueryValueExW(key, &name, None, Some(&mut ty), Some(buf.as_mut_ptr()), Some(&mut len)).ok().ok().map(|_| {
                buf.truncate(len as usize);
                buf
            })
        } else {
            None
        };
        let _ = RegCloseKey(key);
        out
    }
}

/// L'elenco dei desktop virtuali e quale è il corrente, letto dal registro. Sempre almeno un desktop.
pub fn list() -> Vec<Desktop> {
    let ids = read_value(ROOT, "VirtualDesktopIDs").map(|b| parse_ids(&b)).unwrap_or_default();
    let current = read_value(ROOT, "CurrentVirtualDesktop").and_then(|b| guid_from_bytes(&b));
    let mut names = HashMap::new();
    for id in &ids {
        if let Some(n) = read_value(&format!(r"{ROOT}\Desktops\{id}"), "Name") {
            names.insert(id.clone(), utf16_to_string(&n));
        }
    }
    desktops_from(&ids, current.as_deref(), &names)
}

// ------------------------------------------------------------------ desktop di una finestra (documentato)

/// Il desktop a cui appartiene una finestra, con `IVirtualDesktopManager` (documentata). `None` se la
/// finestra non esiste o l'interfaccia non risponde. Chi chiama deve avere COM inizializzato. La usa M5
/// (su quale desktop sta ogni finestra); per ora la verifica un test sul sistema vero.
#[cfg_attr(not(test), allow(dead_code))]
pub fn window_desktop(hwnd: HWND) -> Option<String> {
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
    use windows::Win32::UI::Shell::{IVirtualDesktopManager, VirtualDesktopManager};
    unsafe {
        let mgr: IVirtualDesktopManager = CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_ALL).ok()?;
        let g = mgr.GetWindowDesktopId(hwnd).ok()?;
        guid_from_bytes(&[
            &g.data1.to_le_bytes()[..],
            &g.data2.to_le_bytes()[..],
            &g.data3.to_le_bytes()[..],
            &g.data4[..],
        ]
        .concat())
    }
}

// ------------------------------------------------------------------ tasti (SendInput)

fn key_event(vk: VIRTUAL_KEY, up: bool, extended: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: vk, dwFlags: flags, ..Default::default() } } }
}

/// Preme `Win+Ctrl+<tasto>` e lo rilascia (la freccia è un tasto "esteso"). Restituisce se Windows ha
/// accettato tutti gli eventi.
fn win_ctrl(vk: VIRTUAL_KEY, extended: bool) -> bool {
    let seq = [
        key_event(VK_LWIN, false, false),
        key_event(VK_CONTROL, false, false),
        key_event(vk, false, extended),
        key_event(vk, true, extended),
        key_event(VK_CONTROL, true, false),
        key_event(VK_LWIN, true, false),
    ];
    unsafe { SendInput(&seq, std::mem::size_of::<INPUT>() as i32) == seq.len() as u32 }
}

/// Passa al desktop `to` (indice nell'elenco attuale) con Win+Ctrl+Freccia, un passo alla volta (Windows
/// ha un'animazione per ogni passo). `false` se l'indice non è valido o non c'è nulla da fare.
pub fn switch_to(to: usize) -> bool {
    let list = list();
    let Some((dir, n)) = plan_switch(current_index(&list), to, list.len()) else { return false };
    let mut ok = true;
    for _ in 0..n {
        ok &= match dir {
            Dir::Right => win_ctrl(VK_RIGHT, true),
            Dir::Left => win_ctrl(VK_LEFT, true),
        };
        std::thread::sleep(std::time::Duration::from_millis(220));
    }
    ok
}

/// Crea un nuovo desktop con Win+Ctrl+D (Windows passa subito al nuovo).
pub fn create() -> bool {
    win_ctrl(VK_D, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Il GUID {00112233-4455-6677-8899-AABBCCDDEEFF} come lo scrive Windows (primi tre gruppi little endian).
    const BYTES: [u8; 16] = [0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

    fn id(n: u8) -> String {
        let mut b = BYTES;
        b[0] = n;
        guid_from_bytes(&b).unwrap()
    }

    #[test]
    fn guids_are_read_in_windows_byte_order() {
        assert_eq!(guid_from_bytes(&BYTES).unwrap(), "{00112233-4455-6677-8899-AABBCCDDEEFF}");
        assert_eq!(guid_from_bytes(&[0u8; 16]).unwrap(), "{00000000-0000-0000-0000-000000000000}");
        assert_eq!(guid_from_bytes(&[0xFF; 16]).unwrap(), "{FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF}");
        assert!(guid_from_bytes(&BYTES[..15]).is_none());
        assert!(guid_from_bytes(&[]).is_none());
        assert!(guid_from_bytes(&[0u8; 17]).is_none());
    }

    #[test]
    fn the_id_blob_is_split_in_sixteen_byte_blocks() {
        let mut blob = Vec::new();
        for n in [1u8, 2, 3] {
            let mut b = BYTES;
            b[0] = n;
            blob.extend_from_slice(&b);
        }
        assert_eq!(parse_ids(&blob), vec![id(1), id(2), id(3)]);
        blob.extend_from_slice(&[9, 9, 9]); // resto incompleto: ignorato
        assert_eq!(parse_ids(&blob).len(), 3);
        assert!(parse_ids(&[]).is_empty());
        assert!(parse_ids(&[1, 2, 3]).is_empty());
    }

    #[test]
    fn utf16_names_stop_at_the_terminator() {
        let utf16 = |s: &str, nul: bool| -> Vec<u8> {
            let mut v: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
            if nul {
                v.extend_from_slice(&[0, 0]);
            }
            v
        };
        assert_eq!(utf16_to_string(&utf16("Lavoro", true)), "Lavoro");
        assert_eq!(utf16_to_string(&utf16("Casa è bella", false)), "Casa è bella");
        assert_eq!(utf16_to_string(&[0, 0, b'x', 0]), "", "si ferma al primo NUL");
        assert_eq!(utf16_to_string(&[]), "");
        assert_eq!(utf16_to_string(b"a"), "", "un byte spaiato non basta");
    }

    #[test]
    fn the_list_names_desktops_and_marks_the_current_one() {
        let ids = vec![id(1), id(2), id(3)];
        let mut names = HashMap::new();
        names.insert(id(2), "  Lavoro ".to_string());
        names.insert(id(3), "   ".to_string()); // solo spazi: si usa il nome predefinito
        let l = desktops_from(&ids, Some(&id(2).to_lowercase()), &names);
        assert_eq!(l.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(), ["Desktop 1", "Lavoro", "Desktop 3"]);
        assert_eq!(l.iter().map(|d| d.current).collect::<Vec<_>>(), [false, true, false], "maiuscole ignorate");
        assert_eq!(current_index(&l), 1);
    }

    #[test]
    fn an_unknown_or_missing_current_desktop_falls_back_to_the_first() {
        let ids = vec![id(1), id(2)];
        for cur in [None, Some("{non-un-guid}"), Some(id(9).as_str())] {
            let l = desktops_from(&ids, cur, &HashMap::new());
            assert_eq!(l.iter().filter(|d| d.current).count(), 1, "sempre uno solo evidenziato");
            assert!(l[0].current);
        }
    }

    #[test]
    fn without_registry_data_there_is_a_single_desktop() {
        let l = desktops_from(&[], None, &HashMap::new());
        assert_eq!(l.len(), 1);
        assert!(l[0].current && l[0].name == "Desktop 1");
        assert_eq!(current_index(&l), 0);
        assert_eq!(current_index(&[]), 0);
    }

    #[test]
    fn switching_plans_the_right_number_of_steps() {
        assert_eq!(plan_switch(0, 2, 3), Some((Dir::Right, 2)));
        assert_eq!(plan_switch(2, 0, 3), Some((Dir::Left, 2)));
        assert_eq!(plan_switch(1, 2, 3), Some((Dir::Right, 1)));
        assert_eq!(plan_switch(1, 1, 3), None, "gia' li'");
        assert_eq!(plan_switch(0, 3, 3), None, "fuori elenco");
        assert_eq!(plan_switch(5, 0, 3), None, "partenza fuori elenco");
        assert_eq!(plan_switch(0, 0, 0), None);
    }

    #[test]
    fn the_real_registry_gives_a_consistent_list() {
        let l = list();
        println!("desktop virtuali: {}", l.len());
        assert!(!l.is_empty());
        assert_eq!(l.iter().filter(|d| d.current).count(), 1);
        assert!(l.iter().all(|d| !d.name.is_empty()));
        let mut ids: Vec<&String> = l.iter().map(|d| &d.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), l.len(), "id univoci");
    }

    /// Sul sistema vero, in sola lettura: ogni finestra del desktop corrente deve risultare, per
    /// l'interfaccia documentata, sul desktop che il registro dice corrente.
    #[test]
    fn windows_of_the_current_desktop_agree_with_the_registry() {
        let cur = list().into_iter().find(|d| d.current).map(|d| d.id).unwrap_or_default();
        let (n, ids) = std::thread::spawn(|| {
            let _com = crate::dock::ComScope::new();
            let ws: Vec<usize> = crate::dock::raw_app_windows().into_iter().map(|(h, _, _)| h.0 as usize).collect();
            let ids: Vec<Option<String>> = ws.iter().map(|&h| window_desktop(HWND(h as *mut _))).collect();
            (ws.len(), ids)
        })
        .join()
        .unwrap();
        println!("finestre controllate: {n}");
        if cur.is_empty() {
            return; // un solo desktop mai toccato: nessun GUID nel registro, niente da confrontare
        }
        for id in ids.into_iter().flatten() {
            assert_eq!(id, cur, "una finestra del desktop corrente risulta su un altro desktop");
        }
    }

    #[test]
    fn key_events_are_well_formed() {
        let down = key_event(VK_RIGHT, false, true);
        let up = key_event(VK_RIGHT, true, true);
        unsafe {
            assert_eq!(down.Anonymous.ki.wVk, VK_RIGHT);
            assert!(down.Anonymous.ki.dwFlags.contains(KEYEVENTF_EXTENDEDKEY) && !down.Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP));
            assert!(up.Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP));
            assert!(!key_event(VK_D, false, false).Anonymous.ki.dwFlags.contains(KEYEVENTF_EXTENDEDKEY));
        }
    }
}
