//! «Semafori» in stile macOS al posto dei tasti di Windows (chiudi, riduci a icona, schermo intero) su tutte le finestre.
//!
//! Come funziona (senza toccare le app): per ogni finestra con barra del titolo EosOS crea una finestrina «a strati»
//! senza focus, grande quanto l'area dei tre tasti di Windows (`DWMWA_CAPTION_BUTTON_BOUNDS`), subito sopra di essa
//! nell'ordine delle finestre. La finestrina ha il colore della barra (letto dallo schermo) e disegna i tre cerchi;
//! il clic manda alla finestra vera lo stesso `WM_SYSCOMMAND` dei tasti di Windows. A sinistra (come macOS) la
//! finestrina copre l'icona e l'inizio del titolo, e a destra copre soltanto i tasti di Windows.
//!
//! Sicurezza: se EosOS si chiude le finestrine spariscono e i tasti di Windows tornano da soli. Niente viene
//! modificato nelle altre app (salvo, per Esplora file, il bordo: si ripristina da solo spegnendo l'opzione).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::core::{w, BOOL};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_BUTTON_BOUNDS, DWMWA_CLOAKED,
    DWMWA_EXTENDED_FRAME_BOUNDS, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWINDOWATTRIBUTE,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, GetMonitorInfoW, GetPixel, MonitorFromWindow, ReleaseDC, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, LoadCursorW, IDC_ARROW, DestroyWindow, DispatchMessageW, EnumWindows, GetClassNameW, GetForegroundWindow, GetMessageW, GetWindow, GetWindowLongPtrW,
    GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed, PostMessageW, RegisterClassW, SetTimer, SetWindowPos, ShowWindow,
    TranslateMessage, UpdateLayeredWindow, GWL_EXSTYLE, GWL_STYLE, GW_HWNDNEXT, GW_HWNDPREV, HWND_MESSAGE, HWND_TOP, HWND_TOPMOST, MSG, SC_CLOSE,
    SC_MAXIMIZE, SC_MINIMIZE, SC_RESTORE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SWP_NOOWNERZORDER, SWP_NOSIZE, SW_SHOWNOACTIVATE, ULW_ALPHA, WINDOW_EX_STYLE,
    WM_APP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_SYSCOMMAND, WM_TIMER, WNDCLASSW, WS_CAPTION, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_THICKFRAME, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

const WM_MOUSELEAVE: u32 = 0x02A3;
const TICK_MS: u32 = 100;
const FAST_MS: u32 = 16;
/// Bit che distingue la finestrina di copertura dei tasti di Windows da quella dei cerchi della stessa finestra.
const COVER_BIT: isize = 1 << 60;
/// Ogni quanto si rilegge il colore della barra (cambia con lo sfondo e con l'attivazione).
const RESAMPLE: Duration = Duration::from_millis(700);

static HOST: AtomicIsize = AtomicIsize::new(0);
static PENDING: AtomicBool = AtomicBool::new(false);
/// Finestre di Esplora file alle quali e' stato cambiato il bordo (per ripristinarle).
static TOUCHED: Mutex<Vec<isize>> = Mutex::new(Vec::new());

// ------------------------------------------------------------------ parte pura (testata)

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Close,
    Min,
    Zoom,
}

