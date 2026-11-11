//! Mission Control (fase M6): l'hook di tastiera di basso livello che porta Win+Tab e Alt+Tab a Mission
//! Control. La logica sta in `mckeys.rs` (pura e testata); qui c'è il collegamento a Windows.
//!
//! **L'hook vive in un processo separato** (lo stesso exe lanciato con `--keyhook`, come il watchdog), non
//! nel processo con le finestre. Provato dal vivo: quando una finestra Tauri/WebView2 di EosOS ha il primo
//! piano, i tasti smettono di arrivare a un hook installato nello stesso processo (Win+Tab per chiudere la
//! panoramica faceva comparire la Visualizzazione attività di Windows). Un processo senza finestre non ha il
//! problema.
//!
//! Protocollo, a righe di testo: il figlio scrive su stdout `installed`, `cmd <Nome>` e `swallowed <n>`; il
//! genitore scrive su stdin `cfg <alt 0|1> <win 0|1> <tap 0|1>`. Se lo stdin si chiude (il genitore è morto) il figlio
//! esce da solo.
//!
//! Regole che tengono l'hook innocuo:
//! - il callback fa pochissimo (nessuna attesa, nessun I/O, nessun `SendInput`): legge lo stato, decide e
//!   passa il lavoro a un altro thread; Windows toglie da solo un hook troppo lento, e a quel punto restano
//!   le scorciatoie sue;
//! - ogni panico è fermato dentro il callback (non può attraversare il confine con Windows);
//! - gli eventi iniettati da noi portano un contrassegno e non vengono rielaborati;
//! - se l'hook non si installa o il figlio muore restano Alt+Tab e Win+Tab di Windows e la scorciatoia
//!   registrata di M3: niente si rompe. Il genitore rilancia il figlio al massimo 3 volte in 5 minuti;
//! - una sessione Alt+Tab il cui rilascio di Alt è andato perso si chiude da sola (rete di sicurezza).
//!
//! Limite noto: Windows non manda questi tasti a un hook normale mentre ha il focus una finestra con
//! privilegi più alti (UIPI); lì funzionano i selettori di Windows.

use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, KBDLLHOOKSTRUCT, MSG,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use crate::mckeys::{self, Cmd, Config, Key, State};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Contrassegno degli eventi iniettati da questo modulo ("EOS1").
const MAGIC: usize = 0x454f_5331;
/// Tasto neutro (non assegnato): fa credere a Windows che fra Alt/Win premuto e rilasciato sia successo altro.
const VK_NEUTRAL: u16 = 0xE8;

// ------------------------------------------------------------------ protocollo (pure, testate)

/// Il nome di un comando nel protocollo.
pub fn cmd_name(c: Cmd) -> &'static str {
    match c {
        Cmd::Toggle => "Toggle",
        Cmd::Next => "Next",
        Cmd::Prev => "Prev",
        Cmd::Commit => "Commit",
        Cmd::Cancel => "Cancel",
        Cmd::Launcher => "Launcher",
    }
}

/// Un messaggio del figlio al genitore.
#[derive(Debug, PartialEq, Eq)]
pub enum FromChild {
    Installed,
    Cmd(Cmd),
    Swallowed(u32),
}

/// Legge una riga del figlio; righe sconosciute o malformate si ignorano. Pura, testata.
pub fn parse_child_line(line: &str) -> Option<FromChild> {
    let mut it = line.trim().splitn(2, ' ');
    match (it.next()?, it.next()) {
        ("installed", None) => Some(FromChild::Installed),
        ("swallowed", Some(n)) => n.trim().parse().ok().map(FromChild::Swallowed),
        ("cmd", Some(name)) => {
            let c = match name.trim() {
                "Toggle" => Cmd::Toggle,
                "Next" => Cmd::Next,
                "Prev" => Cmd::Prev,
                "Commit" => Cmd::Commit,
                "Cancel" => Cmd::Cancel,
                "Launcher" => Cmd::Launcher,
                _ => return None,
            };
            Some(FromChild::Cmd(c))
        }
        _ => None,
    }
}

