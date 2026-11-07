//! Dati del Dock (fase D1): app fissate, app aperte e icone.
//!
//! - Fissate: collegamenti `.lnk` della taskbar di Windows.
//! - Aperte: finestre di primo livello visibili, raggruppate per eseguibile.
//! - Icone: estratte dall'eseguibile e codificate in PNG (data URL).
//!
//! Il Dock non nasconde né modifica nulla del sistema: un errore qui non tocca
//! mai la logica di ripristino della taskbar.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use base64::Engine;
use serde::Serialize;
use windows::core::{Interface, BOOL, HSTRING};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, SIZE};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, ShellLink,
    SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindow, GetWindowLongW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE, GW_OWNER, WS_EX_TOOLWINDOW,
};

use crate::foreground::{cached_description, display_name};
use crate::util::image_path;

/// Lato (px) dell'icona richiesta: abbastanza grande per l'ingrandimento del Dock.
const ICON_SIZE: i32 = 128;

/// Una voce del Dock, come la vede il frontend.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DockItem {
    /// Percorso dell'eseguibile in minuscolo: identifica l'app.
    pub id: String,
    pub name: String,
    pub path: String,
    pub pinned: bool,
    pub running: bool,
    /// `data:image/png;base64,...`, se l'icona è stata estratta.
    pub icon: Option<String>,
}

/// App fissata (dati grezzi).
#[derive(Clone, Debug, PartialEq)]
pub struct Pinned {
    pub name: String,
    pub path: PathBuf,
}

/// App con almeno una finestra aperta (dati grezzi).
#[derive(Clone, Debug, PartialEq)]
pub struct Running {
    pub path: PathBuf,
}

fn key(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
}

/// Stessa app? Stesso percorso, oppure stesso nome di file con una cartella che
/// contiene l'altra: le app che si aggiornano in una sottocartella versionata
/// (Discord: `Discord\Discord.exe` fissato, `Discord\app-1.0\Discord.exe` in
/// esecuzione) devono comparire una volta sola. Pura, testata.
pub fn same_app(a: &Path, b: &Path) -> bool {
    if key(a) == key(b) {
        return true;
    }
    let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().to_lowercase());
    if name(a).is_none() || name(a) != name(b) {
        return false;
    }
    let dir = |p: &Path| PathBuf::from(key(p.parent().unwrap_or(p)));
    let (da, db) = (dir(a), dir(b));
    da.starts_with(&db) || db.starts_with(&da)
}

/// Unisce fissate e aperte: prima le fissate (nell'ordine dato), poi le aperte
/// non fissate (nell'ordine di scoperta). Una stessa app compare una sola volta.
/// Funzione pura, testata. Le icone si aggiungono dopo.
pub fn merge(pinned: &[Pinned], running: &[Running]) -> Vec<DockItem> {
    let mut items: Vec<DockItem> = Vec::new();
    for p in pinned {
        if items.iter().any(|i| same_app(Path::new(&i.path), &p.path)) {
            continue;
        }
        items.push(DockItem {
            running: running.iter().any(|r| same_app(&r.path, &p.path)),
            id: key(&p.path),
            name: p.name.clone(),
            path: p.path.to_string_lossy().to_string(),
            pinned: true,
            icon: None,
        });
    }
    for r in running {
        if items.iter().any(|i| same_app(Path::new(&i.path), &r.path)) {
            continue;
        }
        items.push(DockItem {
            id: key(&r.path),
            name: String::new(), // il nome si risolve dopo, dalla descrizione del file
            path: r.path.to_string_lossy().to_string(),
            pinned: false,
            running: true,
            icon: None,
        });
    }
    items
}

// ---------------------------------------------------------------- COM

/// Inizializza COM sul thread corrente per la durata dello scope.
struct ComScope(bool);
impl ComScope {
    fn new() -> Self {
        // S_OK / S_FALSE vanno bilanciati con CoUninitialize; RPC_E_CHANGED_MODE no.
        Self(unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok())
    }
}
impl Drop for ComScope {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

// ----------------------------------------------------------- fissate

/// Cartella dei collegamenti fissati sulla taskbar.
fn pinned_dir() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(
        PathBuf::from(appdata)
            .join(r"Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar"),
    )
}

/// Percorso di destinazione di un `.lnk`. Le app dello Store non hanno un percorso
/// (puntano a un AppUserModelID): restituiscono `None` e per ora sono saltate.
fn resolve_lnk(lnk: &Path) -> Option<PathBuf> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let file: IPersistFile = link.cast().ok()?;
        file.Load(&HSTRING::from(lnk.to_string_lossy().as_ref()), STGM_READ).ok()?;
        let mut buf = [0u16; 1024];
        let mut fd = WIN32_FIND_DATAW::default();
        link.GetPath(&mut buf, &mut fd, 0).ok()?;
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if len == 0 {
            return None;
        }
        let path = PathBuf::from(String::from_utf16_lossy(&buf[..len]));
        path.is_file().then_some(path)
    }
}

