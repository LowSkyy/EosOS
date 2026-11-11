//! La schermata di blocco di EosOS (in stile macOS): copre tutti i monitor con una pagina sua (sfondo sfocato, orologio, nome, campo di accesso).
//!
//! **Cosa e' e cosa non e':** non sostituisce la schermata di accesso di Windows (gira su un desktop protetto, `LogonUI`, che un programma
//! normale non puo' toccare): e' un blocco «da scrivania». Chi preme Ctrl+Alt+Canc o riavvia il PC la salta. Per questo, dopo un tempo
//! scelto dall'utente, EosOS chiama anche il blocco vero di Windows (`LockWorkStation`).
//!
//! **Come si sblocca:** le credenziali le verifica sempre Windows, mai EosOS: la password dell'account con `LogonUser` (EosOS non la salva
//! ne' la scrive in nessun registro), e Windows Hello/PIN, dove ci sono, con `UserConsentVerifier`. Un account senza password si sblocca con
//! un clic. Dopo 5 tentativi sbagliati si aspetta (5 s, poi il doppio fino a 5 minuti): l'attesa la impone Rust, non la pagina.
//!
//! **Come resta bloccato:** una finestra per monitor, sopra a tutto; un thread la rimette in primo piano ogni 250 ms; l'hook di tastiera (processo
//! separato, `mchook`) scarta Win, Alt+Tab, Alt+F4... (ma mai la via d'uscita di emergenza Ctrl+Alt+Maiusc+Esc, che chiude EosOS e ripristina
//! tutto); gli altri punti d'ingresso (Launcher, Mission Control, cattura, Dock...) non si aprono finche' e' bloccato (`is_locked`).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, WPARAM};
use windows::Win32::Security::{LogonUserW, LOGON32_LOGON_INTERACTIVE, LOGON32_PROVIDER_DEFAULT};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, RegisterHotKey, UnregisterHotKey, LASTINPUTINFO, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetMessageW, SetWindowPos, SystemParametersInfoW, HWND_TOPMOST, MSG, SPI_GETDESKWALLPAPER, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WM_HOTKEY,
};
use windows::core::HSTRING;

/// Le finestre di blocco si chiamano «lock-0», «lock-1»... una per monitor.
pub const LABEL_PREFIX: &str = "lock-";
const HOTKEY_ID: i32 = 0x4C0C;
/// Ogni quanto il thread di guardia rimette la schermata in primo piano.
const KEEP_EVERY: Duration = Duration::from_millis(250);
/// Ogni quanto si controllano inattivita' e tempo di blocco.
const TICK: Duration = Duration::from_secs(5);
/// Fino a quanti byte si legge lo sfondo del desktop.
const MAX_WALLPAPER: u64 = 48 * 1024 * 1024;

// ------------------------------------------------------------------ pure

/// Quanto bisogna aspettare dopo `fails` tentativi sbagliati di fila: niente fino al quarto, poi 5 s, 10, 20... fino a 5 minuti. Pura, testata.
pub fn backoff(fails: u32) -> Duration {
    if fails < 5 {
        return Duration::ZERO;
    }
    Duration::from_secs((5u64 << (fails - 5).min(6)).min(300))
}

/// L'inattivita' ha raggiunto `minutes` (0 = mai)? Pura, testata.
pub fn idle_due(idle: Duration, minutes: u32) -> bool {
    minutes > 0 && idle >= Duration::from_secs(u64::from(minutes) * 60)
}

/// E' ora di passare al blocco vero di Windows? `minutes` = 0 vuol dire mai. Pura, testata.
pub fn escalate_due(locked_for: Duration, minutes: u32) -> bool {
    minutes > 0 && locked_for >= Duration::from_secs(u64::from(minutes) * 60)
}

/// Il nome della finestra del monitor `index`. Pura, testata.
pub fn label_for(index: usize) -> String {
    format!("{LABEL_PREFIX}{index}")
}

/// E' la finestra di una schermata di blocco? Pura, testata.
pub fn is_lock_label(label: &str) -> bool {
    label.strip_prefix(LABEL_PREFIX).is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// La lettera nel cerchio dell'avatar: la prima lettera del nome, maiuscola (`?` se vuoto). Pura, testata.
pub fn initial_of(name: &str) -> String {
    name.trim().chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_else(|| "?".to_string())
}

// ------------------------------------------------------------------ stato

static LOCKED: AtomicBool = AtomicBool::new(false);
static FAILS: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// L'account non ha password: basta un clic.
    Blank,
    /// Serve la password (o PIN/Hello).
    Password,
}

