//! Gestione delle finestre (fase «snap»): con una scorciatoia la finestra in primo piano si dispone in una metà, in un terzo o in
//! un quarto dell'area di lavoro del suo monitor, si massimizza, si centra, torna com'era o passa sull'altro monitor.
//!
//! Sono scorciatoie `RegisterHotKey` (come quelle del Dock), SPENTE di base: il modificatore (Ctrl+Alt, Ctrl+Maiusc o Alt+Maiusc) si
//! sceglie nelle Impostazioni. L'area di lavoro è quella di Windows: esclude la barra dei menu di EosOS e il Dock quando riservano spazio.
//!
//! Qui sta la geometria pura (testata: nessuna finestra, nessun monitor) e, in fondo, la parte che parla con Windows.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, IsZoomed,
    PostThreadMessageW, SetWindowPos, ShowWindow, GetMessageW, GWL_STYLE, MSG, SWP_NOACTIVATE, SWP_NOZORDER, SW_RESTORE, WM_APP, WM_HOTKEY,
};

// ------------------------------------------------------------------ geometria pura

/// Un rettangolo in pixel fisici: sinistra, alto, destra, basso (destra e basso esclusi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
}

impl Rect {
    pub const fn new(l: i32, t: i32, r: i32, b: i32) -> Rect {
        Rect { l, t, r, b }
    }
    pub fn w(&self) -> i32 {
        self.r - self.l
    }
    pub fn h(&self) -> i32 {
        self.b - self.t
    }
}

/// Dove va la finestra dentro l'area di lavoro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    LeftThird,
    CenterThird,
    RightThird,
    Maximize,
    Center,
}

/// Il punto che divide `len` pixel a partire da `start` in `parts` parti uguali: `i`-esimo confine. Gli ultimi pixel del resto
/// vanno all'ultima parte, così le parti coprono tutto senza buchi. Pura.
fn cut(start: i32, len: i32, i: i32, parts: i32) -> i32 {
    start + (i64::from(len) * i64::from(i) / i64::from(parts)) as i32
}

/// Il rettangolo VISIBILE che deve avere la finestra: `work` è l'area di lavoro del monitor, `cur` la finestra com'è adesso (serve
/// solo per «centra», che ne tiene la misura). Pura, testata.
pub fn target(layout: Layout, work: Rect, cur: Rect) -> Rect {
    let (w, h) = (work.w(), work.h());
    let x = |i, n| cut(work.l, w, i, n);
    let y = |i, n| cut(work.t, h, i, n);
    match layout {
        Layout::Maximize => work,
        Layout::Left => Rect::new(work.l, work.t, x(1, 2), work.b),
        Layout::Right => Rect::new(x(1, 2), work.t, work.r, work.b),
        Layout::Top => Rect::new(work.l, work.t, work.r, y(1, 2)),
        Layout::Bottom => Rect::new(work.l, y(1, 2), work.r, work.b),
        Layout::TopLeft => Rect::new(work.l, work.t, x(1, 2), y(1, 2)),
        Layout::TopRight => Rect::new(x(1, 2), work.t, work.r, y(1, 2)),
        Layout::BottomLeft => Rect::new(work.l, y(1, 2), x(1, 2), work.b),
        Layout::BottomRight => Rect::new(x(1, 2), y(1, 2), work.r, work.b),
        Layout::LeftThird => Rect::new(work.l, work.t, x(1, 3), work.b),
        Layout::CenterThird => Rect::new(x(1, 3), work.t, x(2, 3), work.b),
        Layout::RightThird => Rect::new(x(2, 3), work.t, work.r, work.b),
        Layout::Center => {
            let (cw, ch) = (cur.w().clamp(1, w.max(1)), cur.h().clamp(1, h.max(1)));
            let l = work.l + (w - cw) / 2;
            let t = work.t + (h - ch) / 2;
            Rect::new(l, t, l + cw, t + ch)
        }
    }
}

/// I bordi invisibili che Windows 10/11 mette attorno a molte finestre: `win` è il rettangolo della finestra, `frame` quello visibile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Insets {
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
}