/// I cerchi e il loro ordine: a sinistra (macOS) chiudi, riduci, schermo intero; a destra (Windows) riduci, schermo intero, chiudi.
/// Con una finestra che ha solo «chiudi» (finestre di dialogo) resta il solo cerchio rosso.
pub fn kinds_order(right: bool, three: bool) -> Vec<Kind> {
    match (three, right) {
        (false, _) => vec![Kind::Close],
        (true, false) => vec![Kind::Close, Kind::Min, Kind::Zoom],
        (true, true) => vec![Kind::Min, Kind::Zoom, Kind::Close],
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub w: i32,
    pub h: i32,
    pub r: f32,
    /// (cerchio, centro x, centro y) nella finestrina.
    pub centers: Vec<(Kind, f32, f32)>,
}

/// Disposizione dei cerchi. A sinistra la finestrina e' larga quanto serve; a destra quanto i tasti di Windows
/// (`native_w`) e il gruppo di cerchi ci sta al centro.
pub fn layout(kinds: &[Kind], scale: f32, right: bool, native_w: i32, h: i32) -> Layout {
    let n = kinds.len().max(1) as f32;
    let r = 6.5 * scale;
    let gap = 8.0 * scale;
    let pad = 10.0 * scale;
    let group = n * 2.0 * r + (n - 1.0) * gap;
    let (w, start) = if right {
        let w = native_w.max((group + 2.0 * 4.0 * scale).ceil() as i32);
        (w, w as f32 - group - 14.0 * scale)
    } else {
        ((group + 2.0 * pad).ceil() as i32, pad)
    };
    let h = h.max((2.0 * r + 4.0).ceil() as i32);
    let centers = kinds.iter().enumerate().map(|(i, k)| (*k, start + r + i as f32 * (2.0 * r + gap), h as f32 / 2.0)).collect();
    Layout { w, h, r, centers }
}

/// Quale cerchio sta sotto il punto (con un po' di margine, per non dover essere precisi).
pub fn hit(l: &Layout, x: f32, y: f32, scale: f32) -> Option<usize> {
    let slack = 2.5 * scale;
    l.centers.iter().position(|(_, cx, cy)| (x - cx).hypot(y - cy) <= l.r + slack)
}

/// Il `WM_SYSCOMMAND` da mandare alla finestra vera per un cerchio.
pub fn command_for(kind: Kind, zoomed: bool) -> u32 {
    match kind {
        Kind::Close => SC_CLOSE,
        Kind::Min => SC_MINIMIZE,
        Kind::Zoom => {
            if zoomed {
                SC_RESTORE
            } else {
                SC_MAXIMIZE
            }
        }
    }
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn circle_cov(px: f32, py: f32, cx: f32, cy: f32, r: f32) -> f32 {
    clamp01(r + 0.5 - (px - cx).hypot(py - cy))
}

fn seg_cov(px: f32, py: f32, a: (f32, f32), b: (f32, f32), hw: f32) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 <= f32::EPSILON { 0.0 } else { clamp01(((px - a.0) * dx + (py - a.1) * dy) / len2) };
    let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
    clamp01(hw + 0.5 - (px - qx).hypot(py - qy))
}

fn in_tri(p: (f32, f32), t: &[(f32, f32); 3]) -> bool {
    let s = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| (a.0 - c.0) * (b.1 - c.1) - (b.0 - c.0) * (a.1 - c.1);
    let (d1, d2, d3) = (s(p, t[0], t[1]), s(p, t[1], t[2]), s(p, t[2], t[0]));
    !((d1 < 0.0 || d2 < 0.0 || d3 < 0.0) && (d1 > 0.0 || d2 > 0.0 || d3 > 0.0))
}

fn tri_cov(px: f32, py: f32, t: &[(f32, f32); 3]) -> f32 {
    let mut n = 0;
    for sy in 0..3 {
        for sx in 0..3 {
            if in_tri((px - 0.5 + (sx as f32 + 0.5) / 3.0, py - 0.5 + (sy as f32 + 0.5) / 3.0), t) {
                n += 1;
            }
        }
    }
    n as f32 / 9.0
}

/// Copertura del simbolo dentro al cerchio (croce, trattino, frecce), in coordinate relative al centro.
fn glyph_cov(kind: Kind, zoomed: bool, dx: f32, dy: f32, r: f32, scale: f32) -> f32 {
    let g = r * 0.5;
    let hw = (0.55 * scale).max(0.55);
    match kind {
        Kind::Close => seg_cov(dx, dy, (-g, -g), (g, g), hw).max(seg_cov(dx, dy, (-g, g), (g, -g), hw)),
        Kind::Min => seg_cov(dx, dy, (-g * 1.15, 0.0), (g * 1.15, 0.0), hw),
        Kind::Zoom => {
            let (a, b) = if zoomed {
                ([(0.1 * g, -0.1 * g), (g, -0.1 * g), (0.1 * g, -g)], [(-0.1 * g, 0.1 * g), (-g, 0.1 * g), (-0.1 * g, g)])
            } else {
                ([(g, -g), (g, 0.1 * g), (-0.1 * g, -g)], [(-g, g), (-g, -0.1 * g), (0.1 * g, g)])
            };
            tri_cov(dx, dy, &a).max(tri_cov(dx, dy, &b))
        }
    }
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn lum(c: [u8; 3]) -> f32 {
    0.299 * f32::from(c[0]) + 0.587 * f32::from(c[1]) + 0.114 * f32::from(c[2])
}

/// Colore di un cerchio (RGB 0..255): vivo se la finestra e' attiva, grigio altrimenti o se non si puo' usare.
fn light_color(kind: Kind, usable: bool, active: bool, bg: [u8; 3]) -> [f32; 3] {
    if !usable || !active {
        return if lum(bg) < 128.0 { [92.0, 92.0, 96.0] } else { [196.0, 196.0, 200.0] };
    }
    match kind {
        Kind::Close => [255.0, 95.0, 87.0],
        Kind::Min => [254.0, 188.0, 46.0],
        Kind::Zoom => [40.0, 200.0, 64.0],
    }
}

/// Stato per disegnare: tutto quello che serve a `render`.
pub struct Paint<'a> {
    pub layout: &'a Layout,
    pub kinds: &'a [(Kind, bool)],
    pub scale: f32,
    pub bg: [u8; 3],
    pub active: bool,
    pub zoomed: bool,
    pub hover: bool,
    pub down: Option<usize>,
    /// Angolo arrotondato della finestra: (a destra?, raggio in pixel); 0 = senza.
    pub corner: (bool, f32),
    /// Forma a capsula (cerchi a sinistra) invece del rettangolo pieno.
    pub capsule: bool,
}

/// Disegna la finestrina: `w*h` pixel BGRA premoltiplicati.
///
/// Due forme: a destra (e per la copertura dei tasti di Windows) un rettangolo pieno del colore della barra; a sinistra una
/// «capsula» scura o chiara (secondo la barra) con i cerchi, che non copre altro che se stessa.
pub fn render(p: &Paint) -> Vec<u32> {
    let (w, h) = (p.layout.w.max(1) as usize, p.layout.h.max(1) as usize);
    let mut out = vec![0u32; w * h];
    let dark = lum(p.bg) < 128.0;
    let bg = if p.capsule {
        if dark { [30.0, 30.0, 33.0] } else { [244.0, 244.0, 247.0] }
    } else {
        [f32::from(p.bg[0]), f32::from(p.bg[1]), f32::from(p.bg[2])]
    };
    let (right_corner, cr) = p.corner;
    // Capsula: alta quanto i cerchi piu' un po' di margine, centrata nella barra.
    let ph = 2.0 * p.layout.r + 9.0 * p.scale;
    let (pcy, prad) = (h as f32 / 2.0, ph / 2.0);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut col = bg;
            let mut alpha = 1.0f32;
            if p.capsule {
                // distanza con segno da una capsula (un segmento orizzontale ingrossato)
                let hx = (w as f32 / 2.0 - prad).max(0.0);
                let d = ((fx - w as f32 / 2.0).abs() - hx).max(0.0).hypot(fy - pcy) - prad;
                alpha = clamp01(0.5 - d) * 0.92;
            }
            for (i, (k, cx, cy)) in p.layout.centers.iter().enumerate() {
                let c = circle_cov(fx, fy, *cx, *cy, p.layout.r);
                if c <= 0.0 {
                    continue;
                }
                let usable = p.kinds.iter().find(|(kk, _)| kk == k).is_none_or(|(_, u)| *u);
                let mut base = light_color(*k, usable, p.active, p.bg);
                if p.down == Some(i) {
                    base = mix(base, [0.0, 0.0, 0.0], 0.2);
                }
                col = mix(col, base, c);
                alpha = alpha.max(c);
                if p.hover && usable && p.active {
                    let gc = glyph_cov(*k, p.zoomed, fx - cx, fy - cy, p.layout.r, p.scale);
                    col = mix(col, mix(base, [0.0, 0.0, 0.0], 0.62), gc * c);
                }
            }
            // Angolo arrotondato della finestra: fuori dal raggio la finestrina e' trasparente.
            if cr > 0.0 && !p.capsule {
                let (ex, ey) = if right_corner { (w as f32 - fx, fy) } else { (fx, fy) };
                if ex < cr && ey < cr {
                    alpha *= clamp01(cr + 0.5 - (cr - ex).hypot(cr - ey));
                }
            }
            let a = (alpha * 255.0).round() as u32;
            let ch = |v: f32| ((v * alpha).round().clamp(0.0, 255.0)) as u32;
            if let Some(px) = out.get_mut(y * w + x) {
                *px = (a << 24) | (ch(col[0]) << 16) | (ch(col[1]) << 8) | ch(col[2]);
            }
        }
    }
    out
}

