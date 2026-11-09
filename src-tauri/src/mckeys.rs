//! Mission Control (fase M6): la logica pura di Win+Tab e Alt+Tab. Nessun hook, nessun Windows: gli eventi
//! della tastiera arrivano già letti (con i modificatori che contano) e escono i comandi da eseguire e se
//! il tasto va "mangiato" (non deve arrivare a Windows). Tutto testato; l'hook vero sta in `mchook.rs`.
//!
//! Scelta: i modificatori NON si ricostruiscono dalla sequenza degli eventi (un rilascio perso, per
//! esempio dopo Win+L o Ctrl+Alt+Canc, li lascerebbe "incastrati" e farebbe scattare il Tab da solo):
//! li fornisce l'hook leggendo lo stato vero della tastiera nel momento in cui arriva il Tab.

pub const VK_TAB: u32 = 0x09;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_LEFT: u32 = 0x25;
pub const VK_RIGHT: u32 = 0x27;

/// Un tasto Alt (generico, sinistro, destro).
pub fn is_alt(vk: u32) -> bool {
    matches!(vk, 0x12 | 0xA4 | 0xA5)
}

/// Quali scorciatoie sostituiscono quelle di Windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub alt_tab: bool,
    pub win_tab: bool,
    /// Un tocco del tasto Windows da solo apre il Launcher (invece del menu Start).
    pub win_tap: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config { alt_tab: true, win_tab: true, win_tap: true }
    }
}

/// Un evento della tastiera con il contesto dei modificatori nel momento in cui arriva.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub vk: u32,
    pub down: bool,
    pub alt: bool,
    pub win: bool,
    pub ctrl: bool,
    pub shift: bool,
}

/// Cosa deve fare Mission Control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    /// Win+Tab: apre la panoramica, o la chiude se è aperta.
    Toggle,
    /// Alt+Tab: passa alla finestra successiva (la prima volta apre la panoramica).
    Next,
    /// Alt+Maiusc+Tab: alla precedente.
    Prev,
    /// Alt rilasciato: attiva la finestra evidenziata.
    Commit,
    /// Esc: chiude senza cambiare finestra.
    Cancel,
    /// Tasto Windows premuto e rilasciato da solo: apre (o chiude) il Launcher.
    Launcher,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Held {
    None,
    Alt,
    Win,
}

/// Stato fra un evento e l'altro: c'è una sessione Alt+Tab in corso e quale Tab è stato mangiato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    active: bool,
    tab: Held,
    /// Un Win+Tab è stato mangiato e Win non è ancora stato rilasciato.
    win_used: bool,
    /// Un Alt+Tab è stato mangiato e Alt non è ancora stato rilasciato (anche dopo Esc).
    alt_used: bool,
    /// Win è premuto e finora nessun altro tasto: se lo si rilascia così è un «tocco».
    win_tap: bool,
    /// Win è tenuto premuto (da prima di questo evento): le ripetizioni non armano un nuovo tocco.
    win_held: bool,
}

impl State {
    pub const DEFAULT: State = State { active: false, tab: Held::None, win_used: false, alt_used: false, win_tap: false, win_held: false };
}

impl Default for State {
    fn default() -> Self {
        State::DEFAULT
    }
}

impl State {
    /// C'è una sessione Alt+Tab in corso (Alt tenuto e panoramica aperta da noi).
    pub fn active(&self) -> bool {
        self.active
    }
}

/// Esito di un evento.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Out {
    /// Il tasto non deve arrivare a Windows.
    pub swallow: bool,
    pub cmd: Option<Cmd>,
    /// Solo sui rilasci di Alt e Win: il rilascio va trattenuto e rimesso DOPO un tasto "neutro": senza, Windows
    /// vede "Alt (o Win) premuto e rilasciato da solo" e apre il menu della finestra o il menu Start.
    pub neutral: bool,
}

const PASS: Out = Out { swallow: false, cmd: None, neutral: false };

