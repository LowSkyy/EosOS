//! Finestra del Dock: una finestra Tauri trasparente, sempre in primo piano,
//! centrata in basso sul monitor primario.
//!
//! La finestra è fissa e più grande della capsula (margine sopra e ai lati per le
//! icone ingrandite e il rimbalzo): non cambia mai dimensione durante l'uso, così
//! non sfarfalla. Un ciclo a ~60 Hz legge il cursore e decide tre cose:
//!
//! - **zona cliccabile** (`SetWindowRgn`): finché il mouse è fuori è solo la
//!   capsula a riposo, quindi i clic sopra e ai lati arrivano alle app sotto;
//!   quando entra nella capsula diventa tutta la finestra, perché le icone
//!   ingrandite devono vedersi e poter essere cliccate;
//! - **auto-nascondi**: la finestra scorre sotto il bordo inferiore e riappare
//!   toccandolo; la parte fuori schermo è ritagliata, per non comparire su un
//!   monitor posto sotto;
//! - il resto (posizione e dimensione) segue monitor, scala e area di lavoro.
//!
//! Niente sfocatura dello sfondo: tutti gli effetti di Windows (acrilico, blur,
//! sfondo di sistema) riempiono l'intero rettangolo della finestra e ignorano il
//! ritaglio, lasciando angoli quadrati. Il vetro è quindi una tinta traslucida
//! con bordo luminoso (CSS) dentro una capsula perfetta. Una vera sfocatura
//! richiederebbe la Composition API (fase D5).
//!
//! Il Dock non è un appbar: non riserva spazio e non tocca la taskbar.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject, SetWindowRgn, HGDIOBJ, HRGN,
    RGN_AND,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowLongPtrW, GetWindowRect, IsWindowVisible, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    GWL_EXSTYLE, GWL_STYLE, SW_SHOWNOACTIVATE,
    SetWindowDisplayAffinity, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WS_CAPTION,
    WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU,
};

use crate::dockgeom::{
    at_bottom_edge, capsule_rect, capture_rect, is_active, reserve_height, slide_y, step_visibility,
    visible_height,
    window_rect, Fit, Rect, HIDE_DELAY_SECONDS, SHRINK_GRACE_SECONDS,
};
use crate::glass;

pub const LABEL: &str = "dock";
/// Stili di finestra che non attiva e non compare in Alt+Tab.
const NO_ACTIVATE_STYLES: isize = (WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0) as isize;
/// Stili di una finestra con cornice: il Dock non ne deve avere.
const CHROME_STYLES: isize =
    (WS_CAPTION.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0) as isize;
/// Contorno (px fisici) catturato attorno al Dock: la sfocatura ne ha bisogno.
const CAPTURE_MARGIN: i32 = 48;
const TICK: Duration = Duration::from_millis(16);
/// Ogni quanto si ricontrollano monitor, scala e area di lavoro.
const RECONCILE_EVERY: Duration = Duration::from_secs(1);

/// Un menu contestuale è aperto: il Dock non deve nascondersi sotto di esso.
static MENU_OPEN: AtomicBool = AtomicBool::new(false);
static AUTOHIDE: AtomicBool = AtomicBool::new(false);
static BLUR: AtomicBool = AtomicBool::new(true);
static RESERVE: AtomicBool = AtomicBool::new(true);

pub fn set_menu_open(open: bool) {
    MENU_OPEN.store(open, Ordering::Relaxed);
}

pub fn autohide() -> bool {
    AUTOHIDE.load(Ordering::Relaxed)
}

/// Inverte "nascondi automaticamente" e lo salva.
pub fn toggle_autohide() {
    let on = !autohide();
    AUTOHIDE.store(on, Ordering::Relaxed);
    crate::dockstore::set_autohide(on);
}

/// Impostazione "riserva spazio alle finestre".
pub fn reserve_enabled() -> bool {
    RESERVE.load(Ordering::Relaxed)
}