/// Vero se le classi della shell di Windows non devono avere i semafori.
fn skipped_class(class: &str) -> bool {
    ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "XamlExplorerHostIslandWindow", "TopLevelWindowForOverflowXamlIsland"].contains(&class)
}

// ------------------------------------------------------------------ finestre

struct Ov {
    hwnd: HWND,
    target: HWND,
    x: i32,
    y: i32,
    layout: Layout,
    kinds: Vec<(Kind, bool)>,
    scale: f32,
    bg: [u8; 3],
    active: bool,
    zoomed: bool,
    corner: (bool, f32),
    hover: bool,
    down: Option<usize>,
    sampled: Instant,
    tracking: bool,
    /// Rettangolo della finestra vera all'ultima passata: serve a accorgersi subito se si e' mossa.
    trect: RECT,
}

#[derive(Default)]
struct State {
    ovs: HashMap<isize, Ov>,
    by_overlay: HashMap<isize, isize>,
    styled: HashSet<isize>,
}

thread_local! {
    static ST: RefCell<State> = RefCell::new(State::default());
}

/// Una finestrina da mostrare: chiave, x, disposizione, cerchi, angolo.
type Part = (isize, i32, Layout, Vec<(Kind, bool)>, (bool, f32));

struct Info {
    hwnd: HWND,
    /// Posizione e altezza dell'area dei tasti di Windows (schermo).
    nx: i32,
    ny: i32,
    native_w: i32,
    h: i32,
    frame_left: i32,
    frame_right: i32,
    frame_top: i32,
    scale: f32,
    kinds: Vec<(Kind, bool)>,
    three: bool,
    zoomed: bool,
    topmost: bool,
    explorer: bool,
}

fn class_of(h: HWND) -> String {
    let mut buf = [0u16; 128];
    let n = unsafe { GetClassNameW(h, &mut buf) };
    String::from_utf16_lossy(buf.get(..n.max(0) as usize).unwrap_or(&[]))
}

fn dwm_rect(h: HWND, attr: DWMWINDOWATTRIBUTE) -> Option<RECT> {
    let mut r = RECT::default();
    unsafe { DwmGetWindowAttribute(h, attr, &mut r as *mut _ as *mut _, std::mem::size_of::<RECT>() as u32) }.ok()?;
    Some(r)
}

/// Se la finestra merita i semafori, dice dove sono i suoi tasti.
fn inspect(h: HWND) -> Option<Info> {
    unsafe {
        if h.0.is_null() || !IsWindowVisible(h).as_bool() || IsIconic(h).as_bool() {
            return None;
        }
        let style = GetWindowLongPtrW(h, GWL_STYLE) as u32;
        let ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as u32;
        let has_caption = style & WS_CAPTION.0 == WS_CAPTION.0;
        if ex & (WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) != 0 {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(h, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            return None;
        }
        let mut cloaked = 0u32;
        if DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, 4).is_ok() && cloaked != 0 {
            return None;
        }
        let class = class_of(h);
        if skipped_class(&class) {
            return None;
        }
        let mut wr = RECT::default();
        GetWindowRect(h, &mut wr).ok()?;
        let fr = dwm_rect(h, DWMWA_EXTENDED_FRAME_BOUNDS).unwrap_or(wr);
        let scale = (GetDpiForWindow(h).max(96) as f32) / 96.0;
        // Con la barra di sistema Windows dice dove sono i tasti; se non lo dice (barra disegnata dall'app: restituisce una larghezza nulla)
        // o la barra non c'e', si usa la posizione di Windows 11.
        let from_dwm = if has_caption { dwm_rect(h, DWMWA_CAPTION_BUTTON_BOUNDS).filter(|r| r.right - r.left >= 20 && r.bottom - r.top >= 10) } else { None };
        let cb = if let Some(r) = from_dwm {
            r
        } else {
            // Finestre senza barra di sistema (VS Code, Discord, Spotify, app Electron...): i tasti li disegna l'app in alto a destra, di
            // solito come Windows 11 (3 tasti da 46 x 33). Solo se la finestra si puo' ridimensionare e ridurre/ingrandire e non e' piccola.
            let sizable = style & WS_THICKFRAME.0 != 0 && style & (WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0) != 0;
            if !sizable || (fr.right - fr.left) < (320.0 * scale) as i32 || (fr.bottom - fr.top) < (200.0 * scale) as i32 {
                return None;
            }
            let w = (3.0 * 46.0 * scale) as i32;
            RECT { left: fr.right - wr.left - w, top: fr.top - wr.top, right: fr.right - wr.left, bottom: fr.top - wr.top + (33.0 * scale) as i32 }
        };
        if cb.right - cb.left < 20 || cb.bottom - cb.top < 10 {
            return None;
        }
        let zoomed = IsZoomed(h).as_bool();
        // A schermo intero vero (video, giochi, F11) niente semafori: coprirebbero il contenuto.
        let mon = MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(mon, &mut mi).as_bool() {
            let m = mi.rcMonitor;
            let covers = fr.left <= m.left && fr.top <= m.top && fr.right >= m.right && fr.bottom >= m.bottom;
            let work_differs = mi.rcWork.left != m.left || mi.rcWork.top != m.top || mi.rcWork.right != m.right || mi.rcWork.bottom != m.bottom;
            if covers && (work_differs || !zoomed) {
                return None;
            }
        }
        let native_w = cb.right - cb.left;
        let three = native_w as f32 >= 100.0 * scale;
        let kinds = vec![(Kind::Close, true), (Kind::Min, three && style & WS_MINIMIZEBOX.0 != 0), (Kind::Zoom, three && style & WS_MAXIMIZEBOX.0 != 0)];
        Some(Info {
            hwnd: h,
            nx: wr.left + cb.left,
            ny: wr.top + cb.top,
            native_w,
            h: cb.bottom - cb.top,
            frame_left: fr.left,
            frame_right: fr.right,
            frame_top: fr.top,
            scale,
            kinds,
            three,
            zoomed,
            topmost: ex & WS_EX_TOPMOST.0 != 0,
            explorer: class == "CabinetWClass",
        })
    }
}