/// App fissate sulla taskbar, ordinate per nome (l'ordine reale di Windows è nel
/// registro `Taskband`, non ancora letto).
pub fn read_pinned() -> Vec<Pinned> {
    let Some(dir) = pinned_dir() else { return Vec::new() };
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let _com = ComScope::new();
    let mut out: Vec<Pinned> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")))
        .filter_map(|lnk| {
            let path = resolve_lnk(&lnk)?;
            let name = lnk.file_stem()?.to_string_lossy().to_string();
            Some(Pinned { name, path })
        })
        .collect();
    out.sort_by_key(|p| p.name.to_lowercase());
    out
}

// ------------------------------------------------------------- aperte

/// Finestre che contano come "app aperta": visibili, senza proprietario, non di
/// tipo strumento, con titolo e non nascoste (cloaked, es. desktop virtuali).
fn is_app_window(hwnd: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }
        if GetWindow(hwnd, GW_OWNER).is_ok_and(|o| !o.0.is_null()) {
            return false;
        }
        if GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0 {
            return false;
        }
        let mut cloaked = 0u32;
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        )
        .is_ok()
            && cloaked != 0
        {
            return false;
        }
        let mut title = [0u16; 8];
        if GetWindowTextW(hwnd, &mut title) == 0 {
            return false;
        }
        let mut class = [0u16; 64];
        let n = GetClassNameW(hwnd, &mut class);
        let class = String::from_utf16_lossy(&class[..n.max(0) as usize]);
        !matches!(class.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd")
    }
}

unsafe extern "system" fn collect_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // Un panic non deve attraversare il confine FFI: lo si assorbe.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let out = unsafe { &mut *(lparam.0 as *mut Vec<Running>) };
        if !is_app_window(hwnd) {
            return;
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid == 0 || pid == std::process::id() {
            return; // mai mostrare EosOS stessa
        }
        let Ok(h) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
            return;
        };
        let path = image_path(h);
        unsafe {
            let _ = CloseHandle(h);
        }
        let Some(path) = path else { return };
        // Le app dello Store girano dentro ApplicationFrameHost: finché non si
        // risolvono i pacchetti, non vanno raggruppate sotto un'unica icona.
        if stem(&path).eq_ignore_ascii_case("ApplicationFrameHost") {
            return;
        }
        if !out.iter().any(|r| key(&r.path) == key(&path)) {
            out.push(Running { path });
        }
    }));
    true.into()
}

/// Eseguibili con almeno una finestra aperta, nell'ordine Z (dall'alto).
pub fn read_running() -> Vec<Running> {
    let mut out: Vec<Running> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect_window), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

// -------------------------------------------------------------- icone

/// Converte un HBITMAP a 32 bit in pixel RGBA non premoltiplicati.
fn bitmap_to_rgba(hbm: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
    unsafe {
        let mut bm = BITMAP::default();
        if GetObjectW(
            HGDIOBJ(hbm.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bm as *mut _ as *mut _),
        ) == 0
        {
            return None;
        }
        let (w, h) = (bm.bmWidth, bm.bmHeight);
        if w <= 0 || h <= 0 {
            return None;
        }
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h, // negativo: righe dall'alto verso il basso
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut px = vec![0u8; (w * h * 4) as usize];
        let dc = GetDC(None);
        let lines = GetDIBits(dc, hbm, 0, h as u32, Some(px.as_mut_ptr() as *mut _), &mut info, DIB_RGB_COLORS);
        ReleaseDC(None, dc);
        if lines == 0 {
            return None;
        }
        Some((w as u32, h as u32, bgra_to_rgba(px)))
    }
}

/// BGRA (eventualmente premoltiplicato) → RGBA non premoltiplicato. Se nessun
/// pixel ha alfa, l'immagine non ha trasparenza: la si rende opaca. Pura, testata.
pub fn bgra_to_rgba(mut px: Vec<u8>) -> Vec<u8> {
    let has_alpha = px.as_chunks::<4>().0.iter().any(|c| c[3] != 0);
    for c in px.as_chunks_mut::<4>().0 {
        let (b, g, r, a) = (c[0], c[1], c[2], if has_alpha { c[3] } else { 255 });
        let un = |v: u8| -> u8 {
            if a == 0 || a == 255 { v } else { ((v as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8 }
        };
        c[0] = un(r);
        c[1] = un(g);
        c[2] = un(b);
        c[3] = a;
    }
    px
}

fn encode_png(w: u32, h: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().ok()?.write_image_data(rgba).ok()?;
    }
    Some(out)
}

/// Icona dell'eseguibile come data URL PNG.
pub fn icon_data_url(path: &Path) -> Option<String> {
    let _com = ComScope::new();
    unsafe {
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(&HSTRING::from(path.to_string_lossy().as_ref()), None).ok()?;
        let hbm = factory
            .GetImage(SIZE { cx: ICON_SIZE, cy: ICON_SIZE }, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK)
            .ok()?;
        let rgba = bitmap_to_rgba(hbm);
        let _ = DeleteObject(HGDIOBJ(hbm.0));
        let (w, h, px) = rgba?;
        let png = encode_png(w, h, &px)?;
        Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png)
        ))
    }
}