/// Inverte la riserva di spazio e la salva. Il resto lo fa il ciclo di controllo.
pub fn toggle_reserve() {
    let on = !reserve_enabled();
    RESERVE.store(on, Ordering::Relaxed);
    crate::dockstore::set_reserve(on);
}

/// `EOSOS_DOCK_RESERVE=0` spegne la riserva qualunque sia l'impostazione: serve agli
/// script di prova che misurano l'area di lavoro della sola barra in alto.
fn reserve_allowed() -> bool {
    static ALLOWED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ALLOWED.get_or_init(|| std::env::var("EOSOS_DOCK_RESERVE").map_or(true, |v| v != "0"))
}

pub fn blur_enabled() -> bool {
    BLUR.load(Ordering::Relaxed)
}

/// Inverte la sfocatura dello sfondo e la salva.
pub fn toggle_blur() {
    let on = !blur_enabled();
    BLUR.store(on, Ordering::Relaxed);
    crate::dockstore::set_blur(on);
}

/// Cosa catturare per la sfocatura: (finestra del Dock, rettangolo da catturare),
/// entrambi in pixel fisici. `None` se la sfocatura è spenta o il Dock è nascosto.
pub fn capture_info() -> Option<(Rect, Rect)> {
    if !blur_enabled() {
        return None;
    }
    let st = lock();
    let (win, geo) = (st.cur_win?, st.geo?);
    if st.p < 0.02 {
        return None;
    }
    capture_rect(win, geo.monitor, CAPTURE_MARGIN).map(|c| (win, c))
}

/// Monitor primario, come serve al ciclo (si ricontrolla ogni secondo).
#[derive(Clone, Copy)]
struct Geometry {
    work: Rect,
    monitor: Rect,
    scale: f64,
}

struct State {
    fit: Option<Fit>,
    geo: Option<Geometry>,
    hwnd: isize,
    /// La finestra è stata mostrata almeno una volta.
    shown: bool,
    /// Dimensione fisica già applicata alla finestra.
    size: (i32, i32),
    /// Il cursore è sul Dock (con isteresi).
    active: bool,
    left_at: Option<Instant>,
    idle_since: Option<Instant>,
    /// Rettangolo attuale della finestra (cambia durante lo scorrimento).
    cur_win: Option<Rect>,
    /// L'esclusione dalla cattura è già applicata (per la sfocatura).
    affinity: bool,
    /// Il Dock tiene riservata una fascia in basso (appbar).
    reserved: bool,
    /// Visibilità: 0 = nascosto, 1 = mostrato.
    p: f64,
    last_tick: Option<Instant>,
    applied_pos: Option<(i32, i32)>,
    applied_region: Option<[i32; 9]>,
}

static STATE: Mutex<State> = Mutex::new(State {
    fit: None,
    geo: None,
    hwnd: 0,
    shown: false,
    size: (0, 0),
    active: false,
    left_at: None,
    idle_since: None,
    cur_win: None,
    affinity: false,
    reserved: false,
    p: 1.0,
    last_tick: None,
    applied_pos: None,
    applied_region: None,
});

fn lock() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

fn create(app: &AppHandle) -> tauri::Result<()> {
    let (pos, scale) = match app.primary_monitor() {
        Ok(Some(m)) => {
            let p = m.work_area().position;
            (PhysicalPosition::new(p.x as f64, p.y as f64), m.scale_factor())
        }
        _ => (PhysicalPosition::new(0.0, 0.0), 1.0),
    };
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=dock".into()))
        .title("EosOS Dock")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .position(pos.x, pos.y)
        // Dimensione provvisoria: la finestra è nascosta finché il frontend non misura.
        .inner_size(1000.0 / scale, 260.0 / scale)
        .build()?;
    // Il Dock non deve mai rubare il focus né comparire in Alt+Tab.
    if let Ok(hwnd) = win.hwnd() {
        lock().hwnd = hwnd.0 as isize;
        unsafe {
            enforce_styles(hwnd);
        }
    }
    Ok(())
}