unsafe extern "system" fn collect(h: HWND, data: LPARAM) -> BOOL {
    // SAFETY: `data` e' il puntatore al Vec di `sync`, valido per tutta l'enumerazione.
    let out = unsafe { &mut *(data.0 as *mut Vec<Info>) };
    if let Some(i) = inspect(h) {
        out.push(i);
    }
    BOOL(1)
}

/// Colore della barra del titolo, letto dallo schermo subito a sinistra dei tasti di Windows.
fn sample_bg(x: i32, y: i32) -> [u8; 3] {
    unsafe {
        let dc = GetDC(None);
        let c = GetPixel(dc, x, y);
        ReleaseDC(None, dc);
        if c.0 == 0xFFFF_FFFF {
            return [32, 32, 32];
        }
        [(c.0 & 0xFF) as u8, ((c.0 >> 8) & 0xFF) as u8, ((c.0 >> 16) & 0xFF) as u8]
    }
}

thread_local! {
    static STRIP: std::cell::RefCell<HashMap<isize, (i32, Instant)>> = std::cell::RefCell::new(HashMap::new());
}

/// Altezza della barra in alto, letta dallo schermo: si scende finche' il colore resta lo stesso. Mai oltre `max`.
/// Il risultato resta in memoria 2 secondi per finestra.
fn strip_height(key: isize, x: i32, y: i32, max: i32, left: i32, right: i32) -> i32 {
    let cached = STRIP.with(|c| c.borrow().get(&key).filter(|(_, t)| t.elapsed() < Duration::from_secs(2)).map(|(h, _)| *h));
    if let Some(h) = cached {
        return h;
    }
    let x = x.max(left + 1).min(right - 1);
    let top = sample_bg(x, y + 2);
    let mut h = 0;
    for dy in 3..max {
        let c = sample_bg(x, y + dy);
        let d = (0..3).map(|i| (c[i] as i32 - top[i] as i32).abs()).max().unwrap_or(0);
        if d > 10 {
            break;
        }
        h = dy;
    }
    STRIP.with(|c| c.borrow_mut().insert(key, (h, Instant::now())));
    h
}

/// Disegna e mostra la finestrina (posizione e contenuto).
fn present(ov: &Ov) {
    unsafe {
        let (w, h) = (ov.layout.w.max(1), ov.layout.h.max(1));
        let pixels = render(&Paint {
            layout: &ov.layout,
            kinds: &ov.kinds,
            scale: ov.scale,
            bg: ov.bg,
            active: ov.active,
            zoomed: ov.zoomed,
            hover: ov.hover,
            down: ov.down,
            corner: ov.corner,
            capsule: !ov.kinds.is_empty() && !ov.corner.0,
        });
        let bmi = BITMAPINFO { bmiHeader: BITMAPINFOHEADER { biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, biWidth: w, biHeight: -h, biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0, ..Default::default() }, ..Default::default() };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let mem = CreateCompatibleDC(None);
        let Ok(bmp) = CreateDIBSection(Some(mem), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) else {
            let _ = DeleteDC(mem);
            return;
        };
        if !bits.is_null() {
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u32, pixels.len().min((w * h) as usize));
        }
        let old = SelectObject(mem, HGDIOBJ(bmp.0));
        let dst = POINT { x: ov.x, y: ov.y };
        let size = SIZE { cx: w, cy: h };
        let src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
        let _ = UpdateLayeredWindow(ov.hwnd, None, Some(&dst), Some(&size), Some(mem), Some(&src), COLORREF(0), Some(&blend), ULW_ALPHA);
        SelectObject(mem, old);
        let _ = DeleteObject(HGDIOBJ(bmp.0));
        let _ = DeleteDC(mem);
    }
}

fn create_overlay(info: &Info) -> Option<HWND> {
    unsafe {
        let hi = GetModuleHandleW(None).ok()?;
        let ex = WS_EX_LAYERED.0 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | if info.topmost { WS_EX_TOPMOST.0 } else { 0 };
        CreateWindowExW(WINDOW_EX_STYLE(ex), w!("EosOSLights"), w!(""), WS_POPUP, info.nx, info.ny, 10, 10, None, None, Some(hi.into()), None).ok()
    }
}