struct Inner {
    since: Option<Instant>,
    retry_at: Option<Instant>,
    mode: Mode,
    hello: bool,
}

static STATE: Mutex<Inner> = Mutex::new(Inner { since: None, retry_at: None, mode: Mode::Password, hello: false });

fn st() -> std::sync::MutexGuard<'static, Inner> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Lo schermo e' bloccato adesso: gli altri punti d'ingresso (Launcher, Mission Control, cattura...) non devono aprirsi.
pub fn is_locked() -> bool {
    LOCKED.load(Ordering::Relaxed)
}

fn wait_left() -> u64 {
    st().retry_at.map(|t| t.saturating_duration_since(Instant::now()).as_secs_f64().ceil() as u64).unwrap_or(0)
}

// ------------------------------------------------------------------ Windows: account

fn user_and_domain() -> (String, String) {
    let user = std::env::var("USERNAME").unwrap_or_default();
    let domain = std::env::var("USERDOMAIN").unwrap_or_else(|_| ".".to_string());
    (user, if domain.is_empty() { ".".to_string() } else { domain })
}

/// Windows accetta questa password per l'utente corrente? (`LogonUser` interattivo: verifica e basta, non apre nessuna sessione.) Nelle build di prova
/// `EOSOS_TEST_LOCK_PASSWORD` sostituisce la verifica, per non dipendere dall'account di chi prova.
fn password_ok(password: &str) -> bool {
    #[cfg(feature = "test-hooks")]
    if let Ok(t) = std::env::var("EOSOS_TEST_LOCK_PASSWORD") {
        return password == t;
    }
    let (user, domain) = user_and_domain();
    if user.is_empty() {
        return false;
    }
    let mut token = HANDLE::default();
    let ok = unsafe {
        LogonUserW(&HSTRING::from(user), &HSTRING::from(domain), &HSTRING::from(password), LOGON32_LOGON_INTERACTIVE, LOGON32_PROVIDER_DEFAULT, &mut token)
    }
    .is_ok();
    if ok {
        unsafe {
            let _ = CloseHandle(token);
        }
    }
    ok
}

/// Windows Hello / PIN e' disponibile per verificare l'utente? Mai nelle build di prova, salvo `EOSOS_TEST_LOCK_HELLO=1`.
fn hello_available() -> bool {
    #[cfg(feature = "test-hooks")]
    {
        return std::env::var("EOSOS_TEST_LOCK_HELLO").is_ok_and(|v| v == "1");
    }
    #[cfg(not(feature = "test-hooks"))]
    {
        use windows::Security::Credentials::UI::{UserConsentVerifier, UserConsentVerifierAvailability};
        UserConsentVerifier::CheckAvailabilityAsync().and_then(|op| op.join()).is_ok_and(|a| a == UserConsentVerifierAvailability::Available)
    }
}

/// Chiede a Windows di verificare l'utente (PIN, impronta, volto): `true` solo se l'ha riconosciuto lui.
fn hello_verified() -> bool {
    #[cfg(feature = "test-hooks")]
    {
        return std::env::var("EOSOS_TEST_LOCK_HELLO").is_ok_and(|v| v == "1");
    }
    #[cfg(not(feature = "test-hooks"))]
    {
        use windows::Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier};
        UserConsentVerifier::RequestVerificationAsync(&HSTRING::from("Sblocca EosOS"))
            .and_then(|op| op.join())
            .is_ok_and(|r| r == UserConsentVerificationResult::Verified)
    }
}

fn display_name() -> String {
    use windows::Win32::Security::Authentication::Identity::{GetUserNameExW, NameDisplay};
    let mut buf = [0u16; 256];
    let mut len = buf.len() as u32;
    let ok = unsafe { GetUserNameExW(NameDisplay, Some(windows::core::PWSTR(buf.as_mut_ptr())), &mut len) };
    let name = if ok { String::from_utf16_lossy(&buf[..len as usize]) } else { String::new() };
    if name.trim().is_empty() { std::env::var("USERNAME").unwrap_or_default() } else { name }
}

// ------------------------------------------------------------------ comandi (per la pagina)

