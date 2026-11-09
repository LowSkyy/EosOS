//! Cattura dello schermo (come macOS): **schermo intero** (il monitor dove sta il cursore), **finestra** (quella in primo piano) e
//! **selezione** (si trascina un rettangolo, vedi `capturewin.rs`). L'immagine si salva in `Immagini\Screenshots` come PNG
//! («Cattura schermo 2026-11-09 alle 17.55.03.png»), si copia negli appunti e si accompagna con un suono.
//!
//! Il Dock, le barre e i pannelli di EosOS sono esclusi dalla cattura di sistema (servono alla sfocatura): per la cattura si
//! riabilitano per un attimo (`with_ui_visible`) così lo screenshot mostra tutto ciò che si vede davvero.
//!
//! La logica (nomi, rettangoli, formati) è pura e testata; sotto c'è la parte che parla con Windows.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use windows::core::GUID;
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::core::w;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT};
use windows::Win32::UI::Shell::{FOLDERID_Pictures, SHGetKnownFolderPath, KF_FLAG_DEFAULT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetMessageW, PostThreadMessageW, MSG, WM_APP, WM_HOTKEY, GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowDisplayAffinity, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    SetWindowDisplayAffinity, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, WDA_EXCLUDEFROMCAPTURE,
    WDA_NONE, WINDOW_DISPLAY_AFFINITY,
};

use crate::dockgeom::Rect;

/// Lato minimo di una selezione (px fisici): sotto, un semplice clic, non una selezione.
pub const MIN_SIDE: i32 = 4;

// ------------------------------------------------------------------ parte pura

/// Che cosa si cattura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Screen,
    Window,
    Region,
}

/// Il nome del tipo nel protocollo con la pagina. Pura, testata.
pub fn kind_named(s: &str) -> Option<Kind> {
    match s {
        "screen" => Some(Kind::Screen),
        "window" => Some(Kind::Window),
        "region" => Some(Kind::Region),
        _ => None,
    }
}

/// Il nome del file: «Cattura schermo 2026-11-09 alle 17.55.03.png». Niente due punti (non ammessi nei nomi di Windows). Pura, testata.
pub fn file_name(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> String {
    format!("Cattura schermo {y:04}-{mo:02}-{d:02} alle {h:02}.{mi:02}.{s:02}.png")
}

/// Un nome che non esiste ancora in `dir`: se c'è già (due catture nello stesso secondo) si aggiunge « (2)», « (3)»… Pura, testata.
pub fn unique_path(dir: &Path, name: &str, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let first = dir.join(name);
    if !exists(&first) {
        return first;
    }
    let (stem, ext) = name.rsplit_once('.').unwrap_or((name, ""));
    for n in 2..1000 {
        let p = dir.join(if ext.is_empty() { format!("{stem} ({n})") } else { format!("{stem} ({n}).{ext}") });
        if !exists(&p) {
            return p;
        }
    }
    first
}

/// Da BGRA (come lo dà lo schermo, con l'alfa a zero) a RGBA opaco per il PNG. Pura, testata.
pub fn to_rgba_opaque(bgra: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bgra.len());
    for p in bgra.as_chunks::<4>().0 {
        out.extend_from_slice(&[p[2], p[1], p[0], 255]);
    }
    out
}

/// L'immagine per gli appunti (formato `CF_DIB`): intestazione da 40 byte + righe dal BASSO verso l'alto, 32 bit, alfa opaco
/// (le app che leggono i 32 bit come alfa non la vedrebbero trasparente). Pura, testata.
pub fn dib_bytes(w: i32, h: i32, bgra: &[u8]) -> Vec<u8> {
    let stride = (w as usize) * 4;
    let mut out = Vec::with_capacity(40 + bgra.len());
    out.extend_from_slice(&40u32.to_le_bytes()); // biSize
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes()); // positivo: righe dal basso
    out.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    out.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    out.extend_from_slice(&((stride * h as usize) as u32).to_le_bytes()); // biSizeImage
    out.extend_from_slice(&[0u8; 16]); // risoluzioni e colori
    for row in bgra.chunks_exact(stride).rev() {
        for p in row.as_chunks::<4>().0 {
            out.extend_from_slice(&[p[0], p[1], p[2], 255]);
        }
    }
    out
}