/// Un passo della macchina a stati. Pura, testata.
pub fn step(s: State, cfg: Config, k: Key) -> (State, Out) {
    let is_win = matches!(k.vk, 0x5B | 0x5C);
    if k.down {
        if is_win {
            // Win da solo (nessun altro modificatore già premuto): potrebbe essere un tocco. Le ripetizioni
            // del tasto tenuto premuto non cambiano nulla.
            let alone = cfg.win_tap && !s.win_held && !s.win_used && !k.alt && !k.ctrl && !k.shift;
            return (State { win_tap: s.win_tap || alone, win_held: true, ..s }, PASS);
        }
        // Qualunque altro tasto premuto con Win giù: non è più un tocco (Win+E, Win+D, Win+Tab, ...).
        let s = if s.win_tap { State { win_tap: false, ..s } } else { s };
        if k.vk == VK_TAB {
            // Alt+Tab (non Ctrl+Alt+Tab, che è il selettore fisso di Windows, né con Windows premuto).
            if cfg.alt_tab && k.alt && !k.ctrl && !k.win {
                let cmd = if k.shift { Cmd::Prev } else { Cmd::Next };
                return (State { active: true, tab: Held::Alt, alt_used: true, ..s }, Out { swallow: true, cmd: Some(cmd), neutral: false });
            }
            // Win+Tab: un solo scatto per pressione (il ripetersi del Tab tenuto premuto non conta).
            if cfg.win_tab && k.win && !k.alt && !k.ctrl {
                if s.tab == Held::Win {
                    return (s, Out { swallow: true, cmd: None, neutral: false });
                }
                return (State { tab: Held::Win, win_used: true, ..s }, Out { swallow: true, cmd: Some(Cmd::Toggle), neutral: false });
            }
            return (s, PASS);
        }
        if s.active {
            match k.vk {
                VK_ESCAPE => {
                    return (State { active: false, tab: Held::None, ..s }, Out { swallow: true, cmd: Some(Cmd::Cancel), neutral: false });
                }
                VK_LEFT => return (s, Out { swallow: true, cmd: Some(Cmd::Prev), neutral: false }),
                VK_RIGHT => return (s, Out { swallow: true, cmd: Some(Cmd::Next), neutral: false }),
                _ => {}
            }
        }
        return (s, PASS);
    }
    // Rilasci.
    if k.vk == VK_TAB && s.tab != Held::None {
        return (State { tab: Held::None, ..s }, Out { swallow: true, cmd: None, neutral: false });
    }
    if is_alt(k.vk) && (s.active || s.alt_used) {
        // Il rilascio di Alt arriva a Windows (altrimenti il tasto resterebbe premuto per le altre app), ma
        // preceduto dal tasto neutro: senza, Windows aprirebbe il menu della finestra. Dopo Esc non c'è
        // nulla da attivare, ma il tasto neutro serve lo stesso.
        let cmd = if s.active { Some(Cmd::Commit) } else { None };
        return (State { active: false, alt_used: false, ..s }, Out { swallow: false, cmd, neutral: true });
    }
    if is_win && s.win_tap && !s.win_used {
        // Tocco: il rilascio di Win passa preceduto dal tasto neutro (senza, si aprirebbe il menu Start).
        return (State { win_tap: false, win_held: false, ..s }, Out { swallow: false, cmd: Some(Cmd::Launcher), neutral: true });
    }
    if is_win && s.win_used {
        return (State { win_used: false, win_held: false, win_tap: false, ..s }, Out { swallow: false, cmd: None, neutral: true });
    }
    if is_win {
        return (State { win_held: false, win_tap: false, ..s }, PASS);
    }
    (s, PASS)
}

/// Rete di sicurezza: una sessione in corso con Alt non più premuto (il rilascio è andato perso) va chiusa.
/// Pura, testata.
pub fn stuck(active: bool, alt_down: bool) -> bool {
    active && !alt_down
}