/// Stili che il Dock deve avere, cioè niente cornice (barra del titolo, menu di
/// sistema, pulsanti) e finestra che non si attiva, non ruba il focus e non sta in
/// Alt+Tab. Tauri li riapplica dopo la creazione e a certe operazioni: si
/// ricontrollano a ogni ciclo (due letture) e si correggono solo se serve. Senza
/// questo cliccare sul Dock toglieva il primo piano all'app che l'utente stava
/// usando e Windows disegnava la cornice in cima al Dock durante i trascinamenti.
unsafe fn enforce_styles(hwnd: HWND) {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let want_ex = (ex | NO_ACTIVATE_STYLES) & !WS_EX_APPWINDOW.0 as isize;
        if style & CHROME_STYLES == 0 && ex == want_ex {
            return;
        }
        // Si rilegge subito prima di scrivere: `show()` (altro thread) può aver aggiunto
        // WS_VISIBLE dopo la prima lettura, e scrivere il valore vecchio lo cancellerebbe
        // lasciando il Dock nascosto.
        let fresh = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(hwnd, GWL_STYLE, fresh & !CHROME_STYLES);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, want_ex);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

fn hwnd_of(raw: isize) -> HWND {
    HWND(raw as *mut _)
}

fn cursor() -> Option<(i32, i32)> {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p).ok()? };
    Some((p.x, p.y))
}

/// Regione cliccabile e visibile della finestra, in coordinate della finestra:
/// la capsula a riposo (cursore fuori) oppure tutta la finestra (cursore dentro),
/// sempre ritagliata alla parte che sta dentro lo schermo.
unsafe fn build_region(w: i32, h: i32, cap: Rect, radius: i32, active: bool, vis_h: i32) -> HRGN {
    unsafe {
        let base = if active {
            CreateRectRgn(0, 0, w, h)
        } else {
            let d = radius * 2;
            CreateRoundRectRgn(cap.x, cap.y, cap.x + cap.w + 1, cap.y + cap.h + 1, d, d)
        };
        let clip = CreateRectRgn(0, 0, w, vis_h.max(0));
        let out = CreateRectRgn(0, 0, 0, 0);
        CombineRgn(Some(out), Some(base), Some(clip), RGN_AND);
        let _ = DeleteObject(HGDIOBJ(base.0));
        let _ = DeleteObject(HGDIOBJ(clip.0));
        out
    }
}