/// Il rettangolo scelto dalla pagina (px CSS del monitor, anche trascinato all'indietro) in pixel fisici dentro `monitor`.
/// `None` se la scala non è sensata o se la selezione è più piccola di `MIN_SIDE`. Pura, testata.
pub fn region_to_rect(x: f64, y: f64, w: f64, h: f64, dpr: f64, monitor: Rect) -> Option<Rect> {
    if ![x, y, w, h, dpr].iter().all(|v| v.is_finite()) || !(0.5..=4.0).contains(&dpr) {
        return None;
    }
    let (x0, x1) = (x.min(x + w), x.max(x + w));
    let (y0, y1) = (y.min(y + h), y.max(y + h));
    let to = |v: f64| (v * dpr).round() as i64;
    let l = (monitor.x as i64 + to(x0)).clamp(monitor.x as i64, (monitor.x + monitor.w) as i64);
    let r = (monitor.x as i64 + to(x1)).clamp(monitor.x as i64, (monitor.x + monitor.w) as i64);
    let t = (monitor.y as i64 + to(y0)).clamp(monitor.y as i64, (monitor.y + monitor.h) as i64);
    let b = (monitor.y as i64 + to(y1)).clamp(monitor.y as i64, (monitor.y + monitor.h) as i64);
    let (rw, rh) = ((r - l) as i32, (b - t) as i32);
    (rw >= MIN_SIDE && rh >= MIN_SIDE).then_some(Rect { x: l as i32, y: t as i32, w: rw, h: rh })
}

/// Il rettangolo di una finestra tenuto dentro lo schermo virtuale (una finestra mezza fuori si taglia). `None` se resta meno di
/// `MIN_SIDE`. Pura, testata.
pub fn clamp_to(r: Rect, screen: Rect) -> Option<Rect> {
    let l = r.x.max(screen.x);
    let t = r.y.max(screen.y);
    let rr = (r.x + r.w).min(screen.x + screen.w);
    let b = (r.y + r.h).min(screen.y + screen.h);
    (rr - l >= MIN_SIDE && b - t >= MIN_SIDE).then_some(Rect { x: l, y: t, w: rr - l, h: b - t })
}

/// Le scelte del modificatore delle scorciatoie: (id, etichetta). «off» è la scelta di base.
pub const MODES: &[(&str, &str)] = &[("off", "Disattivate"), ("ctrl_shift", "Ctrl+Maiusc+3/4/5"), ("ctrl_alt", "Ctrl+Alt+3/4/5"), ("alt_shift", "Alt+Maiusc+3/4/5")];

/// I tasti: 3 = schermo intero, 4 = selezione, 5 = finestra. Pura.
pub const KEYS: &[(u32, Kind)] = &[(0x33, Kind::Screen), (0x34, Kind::Region), (0x35, Kind::Window)];

/// La modalità effettiva dato il valore salvato (vuoto o sconosciuto = spente); `EOSOS_CAPTURE_KEYS=0` le spegne sempre. Pura, testata.
pub fn effective_mode(saved: &str, env: Option<&str>) -> &'static str {
    if env == Some("0") {
        return "off";
    }
    MODES.iter().find(|m| m.0 == saved).map_or("off", |m| m.0)
}

// ------------------------------------------------------------------ Windows

fn to_rect(r: RECT) -> Rect {
    Rect { x: r.left, y: r.top, w: r.right - r.left, h: r.bottom - r.top }
}

/// Lo schermo virtuale (tutti i monitor insieme).
fn virtual_screen() -> Rect {
    unsafe { Rect { x: GetSystemMetrics(SM_XVIRTUALSCREEN), y: GetSystemMetrics(SM_YVIRTUALSCREEN), w: GetSystemMetrics(SM_CXVIRTUALSCREEN), h: GetSystemMetrics(SM_CYVIRTUALSCREEN) } }
}

/// Il monitor (px fisici) dove sta il cursore.
pub fn cursor_monitor() -> Rect {
    unsafe {
        let mut p = POINT::default();
        let _ = GetCursorPos(&mut p);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(MonitorFromPoint(p, MONITOR_DEFAULTTONEAREST), &mut mi).as_bool() { to_rect(mi.rcMonitor) } else { virtual_screen() }
    }
}

