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
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
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
    /// L'app è attiva ma senza finestre (es. ridotta nell'area di notifica): in
    /// macOS 27 ha un puntino grigio chiaro. Mai vero se `running` lo è.
    pub background: bool,
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

pub(crate) fn key(path: &Path) -> String {
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
            background: false,
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
            background: false,
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
        let out = unsafe { &mut *(lparam.0 as *mut Vec<(HWND, u32, PathBuf)>) };
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
        out.push((hwnd, pid, path));
    }));
    true.into()
}

/// Eseguibile del processo proprietario di una finestra.
pub fn window_exe(hwnd: HWND) -> Option<PathBuf> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }
    let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let path = image_path(h);
    unsafe {
        let _ = CloseHandle(h);
    }
    path
}

/// Tabella dei processi: (pid, pid del genitore, nome del file in minuscolo).
pub fn process_table() -> Vec<(u32, u32, String)> {
    let mut out = Vec::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok {
            let len = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(e.szExeFile.len());
            out.push((
                e.th32ProcessID,
                e.th32ParentProcessID,
                String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase(),
            ));
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    out
}

/// L'antenato più vicino di `pid` (al massimo 4 livelli) il cui eseguibile si
/// chiama come uno di `names`. Pura, testata.
///
/// Serve alle app con processi di supporto che possiedono le finestre: la
/// finestra di Steam è di `steamwebhelper.exe`, figlio di `Steam.exe`.
pub fn find_ancestor(table: &[(u32, u32, String)], pid: u32, names: &[String]) -> Option<u32> {
    let mut cur = pid;
    for _ in 0..4 {
        let (_, parent, _) = table.iter().find(|(p, _, _)| *p == cur)?;
        let (ppid, _, name) = table.iter().find(|(p, _, _)| p == parent)?;
        if names.contains(name) {
            return Some(*ppid);
        }
        cur = *ppid;
    }
    None
}

/// Tutte le finestre "da app" con il loro eseguibile, nell'ordine Z (dall'alto).
/// Se l'eseguibile di una finestra non è tra le app fissate ma un suo antenato
/// sì, la finestra è attribuita a quell'app (vedi `find_ancestor`).
pub fn app_windows() -> Vec<(HWND, PathBuf)> {
    let mut raw: Vec<(HWND, u32, PathBuf)> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect_window), LPARAM(&mut raw as *mut _ as isize));
    }
    let pinned: Vec<PathBuf> = crate::dockstore::load().into_iter().map(|p| p.path).collect();
    let names: Vec<String> = pinned
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_lowercase())
        .collect();
    // La tabella dei processi si costruisce solo se serve.
    let mut table: Option<Vec<(u32, u32, String)>> = None;
    raw.into_iter()
        .map(|(hwnd, pid, path)| {
            if pinned.iter().any(|p| same_app(p, &path)) {
                return (hwnd, path);
            }
            let t = table.get_or_insert_with(process_table);
            let owner = find_ancestor(t, pid, &names)
                .and_then(|apid| unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, apid).ok() })
                .and_then(|h| {
                    let p = image_path(h);
                    unsafe {
                        let _ = CloseHandle(h);
                    }
                    p
                })
                .filter(|p| pinned.iter().any(|x| same_app(x, p)));
            (hwnd, owner.unwrap_or(path))
        })
        .collect()
}