static ICONS: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

fn cached_icon(path: &Path) -> Option<String> {
    let cache = ICONS.get_or_init(Default::default);
    let k = key(path);
    if let Some(v) = cache.lock().ok()?.get(&k) {
        return v.clone();
    }
    let icon = icon_data_url(path);
    cache.lock().ok()?.insert(k, icon.clone());
    icon
}

// ------------------------------------------------------------- insieme

/// Elenco completo del Dock: fissate + aperte, con nomi e icone.
pub fn items() -> Vec<DockItem> {
    let mut items = merge(&read_pinned(), &read_running());
    for it in &mut items {
        let path = PathBuf::from(&it.path);
        if it.name.is_empty() {
            let desc = cached_description(&path);
            it.name = display_name(desc.as_deref(), &stem(&path), "", "");
        }
        it.icon = cached_icon(&path);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(name: &str, path: &str) -> Pinned {
        Pinned { name: name.into(), path: PathBuf::from(path) }
    }
    fn run(path: &str) -> Running {
        Running { path: PathBuf::from(path) }
    }

    #[test]
    fn merge_puts_pinned_first_then_unpinned_running() {
        let items = merge(
            &[pin("Edge", r"C:\A\edge.exe"), pin("Note", r"C:\A\note.exe")],
            &[run(r"C:\A\code.exe"), run(r"c:\a\NOTE.exe")],
        );
        let ids: Vec<_> = items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, [r"c:\a\edge.exe", r"c:\a\note.exe", r"c:\a\code.exe"]);
        assert!(items[0].pinned && !items[0].running);
        assert!(items[1].pinned && items[1].running, "il confronto ignora le maiuscole");
        assert!(!items[2].pinned && items[2].running);
    }

    #[test]
    fn versioned_subfolder_matches_the_pinned_launcher() {
        let items = merge(
            &[pin("Discord", r"C:\U\Local\Discord\Discord.exe")],
            &[run(r"C:\U\Local\Discord\app-1.0.9267\Discord.exe")],
        );
        assert_eq!(items.len(), 1);
        assert!(items[0].pinned && items[0].running);
        // stesso nome ma cartelle non annidate: sono app diverse
        assert!(!same_app(Path::new(r"C:\A\FiveM\FiveM.exe"), Path::new(r"C:\B\FiveM.exe")));
        assert!(!same_app(Path::new(r"C:\A\a.exe"), Path::new(r"C:\A\b.exe")));
    }

    #[test]
    fn merge_has_no_duplicates() {
        let items = merge(
            &[pin("A", r"C:\x\a.exe"), pin("A2", r"C:\X\A.exe")],
            &[run(r"C:\x\a.exe"), run(r"C:\x\a.exe")],
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "A");
    }

    #[test]
    fn bgra_is_swapped_and_unpremultiplied() {
        // pixel opaco blu (BGRA) -> RGBA rosso/verde/blu corretti
        assert_eq!(bgra_to_rgba(vec![255, 0, 0, 255]), vec![0, 0, 255, 255]);
        // premoltiplicato a metà alfa: 100 -> ~200
        let out = bgra_to_rgba(vec![0, 0, 100, 128, 0, 0, 0, 255]);
        assert_eq!(out[3], 128);
        assert!((out[0] as i32 - 199).abs() <= 1);
        // nessun alfa nell'immagine: resa opaca
        assert_eq!(bgra_to_rgba(vec![1, 2, 3, 0]), vec![3, 2, 1, 255]);
    }

    #[test]
    fn icon_of_notepad_is_a_valid_png() {
        let path = Path::new(r"C:\Windows\System32\notepad.exe");
        let url = icon_data_url(path).expect("icona di notepad");
        let b64 = url.strip_prefix("data:image/png;base64,").expect("data URL PNG");
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let dec = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut reader = dec.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert!(info.width >= 32 && info.width == info.height);
        assert!(buf.as_chunks::<4>().0.iter().any(|p| p[3] > 0), "almeno un pixel visibile");
    }

    #[test]
    fn live_items_do_not_crash_and_have_names() {
        let items = items();
        for it in &items {
            assert!(!it.name.is_empty(), "nome vuoto per {}", it.path);
        }
    }
    /// Aiuto per lo sviluppo: `cargo test dump_dock -- --ignored --nocapture`
    /// stampa le voci reali e salva le icone in %TEMP%\eosos-dock.
    #[test]
    #[ignore]
    fn dump_dock() {
        let dir = std::env::temp_dir().join("eosos-dock");
        let _ = std::fs::create_dir_all(&dir);
        for it in items() {
            println!("{:<28} fissata={:<5} aperta={:<5} {}", it.name, it.pinned, it.running, it.path);
            if let Some(b64) = it.icon.as_deref().and_then(|u| u.strip_prefix("data:image/png;base64,")) {
                let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
                let _ = std::fs::write(dir.join(format!("{}.png", it.name.replace(['/', '\\', ':'], "_"))), bytes);
            } else {
                println!("   (nessuna icona)");
            }
        }
        println!("icone in {}", dir.display());
    }
}