/// La parte visibile della finestra in primo piano (senza i bordi invisibili), se è catturabile: visibile, non ridotta a icona e di un
/// altro programma (mai una finestra di EosOS: sarebbe il pannello da cui si è premuto).
fn foreground_rect() -> Option<Rect> {
    window_rect(unsafe { GetForegroundWindow() })
}

/// La parte visibile della finestra `h` se è catturabile (vedi `foreground_rect`).
fn window_rect(h: HWND) -> Option<Rect> {
    unsafe {
        if h.0.is_null() || !IsWindowVisible(h).as_bool() || IsIconic(h).as_bool() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(h, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            return None;
        }
        let mut r = RECT::default();
        DwmGetWindowAttribute(h, DWMWA_EXTENDED_FRAME_BOUNDS, &mut r as *mut _ as *mut _, std::mem::size_of::<RECT>() as u32).ok()?;
        clamp_to(to_rect(r), virtual_screen())
    }
}

unsafe extern "system" fn collect_own(h: HWND, data: LPARAM) -> windows::core::BOOL {
    // SAFETY: `data` è il puntatore al Vec di `with_ui_visible`, valido per tutta l'enumerazione.
    let out = unsafe { &mut *(data.0 as *mut Vec<(isize, WINDOW_DISPLAY_AFFINITY)>) };
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(h, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            let mut a = 0u32;
            if GetWindowDisplayAffinity(h, &mut a).is_ok() && a == WDA_EXCLUDEFROMCAPTURE.0 {
                out.push((h.0 as isize, WINDOW_DISPLAY_AFFINITY(a)));
            }
        }
    }
    windows::core::BOOL(1)
}

/// Esegue `f` con le finestre di EosOS escluse dalla cattura rese visibili (e poi le rimette com'erano).
fn with_ui_visible<T>(f: impl FnOnce() -> T) -> T {
    let mut own: Vec<(isize, WINDOW_DISPLAY_AFFINITY)> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect_own), LPARAM(&mut own as *mut _ as isize));
        for (h, _) in &own {
            let _ = SetWindowDisplayAffinity(HWND(*h as *mut _), WDA_NONE);
        }
    }
    if !own.is_empty() {
        std::thread::sleep(Duration::from_millis(160)); // il tempo che DWM ricomponga
    }
    let out = f();
    unsafe {
        for (h, a) in &own {
            let _ = SetWindowDisplayAffinity(HWND(*h as *mut _), *a);
        }
    }
    out
}

/// Dove si salvano le catture: `Immagini\Screenshots` (come lo Strumento di cattura). Nelle build di prova `EOSOS_CAPTURE_DIR` lo cambia.
fn save_dir() -> Option<PathBuf> {
    #[cfg(feature = "test-hooks")]
    if let Some(d) = std::env::var_os("EOSOS_CAPTURE_DIR") {
        return Some(PathBuf::from(d));
    }
    let pictures = unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Pictures as *const GUID, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        PathBuf::from(s?)
    };
    Some(pictures.join("Screenshots"))
}

/// Appunti e suono vanno toccati? Le prove automatiche di solito no (`EOSOS_TEST_QUIET`, solo build di prova).
fn side_effects_enabled() -> bool {
    !(cfg!(feature = "test-hooks") && std::env::var_os("EOSOS_TEST_QUIET").is_some())
}