/// Quanto la finestra «intera» sporge oltre la parte visibile. Mai negativo (una lettura strana vale zero). Pura, testata.
pub fn insets(win: Rect, frame: Rect) -> Insets {
    Insets { l: (frame.l - win.l).max(0), t: (frame.t - win.t).max(0), r: (win.r - frame.r).max(0), b: (win.b - frame.b).max(0) }
}

/// Il rettangolo da dare a `SetWindowPos` perché la parte VISIBILE finisca in `visible`. Pura, testata.
pub fn outer_for(visible: Rect, ins: Insets) -> Rect {
    Rect::new(visible.l - ins.l, visible.t - ins.t, visible.r + ins.r, visible.b + ins.b)
}

/// Sposta la finestra da un monitor all'altro mantenendo posizione e misura in proporzione, senza uscire dal monitor di arrivo. Pura, testata.
pub fn move_between(cur: Rect, from: Rect, to: Rect) -> Rect {
    let (fw, fh) = (from.w().max(1), from.h().max(1));
    let scale = |v: i32, origin_from: i32, origin_to: i32, len_from: i32, len_to: i32| -> i32 {
        origin_to + (i64::from(v - origin_from) * i64::from(len_to) / i64::from(len_from)) as i32
    };
    let l = scale(cur.l, from.l, to.l, fw, to.w());
    let t = scale(cur.t, from.t, to.t, fh, to.h());
    let w = (i64::from(cur.w()) * i64::from(to.w()) / i64::from(fw)) as i32;
    let h = (i64::from(cur.h()) * i64::from(to.h()) / i64::from(fh)) as i32;
    let (w, h) = (w.clamp(1, to.w().max(1)), h.clamp(1, to.h().max(1)));
    let l = l.clamp(to.l, (to.r - w).max(to.l));
    let t = t.clamp(to.t, (to.b - h).max(to.t));
    Rect::new(l, t, l + w, t + h)
}

/// Il monitor che segue (`delta` = 1) o precede (-1) quello corrente, in giro. `None` se c'è un solo monitor. Pura, testata.
pub fn next_index(count: usize, current: usize, delta: i32) -> Option<usize> {
    if count < 2 {
        return None;
    }
    Some((current as i64 + i64::from(delta)).rem_euclid(count as i64) as usize)
}

/// Va ricordata la posizione di prima? Sì se la finestra non è quella che abbiamo appena disposto noi (se l'utente l'ha spostata
/// o ridimensionata in mezzo, quella di adesso è la nuova «originale»). Pura, testata.
pub fn should_remember(last_applied: Option<Rect>, cur: Rect) -> bool {
    last_applied != Some(cur)
}

/// La finestra si può disporre? Serve il bordo ridimensionabile (`WS_THICKFRAME`): niente finestre a misura fissa, né finestre di
/// sistema (desktop, barra). Pura, testata.
pub fn snappable(style: u32, class: &str) -> bool {
    const WS_THICKFRAME: u32 = 0x0004_0000;
    style & WS_THICKFRAME != 0 && !matches!(class, "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd" | "XamlExplorerHostIslandWindow")
}

/// Che cosa fa una scorciatoia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Snap(Layout),
    /// Torna alla posizione di prima.
    Restore,
    /// Sposta sul monitor successivo (+1) o precedente (-1).
    Monitor(i32),
}

/// Le scorciatoie: (nome, tasto, azione). Il nome serve alle prove. Niente freccia in su (è di Mission Control) e niente numeri (sono
/// quelli del Dock). Pura, testata.
pub const KEYS: &[(&str, u32, Action)] = &[
    ("left", 0x25, Action::Snap(Layout::Left)),
    ("right", 0x27, Action::Snap(Layout::Right)),
    ("top", 0x54, Action::Snap(Layout::Top)),         // T
    ("bottom", 0x42, Action::Snap(Layout::Bottom)),   // B
    ("topleft", 0x55, Action::Snap(Layout::TopLeft)), // U
    ("topright", 0x49, Action::Snap(Layout::TopRight)), // I
    ("bottomleft", 0x4A, Action::Snap(Layout::BottomLeft)), // J
    ("bottomright", 0x4B, Action::Snap(Layout::BottomRight)), // K
    ("leftthird", 0x44, Action::Snap(Layout::LeftThird)), // D
    ("centerthird", 0x46, Action::Snap(Layout::CenterThird)), // F
    ("rightthird", 0x47, Action::Snap(Layout::RightThird)), // G
    ("maximize", 0x0D, Action::Snap(Layout::Maximize)), // Invio
    ("center", 0x43, Action::Snap(Layout::Center)),   // C
    ("restore", 0x08, Action::Restore),               // Backspace
    ("nextmonitor", 0x22, Action::Monitor(1)),        // PagGiù
    ("prevmonitor", 0x21, Action::Monitor(-1)),       // PagSu
];