/// Cosa serve alla pagina per disegnarsi.
#[derive(Serialize)]
pub struct Info {
    pub name: String,
    pub initial: String,
    /// «blank» (un clic) o «password».
    pub mode: &'static str,
    pub hello: bool,
    /// Secondi da aspettare prima del prossimo tentativo.
    pub wait_secs: u64,
}

pub fn info() -> Info {
    let name = display_name();
    let (mode, hello) = {
        let s = st();
        (s.mode, s.hello)
    };
    Info { initial: initial_of(&name), name, mode: if mode == Mode::Blank { "blank" } else { "password" }, hello, wait_secs: wait_left() }
}

/// Esito di un tentativo.
#[derive(Serialize)]
pub struct Attempt {
    pub ok: bool,
    pub wait_secs: u64,
}

/// Prova una password. Dentro l'attesa imposta dopo troppi errori non si verifica nemmeno.
pub fn try_password(app: &AppHandle, password: &str) -> Attempt {
    if !is_locked() {
        return Attempt { ok: false, wait_secs: 0 };
    }
    let wait = wait_left();
    if wait > 0 {
        return Attempt { ok: false, wait_secs: wait };
    }
    if password_ok(password) {
        unlock(app);
        return Attempt { ok: true, wait_secs: 0 };
    }
    let fails = FAILS.fetch_add(1, Ordering::SeqCst) + 1;
    let pause = backoff(fails);
    st().retry_at = (!pause.is_zero()).then(|| Instant::now() + pause);
    Attempt { ok: false, wait_secs: pause.as_secs() }
}

/// Account senza password: un clic sblocca (solo se alla partenza il blocco ha verificato che la password e' vuota).
pub fn try_blank(app: &AppHandle) -> bool {
    if !is_locked() || st().mode != Mode::Blank {
        return false;
    }
    unlock(app);
    true
}

/// Windows Hello / PIN: la verifica la fa Windows (con la sua finestra); si chiama su un thread a parte perche' aspetta la risposta.
pub fn try_hello(app: &AppHandle) -> bool {
    if !is_locked() || !st().hello || wait_left() > 0 {
        return false;
    }
    if hello_verified() {
        unlock(app);
        return true;
    }
    false
}

/// I byte dello sfondo del desktop (vuoto se non si trova: la pagina usa un fondo suo).
pub fn wallpaper() -> Vec<u8> {
    let mut buf = [0u16; 520];
    let from_system = unsafe { SystemParametersInfoW(SPI_GETDESKWALLPAPER, buf.len() as u32, Some(buf.as_mut_ptr().cast()), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) }.is_ok();
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let mut path = if from_system { String::from_utf16_lossy(&buf[..end]) } else { String::new() };
    if path.is_empty() || !std::path::Path::new(&path).is_file() {
        path = std::env::var("APPDATA").map(|a| format!("{a}\\Microsoft\\Windows\\Themes\\TranscodedWallpaper")).unwrap_or_default();
    }
    match std::fs::metadata(&path) {
        Ok(m) if m.is_file() && m.len() <= MAX_WALLPAPER => std::fs::read(&path).unwrap_or_default(),
        _ => Vec::new(),
    }
}

// ------------------------------------------------------------------ blocco e sblocco

/// Il blocco che l'utente ha chiesto: quello di EosOS (di base) o, se l'ha spento nelle Impostazioni, quello di Windows.
pub fn request_lock(app: &AppHandle) {
    if crate::mcstore::load().lock_off {
        lock_windows();
    } else {
        lock(app);
    }
}

/// Il blocco vero di Windows (nelle build di prova scrive solo la riga di prova, senza bloccare il PC di chi prova).
pub fn lock_windows() {
    let _ = crate::launcheract::run_now("action:lock");
}