fn set_clipboard_dib(dib: &[u8]) -> bool {
    const CF_DIB: u32 = 8;
    unsafe {
        let mut opened = false;
        for _ in 0..8 {
            if OpenClipboard(None).is_ok() {
                opened = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        if !opened {
            return false;
        }
        let ok = (|| {
            EmptyClipboard().ok()?;
            let g = GlobalAlloc(GMEM_MOVEABLE, dib.len()).ok()?;
            let p = GlobalLock(g) as *mut u8;
            if p.is_null() {
                return None;
            }
            std::ptr::copy_nonoverlapping(dib.as_ptr(), p, dib.len());
            let _ = GlobalUnlock(g);
            SetClipboardData(CF_DIB, Some(HANDLE(g.0))).ok()?;
            Some(())
        })()
        .is_some();
        let _ = CloseClipboard();
        ok
    }
}

fn png_bytes(w: u32, h: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().ok()?.write_image_data(rgba).ok()?;
    }
    Some(out)
}

/// L'esito di una cattura.
#[derive(serde::Serialize, Debug, Clone)]
pub struct Shot {
    pub path: String,
    pub w: i32,
    pub h: i32,
}

/// Scatta il rettangolo `r` (px fisici): PNG nella cartella delle catture, copia negli appunti, suono. `None` se qualcosa non riesce.
pub fn shoot(r: Rect) -> Option<Shot> {
    let bgra = with_ui_visible(|| crate::backdrop::grab(r))?;
    let png = png_bytes(r.w as u32, r.h as u32, &to_rgba_opaque(&bgra))?;
    let dir = save_dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    let now = local_time();
    let path = unique_path(&dir, &file_name(now.0, now.1, now.2, now.3, now.4, now.5), |p| p.exists());
    std::fs::write(&path, png).ok()?;
    if side_effects_enabled() {
        let _ = set_clipboard_dib(&dib_bytes(r.w, r.h, &bgra));
        unsafe {
            let _ = PlaySoundW(w!("SystemAsterisk"), None, SND_ALIAS | SND_ASYNC);
        }
    }
    Some(Shot { path: path.to_string_lossy().into_owned(), w: r.w, h: r.h })
}

/// L'ora locale (anno, mese, giorno, ore, minuti, secondi).
fn local_time() -> (i32, u32, u32, u32, u32, u32) {
    use windows::Win32::System::SystemInformation::GetLocalTime;
    let t = unsafe { GetLocalTime() };
    (i32::from(t.wYear), u32::from(t.wMonth), u32::from(t.wDay), u32::from(t.wHour), u32::from(t.wMinute), u32::from(t.wSecond))
}

/// Solo per le prove: cattura la finestra `h` (invece di quella in primo piano, che Windows non lascia prendere a una finestra di prova).
#[allow(dead_code)]
pub fn run_window(h: HWND) -> Option<Shot> {
    shoot(window_rect(h)?)
}

/// Cattura schermo intero (monitor del cursore) o finestra in primo piano. La selezione passa da `capturewin`.
pub fn run(kind: Kind) -> Option<Shot> {
    match kind {
        Kind::Screen => shoot(cursor_monitor()),
        Kind::Window => shoot(foreground_rect()?),
        Kind::Region => None,
    }
}

// ------------------------------------------------------------------ avvio e scorciatoie

static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
static THREAD_ID: AtomicU32 = AtomicU32::new(0);
static REGISTERED: AtomicUsize = AtomicUsize::new(0);
const BASE_ID: i32 = 0x0E90;
const RELOAD: u32 = WM_APP + 0x23;

/// I modificatori della modalità (0 = spente). Pura, testata.
pub fn modifiers(mode: &str) -> u32 {
    match mode {
        "ctrl_shift" => MOD_CONTROL.0 | MOD_SHIFT.0,
        "ctrl_alt" => MOD_CONTROL.0 | MOD_ALT.0,
        "alt_shift" => MOD_ALT.0 | MOD_SHIFT.0,
        _ => 0,
    }
}

/// Quante combinazioni sono registrate adesso (per le prove).
#[allow(dead_code)]
pub fn registered_count() -> usize {
    REGISTERED.load(Ordering::Relaxed)
}

fn current_mode() -> &'static str {
    effective_mode(&crate::mcstore::load().capture_keys, std::env::var("EOSOS_CAPTURE_KEYS").ok().as_deref())
}

fn register_all() {
    unsafe {
        for i in 0..KEYS.len() as i32 {
            let _ = UnregisterHotKey(None, BASE_ID + i);
        }
    }
    let mods = modifiers(current_mode());
    let mut ok = 0;
    if mods != 0 {
        for (i, (vk, _)) in KEYS.iter().enumerate() {
            if unsafe { RegisterHotKey(None, BASE_ID + i as i32, HOT_KEY_MODIFIERS(mods) | MOD_NOREPEAT, *vk) }.is_ok() {
                ok += 1;
            }
        }
    }
    REGISTERED.store(ok, Ordering::Relaxed);
}