fn destroy_all(st: &mut State) {
    for (_, ov) in st.ovs.drain() {
        unsafe {
            let _ = DestroyWindow(ov.hwnd);
        }
    }
    st.by_overlay.clear();
}

fn style_explorer(st: &mut State, info: &Info, want: bool) {
    let key = info.hwnd.0 as isize;
    unsafe {
        if want && !st.styled.contains(&key) {
            let none: u32 = 0xFFFF_FFFE; // DWMWA_COLOR_NONE: niente bordo
            let round: u32 = 2; // DWMWCP_ROUND
            let _ = DwmSetWindowAttribute(info.hwnd, DWMWA_BORDER_COLOR, &none as *const _ as *const _, 4);
            let _ = DwmSetWindowAttribute(info.hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &round as *const _ as *const _, 4);
            st.styled.insert(key);
            if let Ok(mut t) = TOUCHED.lock() {
                t.push(key);
            }
        }
    }
}

fn unstyle_all(st: &mut State) {
    restore_all();
    st.styled.clear();
}

/// Rimette il bordo di Windows alle finestre di Esplora file toccate.
pub fn restore_all() {
    let list: Vec<isize> = TOUCHED.lock().map(|mut t| std::mem::take(&mut *t)).unwrap_or_default();
    for k in list {
        let h = HWND(k as *mut _);
        unsafe {
            let default: u32 = 0xFFFF_FFFF; // DWMWA_COLOR_DEFAULT
            let _ = DwmSetWindowAttribute(h, DWMWA_BORDER_COLOR, &default as *const _ as *const _, 4);
        }
    }
}

/// Una passata: allinea le finestrine alle finestre vere (crea, sposta, ridisegna, ordina, toglie).
fn sync() {
    PENDING.store(false, Ordering::SeqCst);
    let cfg = crate::mcstore::load();
    let enabled = !cfg.lights_off && !crate::lockscreen::is_locked();
    let _ = ST.try_with(|cell| {
        let Ok(mut st) = cell.try_borrow_mut() else { return };
        if !enabled {
            destroy_all(&mut st);
            if cfg.finder_off {
                unstyle_all(&mut st);
            }
            return;
        }
        if cfg.finder_off && !st.styled.is_empty() {
            unstyle_all(&mut st);
        }
        let mut infos: Vec<Info> = Vec::new();
        unsafe {
            let _ = EnumWindows(Some(collect), LPARAM(&mut infos as *mut _ as isize));
        }
        let fg = unsafe { GetForegroundWindow() };
        let right = !cfg.lights_left;
        let mut seen: HashSet<isize> = HashSet::new();
        for info in &infos {
            let base = info.hwnd.0 as isize;
            if info.explorer {
                style_explorer(&mut st, info, !cfg.finder_off);
            }
            let order = kinds_order(right, info.three);
            let kinds: Vec<(Kind, bool)> = order.iter().map(|k| (*k, info.kinds.iter().find(|(kk, _)| kk == k).is_none_or(|(_, u)| *u))).collect();
            let radius = if info.zoomed { 0.0 } else { 8.0 * info.scale };
            let active = fg == info.hwnd;
            // La finestrina parte dal bordo alto visibile (di 1 pixel sopra i tasti di Windows) per non lasciare una riga.
            let top = info.ny.min(info.frame_top);
            let extra = info.ny - top;
            // Le parti: i cerchi e, se stanno a sinistra, una finestrina liscia che copre i tasti di Windows a destra.
            // Barra piu' alta dei tasti (Chrome: la fila delle schede): i cerchi stanno al centro della barra vera.
            let strip = strip_height(base, info.nx - 3, info.ny, (info.h as f32 * 1.6) as i32, info.frame_left, info.frame_right);
            let main = layout(&order, info.scale, right, info.native_w, (info.h + extra).max(strip + extra));
            let mut parts: Vec<Part> = vec![(base, if right { info.nx } else { info.frame_left }, main, kinds, (right, radius))];
            if !right {
                parts.push((base ^ COVER_BIT, info.nx, Layout { w: info.native_w, h: info.h + extra, r: 0.0, centers: Vec::new() }, Vec::new(), (true, radius)));
            }
            for (key, x, lay, kinds, corner) in parts {
                seen.insert(key);
                let y = top;
                let existing = st.ovs.contains_key(&key);
                if !existing {
                    let Some(hwnd) = create_overlay(info) else { continue };
                    st.by_overlay.insert(hwnd.0 as isize, key);
                    st.ovs.insert(
                        key,
                        Ov { hwnd, target: info.hwnd, x, y, layout: lay.clone(), kinds: kinds.clone(), scale: info.scale, bg: [32, 32, 32], active, zoomed: info.zoomed, corner, hover: false, down: None, sampled: Instant::now() - RESAMPLE, tracking: false, trect: RECT::default() },
                    );
                }
                let State { ovs, by_overlay, .. } = &mut *st;
                let Some(ov) = ovs.get_mut(&key) else { continue };
                // Il colore della barra: a ogni cambio di aspetto o ogni tanto.
                let mut changed = !existing;
                let moved = ov.x != x || ov.y != y;
                if ov.layout != lay || ov.kinds != kinds || ov.active != active || ov.zoomed != info.zoomed || ov.corner != corner || ov.scale != info.scale {
                    changed = true;
                }
                if changed || ov.sampled.elapsed() >= RESAMPLE {
                    let bg = sample_bg((info.nx - 3).max(info.frame_left + 1).min(info.frame_right - 1), info.ny + 2);
                    if bg != ov.bg {
                        changed = true;
                    }
                    ov.bg = bg;
                    ov.sampled = Instant::now();
                }
                ov.x = x;
                ov.y = y;
                if moved && !changed {
                    // Solo spostata: basta spostare la finestrina, senza ridisegnarla.
                    unsafe {
                        let _ = SetWindowPos(ov.hwnd, None, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER);
                    }
                }
                ov.layout = lay;
                ov.kinds = kinds;
                ov.active = active;
                ov.zoomed = info.zoomed;
                ov.corner = corner;
                ov.scale = info.scale;
                unsafe {
                    let _ = GetWindowRect(info.hwnd, &mut ov.trect);
                }
                if changed {
                    present(ov);
                }
                // Ordine: la finestrina sta subito sopra la sua finestra (o sopra l'altra finestrina della stessa finestra).
                unsafe {
                    let next = GetWindow(ov.hwnd, GW_HWNDNEXT).ok();
                    let in_place = next == Some(info.hwnd) || next.is_some_and(|n| by_overlay.get(&(n.0 as isize)).is_some_and(|k| (*k & !COVER_BIT) == base));
                    if !in_place {
                        let prev = GetWindow(info.hwnd, GW_HWNDPREV).ok();
                        let after = match prev {
                            Some(p) if p != ov.hwnd => p,
                            Some(_) => info.hwnd,
                            None => {
                                if info.topmost {
                                    HWND_TOPMOST
                                } else {
                                    HWND_TOP
                                }
                            }
                        };
                        let _ = SetWindowPos(ov.hwnd, Some(after), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER);
                    }
                    if !IsWindowVisible(ov.hwnd).as_bool() {
                        let _ = ShowWindow(ov.hwnd, SW_SHOWNOACTIVATE);
                    }
                }
            }
        }
        // Via le finestrine delle finestre sparite (chiuse, ridotte, a schermo intero).
        let gone: Vec<isize> = st.ovs.keys().copied().filter(|k| !seen.contains(k)).collect();
        for k in gone {
            if let Some(ov) = st.ovs.remove(&k) {
                st.by_overlay.remove(&(ov.hwnd.0 as isize));
                unsafe {
                    let _ = DestroyWindow(ov.hwnd);
                }
            }
        }
        st.styled.retain(|k| seen.contains(k));
    });
}