/// La finestra evidenziata dopo `delta` passi (+1 avanti, -1 indietro) su `len` finestre, in ordine di uso
/// recente (la prima è quella in primo piano). Alla prima pressione si parte dalla seconda, come Windows;
/// all'indietro dall'ultima. Si gira in tondo. `None` se non ci sono finestre. Pura, testata.
pub fn cycle(len: usize, current: Option<usize>, delta: i32) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let n = len as i64;
    let next = match current {
        Some(c) if (c as i64) < n => (c as i64 + i64::from(delta)).rem_euclid(n),
        _ if delta >= 0 => 1_i64.rem_euclid(n),
        _ => n - 1,
    };
    Some(next as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(vk: u32, down: bool) -> Key {
        Key { vk, down, alt: false, win: false, ctrl: false, shift: false }
    }
    fn alt_tab(down: bool) -> Key {
        Key { alt: true, ..key(VK_TAB, down) }
    }
    fn win_tab(down: bool) -> Key {
        Key { win: true, ..key(VK_TAB, down) }
    }

    #[test]
    fn alt_tab_opens_then_cycles_and_commits_on_alt_release() {
        let cfg = Config::default();
        let (s, o) = step(State::default(), cfg, alt_tab(true));
        assert_eq!(o, Out { swallow: true, cmd: Some(Cmd::Next), neutral: false });
        assert!(s.active());
        // Tab rilasciato e premuto ancora: scorre, senza un altro tasto neutro.
        let (s, o) = step(s, cfg, alt_tab(false));
        assert_eq!(o, Out { swallow: true, cmd: None, neutral: false });
        let (s, o) = step(s, cfg, alt_tab(true));
        assert_eq!(o, Out { swallow: true, cmd: Some(Cmd::Next), neutral: false });
        // Il rilascio di Alt arriva a Windows ma attiva la finestra.
        let (s, o) = step(s, cfg, key(0xA4, false));
        assert_eq!(o, Out { swallow: false, cmd: Some(Cmd::Commit), neutral: true }, "il tasto neutro precede il rilascio di Alt");
        assert!(!s.active());
    }

    #[test]
    fn shift_goes_backwards_and_arrows_move_during_a_session() {
        let cfg = Config::default();
        let (s, o) = step(State::default(), cfg, Key { shift: true, ..alt_tab(true) });
        assert_eq!(o.cmd, Some(Cmd::Prev));
        let (s, o) = step(s, cfg, Key { alt: true, ..key(VK_RIGHT, true) });
        assert_eq!((o.swallow, o.cmd), (true, Some(Cmd::Next)));
        let (_, o) = step(s, cfg, Key { alt: true, ..key(VK_LEFT, true) });
        assert_eq!((o.swallow, o.cmd), (true, Some(Cmd::Prev)));
        // Fuori da una sessione le frecce non si toccano.
        let (_, o) = step(State::default(), cfg, Key { alt: true, ..key(VK_LEFT, true) });
        assert_eq!(o, PASS);
    }

    #[test]
    fn escape_cancels_and_a_later_alt_release_does_not_commit() {
        let cfg = Config::default();
        let (s, _) = step(State::default(), cfg, alt_tab(true));
        let (s, o) = step(s, cfg, Key { alt: true, ..key(VK_ESCAPE, true) });
        assert_eq!(o, Out { swallow: true, cmd: Some(Cmd::Cancel), neutral: false });
        let (_, o) = step(s, cfg, key(0xA4, false));
        assert_eq!(o, Out { swallow: false, cmd: None, neutral: true }, "sessione chiusa: Alt non attiva niente, ma serve il tasto neutro");
        // Esc senza sessione passa a Windows.
        let (_, o) = step(State::default(), cfg, key(VK_ESCAPE, true));
        assert_eq!(o, PASS);
    }

    #[test]
    fn win_tab_toggles_once_per_press_and_swallows_the_release() {
        let cfg = Config::default();
        let (s, o) = step(State::default(), cfg, win_tab(true));
        assert_eq!(o, Out { swallow: true, cmd: Some(Cmd::Toggle), neutral: false });
        // Il Tab tenuto premuto si ripete: non deve riaprire e richiudere.
        let (s, o) = step(s, cfg, win_tab(true));
        assert_eq!(o, Out { swallow: true, cmd: None, neutral: false });
        let (s, o) = step(s, cfg, win_tab(false));
        assert_eq!(o, Out { swallow: true, cmd: None, neutral: false });
        // Il rilascio di Win passa, preceduto dal tasto neutro (senza si aprirebbe il menu Start), una volta sola.
        let (s2, o) = step(s, cfg, key(0x5B, false));
        assert_eq!(o, Out { swallow: false, cmd: None, neutral: true });
        assert_eq!(step(s2, cfg, key(0x5B, false)).1, PASS);
        // Nuova pressione = nuovo scatto.
        let (_, o) = step(s, cfg, win_tab(true));
        assert_eq!(o.cmd, Some(Cmd::Toggle));
    }

    #[test]
    fn other_combinations_are_left_to_windows() {
        let cfg = Config::default();
        // Tab da solo, Ctrl+Tab, Ctrl+Alt+Tab (selettore fisso), Alt+Win+Tab: tutto a Windows.
        for k in [
            key(VK_TAB, true),
            Key { ctrl: true, ..key(VK_TAB, true) },
            Key { ctrl: true, alt: true, ..key(VK_TAB, true) },
            Key { alt: true, win: true, ..key(VK_TAB, true) },
            Key { ctrl: true, win: true, ..key(VK_TAB, true) },
        ] {
            assert_eq!(step(State::default(), cfg, k), (State::default(), PASS), "{k:?}");
        }
        // Un Tab rilasciato che non abbiamo mangiato passa.
        assert_eq!(step(State::default(), cfg, key(VK_TAB, false)).1, PASS);
        assert_eq!(step(State::default(), cfg, key(0x41, true)).1, PASS);
    }

    #[test]
    fn disabled_shortcuts_are_never_touched() {
        let off = Config { alt_tab: false, win_tab: false, win_tap: false };
        assert_eq!(step(State::default(), off, alt_tab(true)).1, PASS);
        assert_eq!(step(State::default(), off, win_tab(true)).1, PASS);
        let only_win = Config { alt_tab: false, win_tab: true, win_tap: false };
        assert_eq!(step(State::default(), only_win, alt_tab(true)).1, PASS);
        assert_eq!(step(State::default(), only_win, win_tab(true)).1.cmd, Some(Cmd::Toggle));
        let only_alt = Config { alt_tab: true, win_tab: false, win_tap: false };
        assert_eq!(step(State::default(), only_alt, win_tab(true)).1, PASS);
    }

    const WIN: u32 = 0x5B;

    #[test]
    fn a_lone_win_tap_opens_the_launcher_and_hides_the_release_from_start() {
        let cfg = Config::default();
        let (s, o) = step(State::default(), cfg, key(WIN, true));
        assert_eq!(o, PASS, "la pressione passa: Windows non apre niente finché Win è giù");
        // Il tasto tenuto si ripete: sempre un tocco.
        let (s, o) = step(s, cfg, key(WIN, true));
        assert_eq!(o, PASS);
        let (s, o) = step(s, cfg, key(WIN, false));
        assert_eq!(o, Out { swallow: false, cmd: Some(Cmd::Launcher), neutral: true });
        assert_eq!(s, State::default());
        // Il Win destro (0x5C) vale lo stesso.
        let (s, _) = step(State::default(), cfg, key(0x5C, true));
        assert_eq!(step(s, cfg, key(0x5C, false)).1.cmd, Some(Cmd::Launcher));
    }

    #[test]
    fn win_with_any_other_key_is_not_a_tap() {
        let cfg = Config::default();
        // Win+E, Win+D, Win+Maiusc+S...: tutto a Windows, nessun Launcher, nessun tasto neutro.
        for other in [0x45_u32, 0x44, 0x10, 0x11, 0x12, 0x1B, 0x70] {
            let (s, _) = step(State::default(), cfg, key(WIN, true));
            let (s, o) = step(s, cfg, key(other, true));
            assert_eq!(o, PASS);
            let (s, _) = step(s, cfg, key(other, false));
            // Una ripetizione di Win dopo l'altro tasto non riarma il tocco.
            let (s, _) = step(s, cfg, key(WIN, true));
            let (s, o) = step(s, cfg, key(WIN, false));
            assert_eq!(o, PASS, "tasto {other:#x}");
            assert_eq!(s, State::default());
        }
    }

    #[test]
    fn win_tab_is_not_a_tap_and_keeps_its_own_release() {
        let cfg = Config::default();
        let (s, _) = step(State::default(), cfg, key(WIN, true));
        let (s, o) = step(s, cfg, win_tab(true));
        assert_eq!(o.cmd, Some(Cmd::Toggle));
        let (s, _) = step(s, cfg, win_tab(false));
        let (s, o) = step(s, cfg, key(WIN, false));
        assert_eq!(o, Out { swallow: false, cmd: None, neutral: true }, "solo il tasto neutro, niente Launcher");
        assert_eq!(s, State::default());
        // Anche con Win+Tab lasciato a Windows (win_tab spento) non si apre il Launcher.
        let cfg = Config { win_tab: false, ..cfg };
        let (s, _) = step(State::default(), cfg, key(WIN, true));
        let (s, _) = step(s, cfg, win_tab(true));
        let (s, _) = step(s, cfg, win_tab(false));
        assert_eq!(step(s, cfg, key(WIN, false)).1, PASS);
    }

    #[test]
    fn a_modifier_held_before_win_or_the_setting_off_means_no_tap() {
        let cfg = Config::default();
        for k in [Key { ctrl: true, ..key(WIN, true) }, Key { shift: true, ..key(WIN, true) }, Key { alt: true, ..key(WIN, true) }] {
            let (s, _) = step(State::default(), cfg, k);
            assert_eq!(step(s, cfg, key(WIN, false)).1, PASS, "{k:?}");
        }
        let off = Config { win_tap: false, ..cfg };
        let (s, _) = step(State::default(), off, key(WIN, true));
        assert_eq!(step(s, off, key(WIN, false)).1, PASS);
        // Un rilascio di Win senza pressione vista (stato pulito) passa e non apre niente.
        assert_eq!(step(State::default(), cfg, key(WIN, false)).1, PASS);
    }

    #[test]
    fn a_lost_alt_release_is_noticed_by_the_safety_net() {
        assert!(stuck(true, false));
        assert!(!stuck(true, true));
        assert!(!stuck(false, false));
    }

    #[test]
    fn cycling_starts_from_the_second_window_and_wraps() {
        assert_eq!(cycle(0, None, 1), None);
        assert_eq!(cycle(1, None, 1), Some(0), "una sola finestra: resta quella");
        assert_eq!(cycle(5, None, 1), Some(1));
        assert_eq!(cycle(5, Some(1), 1), Some(2));
        assert_eq!(cycle(5, Some(4), 1), Some(0), "gira in tondo");
        assert_eq!(cycle(5, None, -1), Some(4));
        assert_eq!(cycle(5, Some(0), -1), Some(4));
        assert_eq!(cycle(3, Some(9), 1), Some(1), "indice vecchio fuori elenco: si riparte");
    }
}