/// Blocca: una finestra per monitor, sopra a tutto. Se e' gia' bloccato non fa niente.
pub fn lock(app: &AppHandle) {
    if LOCKED.swap(true, Ordering::SeqCst) {
        return;
    }
    FAILS.store(0, Ordering::SeqCst);
    {
        let mut s = st();
        s.since = Some(Instant::now());
        s.retry_at = None;
        s.mode = Mode::Password;
        s.hello = false;
    }
    crate::mchook::set_locked(true);
    crate::panelwin::hide(app);
    crate::launcherwin::hide(app);
    let app = app.clone();
    // Si crea su un thread a parte: costruire finestre dal thread principale di Tauri puo' bloccarsi.
    std::thread::spawn(move || {
        let (mode, hello) = if cfg!(feature = "test-hooks") && std::env::var("EOSOS_TEST_LOCK_PASSWORD").is_ok() {
            (Mode::Password, hello_available())
        } else {
            (if password_ok("") { Mode::Blank } else { Mode::Password }, hello_available())
        };
        {
            let mut s = st();
            s.mode = mode;
            s.hello = hello;
        }
        create_windows(&app);
        // Uscita di sicurezza: se non e' comparsa nessuna finestra non si resta con i tasti scartati e niente da vedere: si toglie il blocco EosOS e
        // si passa a quello vero di Windows (l'effetto che l'utente voleva).
        if lock_hwnds(&app).is_empty() {
            eprintln!("blocco: nessuna finestra creata: passo al blocco di Windows");
            unlock(&app);
            lock_windows();
            return;
        }
        keep_on_top(&app);
    });
}

/// Sblocca: chiude le finestre e riabilita i tasti.
pub fn unlock(app: &AppHandle) {
    if !LOCKED.swap(false, Ordering::SeqCst) {
        return;
    }
    crate::mchook::set_locked(false);
    close_windows(app);
    let mut s = st();
    s.since = None;
    s.retry_at = None;
    FAILS.store(0, Ordering::SeqCst);
}

fn close_windows(app: &AppHandle) {
    for (label, win) in app.webview_windows() {
        if is_lock_label(&label) {
            let _ = win.destroy();
        }
    }
}

fn create_windows(app: &AppHandle) {
    let monitors = app.available_monitors().unwrap_or_default();
    let cursor = crate::capture::cursor_monitor();
    for (i, m) in monitors.iter().enumerate() {
        let label = label_for(i);
        if let Some(old) = app.get_webview_window(&label) {
            let _ = old.destroy();
        }
        let (pos, size) = (m.position(), m.size());
        let built = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(format!("index.html?view=lock&i={i}").into()))
            .title(format!("EosOS Blocco {i}"))
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .focused(false)
            .visible(false)
            .background_color(tauri::window::Color(0, 0, 0, 255))
            .build();
        let Ok(win) = built else {
            eprintln!("blocco: finestra {label} non creata");
            continue;
        };
        crate::chromeguard::protect(&win);
        let _ = win.set_position(PhysicalPosition::new(pos.x, pos.y));
        let _ = win.set_size(PhysicalSize::new(size.width.max(1), size.height.max(1)));
        let _ = win.show();
        if let Ok(hwnd) = win.hwnd() {
            unsafe {
                let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
            let under_cursor = cursor.x >= pos.x && cursor.x < pos.x + size.width as i32 && cursor.y >= pos.y && cursor.y < pos.y + size.height as i32;
            if under_cursor || i == 0 && monitors.len() == 1 {
                crate::dockact::focus(hwnd);
            }
        }
    }
    // Se il monitor col cursore non e' stato trovato, il focus va alla prima finestra.
    if let Some(first) = app.get_webview_window(&label_for(0))
        && let Ok(hwnd) = first.hwnd()
        && unsafe { GetForegroundWindow() } != hwnd
        && !lock_windows_have_focus(app)
    {
        crate::dockact::focus(hwnd);
    }
}

fn lock_hwnds(app: &AppHandle) -> Vec<(String, HWND)> {
    app.webview_windows().into_iter().filter(|(l, _)| is_lock_label(l)).filter_map(|(l, w)| w.hwnd().ok().map(|h| (l, h))).collect()
}

fn lock_windows_have_focus(app: &AppHandle) -> bool {
    let fg = unsafe { GetForegroundWindow() };
    lock_hwnds(app).iter().any(|(_, h)| *h == fg)
}