/// Avvia una cattura: `delay_ms` dà il tempo a un pannello o al Launcher da cui parte il comando di chiudersi (così non finiscono
/// nell'immagine). La selezione apre il velo; le altre scattano e basta.
pub fn trigger(kind: Kind, delay_ms: u64) {
    let Some(app) = APP.get().cloned() else { return };
    std::thread::spawn(move || {
        if delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(delay_ms));
        }
        match kind {
            Kind::Region => crate::capturewin::begin(&app),
            k => {
                let _ = run(k);
            }
        }
    });
}

/// Si ricordano l'app e si avvia il thread delle scorciatoie (una volta).
pub fn start(app: &tauri::AppHandle) {
    if APP.set(app.clone()).is_err() {
        return;
    }
    std::thread::spawn(|| {
        THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
        register_all();
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            if msg.message == WM_HOTKEY {
                let i = msg.wParam.0 as i32 - BASE_ID;
                if let Some((_, kind)) = usize::try_from(i).ok().and_then(|i| KEYS.get(i)) {
                    trigger(*kind, 0);
                }
            } else if msg.message == RELOAD {
                register_all();
            }
        }
    });
}

/// Le impostazioni sono cambiate: si registrano di nuovo le combinazioni.
pub fn reload() {
    let id = THREAD_ID.load(Ordering::Relaxed);
    if id != 0 {
        unsafe {
            let _ = PostThreadMessageW(id, RELOAD, WPARAM(0), windows::Win32::Foundation::LPARAM(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MON: Rect = Rect { x: -1920, y: 0, w: 1920, h: 1080 };

    #[test]
    fn the_file_name_has_no_colons_and_sorts_by_time() {
        let n = file_name(2026, 11, 9, 7, 5, 3);
        assert_eq!(n, "Cattura schermo 2026-11-09 alle 07.05.03.png");
        assert!(!n.contains(':'));
        assert!(file_name(2026, 11, 9, 7, 5, 3) < file_name(2026, 11, 9, 17, 0, 0));
    }

    #[test]
    fn a_taken_name_gets_a_counter() {
        let dir = Path::new("D:\\x");
        let taken = [dir.join("a.png"), dir.join("a (2).png")];
        let exists = |p: &Path| taken.iter().any(|t| t == p);
        assert_eq!(unique_path(dir, "b.png", exists), dir.join("b.png"));
        assert_eq!(unique_path(dir, "a.png", exists), dir.join("a (3).png"));
        assert_eq!(unique_path(dir, "senzaestensione", |p| p == dir.join("senzaestensione")), dir.join("senzaestensione (2)"));
    }

    #[test]
    fn screen_pixels_become_opaque_rgba() {
        let bgra = [10u8, 20, 30, 0, 1, 2, 3, 0]; // B G R A
        assert_eq!(to_rgba_opaque(&bgra), vec![30, 20, 10, 255, 3, 2, 1, 255]);
        assert!(to_rgba_opaque(&[]).is_empty());
    }

    #[test]
    fn the_clipboard_image_is_a_bottom_up_32_bit_dib() {
        // 2 x 2: riga alta (A B) e riga bassa (C D), ognuno BGRA.
        let px = |v: u8| [v, v + 1, v + 2, 0];
        let bgra: Vec<u8> = [px(10), px(20), px(30), px(40)].concat();
        let d = dib_bytes(2, 2, &bgra);
        assert_eq!(d.len(), 40 + 16);
        assert_eq!(u32::from_le_bytes(d[0..4].try_into().unwrap()), 40, "biSize");
        assert_eq!(i32::from_le_bytes(d[4..8].try_into().unwrap()), 2, "larghezza");
        assert_eq!(i32::from_le_bytes(d[8..12].try_into().unwrap()), 2, "altezza positiva = dal basso");
        assert_eq!(u16::from_le_bytes(d[14..16].try_into().unwrap()), 32, "bit per pixel");
        assert_eq!(u32::from_le_bytes(d[20..24].try_into().unwrap()), 16, "biSizeImage");
        // La prima riga scritta e la bassa (C D), e l'alfa e 255.
        assert_eq!(&d[40..48], &[30, 31, 32, 255, 40, 41, 42, 255]);
        assert_eq!(&d[48..56], &[10, 11, 12, 255, 20, 21, 22, 255]);
    }

    #[test]
    fn a_selection_is_converted_to_physical_pixels_inside_the_monitor() {
        // 150% di scala: 100,50 per 200x100 CSS = 150,75 per 300x150 fisici, dentro il monitor a sinistra.
        assert_eq!(region_to_rect(100.0, 50.0, 200.0, 100.0, 1.5, MON), Some(Rect { x: -1920 + 150, y: 75, w: 300, h: 150 }));
        // Trascinato all'indietro (verso l'alto e a sinistra): stesso rettangolo.
        assert_eq!(region_to_rect(300.0, 150.0, -200.0, -100.0, 1.5, MON), Some(Rect { x: -1920 + 150, y: 75, w: 300, h: 150 }));
        // Che esce dal monitor: si taglia al bordo.
        let r = region_to_rect(1800.0, 1000.0, 400.0, 400.0, 1.0, MON).unwrap();
        assert_eq!((r.x + r.w, r.y + r.h), (MON.x + MON.w, MON.y + MON.h));
        assert_eq!(region_to_rect(-50.0, -50.0, 100.0, 100.0, 1.0, MON), Some(Rect { x: -1920, y: 0, w: 50, h: 50 }));
    }

    #[test]
    fn tiny_or_nonsense_selections_are_refused() {
        assert_eq!(region_to_rect(10.0, 10.0, 2.0, 50.0, 1.0, MON), None, "un clic o quasi");
        assert_eq!(region_to_rect(10.0, 10.0, 50.0, 3.0, 1.0, MON), None);
        assert_eq!(region_to_rect(f64::NAN, 0.0, 10.0, 10.0, 1.0, MON), None);
        assert_eq!(region_to_rect(0.0, 0.0, f64::INFINITY, 10.0, 1.0, MON), None);
        assert_eq!(region_to_rect(0.0, 0.0, 100.0, 100.0, 0.0, MON), None, "scala zero");
        assert_eq!(region_to_rect(0.0, 0.0, 100.0, 100.0, 9.0, MON), None, "scala assurda");
        assert_eq!(region_to_rect(5000.0, 5000.0, 100.0, 100.0, 1.0, MON), None, "tutta fuori dal monitor");
    }

    #[test]
    fn a_window_partly_off_screen_is_cut_to_the_screen() {
        let screen = Rect { x: -1920, y: 0, w: 3840, h: 1080 };
        assert_eq!(clamp_to(Rect { x: 100, y: 100, w: 400, h: 300 }, screen), Some(Rect { x: 100, y: 100, w: 400, h: 300 }));
        assert_eq!(clamp_to(Rect { x: -2000, y: -50, w: 500, h: 300 }, screen), Some(Rect { x: -1920, y: 0, w: 420, h: 250 }));
        assert_eq!(clamp_to(Rect { x: 4000, y: 0, w: 100, h: 100 }, screen), None, "tutta fuori");
        assert_eq!(clamp_to(Rect { x: -1925, y: 0, w: 8, h: 100 }, screen), None, "resta una striscia di 3 px");
    }

    #[test]
    fn modifiers_follow_the_mode() {
        assert_eq!(modifiers("off"), 0);
        assert_eq!(modifiers("boh"), 0);
        for (id, _) in MODES.iter().filter(|m| m.0 != "off") {
            assert_eq!(modifiers(id).count_ones(), 2, "{id}");
        }
        assert_eq!(KEYS.iter().map(|k| k.0).collect::<Vec<_>>(), [0x33, 0x34, 0x35], "3, 4, 5");
    }

    #[test]
    fn the_kinds_and_keys_are_fixed() {
        assert_eq!(kind_named("screen"), Some(Kind::Screen));
        assert_eq!(kind_named("window"), Some(Kind::Window));
        assert_eq!(kind_named("region"), Some(Kind::Region));
        assert_eq!(kind_named("Screen"), None);
        assert_eq!(kind_named(""), None);
        assert_eq!(KEYS.len(), 3);
        assert_eq!(effective_mode("", None), "off");
        assert_eq!(effective_mode("boh", None), "off");
        assert_eq!(effective_mode("ctrl_shift", None), "ctrl_shift");
        assert_eq!(effective_mode("ctrl_shift", Some("0")), "off");
    }
}