/// Controllo leggero (ogni pochi millisecondi): la finestra vera si e' mossa o e' finita sopra la sua finestrina? Se si', passata completa.
fn needs_sync() -> bool {
    ST.try_with(|cell| {
        let Ok(st) = cell.try_borrow() else { return false };
        st.ovs.values().any(|ov| unsafe {
            let mut r = RECT::default();
            if GetWindowRect(ov.target, &mut r).is_err() {
                return true; // la finestra non c'e' piu'
            }
            if r != ov.trect {
                return true;
            }
            let next = GetWindow(ov.hwnd, GW_HWNDNEXT).ok();
            let base = ov.target.0 as isize;
            !(next == Some(ov.target) || next.is_some_and(|n| st.by_overlay.get(&(n.0 as isize)).is_some_and(|k| (*k & !COVER_BIT) == base)))
        })
    })
    .unwrap_or(false)
}

/// Chiede una passata subito (cambio di impostazione, evento delle finestre).
pub fn poke() {
    let h = HOST.load(Ordering::SeqCst);
    if h != 0 && !PENDING.swap(true, Ordering::SeqCst) {
        unsafe {
            let _ = PostMessageW(Some(HWND(h as *mut _)), WM_APP, WPARAM(0), LPARAM(0));
        }
    }
}

/// Sposta le finestrine di `target` di quanto si e' mossa la finestra. `false` se non c'e' nulla da spostare o e' cambiata la dimensione
/// (serve la passata completa).
fn nudge(target: HWND) -> bool {
    let key = target.0 as isize;
    ST.try_with(|cell| {
        let Ok(mut st) = cell.try_borrow_mut() else { return false };
        let mut any = false;
        for (k, ov) in st.ovs.iter_mut() {
            if (*k & !COVER_BIT) != key {
                continue;
            }
            any = true;
            let mut r = RECT::default();
            unsafe {
                if GetWindowRect(ov.target, &mut r).is_err() {
                    return false;
                }
            }
            let t = ov.trect;
            if r.right - r.left != t.right - t.left || r.bottom - r.top != t.bottom - t.top {
                return false;
            }
            let (dx, dy) = (r.left - t.left, r.top - t.top);
            if dx == 0 && dy == 0 {
                continue;
            }
            ov.x += dx;
            ov.y += dy;
            ov.trect = r;
            unsafe {
                let _ = SetWindowPos(ov.hwnd, None, ov.x, ov.y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER);
            }
        }
        any
    })
    .unwrap_or(false)
}

/// Durante il trascinamento (o il ridimensionamento) con il mouse: a ogni immagine del desktop (`DwmFlush`) le finestrine vanno dove
/// e' la finestra, cosi' i tasti di Windows non fanno in tempo a vedersi. Al massimo 60 secondi.
fn follow_drag(target: HWND) {
    use windows::Win32::Graphics::Dwm::DwmFlush;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    let start = Instant::now();
    while unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } as u16 & 0x8000 != 0 && start.elapsed() < Duration::from_secs(60) {
        if !nudge(target) {
            sync();
        }
        unsafe {
            let _ = DwmFlush();
        }
    }
    poke();
}

unsafe extern "system" fn on_event(_h: HWINEVENTHOOK, event: u32, _hwnd: HWND, id_object: i32, id_child: i32, _t: u32, _time: u32) {
    // Solo gli eventi delle finestre (non del cursore o dei figli): quello del cursore arriva a ogni movimento del mouse.
    if event == 0x800B && (id_object != 0 || id_child != 0) {
        return;
    }
    // Inizio del trascinamento: si segue la finestra a ogni immagine del desktop finche' il tasto del mouse e' premuto.
    if event == 0x000A {
        follow_drag(_hwnd);
        return;
    }
    // Spostamento puro di una finestra con i semafori: si spostano subito le sue finestrine, senza la passata completa.
    if event == 0x800B && nudge(_hwnd) {
        return;
    }
    poke();
}