/// L'azione di nome `name` (serve ai comandi di prova). Pura.
#[allow(dead_code)]
pub fn action_named(name: &str) -> Option<Action> {
    KEYS.iter().find(|k| k.0 == name).map(|k| k.2)
}

/// Le scelte del modificatore: (id, etichetta). «off» è la scelta di base.
pub const MODES: &[(&str, &str)] = &[("off", "Disattivate"), ("ctrl_alt", "Ctrl+Alt+…"), ("ctrl_shift", "Ctrl+Maiusc+…"), ("alt_shift", "Alt+Maiusc+…")];

/// La modalità effettiva dato il valore salvato (vuoto = spente). `EOSOS_SNAP_KEYS=0` le spegne sempre. Pura, testata.
pub fn effective_mode(saved: &str, env: Option<&str>) -> &'static str {
    if env == Some("0") {
        return "off";
    }
    MODES.iter().find(|m| m.0 == saved).map_or("off", |m| m.0)
}

/// I modificatori della modalità (0 = spente). Pura, testata.
pub fn modifiers(mode: &str) -> u32 {
    match mode {
        "ctrl_alt" => MOD_CONTROL.0 | MOD_ALT.0,
        "ctrl_shift" => MOD_CONTROL.0 | MOD_SHIFT.0,
        "alt_shift" => MOD_ALT.0 | MOD_SHIFT.0,
        _ => 0,
    }
}

// ------------------------------------------------------------------ Windows

const BASE_ID: i32 = 0x0E60;
const RELOAD: u32 = WM_APP + 0x22;
static THREAD_ID: AtomicU32 = AtomicU32::new(0);
static REGISTERED: AtomicUsize = AtomicUsize::new(0);
static STARTED: OnceLock<()> = OnceLock::new();

/// Per ogni finestra toccata: (posizione di prima, ultima posizione che le abbiamo dato).
static MEMORY: Mutex<Option<HashMap<isize, (Rect, Rect)>>> = Mutex::new(None);
const MEMORY_MAX: usize = 64;

/// Quante combinazioni sono registrate adesso (per le prove).
#[allow(dead_code)]
pub fn registered_count() -> usize {
    REGISTERED.load(Ordering::Relaxed)
}

fn current_mode() -> &'static str {
    effective_mode(&crate::mcstore::load().snap_keys, std::env::var("EOSOS_SNAP_KEYS").ok().as_deref())
}

fn to_rect(r: RECT) -> Rect {
    Rect::new(r.left, r.top, r.right, r.bottom)
}

unsafe extern "system" fn collect_monitor(m: HMONITOR, _dc: HDC, _r: *mut RECT, data: LPARAM) -> BOOL {
    // SAFETY: `data` è il puntatore al Vec passato da `monitors()` e vale per tutta l'enumerazione.
    let out = unsafe { &mut *(data.0 as *mut Vec<(isize, Rect)>) };
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    if unsafe { GetMonitorInfoW(m, &mut mi) }.as_bool() {
        out.push((m.0 as isize, to_rect(mi.rcWork)));
    }
    BOOL(1)
}

/// I monitor con la loro area di lavoro, da sinistra a destra (a pari sinistra, dall'alto).
fn monitors() -> Vec<(isize, Rect)> {
    let mut v: Vec<(isize, Rect)> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(collect_monitor), LPARAM(&mut v as *mut _ as isize));
    }
    v.sort_by_key(|(_, r)| (r.l, r.t));
    v
}

