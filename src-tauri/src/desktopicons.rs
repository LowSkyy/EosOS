//! Le icone del desktop restano dove le aveva messe l'utente.
//!
//! EosOS cambia l'area di lavoro dello schermo (barra in alto, barra di Windows nascosta): Windows allora risistema le icone del
//! desktop (le sposta, a volte ne cambia la griglia). All'avvio, PRIMA di toccare l'area di lavoro, si fotografano posizione di
//! ogni icona e loro dimensione; poi si rimettono come erano, sia dopo i cambi di EosOS sia alla chiusura. Tutto con le interfacce
//! documentate della shell (`IFolderView2` del desktop), senza toccare la memoria di Esplora file.
//!
//! La foto sta in `%LOCALAPPDATA%\EosOS\desktop-icons.json`: anche il processo di guardia la usa se EosOS si chiude di colpo.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use windows::core::{Interface, PCWSTR, PWSTR};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IServiceProvider, CLSCTX_ALL, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Shell::Common::{ITEMIDLIST, STRRET};
use windows::Win32::UI::Shell::{
    IEnumIDList, IFolderView2, IShellBrowser, IShellFolder, IShellWindows, ShellWindows, StrRetToStrW, FOLDERVIEWMODE, SHGDN_FORPARSING, SHGDN_INFOLDER, SID_STopLevelBrowser,
    SVGIO_ALLVIEW, SVSI_POSITIONITEM, SWC_DESKTOP, SWFO_NEEDDISPATCH,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Layout {
    pub mode: i32,
    pub size: i32,
    /// (nome dell'elemento nella cartella del desktop, x, y).
    pub items: Vec<(String, i32, i32)>,
}

fn path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\desktop-icons.json"))
}

/// La vista delle icone del desktop (quella di Esplora file).
fn desktop_view() -> Option<IFolderView2> {
    unsafe {
        let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL).ok()?;
        let loc = VARIANT::from(0i32); // CSIDL_DESKTOP
        let empty = VARIANT::default();
        let mut hwnd = 0i32;
        let disp = windows.FindWindowSW(&loc, &empty, SWC_DESKTOP, &mut hwnd, SWFO_NEEDDISPATCH).ok()?;
        let sp: IServiceProvider = disp.cast().ok()?;
        let browser: IShellBrowser = sp.QueryService(&SID_STopLevelBrowser).ok()?;
        browser.QueryActiveShellView().ok()?.cast().ok()
    }
}

/// Esegue `f` con COM inizializzato su questo thread.
fn with_com<T>(f: impl FnOnce() -> T) -> T {
    let inited = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok();
    let out = f();
    if inited {
        unsafe { CoUninitialize() };
    }
    out
}

fn name_of(folder: &IShellFolder, pidl: *const ITEMIDLIST) -> Option<String> {
    unsafe {
        let mut sr = STRRET::default();
        folder.GetDisplayNameOf(pidl, windows::Win32::UI::Shell::SHGDNF(SHGDN_INFOLDER.0 | SHGDN_FORPARSING.0), &mut sr).ok()?;
        let mut out = PWSTR::null();
        StrRetToStrW(&mut sr, Some(pidl), &mut out).ok()?;
        let s = out.to_string().ok();
        CoTaskMemFree(Some(out.0 as *const _));
        s
    }
}

/// La foto delle icone del desktop adesso.
pub fn snapshot() -> Option<Layout> {
    with_com(|| unsafe {
        let view = desktop_view()?;
        let folder: IShellFolder = view.GetFolder().ok()?;
        let mut mode = FOLDERVIEWMODE(0);
        let mut size = 0i32;
        view.GetViewModeAndIconSize(&mut mode, &mut size).ok()?;
        let list: IEnumIDList = view.Items(SVGIO_ALLVIEW).ok()?;
        let mut items = Vec::new();
        loop {
            let mut one = [std::ptr::null_mut::<ITEMIDLIST>(); 1];
            let mut got = 0u32;
            if list.Next(&mut one, Some(&mut got)).is_err() || got == 0 {
                break;
            }
            let pidl = one[0];
            if let (Some(name), Ok(pt)) = (name_of(&folder, pidl), view.GetItemPosition(pidl)) {
                items.push((name, pt.x, pt.y));
            }
            CoTaskMemFree(Some(pidl as *const _));
        }
        (!items.is_empty()).then_some(Layout { mode: mode.0, size, items })
    })
}