/// Finche' e' bloccato: rimette le finestre di blocco sopra a tutto (anche sopra a barra e Dock, che sono «sempre in primo piano») e il focus su una di esse.
fn keep_on_top(app: &AppHandle) {
    while is_locked() {
        std::thread::sleep(KEEP_EVERY);
        if !is_locked() {
            break;
        }
        let wins = lock_hwnds(app);
        if wins.is_empty() {
            // Le finestre non ci sono piu' (chiuse da fuori): si ricreano; se non si riesce, uscita di sicurezza come all'inizio.
            create_windows(app);
            if lock_hwnds(app).is_empty() {
                eprintln!("blocco: finestre non ricreate: passo al blocco di Windows");
                unlock(app);
                lock_windows();
                return;
            }
            continue;
        }
        for (_, h) in &wins {
            unsafe {
                let _ = SetWindowPos(*h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
        if !lock_windows_have_focus(app)
            && let Some((_, h)) = wins.iter().min_by_key(|(l, _)| l.clone())
        {
            crate::dockact::focus(*h);
        }
    }
}

// ------------------------------------------------------------------ inattivita', tempo di blocco e scorciatoia

fn idle_now() -> Duration {
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    if unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        Duration::from_millis(u64::from(unsafe { GetTickCount() }.wrapping_sub(info.dwTime)))
    } else {
        Duration::ZERO
    }
}

fn tick(app: &AppHandle) {
    let s = crate::mcstore::load();
    if is_locked() {
        let since = st().since;
        if let Some(t) = since
            && escalate_due(t.elapsed(), s.lock_windows_minutes())
        {
            // Passa il testimone a Windows: la schermata EosOS si toglie (altrimenti, dopo l'accesso di Windows, ne troveresti un'altra).
            unlock(app);
            lock_windows();
        }
    } else if !s.lock_off && idle_due(idle_now(), s.lock_idle) {
        lock(app);
    }
}

/// Avvia la scorciatoia (Ctrl+Alt+L) e il controllo periodico di inattivita' e tempo di blocco.
pub fn start(app: &AppHandle) {
    let a = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(TICK);
        tick(&a);
    });
    let a = app.clone();
    std::thread::spawn(move || unsafe {
        let _ = GetCurrentThreadId();
        // 'L' = 0x4C. Se la combinazione e' gia' usata da un altro programma non si fa nulla: resta il menu.
        if RegisterHotKey(None, HOTKEY_ID, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, 0x4C).is_err() {
            eprintln!("blocco: Ctrl+Alt+L e' gia' usata da un altro programma");
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY && msg.wParam == WPARAM(HOTKEY_ID as usize) {
                request_lock(&a);
            }
        }
        let _ = UnregisterHotKey(None, HOTKEY_ID);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_grows_after_five_failures_up_to_five_minutes() {
        let s = Duration::from_secs;
        for n in 0..5 {
            assert_eq!(backoff(n), Duration::ZERO, "{n} errori: nessuna attesa");
        }
        assert_eq!(backoff(5), s(5));
        assert_eq!(backoff(6), s(10));
        assert_eq!(backoff(7), s(20));
        assert_eq!(backoff(8), s(40));
        assert_eq!(backoff(10), s(160));
        assert_eq!(backoff(11), s(300), "tetto a 5 minuti");
        assert_eq!(backoff(500), s(300));
        assert_eq!(backoff(u32::MAX), s(300));
    }

    #[test]
    fn idle_and_escalation_thresholds_are_in_minutes_and_zero_means_never() {
        let m = |n: u64| Duration::from_secs(n * 60);
        assert!(!idle_due(m(999), 0) && !escalate_due(m(999), 0));
        assert!(!idle_due(m(4) + Duration::from_secs(59), 5));
        assert!(idle_due(m(5), 5));
        assert!(escalate_due(m(5), 5) && !escalate_due(m(1), 5));
    }

    #[test]
    fn lock_window_labels_are_recognized_and_nothing_else() {
        assert_eq!(label_for(0), "lock-0");
        assert_eq!(label_for(12), "lock-12");
        for ok in ["lock-0", "lock-3", "lock-10"] {
            assert!(is_lock_label(ok), "{ok}");
        }
        for bad in ["", "lock-", "lock", "lock-x", "lock-1a", "main", "bar-0", "launcher", "panel", "Lock-0", "lock--1"] {
            assert!(!is_lock_label(bad), "{bad:?}");
        }
    }

    #[test]
    fn the_avatar_letter_is_the_first_one_in_capitals() {
        assert_eq!(initial_of("Raffaele"), "R");
        assert_eq!(initial_of("  mario rossi"), "M");
        assert_eq!(initial_of("élodie"), "É");
        assert_eq!(initial_of(""), "?");
        assert_eq!(initial_of("   "), "?");
    }
}