/// Eseguibili con almeno una finestra aperta, nell'ordine Z (dall'alto).
pub fn read_running() -> Vec<Running> {
    let mut out: Vec<Running> = Vec::new();
    for (_, path) in app_windows() {
        if !out.iter().any(|r| key(&r.path) == key(&path)) {
            out.push(Running { path });
        }
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

/// L'estrazione dalla shell non regge chiamate simultanee (fallisce a caso): si
/// serializza. Così un fallimento è genuino e si può ricordare nella cache.
static ICON_LOCK: Mutex<()> = Mutex::new(());

/// Icona dell'eseguibile come data URL PNG.
pub fn icon_data_url(path: &Path) -> Option<String> {
    let _serial = ICON_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

/// Segna come "in background" le app fissate senza finestre il cui eseguibile ha
/// un processo attivo (`background_paths`). Pura, testata.
pub fn mark_background(items: &mut [DockItem], background_paths: &[PathBuf]) {
    for it in items.iter_mut().filter(|i| !i.running) {
        it.background = background_paths.iter().any(|p| same_app(p, Path::new(&it.path)));
    }
}

/// Percorsi degli eseguibili con almeno un processo attivo, limitati ai file
/// chiamati come uno di `names` (minuscoli): così si apre un handle solo per i
/// pochi processi che interessano, non per tutti.
pub fn processes_named(names: &[String]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    if names.is_empty() {
        return out;
    }
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok {
            let len = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(e.szExeFile.len());
            let exe = String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase();
            if e.th32ProcessID != std::process::id()
                && names.contains(&exe)
                && let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, e.th32ProcessID)
            {
                if let Some(p) = image_path(h)
                    && !out.iter().any(|x| key(x) == key(&p))
                {
                    out.push(p);
                }
                let _ = CloseHandle(h);
            }
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    out
}

/// Mantiene l'ordine di comparsa delle app aperte: quelle già note restano dove
/// sono, le nuove si accodano nell'ordine dato, quelle chiuse spariscono. Senza
/// questo le icone si riordinerebbero a ogni cambio di finestra (l'ordine Z).
/// Pura, testata: `known` è lo stato da una chiamata all'altra.
pub fn stable_order(known: &mut Vec<String>, current: &[String]) -> Vec<String> {
    known.retain(|k| current.contains(k));
    for k in current {
        if !known.contains(k) {
            known.push(k.clone());
        }
    }
    known.clone()
}

static RUN_ORDER: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Riordina le voci non fissate secondo l'ordine di comparsa.
fn apply_run_order(items: &mut [DockItem]) {
    let first = items.iter().position(|i| !i.pinned).unwrap_or(items.len());
    let tail = &mut items[first..];
    let current: Vec<String> = tail.iter().map(|i| i.id.clone()).collect();
    let order = stable_order(&mut RUN_ORDER.lock().unwrap_or_else(|e| e.into_inner()), &current);
    tail.sort_by_key(|i| order.iter().position(|k| *k == i.id));
}

/// Voci senza icone, con i nomi risolti. Serve a ritrovare una voce dall'`id`.
pub fn items_basic() -> Vec<DockItem> {
    let mut items = merge(&crate::dockstore::load(), &read_running());
    for it in &mut items {
        if it.name.is_empty() {
            let path = PathBuf::from(&it.path);
            let desc = cached_description(&path);
            it.name = display_name(desc.as_deref(), &stem(&path), "", "");
        }
    }
    apply_run_order(&mut items);
    // Fissate senza finestre ma con un processo attivo: puntino grigio.
    let idle: Vec<String> = items
        .iter()
        .filter(|i| i.pinned && !i.running)
        .filter_map(|i| Path::new(&i.path).file_name())
        .map(|n| n.to_string_lossy().to_lowercase())
        .collect();
    mark_background(&mut items, &processes_named(&idle));
    items
}

/// Elenco completo del Dock: fissate + aperte, con nomi e icone.
pub fn items() -> Vec<DockItem> {
    let mut items = items_basic();
    for it in &mut items {
        it.icon = cached_icon(Path::new(&it.path));
    }
    items
}

/// La voce con questo `id`, se è davvero nel Dock in questo momento. Le azioni
/// partono sempre da qui: il frontend non può far eseguire percorsi arbitrari.
pub fn find(id: &str) -> Option<DockItem> {
    items_basic().into_iter().find(|i| i.id == id)
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
    fn background_apps_get_the_grey_dot_only_when_pinned_and_without_windows() {
        let mut items = merge(
            &[pin("Discord", "C:/U/Discord/Discord.exe"), pin("Steam", "C:/S/steam.exe"), pin("Chrome", "C:/C/chrome.exe")],
            &[run("C:/C/chrome.exe")],
        );
        // Discord ha un processo attivo (in una sottocartella versionata) ma niente finestre;
        // Chrome ha finestre e un processo; Steam non è attivo.
        mark_background(
            &mut items,
            &[PathBuf::from("C:/U/Discord/app-1.0/Discord.exe"), PathBuf::from("C:/C/chrome.exe")],
        );
        let by = |n: &str| items.iter().find(|i| i.name == n).unwrap();
        assert!(by("Discord").background && !by("Discord").running);
        assert!(!by("Steam").background);
        assert!(by("Chrome").running && !by("Chrome").background, "con finestre vale il puntino normale");
    }

    #[test]
    fn live_process_scan_excludes_self_and_finds_explorer() {
        let me = std::env::current_exe().unwrap();
        let name = me.file_name().unwrap().to_string_lossy().to_lowercase();
        // Il processo corrente è escluso di proposito: non deve comparire.
        assert!(!processes_named(&[name]).iter().any(|p| key(p) == key(&me)));
        // Nessun nome: nessun lavoro.
        assert!(processes_named(&[]).is_empty());
        // Un processo che esiste sempre.
        assert!(processes_named(&["explorer.exe".to_string()]).iter().any(|p| key(p).ends_with("explorer.exe")));
    }

    #[test]
    fn helper_windows_are_attributed_to_the_pinned_parent() {
        // Steam.exe (10) -> steamwebhelper.exe (20) -> steamwebhelper.exe (30); chrome.exe (40) isolato.
        let table = vec![
            (4, 0, "system".to_string()),
            (10, 4, "steam.exe".to_string()),
            (20, 10, "steamwebhelper.exe".to_string()),
            (30, 20, "steamwebhelper.exe".to_string()),
            (40, 4, "chrome.exe".to_string()),
        ];
        let names = vec!["steam.exe".to_string(), "chrome.exe".to_string()];
        assert_eq!(find_ancestor(&table, 20, &names), Some(10));
        assert_eq!(find_ancestor(&table, 30, &names), Some(10), "anche due livelli sotto");
        assert_eq!(find_ancestor(&table, 40, &names), None, "il genitore e' system, non fissato");
        assert_eq!(find_ancestor(&table, 10, &names), None, "un'app non e' antenata di se stessa");
        assert_eq!(find_ancestor(&table, 999, &names), None, "pid sconosciuto");
        assert_eq!(find_ancestor(&table, 20, &[]), None);
    }

    #[test]
    fn ancestor_search_stops_at_four_levels_and_survives_cycles() {
        // Cinque livelli sotto l'antenato: oltre il limite.
        let mut table = vec![(1, 0, "app.exe".to_string())];
        for i in 2..=7u32 {
            table.push((i, i - 1, "helper.exe".to_string()));
        }
        let names = vec!["app.exe".to_string()];
        assert_eq!(find_ancestor(&table, 5, &names), Some(1));
        assert_eq!(find_ancestor(&table, 7, &names), None);
        // Un ciclo (pid riusati) non deve bloccare la ricerca.
        let cyc = vec![(1, 2, "a.exe".to_string()), (2, 1, "b.exe".to_string())];
        assert_eq!(find_ancestor(&cyc, 1, &["zzz.exe".to_string()]), None);
    }

    #[test]
    fn stable_order_keeps_known_appends_new_and_drops_closed() {
        let v = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let mut known = Vec::new();
        assert_eq!(stable_order(&mut known, &v(&["a", "b"])), v(&["a", "b"]));
        // Cambia l'ordine Z e compare una nuova app: l'ordine di comparsa resta.
        assert_eq!(stable_order(&mut known, &v(&["c", "b", "a"])), v(&["a", "b", "c"]));
        // "b" si chiude e poi riapre: va in fondo.
        assert_eq!(stable_order(&mut known, &v(&["a", "c"])), v(&["a", "c"]));
        assert_eq!(stable_order(&mut known, &v(&["b", "a", "c"])), v(&["a", "c", "b"]));
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