/// La riga di configurazione per il figlio.
pub fn config_line(c: Config) -> String {
    format!("cfg {} {} {}\n", u8::from(c.alt_tab), u8::from(c.win_tab), u8::from(c.win_tap))
}

/// Legge una riga di configurazione del genitore. Pura, testata.
pub fn parse_config_line(line: &str) -> Option<Config> {
    let mut it = line.split_whitespace();
    if it.next()? != "cfg" {
        return None;
    }
    let flag = |s: Option<&str>| match s {
        Some("0") => Some(false),
        Some("1") => Some(true),
        _ => None,
    };
    let alt_tab = flag(it.next())?;
    let win_tab = flag(it.next())?;
    let win_tap = flag(it.next())?;
    if it.next().is_some() {
        return None;
    }
    Some(Config { alt_tab, win_tab, win_tap })
}

/// La riga con cui il genitore dice al figlio che lo schermo e' bloccato (1) o sbloccato (0).
pub fn lock_line(locked: bool) -> String {
    format!("lock {}
", u8::from(locked))
}

/// Legge la riga del blocco. Pura, testata.
pub fn parse_lock_line(line: &str) -> Option<bool> {
    let mut it = line.split_whitespace();
    if it.next()? != "lock" {
        return None;
    }
    let v = match it.next()? {
        "0" => false,
        "1" => true,
        _ => return None,
    };
    it.next().is_none().then_some(v)
}

/// Con lo schermo bloccato questi tasti non devono arrivare a nessuno: i tasti Windows e Applicazioni, Alt+Tab, Alt+Esc, Ctrl+Esc, Alt+F4, Alt+Spazio e
/// Ctrl+Maiusc+Esc (Gestione attivita'). **Mai** Ctrl+Alt+Maiusc+Esc: e' la via d'uscita di emergenza di EosOS (il watchdog) e deve restare libera. Pura, testata.
pub fn lock_blocks(vk: u32, alt: bool, ctrl: bool, shift: bool) -> bool {
    const VK_TAB: u32 = 0x09;
    const VK_ESCAPE: u32 = 0x1B;
    const VK_SPACE: u32 = 0x20;
    const VK_F4: u32 = 0x73;
    match vk {
        0x5B..=0x5D => true,
        VK_TAB => alt,
        VK_F4 | VK_SPACE => alt,
        VK_ESCAPE => !(ctrl && shift && alt) && (alt || ctrl),
        _ => false,
    }
}

/// Si può rilanciare il figlio? Al massimo 3 avvii negli ultimi 5 minuti. Pura, testata.
pub fn may_restart(recent_starts: &[Duration], now: Duration) -> bool {
    recent_starts.iter().filter(|t| now.saturating_sub(**t) < Duration::from_secs(300)).count() < 3
}

// ------------------------------------------------------------------ genitore

static CFG: AtomicU8 = AtomicU8::new(0b111);
static INSTALLED: AtomicBool = AtomicBool::new(false);
static SESSION: AtomicBool = AtomicBool::new(false);
static SWALLOWED: AtomicU32 = AtomicU32::new(0);
static CHILD_IN: Mutex<Option<ChildStdin>> = Mutex::new(None);
/// Lo schermo e' bloccato: il figlio deve scartare i tasti di `lock_blocks`.
static LOCKED: AtomicBool = AtomicBool::new(false);

fn cfg() -> Config {
    let b = CFG.load(Ordering::Relaxed);
    Config { alt_tab: b & 1 != 0, win_tab: b & 2 != 0, win_tap: b & 4 != 0 }
}

fn send_cfg() {
    if let Some(stdin) = CHILD_IN.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        let _ = stdin.write_all(config_line(cfg()).as_bytes());
        let _ = stdin.write_all(lock_line(LOCKED.load(Ordering::Relaxed)).as_bytes());
        let _ = stdin.flush();
    }
}

/// Lo schermo e' bloccato (o sbloccato): vale subito anche nel figlio. Se il figlio non c'e' i tasti di Windows restano quelli di Windows.
pub fn set_locked(locked: bool) {
    LOCKED.store(locked, Ordering::Relaxed);
    send_cfg();
}