unsafe extern "system" fn host_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_TIMER => {
            // 1 = passata completa ogni TICK_MS; 2 = controllo leggero molto piu' frequente.
            if wp.0 == 2 {
                if needs_sync() {
                    sync();
                }
            } else {
                sync();
            }
            LRESULT(0)
        }
        WM_APP => {
            sync();
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
    }
}

fn lparam_xy(lp: LPARAM) -> (f32, f32) {
    ((lp.0 & 0xFFFF) as i16 as f32, ((lp.0 >> 16) & 0xFFFF) as i16 as f32)
}

unsafe extern "system" fn ov_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => LRESULT(3), // MA_NOACTIVATE: il clic non ruba il focus
        WM_MOUSEMOVE | WM_MOUSELEAVE | WM_LBUTTONDOWN | WM_LBUTTONUP => {
            let _ = ST.try_with(|cell| {
                let Ok(mut st) = cell.try_borrow_mut() else { return };
                let Some(key) = st.by_overlay.get(&(hwnd.0 as isize)).copied() else { return };
                let Some(ov) = st.ovs.get_mut(&key) else { return };
                let (x, y) = lparam_xy(lp);
                let hot = hit(&ov.layout, x, y, ov.scale);
                let before = (ov.hover, ov.down);
                match msg {
                    WM_MOUSEMOVE => {
                        ov.hover = true;
                        if !ov.tracking {
                            ov.tracking = true;
                            let mut t = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: hwnd, dwHoverTime: 0 };
                            unsafe {
                                let _ = TrackMouseEvent(&mut t);
                            }
                        }
                    }
                    WM_MOUSELEAVE => {
                        ov.hover = false;
                        ov.tracking = false;
                    }
                    WM_LBUTTONDOWN => {
                        ov.down = hot;
                        unsafe {
                            SetCapture(hwnd);
                        }
                    }
                    _ => {
                        unsafe {
                            let _ = ReleaseCapture();
                        }
                        let pressed = ov.down.take();
                        if let (Some(p), Some(h)) = (pressed, hot)
                            && p == h
                            && let Some(kind) = ov.layout.centers.get(h).map(|c| c.0)
                            && ov.kinds.iter().any(|(k, usable)| *k == kind && *usable)
                        {
                            let cmd = command_for(kind, ov.zoomed);
                            unsafe {
                                let _ = PostMessageW(Some(ov.target), WM_SYSCOMMAND, WPARAM(cmd as usize), LPARAM(0));
                            }
                        }
                    }
                }
                if (ov.hover, ov.down) != before {
                    present(ov);
                }
            });
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
    }
}