fn class_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// La finestra si può toccare? Visibile, di un altro processo (mai quelle di EosOS), con il bordo ridimensionabile.
fn eligible(hwnd: HWND) -> bool {
    unsafe {
        if hwnd.0.is_null() || !IsWindow(Some(hwnd)).as_bool() || !IsWindowVisible(hwnd).as_bool() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == std::process::id() {
            return false;
        }
        snappable(GetWindowLongPtrW(hwnd, GWL_STYLE) as u32, &class_of(hwnd))
    }
}

/// La parte visibile della finestra e i suoi bordi invisibili.
fn visible_and_insets(hwnd: HWND) -> Option<(Rect, Insets)> {
    let mut win = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut win) }.ok()?;
    let mut frame = RECT::default();
    let ok = unsafe { DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut frame as *mut _ as *mut _, std::mem::size_of::<RECT>() as u32) }.is_ok();
    let win = to_rect(win);
    // Senza la cornice visibile (finestre senza DWM) si usa la finestra intera: nessun bordo invisibile da compensare.
    let frame = if ok { to_rect(frame) } else { win };
    Some((frame, insets(win, frame)))
}

fn place(hwnd: HWND, visible: Rect, ins: Insets) {
    let o = outer_for(visible, ins);
    unsafe {
        let _ = SetWindowPos(hwnd, None, o.l, o.t, o.w(), o.h(), SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

/// Esegue `action` sulla finestra `hwnd`. `false` se non si può (finestra non adatta, un solo monitor per lo spostamento, nulla da ripristinare).
pub fn apply(hwnd: HWND, action: Action) -> bool {
    if !eligible(hwnd) {
        return false;
    }
    let key = hwnd.0 as isize;
    let mons = monitors();
    if mons.is_empty() {
        return false;
    }
    // Una finestra ridotta a icona o massimizzata si riporta prima alla misura normale: poi se ne legge la posizione.
    unsafe {
        if IsIconic(hwnd).as_bool() || IsZoomed(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
    }
    let Some((cur, ins)) = visible_and_insets(hwnd) else { return false };
    let cur_mon = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) }.0 as isize;
    let idx = mons.iter().position(|(m, _)| *m == cur_mon).unwrap_or(0);
    let work = mons[idx].1;

    let mut guard = MEMORY.lock().unwrap_or_else(|e| e.into_inner());
    let mem = guard.get_or_insert_with(HashMap::new);
    if action == Action::Restore {
        return match mem.remove(&key) {
            Some((orig, _)) => {
                place(hwnd, orig, ins);
                true
            }
            None => false,
        };
    }
    let dest = match action {
        Action::Snap(layout) => target(layout, work, cur),
        Action::Monitor(delta) => {
            let Some(n) = next_index(mons.len(), idx, delta) else { return false };
            move_between(cur, work, mons[n].1)
        }
        Action::Restore => return false,
    };
    let last = mem.get(&key).map(|v| v.1);
    let original = if should_remember(last, cur) { cur } else { mem.get(&key).map_or(cur, |v| v.0) };
    if mem.len() >= MEMORY_MAX && !mem.contains_key(&key) {
        mem.clear();
    }
    mem.insert(key, (original, dest));
    drop(guard);
    place(hwnd, dest, ins);
    true
}

/// Esegue `action` sulla finestra in primo piano.
pub fn apply_foreground(action: Action) -> bool {
    apply(unsafe { GetForegroundWindow() }, action)
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
        for (i, k) in KEYS.iter().enumerate() {
            if unsafe { RegisterHotKey(None, BASE_ID + i as i32, HOT_KEY_MODIFIERS(mods) | MOD_NOREPEAT, k.1) }.is_ok() {
                ok += 1;
            }
        }
    }
    REGISTERED.store(ok, Ordering::Relaxed);
}

/// Avvia il thread delle scorciatoie (una volta).
pub fn start() {
    if STARTED.set(()).is_err() {
        return;
    }
    std::thread::spawn(|| {
        THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
        register_all();
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            if msg.message == WM_HOTKEY {
                let i = msg.wParam.0 as i32 - BASE_ID;
                if let Some(k) = usize::try_from(i).ok().and_then(|i| KEYS.get(i)) {
                    let action = k.2;
                    // Fuori dal ciclo dei messaggi: la disposizione enumera monitor e sposta finestre.
                    std::thread::spawn(move || {
                        apply_foreground(action);
                    });
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
            let _ = PostThreadMessageW(id, RELOAD, WPARAM(0), LPARAM(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Rect = Rect::new(0, 28, 1920, 1080); // area di lavoro: 1920 x 1052 sotto la barra dei menu
    const CUR: Rect = Rect::new(300, 200, 1100, 800);

    #[test]
    fn halves_and_quarters_tile_the_work_area_without_gaps() {
        use Layout::*;
        assert_eq!(target(Left, W, CUR), Rect::new(0, 28, 960, 1080));
        assert_eq!(target(Right, W, CUR), Rect::new(960, 28, 1920, 1080));
        assert_eq!(target(Top, W, CUR), Rect::new(0, 28, 1920, 554));
        assert_eq!(target(Bottom, W, CUR), Rect::new(0, 554, 1920, 1080));
        assert_eq!(target(TopLeft, W, CUR), Rect::new(0, 28, 960, 554));
        assert_eq!(target(TopRight, W, CUR), Rect::new(960, 28, 1920, 554));
        assert_eq!(target(BottomLeft, W, CUR), Rect::new(0, 554, 960, 1080));
        assert_eq!(target(BottomRight, W, CUR), Rect::new(960, 554, 1920, 1080));
        assert_eq!(target(Maximize, W, CUR), W);
    }

    #[test]
    fn thirds_cover_everything_even_when_the_width_does_not_divide() {
        let w = Rect::new(-1920, 0, -1920 + 1000, 600); // monitor a sinistra, 1000 non divisibile per 3
        let (a, b, c) = (target(Layout::LeftThird, w, CUR), target(Layout::CenterThird, w, CUR), target(Layout::RightThird, w, CUR));
        assert_eq!((a.l, c.r), (w.l, w.r));
        assert_eq!((a.r, b.r), (b.l, c.l), "le parti si toccano");
        assert!(a.w() >= 333 && b.w() >= 333 && c.w() >= 333 && a.w() + b.w() + c.w() == 1000);
        assert_eq!((a.t, a.b), (w.t, w.b));
    }

    #[test]
    fn odd_sizes_never_leave_a_gap_between_halves() {
        let w = Rect::new(0, 0, 1921, 1081);
        assert_eq!(target(Layout::Left, w, CUR).r, target(Layout::Right, w, CUR).l);
        assert_eq!(target(Layout::Top, w, CUR).b, target(Layout::Bottom, w, CUR).t);
        assert_eq!(target(Layout::Right, w, CUR).r, 1921);
    }

    #[test]
    fn center_keeps_the_size_and_never_exceeds_the_work_area() {
        // CUR è 800x600: al centro di W (1920x1052, alto a 28) sta a sinistra 560 e in alto 28 + 226.
        assert_eq!(target(Layout::Center, W, CUR), Rect::new(560, 254, 1360, 854));
        let big = Rect::new(0, 0, 5000, 4000);
        assert_eq!(target(Layout::Center, W, big), W, "una finestra piu grande dell'area si ferma all'area");
        let tiny = target(Layout::Center, W, Rect::new(10, 10, 10, 10));
        assert!(tiny.w() >= 1 && tiny.h() >= 1);
    }

    #[test]
    fn invisible_borders_are_compensated_so_the_visible_part_lands_exactly() {
        // Finestra di Windows 11: 7 px invisibili a sinistra, destra e in basso, niente in alto.
        let win = Rect::new(93, 100, 1107, 807);
        let frame = Rect::new(100, 100, 1100, 800);
        let ins = insets(win, frame);
        assert_eq!(ins, Insets { l: 7, t: 0, r: 7, b: 7 });
        assert_eq!(outer_for(Rect::new(0, 28, 960, 1080), ins), Rect::new(-7, 28, 967, 1087));
        // Una lettura strana (cornice fuori dalla finestra) vale zero, mai negativa.
        assert_eq!(insets(Rect::new(100, 100, 200, 200), Rect::new(90, 90, 210, 210)), Insets::default());
    }

    #[test]
    fn moving_to_another_monitor_keeps_the_proportions_and_stays_inside() {
        let from = Rect::new(0, 0, 1920, 1080);
        let to = Rect::new(-1920, 0, 0, 1080);
        // Metà sinistra del primo = metà sinistra del secondo.
        assert_eq!(move_between(Rect::new(0, 0, 960, 1080), from, to), Rect::new(-1920, 0, -960, 1080));
        // Verso un monitor più piccolo la finestra si riduce e non esce.
        let small = Rect::new(2000, 100, 2800, 700);
        let m = move_between(Rect::new(960, 540, 1920, 1080), from, small);
        assert!(m.l >= small.l && m.r <= small.r && m.t >= small.t && m.b <= small.b, "{m:?}");
        // Una finestra che sporgeva dal monitor di partenza rientra in quello di arrivo.
        let m = move_between(Rect::new(1800, 900, 2200, 1200), from, to);
        assert!(m.l >= to.l && m.r <= to.r && m.t >= to.t && m.b <= to.b, "{m:?}");
    }

    #[test]
    fn the_next_monitor_wraps_and_needs_at_least_two() {
        assert_eq!(next_index(1, 0, 1), None);
        assert_eq!(next_index(0, 0, 1), None);
        assert_eq!(next_index(2, 0, 1), Some(1));
        assert_eq!(next_index(2, 1, 1), Some(0));
        assert_eq!(next_index(3, 0, -1), Some(2));
    }

    #[test]
    fn the_previous_position_is_remembered_only_if_the_user_moved_the_window_in_between() {
        let before = Rect::new(10, 10, 500, 400);
        let applied = Rect::new(0, 28, 960, 1080);
        assert!(should_remember(None, before), "prima volta");
        assert!(!should_remember(Some(applied), applied), "e ancora dove l'abbiamo messa: l'originale resta quello di prima");
        assert!(should_remember(Some(applied), Rect::new(50, 50, 600, 500)), "spostata a mano: la posizione di adesso e la nuova originale");
    }

    #[test]
    fn only_resizable_normal_windows_are_touched() {
        const THICK: u32 = 0x0004_0000;
        assert!(snappable(THICK, "Notepad"));
        assert!(!snappable(0, "Notepad"), "misura fissa");
        for c in ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"] {
            assert!(!snappable(THICK, c), "{c}");
        }
    }

    #[test]
    fn shortcuts_are_unique_and_avoid_the_keys_of_mission_control_and_the_dock() {
        let mut vks: Vec<u32> = KEYS.iter().map(|k| k.1).collect();
        vks.sort_unstable();
        let n = vks.len();
        vks.dedup();
        assert_eq!(vks.len(), n, "due azioni sullo stesso tasto");
        let mut names: Vec<&str> = KEYS.iter().map(|k| k.0).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), n, "nomi doppi");
        for k in KEYS {
            assert_ne!(k.1, 0x26, "freccia in su: e di Mission Control ({})", k.0);
            assert!(!(0x31..=0x39).contains(&k.1), "i numeri sono del Dock ({})", k.0);
        }
        assert_eq!(action_named("left"), Some(Action::Snap(Layout::Left)));
        assert_eq!(action_named("restore"), Some(Action::Restore));
        assert_eq!(action_named("boh"), None);
    }

    #[test]
    fn the_shortcuts_are_off_unless_the_user_chose_a_modifier() {
        assert_eq!(effective_mode("", None), "off");
        assert_eq!(effective_mode("boh", None), "off");
        assert_eq!(effective_mode("ctrl_alt", None), "ctrl_alt");
        assert_eq!(effective_mode("alt_shift", None), "alt_shift");
        assert_eq!(effective_mode("ctrl_alt", Some("0")), "off", "«0» le spegne sempre");
        assert_eq!(modifiers("off"), 0);
        assert_eq!(modifiers("ctrl_alt"), MOD_CONTROL.0 | MOD_ALT.0);
        for (id, _) in MODES.iter().filter(|m| m.0 != "off") {
            assert_eq!(modifiers(id).count_ones(), 2, "{id}");
        }
    }
}