/// Un passo di animazione e di aggiornamento della zona cliccabile.
fn tick() {
    let mut st = lock();
    let (Some(fit), Some(geo)) = (st.fit, st.geo) else { return };
    if !st.shown || st.hwnd == 0 {
        return;
    }
    unsafe {
        enforce_styles(hwnd_of(st.hwnd));
        // Dopo la prima visualizzazione il Dock è sempre visibile (l'auto-nascondi lo
        // sposta fuori schermo, non lo nasconde): se qualcosa lo ha nascosto, si rimostra.
        if !IsWindowVisible(hwnd_of(st.hwnd)).as_bool() {
            let _ = ShowWindow(hwnd_of(st.hwnd), SW_SHOWNOACTIVATE);
        }
    }
    let now = Instant::now();
    let dt = st.last_tick.map_or(0.0, |t| now.duration_since(t).as_secs_f64()).min(0.1);
    st.last_tick = Some(now);

    let shown = window_rect(geo.work, geo.scale, &fit);
    let hidden_y = geo.monitor.y + geo.monitor.h;
    let cap = capsule_rect(geo.scale, &fit);
    let cur_y = slide_y(shown.y, hidden_y, st.p);
    let win = Rect { y: cur_y, ..shown };
    let cap_abs = Rect { x: win.x + cap.x, y: win.y + cap.y, w: cap.w, h: cap.h };

    let cur = cursor();
    let active = cur.is_some_and(|c| is_active(c, win, cap_abs, st.active));
    if st.active && !active {
        st.left_at = Some(now);
    }
    if active {
        st.left_at = None;
    }
    st.active = active;

    // Visibilità: sempre visibile senza auto-nascondi; con auto-nascondi si
    // mostra se il cursore è sul Dock o tocca il bordo inferiore, o se un menu
    // è aperto, e si nasconde dopo un breve ritardo.
    let want = !autohide()
        || active
        || MENU_OPEN.load(Ordering::Relaxed)
        || cur.is_some_and(|c| at_bottom_edge(c, geo.monitor));
    let target = if want {
        st.idle_since = None;
        true
    } else {
        let since = *st.idle_since.get_or_insert(now);
        now.duration_since(since).as_secs_f64() < HIDE_DELAY_SECONDS
    };
    st.p = step_visibility(st.p, target, dt);

    let y = slide_y(shown.y, hidden_y, st.p);
    st.cur_win = Some(Rect { y, ..shown });
    // Con la sfocatura la finestra è esclusa dalla cattura (altrimenti si sfocherebbe da sola).
    let want_affinity = blur_enabled();
    if st.affinity != want_affinity {
        let applied = unsafe {
            SetWindowDisplayAffinity(
                hwnd_of(st.hwnd),
                if want_affinity { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE },
            )
        };
        if want_affinity && applied.is_err() {
            // Windows più vecchio della versione 2004: senza l'esclusione il Dock
            // sfocherebbe se stesso. Meglio niente sfocatura (non si salva: è un limite
            // del sistema, non una scelta).
            BLUR.store(false, Ordering::Relaxed);
        } else {
            st.affinity = want_affinity;
        }
    }
    // Si guarda la posizione REALE: quando cambia la scala il sistema sposta le finestre
    // per conto suo, e la cache di quanto già ordinato (`applied_pos`) non se ne accorge.
    let mut actual = RECT::default();
    let actual_pos = unsafe { GetWindowRect(hwnd_of(st.hwnd), &mut actual).ok() }.map(|_| (actual.left, actual.top));
    if st.applied_pos != Some((shown.x, y)) || actual_pos.is_some_and(|p| p != (shown.x, y)) {
        unsafe {
            let _ = SetWindowPos(
                hwnd_of(st.hwnd),
                None,
                shown.x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        st.applied_pos = Some((shown.x, y));
    }

    // La zona resta estesa un attimo dopo l'uscita: la capsula si rimpicciolisce
    // con una transizione e non va tagliata di colpo.
    let grace = st
        .left_at
        .is_some_and(|t| now.duration_since(t).as_secs_f64() < SHRINK_GRACE_SECONDS);
    let region_active = active || grace;
    let vis_h = visible_height(y, shown.h, geo.monitor.y + geo.monitor.h);
    let radius = (fit.radius * geo.scale).round() as i32;
    let key = [shown.w, shown.h, vis_h, region_active as i32, cap.x, cap.y, cap.w, cap.h, radius];
    if st.applied_region != Some(key) {
        unsafe {
            let rgn = build_region(shown.w, shown.h, cap, radius, region_active, vis_h);
            // Dopo la chiamata la regione appartiene al sistema.
            SetWindowRgn(hwnd_of(st.hwnd), Some(rgn), true);
        }
        st.applied_region = Some(key);
    }
}

/// Segue monitor, scala e area di lavoro; applica la dimensione della finestra e
/// la mostra la prima volta.
pub fn reconcile(app: &AppHandle) {
    let Some(fit) = lock().fit else { return };
    let Some(win) = app.get_webview_window(LABEL) else { return };
    let Ok(Some(mon)) = app.primary_monitor() else { return };
    let wa = mon.work_area();
    let work = Rect {
        x: wa.position.x,
        y: wa.position.y,
        w: wa.size.width as i32,
        h: wa.size.height as i32,
    };
    let (mp, ms) = (mon.position(), mon.size());
    let monitor = Rect { x: mp.x, y: mp.y, w: ms.width as i32, h: ms.height as i32 };
    let scale = mon.scale_factor();
    // Con la riserva di spazio il Dock è un appbar in basso: le finestre massimizzate
    // si fermano sopra di lui. Il bordo da cui si appoggia è però quello *disponibile*
    // (sopra la taskbar, se c'è), non l'area di lavoro, che già lo esclude: altrimenti
    // si sposterebbe verso l'alto a ogni aggiornamento.
    let hwnd = lock().hwnd;
    let want_reserve = reserve_enabled() && reserve_allowed() && !AUTOHIDE.load(Ordering::Relaxed);
    let mut work = work;
    if hwnd != 0 && want_reserve {
        let strip = reserve_height(scale, fit.cap_h);
        let rc = crate::appbar::reserve_bottom(
            hwnd_of(hwnd),
            RECT { left: monitor.x, top: monitor.y, right: monitor.x + monitor.w, bottom: monitor.y + monitor.h },
            strip,
        );
        work.h = (rc.bottom - work.y).max(1);
        lock().reserved = true;
    } else if lock().reserved {
        crate::appbar::release_bottom(hwnd_of(hwnd));
        lock().reserved = false;
    }
    let rect = window_rect(work, scale, &fit);

    // Si guarda anche la dimensione REALE della finestra, non solo quella già ordinata
    // (`st.size`): un cambio di scala o un'altra operazione di Tauri può lasciare la
    // finestra della misura vecchia senza che il Dock se ne accorga. Così si autoripara
    // entro un secondo, qualunque sia la causa.
    let actual = win.outer_size().ok().map(|s| (s.width as i32, s.height as i32));
    let (size_changed, first) = {
        let mut st = lock();
        st.geo = Some(Geometry { work, monitor, scale });
        let changed = st.size != (rect.w, rect.h) || actual.is_some_and(|a| a != (rect.w, rect.h));
        st.size = (rect.w, rect.h);
        if changed {
            // Nuova dimensione: posizione e zona vanno riapplicate.
            st.applied_pos = None;
            st.applied_region = None;
        }
        (changed, !st.shown)
    };
    if size_changed {
        let _ = win.set_size(PhysicalSize::new(rect.w as u32, rect.h as u32));
    }
    if first {
        // Prima visualizzazione: già nella posizione giusta, poi la mostra.
        glass::square_borderless(&win);
        let hidden = autohide();
        {
            let mut st = lock();
            st.p = if hidden { 0.0 } else { 1.0 };
            st.shown = true;
        }
        let y = slide_y(rect.y, monitor.y + monitor.h, if hidden { 0.0 } else { 1.0 });
        let _ = win.set_position(PhysicalPosition::new(rect.x, y));
        let _ = win.show();
        // La prima visualizzazione può spostare la finestra: il ciclo riafferma
        // posizione e regione.
        let mut st = lock();
        st.applied_pos = None;
        st.applied_region = None;
    }
}

/// Il frontend ha misurato la capsula e la zona.
pub fn fit(app: &AppHandle, fit: Fit) {
    lock().fit = Some(fit);
    reconcile(app);
}

/// Crea la finestra e avvia il ciclo di animazione.
pub fn start(app: &AppHandle) {
    let saved = crate::dockstore::settings();
    AUTOHIDE.store(saved.autohide, Ordering::Relaxed);
    RESERVE.store(saved.reserve, Ordering::Relaxed);
    BLUR.store(saved.blur, Ordering::Relaxed);
    if let Err(e) = create(app) {
        eprintln!("EosOS: creazione del Dock fallita: {e}");
        return;
    }
    // Due thread distinti. `reconcile` parla con la shell di Windows (appbar, monitor) e può
    // restare bloccato per secondi: per esempio `ABM_SETPOS` avvisa tutte le finestre del
    // cambio dell'area di lavoro e aspetta quelle lente. Se girasse nello stesso thread del
    // ciclo veloce, in quel tempo si fermerebbero anche la zona cliccabile e lo scorrimento.
    let slow = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(RECONCILE_EVERY);
        // Un panic non deve fermare il Dock.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reconcile(&slow)));
    });
    std::thread::spawn(|| loop {
        std::thread::sleep(TICK);
        let _ = std::panic::catch_unwind(tick);
    });
}