/// Avvia il thread dei semafori (finestre, hook degli eventi delle finestre, timer).
pub fn start() {
    std::thread::spawn(|| unsafe {
        let Ok(hi) = GetModuleHandleW(None) else { return };
        let arrow = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let ov_class = WNDCLASSW { lpfnWndProc: Some(ov_proc), hInstance: hi.into(), lpszClassName: w!("EosOSLights"), hCursor: arrow, ..Default::default() };
        let host_class = WNDCLASSW { lpfnWndProc: Some(host_proc), hInstance: hi.into(), lpszClassName: w!("EosOSLightsHost"), ..Default::default() };
        RegisterClassW(&ov_class);
        RegisterClassW(&host_class);
        let Ok(host) = CreateWindowExW(WINDOW_EX_STYLE(0), w!("EosOSLightsHost"), w!(""), WS_POPUP, 0, 0, 0, 0, Some(HWND_MESSAGE), None, Some(hi.into()), None) else {
            return;
        };
        HOST.store(host.0 as isize, Ordering::SeqCst);
        SetTimer(Some(host), 1, TICK_MS, None);
        SetTimer(Some(host), 2, FAST_MS, None);
        let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
        let hooks: Vec<HWINEVENTHOOK> = [(0x0003u32, 0x0003u32), (0x000A, 0x000A), (0x0016, 0x0017), (0x800B, 0x800B), (0x8002, 0x8004)]
            .iter()
            .map(|(a, b)| SetWinEventHook(*a, *b, None, Some(on_event), 0, 0, flags))
            .collect();
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        for h in hooks {
            if !h.0.is_null() {
                let _ = UnhookWinEvent(h);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_follows_the_chosen_side() {
        assert_eq!(kinds_order(false, true), vec![Kind::Close, Kind::Min, Kind::Zoom]);
        assert_eq!(kinds_order(true, true), vec![Kind::Min, Kind::Zoom, Kind::Close]);
        assert_eq!(kinds_order(false, false), vec![Kind::Close]);
        assert_eq!(kinds_order(true, false), vec![Kind::Close]);
    }

    #[test]
    fn left_layout_is_compact_and_right_layout_fills_the_native_buttons() {
        let k = kinds_order(false, true);
        let l = layout(&k, 1.0, false, 138, 32);
        assert_eq!(l.centers.len(), 3);
        assert!(l.w < 100, "{}", l.w);
        assert!(l.centers[0].1 < l.centers[1].1 && l.centers[1].1 < l.centers[2].1);
        let r = layout(&kinds_order(true, true), 1.0, true, 138, 32);
        assert_eq!(r.w, 138);
        let last = r.centers.last().map(|c| c.1).unwrap_or(0.0);
        assert!(last < 138.0 && last > 69.0);
        // scalare con lo zoom di Windows
        assert!(layout(&k, 2.0, false, 276, 64).w > l.w);
    }

    #[test]
    fn single_close_fits_in_the_native_width() {
        let l = layout(&kinds_order(true, false), 1.0, true, 46, 32);
        assert_eq!(l.w, 46);
        assert_eq!(l.centers.len(), 1);
    }

    #[test]
    fn hit_test_finds_the_circle_with_a_little_slack() {
        let l = layout(&kinds_order(false, true), 1.0, false, 138, 32);
        let (_, cx, cy) = l.centers[1];
        assert_eq!(hit(&l, cx, cy, 1.0), Some(1));
        assert_eq!(hit(&l, cx + l.r + 1.0, cy, 1.0), Some(1));
        assert_eq!(hit(&l, cx, cy + 40.0, 1.0), None);
        assert_eq!(hit(&l, -50.0, 5.0, 1.0), None);
    }

    #[test]
    fn commands_match_the_windows_buttons() {
        assert_eq!(command_for(Kind::Close, false), SC_CLOSE);
        assert_eq!(command_for(Kind::Min, true), SC_MINIMIZE);
        assert_eq!(command_for(Kind::Zoom, false), SC_MAXIMIZE);
        assert_eq!(command_for(Kind::Zoom, true), SC_RESTORE);
    }

    fn paint(l: &Layout, kinds: &[(Kind, bool)], active: bool, hover: bool, corner: (bool, f32)) -> Vec<u32> {
        render(&Paint { layout: l, kinds, scale: 1.0, bg: [30, 30, 30], active, zoomed: false, hover, down: None, corner, capsule: false })
    }

    #[test]
    fn render_draws_the_three_colours_on_the_bar_colour() {
        let k = kinds_order(false, true);
        let l = layout(&k, 1.0, false, 138, 32);
        let kinds: Vec<(Kind, bool)> = k.iter().map(|x| (*x, true)).collect();
        let px = paint(&l, &kinds, true, false, (false, 0.0));
        assert_eq!(px.len(), (l.w * l.h) as usize);
        let at = |cx: f32, cy: f32| px[(cy as usize) * l.w as usize + cx as usize] & 0xFFFFFF;
        assert_eq!(at(l.centers[0].1, l.centers[0].2), 0xFF5F57);
        assert_eq!(at(l.centers[1].1, l.centers[1].2), 0xFEBC2E);
        assert_eq!(at(l.centers[2].1, l.centers[2].2), 0x28C840);
        // fuori dai cerchi: il colore della barra, opaco
        assert_eq!(px[0], 0xFF1E1E1E);
    }

    #[test]
    fn inactive_or_unusable_circles_are_grey() {
        let k = kinds_order(false, true);
        let l = layout(&k, 1.0, false, 138, 32);
        let kinds: Vec<(Kind, bool)> = vec![(Kind::Close, true), (Kind::Min, false), (Kind::Zoom, true)];
        let px = paint(&l, &kinds, true, false, (false, 0.0));
        let at = |i: usize| px[(l.centers[i].2 as usize) * l.w as usize + l.centers[i].1 as usize] & 0xFFFFFF;
        assert_eq!(at(0), 0xFF5F57);
        assert_eq!(at(1), 0x5C5C60);
        let grey = paint(&l, &kinds, false, false, (false, 0.0));
        assert_eq!(grey[(l.centers[0].2 as usize) * l.w as usize + l.centers[0].1 as usize] & 0xFFFFFF, 0x5C5C60);
    }

    #[test]
    fn hover_adds_the_symbols() {
        let k = kinds_order(false, true);
        let l = layout(&k, 1.0, false, 138, 32);
        let kinds: Vec<(Kind, bool)> = k.iter().map(|x| (*x, true)).collect();
        let plain = paint(&l, &kinds, true, false, (false, 0.0));
        let hov = paint(&l, &kinds, true, true, (false, 0.0));
        assert_ne!(plain, hov);
        // il centro del cerchio rosso con la croce e' piu' scuro
        let i = (l.centers[0].2 as usize) * l.w as usize + l.centers[0].1 as usize;
        assert!((hov[i] & 0xFF) < (plain[i] & 0xFF));
    }

    #[test]
    fn rounded_corner_is_transparent_outside_the_window_shape() {
        let k = kinds_order(true, true);
        let l = layout(&k, 1.0, true, 138, 32);
        let kinds: Vec<(Kind, bool)> = k.iter().map(|x| (*x, true)).collect();
        let px = paint(&l, &kinds, true, false, (true, 8.0));
        assert_eq!(px[l.w as usize - 1] >> 24, 0, "l'angolo in alto a destra e' fuori dalla finestra");
        assert_eq!(px[(l.w / 2) as usize] >> 24, 255);
    }

    #[test]
    fn capsule_has_transparent_corners_and_keeps_the_circle_colours() {
        let k = kinds_order(false, true);
        let l = layout(&k, 1.0, false, 138, 32);
        let kinds: Vec<(Kind, bool)> = k.iter().map(|x| (*x, true)).collect();
        let px = render(&Paint { layout: &l, kinds: &kinds, scale: 1.0, bg: [30, 30, 30], active: true, zoomed: false, hover: false, down: None, corner: (false, 8.0), capsule: true });
        assert_eq!(px[0] >> 24, 0, "fuori dalla capsula si vede l'app");
        let c = l.centers[0];
        assert_eq!(px[(c.2 as usize) * l.w as usize + c.1 as usize], 0xFFFF5F57);
        // fra due cerchi: il fondo della capsula (scuro, quasi opaco)
        let mid = ((l.centers[0].1 + l.centers[1].1) / 2.0) as usize;
        assert!(px[(c.2 as usize) * l.w as usize + mid] >> 24 > 200);
    }

    #[test]
    fn shell_windows_are_skipped() {
        assert!(skipped_class("Progman"));
        assert!(skipped_class("Shell_TrayWnd"));
        assert!(!skipped_class("Notepad"));
    }
}