/// Rimette le icone come nella foto (dimensione e posizione). Le icone nuove restano dove sono. Risponde con quante ne ha messe.
pub fn apply(layout: &Layout) -> usize {
    with_com(|| unsafe {
        let Some(view) = desktop_view() else { return 0 };
        let Ok(folder) = view.GetFolder::<IShellFolder>() else { return 0 };
        let (mut mode, mut size) = (FOLDERVIEWMODE(0), 0i32);
        if view.GetViewModeAndIconSize(&mut mode, &mut size).is_ok() && (mode.0 != layout.mode || size != layout.size) && layout.size > 0 {
            let _ = view.SetViewModeAndIconSize(FOLDERVIEWMODE(layout.mode), layout.size);
        }
        let mut n = 0;
        for (name, x, y) in &layout.items {
            let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
            let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
            if folder.ParseDisplayName(HWND::default(), None, PCWSTR(wide.as_ptr()), None, &mut pidl, std::ptr::null_mut()).is_err() || pidl.is_null() {
                continue;
            }
            let now = view.GetItemPosition(pidl).ok();
            if now != Some(POINT { x: *x, y: *y }) {
                let list = [pidl as *const ITEMIDLIST];
                let pts = [POINT { x: *x, y: *y }];
                if view.SelectAndPositionItems(1, list.as_ptr(), Some(pts.as_ptr()), SVSI_POSITIONITEM.0 as u32).is_ok() {
                    n += 1;
                }
            }
            CoTaskMemFree(Some(pidl as *const _));
        }
        n
    })
}

fn marker() -> Option<PathBuf> {
    Some(path()?.with_extension("live"))
}

/// All'avvio di EosOS, prima di cambiare l'area di lavoro. Se la sessione precedente non si e' chiusa bene (resta il segno) la
/// disposizione attuale potrebbe essere gia' quella spostata: si tiene la foto vecchia e la si rimette.
pub fn begin() {
    let dirty = marker().is_some_and(|m| m.exists()) && saved().is_some();
    if dirty {
        restore_now();
    } else {
        remember();
    }
    if let Some(m) = marker() {
        let _ = std::fs::write(m, b"1");
    }
}

/// Alla chiusura (o dal processo di guardia), dopo aver rimesso l'area di lavoro: le icone tornano dove erano e il segno si toglie.
pub fn end() {
    if !marker().is_some_and(|m| m.exists()) {
        return;
    }
    std::thread::sleep(std::time::Duration::from_millis(800));
    restore_now();
    if let Some(m) = marker() {
        let _ = std::fs::remove_file(m);
    }
}

/// Si salva la disposizione di Windows.
pub fn remember() {
    if let (Some(l), Some(p)) = (snapshot(), path()) {
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Ok(s) = serde_json::to_string(&l) {
            let _ = std::fs::write(p, s);
        }
    }
}

fn saved() -> Option<Layout> {
    serde_json::from_str(&std::fs::read_to_string(path()?).ok()?).ok()
}

/// Rimette la disposizione salvata (in un thread, piu' volte: Windows risistema le icone qualche istante dopo il cambio dell'area).
pub fn restore_soon() {
    std::thread::spawn(|| {
        for ms in [600u64, 1500, 3500, 6000] {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            if let Some(l) = saved() {
                apply(&l);
            }
        }
    });
}

/// Rimette subito la disposizione salvata (chiusura di EosOS, processo di guardia).
pub fn restore_now() -> usize {
    saved().map_or(0, |l| apply(&l))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_survives_the_file() {
        let l = Layout { mode: 1, size: 48, items: vec![("Cestino".into(), 10, 20), ("C:\\Users\\x\\Desktop\\nota.txt".into(), 1900, 900)] };
        let s = serde_json::to_string(&l).unwrap();
        assert_eq!(serde_json::from_str::<Layout>(&s).unwrap(), l);
    }

    /// Prova sul desktop vero (non sposta nulla: rimette ogni icona dove gia' sta). `cargo test desktop_round -- --ignored`
    #[test]
    #[ignore]
    fn desktop_round_trip_is_a_no_op() {
        let l = snapshot().expect("vista del desktop");
        assert!(!l.items.is_empty());
        assert_eq!(apply(&l), 0, "nessuna icona deve muoversi");
    }
}