/// Cambia quali scorciatoie sostituiscono quelle di Windows (vale subito, anche nel figlio).
pub fn set_config(c: Config) {
    CFG.store(u8::from(c.alt_tab) | (u8::from(c.win_tab) << 1) | (u8::from(c.win_tap) << 2), Ordering::Relaxed);
    if !c.alt_tab {
        SESSION.store(false, Ordering::Relaxed);
    }
    send_cfg();
}

/// Stato per la diagnostica e le prove.
#[derive(Serialize, Debug, Clone)]
pub struct Diag {
    pub installed: bool,
    pub alt_tab: bool,
    pub win_tab: bool,
    pub session: bool,
    /// Tasti mangiati finora.
    pub swallowed: u32,
    /// Lo schermo e' bloccato: il figlio scarta Win, Alt+Tab, Alt+F4...
    pub locked: bool,
}

pub fn diag() -> Diag {
    let c = cfg();
    Diag {
        installed: INSTALLED.load(Ordering::Relaxed),
        alt_tab: c.alt_tab,
        win_tab: c.win_tab,
        session: SESSION.load(Ordering::Relaxed),
        swallowed: SWALLOWED.load(Ordering::Relaxed),
        locked: LOCKED.load(Ordering::Relaxed),
    }
}

fn spawn_child() -> std::io::Result<Child> {
    Command::new(std::env::current_exe()?)
        .arg("--keyhook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
}

/// Avvia il processo dell'hook e il thread che ne esegue i comandi con `handler`. Si chiama una volta
/// all'avvio. Se il figlio non parte o muore, non succede nulla di grave (restano le scorciatoie di Windows).
pub fn start(handler: fn(Cmd), initial: Config) {
    set_config(initial);
    std::thread::spawn(move || {
        let t0 = Instant::now();
        let mut starts: Vec<Duration> = Vec::new();
        loop {
            if !may_restart(&starts, t0.elapsed()) {
                eprintln!("mission control: l'hook di tastiera è terminato troppe volte: restano Alt+Tab e Win+Tab di Windows");
                return;
            }
            starts.push(t0.elapsed());
            let mut child = match spawn_child() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("mission control: hook di tastiera non avviato ({e}): restano Alt+Tab e Win+Tab di Windows");
                    return;
                }
            };
            *CHILD_IN.lock().unwrap_or_else(|e| e.into_inner()) = child.stdin.take();
            send_cfg();
            if let Some(out) = child.stdout.take() {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    match parse_child_line(&line) {
                        Some(FromChild::Installed) => INSTALLED.store(true, Ordering::Relaxed),
                        Some(FromChild::Swallowed(n)) => SWALLOWED.store(n, Ordering::Relaxed),
                        Some(FromChild::Cmd(c)) => {
                            match c {
                                Cmd::Next | Cmd::Prev => SESSION.store(true, Ordering::Relaxed),
                                Cmd::Commit | Cmd::Cancel => SESSION.store(false, Ordering::Relaxed),
                                Cmd::Toggle | Cmd::Launcher => {}
                            }
                            handler(c);
                        }
                        None => {}
                    }
                }
            }
            // Lo stdout si è chiuso: il figlio è finito (o è stato ucciso).
            INSTALLED.store(false, Ordering::Relaxed);
            SESSION.store(false, Ordering::Relaxed);
            *CHILD_IN.lock().unwrap_or_else(|e| e.into_inner()) = None;
            let _ = child.wait();
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

// ------------------------------------------------------------------ figlio (processo --keyhook)

static C_CFG: AtomicU8 = AtomicU8::new(0b111);
static C_STATE: Mutex<State> = Mutex::new(State::DEFAULT);
static C_TX: OnceLock<Mutex<Sender<Msg>>> = OnceLock::new();
static C_SWALLOWED: AtomicU32 = AtomicU32::new(0);
static C_LOCKED: AtomicBool = AtomicBool::new(false);

/// Messaggi per il thread che esegue: un comando, oppure "rilascia questo tasto dopo il tasto neutro".
enum Msg {
    Run(Cmd),
    Release { vk: u32, then: Option<Cmd> },
    /// Un tasto è stato mangiato (solo per il conteggio).
    Swallowed,
}

fn c_cfg() -> Config {
    let b = C_CFG.load(Ordering::Relaxed);
    Config { alt_tab: b & 1 != 0, win_tab: b & 2 != 0, win_tap: b & 4 != 0 }
}

fn down(vk: i32) -> bool {
    unsafe { GetAsyncKeyState(vk) < 0 }
}

/// Tasti con il codice "esteso" (Windows, Alt destro).
fn is_extended(vk: u32) -> bool {
    matches!(vk, 0x5B | 0x5C | 0xA5)
}

/// Dal thread che esegue (MAI dentro il callback dell'hook): il tasto neutro e poi il rilascio di `vk` che
/// avevamo trattenuto. Senza il neutro in mezzo, Windows vedrebbe "Alt (o Win) premuto e rilasciato da solo" e
/// aprirebbe il menu della finestra o il menu Start.
fn release_after_neutral(vk: u32) {
    let ev = |vk: u32, flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: VIRTUAL_KEY(vk as u16), wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: MAGIC },
        },
    };
    let ext = if is_extended(vk) { KEYEVENTF_EXTENDEDKEY } else { Default::default() };
    let seq = [ev(u32::from(VK_NEUTRAL), Default::default()), ev(u32::from(VK_NEUTRAL), KEYEVENTF_KEYUP), ev(vk, KEYEVENTF_KEYUP | ext)];
    unsafe { SendInput(&seq, std::mem::size_of::<INPUT>() as i32) };
}

fn c_send(m: Msg) {
    if let Some(tx) = C_TX.get() {
        let _ = tx.lock().unwrap_or_else(|e| e.into_inner()).send(m);
    }
}

fn say(line: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

fn handle(code: i32, w: WPARAM, l: LPARAM) -> bool {
    if code < 0 {
        return false;
    }
    // SAFETY: per WH_KEYBOARD_LL con code >= 0 `l` punta a una KBDLLHOOKSTRUCT valida per la durata del callback.
    let info = unsafe { &*(l.0 as *const KBDLLHOOKSTRUCT) };
    if info.dwExtraInfo == MAGIC {
        return false;
    }
    let vk = info.vkCode;
    // Schermo bloccato: i tasti che portano altrove (Windows, Alt+Tab...) si scartano, giu' e su; gli altri passano alla schermata di blocco.
    if C_LOCKED.load(Ordering::Relaxed) {
        let alt = info.flags.0 & 0x20 != 0; // LLKHF_ALTDOWN
        if lock_blocks(vk, alt, down(0x11), down(0x10)) {
            return true;
        }
    }
    let is_down = matches!(w.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    // I modificatori si leggono solo quando servono (il Tab): costa poco, ma questo è il percorso di ogni tasto.
    let (mut alt, mut win, mut ctrl, mut shift) = (false, false, false, false);
    if vk == mckeys::VK_TAB || matches!(vk, 0x5B | 0x5C) {
        alt = info.flags.0 & 0x20 != 0; // LLKHF_ALTDOWN
        win = down(0x5B) || down(0x5C);
        ctrl = down(0x11);
        shift = down(0x10);
    }
    let key = Key { vk, down: is_down, alt, win, ctrl, shift };
    let out = {
        let mut s = C_STATE.lock().unwrap_or_else(|e| e.into_inner());
        // Rete di sicurezza: se lo stato crede Win premuto ma la tastiera dice di no (rilascio perso, per esempio
        // dopo Win+L o un cambio di desktop), si dimentica, così il prossimo tocco funziona.
        if is_down && !matches!(vk, 0x5B | 0x5C) && s.win_held() && !(down(0x5B) || down(0x5C)) {
            *s = s.forget_win();
        }
        let (next, out) = mckeys::step(*s, c_cfg(), key);
        *s = next;
        out
    };
    if out.neutral {
        // Si trattiene il rilascio vero: lo rimette il thread che esegue, dopo il tasto neutro.
        c_send(Msg::Release { vk, then: out.cmd });
        return true;
    }
    if let Some(c) = out.cmd {
        c_send(Msg::Run(c));
    }
    if out.swallow {
        C_SWALLOWED.fetch_add(1, Ordering::Relaxed);
        c_send(Msg::Swallowed);
    }
    out.swallow
}

unsafe extern "system" fn proc(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    let swallow = std::panic::catch_unwind(|| handle(code, w, l)).unwrap_or(false);
    if swallow {
        return LRESULT(1);
    }
    unsafe { CallNextHookEx(None, code, w, l) }
}

/// Hook del mouse (solo osservatore, non mangia mai niente): un clic o la rotellina con Win premuto annullano il «tocco».
/// Va in fretta: i movimenti escono subito e il resto costa solo se Win è premuto.
unsafe extern "system" fn mouse_proc(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    // WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN, WM_MOUSEWHEEL, WM_XBUTTONDOWN, WM_MOUSEHWHEEL
    if code >= 0 && matches!(w.0 as u32, 0x201 | 0x204 | 0x207 | 0x20A | 0x20B | 0x20E) {
        let _ = std::panic::catch_unwind(|| {
            let mut s = C_STATE.lock().unwrap_or_else(|e| e.into_inner());
            if s.win_held() {
                *s = s.cancel_tap();
            }
        });
    }
    unsafe { CallNextHookEx(None, code, w, l) }
}

/// Il processo `--keyhook`: installa l'hook, esegue i messaggi e dialoga con il genitore. Non ritorna.
pub fn run_child() -> ! {
    let (tx, rx) = channel::<Msg>();
    let _ = C_TX.set(Mutex::new(tx));
    // Configurazione dal genitore; fine dello stdin = genitore morto = si esce.
    std::thread::spawn(|| {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            if let Some(locked) = parse_lock_line(&line) {
                C_LOCKED.store(locked, Ordering::Relaxed);
                if locked {
                    // Niente sessioni di Alt+Tab o di Win rimaste a meta' quando si blocca.
                    *C_STATE.lock().unwrap_or_else(|e| e.into_inner()) = State::DEFAULT;
                }
                continue;
            }
            if let Some(c) = parse_config_line(&line) {
                C_CFG.store(u8::from(c.alt_tab) | (u8::from(c.win_tab) << 1) | (u8::from(c.win_tap) << 2), Ordering::Relaxed);
                if !c.alt_tab {
                    *C_STATE.lock().unwrap_or_else(|e| e.into_inner()) = State::DEFAULT;
                }
            }
        }
        std::process::exit(0);
    });
    // Esecuzione dei messaggi, fuori dal callback.
    std::thread::spawn(move || loop {
        match rx.recv_timeout(Duration::from_millis(150)) {
            Ok(Msg::Run(cmd)) => say(&format!("cmd {}", cmd_name(cmd))),
            Ok(Msg::Release { vk, then }) => {
                release_after_neutral(vk);
                if let Some(cmd) = then {
                    say(&format!("cmd {}", cmd_name(cmd)));
                }
            }
            Ok(Msg::Swallowed) => say(&format!("swallowed {}", C_SWALLOWED.load(Ordering::Relaxed))),
            Err(RecvTimeoutError::Timeout) => {
                // Rete di sicurezza: sessione in corso ma Alt non più premuto = rilascio perso.
                let mut s = C_STATE.lock().unwrap_or_else(|e| e.into_inner());
                if mckeys::stuck(s.active(), down(0x12) || down(0xA4) || down(0xA5)) {
                    *s = State::DEFAULT;
                    drop(s);
                    release_after_neutral(0xA4);
                    say("cmd Commit");
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    });
    unsafe {
        let module = GetModuleHandleW(None).ok();
        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(proc), module.map(Into::into), 0) {
            Ok(_) => say("installed"),
            Err(_) => std::process::exit(2),
        }
        // Se l'hook del mouse non si installa, resta il limite noto (Win+clic apre il Launcher): niente di grave.
        let _ = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module.map(Into::into), 0);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    std::process::exit(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_lines_round_trip_and_garbage_is_ignored() {
        for c in [Cmd::Toggle, Cmd::Next, Cmd::Prev, Cmd::Commit, Cmd::Cancel, Cmd::Launcher] {
            assert_eq!(parse_child_line(&format!("cmd {}", cmd_name(c))), Some(FromChild::Cmd(c)));
        }
        assert_eq!(parse_child_line("installed"), Some(FromChild::Installed));
        assert_eq!(parse_child_line("swallowed 12\r\n"), Some(FromChild::Swallowed(12)));
        for bad in ["", "cmd", "cmd Boom", "swallowed x", "installed now", "altro 1", "cmd Next extra"] {
            assert_eq!(parse_child_line(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn config_lines_round_trip_and_garbage_is_ignored() {
        for (a, w, t) in [(true, true, true), (true, false, false), (false, true, true), (false, false, false), (true, true, false), (false, false, true)] {
            let c = Config { alt_tab: a, win_tab: w, win_tap: t };
            assert_eq!(parse_config_line(&config_line(c)), Some(c));
        }
        for bad in ["", "cfg", "cfg 1", "cfg 1 2", "cfg 1 1", "cfg 1 1 2", "cfg 1 1 1 1", "conf 1 1 1", "cfg a b c"] {
            assert_eq!(parse_config_line(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn lock_lines_round_trip_and_garbage_is_ignored() {
        assert_eq!(parse_lock_line(&lock_line(true)), Some(true));
        assert_eq!(parse_lock_line(&lock_line(false)), Some(false));
        for bad in ["", "lock", "lock 2", "lock x", "lock 1 1", "cfg 1 1 1", "unlock 1"] {
            assert_eq!(parse_lock_line(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn the_lock_swallows_the_ways_out_but_never_the_emergency_chord_or_plain_typing() {
        let (none, alt, ctrl, shift) = ((false, false, false), (true, false, false), (false, true, false), (false, false, true));
        let b = |vk: u32, m: (bool, bool, bool)| lock_blocks(vk, m.0, m.1, m.2);
        // I tasti Windows e Applicazioni, da soli.
        for vk in [0x5B, 0x5C, 0x5D] {
            assert!(b(vk, none), "{vk:#x}");
        }
        assert!(b(0x09, alt) && !b(0x09, none), "Alt+Tab si', Tab da solo no (serve alla schermata)");
        assert!(b(0x73, alt) && !b(0x73, none), "Alt+F4");
        assert!(b(0x20, alt) && !b(0x20, none), "Alt+Spazio si', la barra spaziatrice da sola no");
        assert!(b(0x1B, alt) && b(0x1B, ctrl), "Alt+Esc e Ctrl+Esc");
        assert!(!b(0x1B, none), "Esc da solo serve alla schermata (cancella la password)");
        assert!(b(0x1B, (false, true, true)), "Ctrl+Maiusc+Esc (Gestione attivita')");
        assert!(!lock_blocks(0x1B, true, true, true), "Ctrl+Alt+Maiusc+Esc e' la via d'uscita di emergenza: mai bloccata");
        // Scrivere una password: lettere, numeri, invio, backspace non si toccano.
        for vk in [0x41, 0x5A, 0x30, 0x39, 0x0D, 0x08, 0x10, 0x11, 0x12, 0xA0, 0xA2, 0xA4] {
            assert!(!b(vk, none) && !b(vk, shift), "{vk:#x}");
        }
    }

    #[test]
    fn the_child_is_restarted_at_most_three_times_in_five_minutes() {
        let s = Duration::from_secs;
        assert!(may_restart(&[], s(0)));
        assert!(may_restart(&[s(0), s(10)], s(20)));
        assert!(!may_restart(&[s(0), s(10), s(20)], s(30)), "il terzo rilancio e' l'ultimo");
        assert!(may_restart(&[s(0), s(10), s(20)], s(400)), "dopo 5 minuti si puo' riprovare");
    }
}
