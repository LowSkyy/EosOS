//! Mission Control (fase M3): animazione e angolo attivo. Solo funzioni pure, testate: nessuna finestra,
//! nessun Windows, nessun orologio (il tempo arriva dal chiamante in millisecondi).

use crate::dockgeom::Rect;

/// Durata dell'animazione di ingresso e di uscita, in millisecondi.
pub const OPEN_MS: u64 = 220;
pub const CLOSE_MS: u64 = 180;
/// Intervallo fra due fotogrammi (circa 60 Hz).
pub const FRAME_MS: u64 = 16;

/// Avanzamento 0..=1 dopo `elapsed_ms` su una durata `total_ms` (durata nulla = già finita). Pura, testata.
pub fn progress(elapsed_ms: u64, total_ms: u64) -> f64 {
    if total_ms == 0 {
        return 1.0;
    }
    (elapsed_ms as f64 / total_ms as f64).clamp(0.0, 1.0)
}

/// Curva di accelerazione "ease-out cubica": parte veloce e rallenta arrivando. Pura, testata.
pub fn ease_out_cubic(t: f64) -> f64 {
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 1.0 };
    1.0 - (1.0 - t).powi(3)
}

/// Rettangolo intermedio fra `a` (t = 0) e `b` (t = 1), con ogni misura arrotondata. Pura, testata.
pub fn lerp_rect(a: Rect, b: Rect, t: f64) -> Rect {
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 1.0 };
    let l = |x: i32, y: i32| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as i32;
    Rect { x: l(a.x, b.x), y: l(a.y, b.y), w: l(a.w, b.w).max(1), h: l(a.h, b.h).max(1) }
}

/// Il rettangolo di una miniatura dopo `elapsed_ms` di un'animazione da `from` a `to` (o al contrario
/// con `reverse`), con la curva di accelerazione. Pura, testata.
pub fn thumb_at(from: Rect, to: Rect, elapsed_ms: u64, total_ms: u64, reverse: bool) -> Rect {
    let e = ease_out_cubic(progress(elapsed_ms, total_ms));
    if reverse { lerp_rect(to, from, e) } else { lerp_rect(from, to, e) }
}

// ------------------------------------------------------------------ angolo attivo

/// Lato (px) della zona dell'angolo in alto a sinistra di ogni monitor.
pub const CORNER_PX: i32 = 3;
/// Quanto il cursore deve restare nell'angolo prima che scatti, in millisecondi.
pub const DWELL_MS: u64 = 500;

/// Il cursore è nell'angolo in alto a sinistra di uno dei monitor? Pura, testata.
pub fn in_corner(cursor: (i32, i32), monitors: &[Rect], size: i32) -> bool {
    monitors.iter().any(|m| cursor.0 >= m.x && cursor.0 < m.x + size && cursor.1 >= m.y && cursor.1 < m.y + size)
}

/// Stato dell'angolo attivo: da quando il cursore è dentro e se ha già fatto scattare.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CornerState {
    since: Option<u64>,
    fired: bool,
}

/// Un passo dell'angolo attivo. `inside`: il cursore è nell'angolo; `blocked`: un motivo per non
/// scattare (panoramica già aperta, pulsante del mouse premuto, app a schermo intero). Scatta UNA volta
/// dopo `dwell` ms di permanenza continua e senza blocchi; per riarmarlo bisogna uscire dall'angolo.
/// Restituisce lo stato nuovo e se deve scattare. Pura, testata.
pub fn corner_step(s: CornerState, inside: bool, blocked: bool, now: u64, dwell: u64) -> (CornerState, bool) {
    if !inside {
        return (CornerState::default(), false);
    }
    if blocked {
        // Dentro ma bloccato: il tempo di permanenza riparte quando il blocco finisce.
        return (CornerState { since: None, fired: s.fired }, false);
    }
    let since = s.since.unwrap_or(now);
    if !s.fired && now.saturating_sub(since) >= dwell {
        return (CornerState { since: Some(since), fired: true }, true);
    }
    (CornerState { since: Some(since), fired: s.fired }, false)
}

/// Valori di `SHQueryUserNotificationState` per cui l'angolo non deve scattare: app a schermo intero
/// (2), gioco a schermo intero Direct3D (3), modalità presentazione (4). Pura, testata.
pub fn blocks_corner(notification_state: i32) -> bool {
    matches!(notification_state, 2..=4)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn progress_is_clamped_and_handles_zero_duration() {
        assert_eq!(progress(0, 200), 0.0);
        assert_eq!(progress(100, 200), 0.5);
        assert_eq!(progress(200, 200), 1.0);
        assert_eq!(progress(9999, 200), 1.0);
        assert_eq!(progress(5, 0), 1.0, "durata nulla: finita");
    }

    #[test]
    fn easing_starts_fast_ends_at_one_and_never_leaves_the_range() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        assert!(ease_out_cubic(0.5) > 0.5, "ease-out: a meta strada e' gia' oltre meta");
        let mut prev = 0.0;
        for i in 0..=100 {
            let v = ease_out_cubic(f64::from(i) / 100.0);
            assert!(v >= prev && (0.0..=1.0).contains(&v), "monotona e dentro 0..1");
            prev = v;
        }
        assert_eq!(ease_out_cubic(-3.0), 0.0);
        assert_eq!(ease_out_cubic(7.0), 1.0);
        assert_eq!(ease_out_cubic(f64::NAN), 1.0, "valori assurdi: stato finale");
    }

    #[test]
    fn lerp_goes_from_a_to_b_and_never_collapses() {
        let (a, b) = (r(0, 0, 1000, 600), r(200, 100, 300, 180));
        assert_eq!(lerp_rect(a, b, 0.0), a);
        assert_eq!(lerp_rect(a, b, 1.0), b);
        let m = lerp_rect(a, b, 0.5);
        assert_eq!(m, r(100, 50, 650, 390));
        assert_eq!(lerp_rect(a, b, f64::NAN), b);
        assert_eq!(lerp_rect(a, b, -1.0), a);
        assert_eq!(lerp_rect(a, b, 9.0), b);
        // Misure mai nulle anche se la sorgente lo fosse.
        let z = lerp_rect(r(0, 0, 0, 0), r(0, 0, 0, 0), 0.5);
        assert!(z.w >= 1 && z.h >= 1);
        // Posizioni negative (monitor a sinistra) vanno bene.
        assert_eq!(lerp_rect(r(-100, -50, 100, 100), r(100, 50, 100, 100), 0.5).x, 0);
    }

    #[test]
    fn a_thumbnail_travels_there_and_back() {
        let (from, to) = (r(100, 100, 800, 500), r(40, 60, 300, 190));
        assert_eq!(thumb_at(from, to, 0, OPEN_MS, false), from);
        assert_eq!(thumb_at(from, to, OPEN_MS, OPEN_MS, false), to);
        assert_eq!(thumb_at(from, to, 10_000, OPEN_MS, false), to, "dopo la fine resta li'");
        // All'indietro parte dalla cella e torna alla posizione reale.
        assert_eq!(thumb_at(from, to, 0, CLOSE_MS, true), to);
        assert_eq!(thumb_at(from, to, CLOSE_MS, CLOSE_MS, true), from);
        // A meta tempo e' gia' oltre meta strada (curva ease-out).
        let mid = thumb_at(from, to, OPEN_MS / 2, OPEN_MS, false);
        assert!(mid.x < 100 - (100 - 40) / 2, "{mid:?}");
    }

    #[test]
    fn the_frame_rate_gives_enough_frames_for_a_smooth_animation() {
        const { assert!(OPEN_MS / FRAME_MS >= 12 && CLOSE_MS / FRAME_MS >= 10) };
    }

    #[test]
    fn the_corner_is_the_top_left_of_each_monitor() {
        let mons = [r(-1920, 0, 1920, 1080), r(0, 0, 2880, 1620)];
        assert!(in_corner((0, 0), &mons, CORNER_PX));
        assert!(in_corner((2, 2), &mons, CORNER_PX));
        assert!(!in_corner((3, 3), &mons, CORNER_PX), "il limite e' escluso");
        assert!(in_corner((-1920, 0), &mons, CORNER_PX), "angolo del monitor a sinistra");
        assert!(in_corner((-1918, 2), &mons, CORNER_PX));
        assert!(!in_corner((-1, 0), &mons, CORNER_PX), "-1 e' nel monitor di sinistra ma non nel suo angolo in alto a sinistra");
        assert!(!in_corner((1000, 5), &mons, CORNER_PX));
        assert!(!in_corner((0, 0), &[], CORNER_PX));
        // Gli altri angoli non contano.
        assert!(!in_corner((2879, 0), &mons, CORNER_PX));
        assert!(!in_corner((0, 1619), &mons, CORNER_PX));
    }

    #[test]
    fn the_corner_fires_once_after_the_dwell() {
        let s = CornerState::default();
        let (s, f) = corner_step(s, true, false, 1000, DWELL_MS);
        assert!(!f, "appena entrato: ancora no");
        let (s, f) = corner_step(s, true, false, 1400, DWELL_MS);
        assert!(!f, "400 ms: ancora no");
        let (s, f) = corner_step(s, true, false, 1500, DWELL_MS);
        assert!(f, "500 ms: scatta");
        let (s, f) = corner_step(s, true, false, 3000, DWELL_MS);
        assert!(!f, "una sola volta finche' resta dentro");
        // Esce e rientra: si riarma.
        let (s, f) = corner_step(s, false, false, 3100, DWELL_MS);
        assert!(!f);
        assert_eq!(s, CornerState::default());
        let (s, _) = corner_step(s, true, false, 4000, DWELL_MS);
        let (_, f) = corner_step(s, true, false, 4500, DWELL_MS);
        assert!(f, "scatta di nuovo dopo essere uscito");
    }

    #[test]
    fn a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked() {
        let (s, _) = corner_step(CornerState::default(), true, true, 1000, DWELL_MS);
        let (s, f) = corner_step(s, true, true, 5000, DWELL_MS);
        assert!(!f, "bloccato per molto tempo: non scatta");
        // Il blocco finisce (pulsante rilasciato): la permanenza riparte da ora.
        let (s, f) = corner_step(s, true, false, 5100, DWELL_MS);
        assert!(!f);
        let (s, f) = corner_step(s, true, false, 5400, DWELL_MS);
        assert!(!f, "300 ms dopo lo sblocco");
        let (_, f) = corner_step(s, true, false, 5600, DWELL_MS);
        assert!(f, "500 ms dopo lo sblocco");
        // Uscire resetta anche se bloccato.
        let (s, f) = corner_step(CornerState::default(), false, true, 10, DWELL_MS);
        assert_eq!((s, f), (CornerState::default(), false));
    }

    #[test]
    fn a_clock_going_backwards_never_fires() {
        let (s, _) = corner_step(CornerState::default(), true, false, 10_000, DWELL_MS);
        let (_, f) = corner_step(s, true, false, 100, DWELL_MS);
        assert!(!f);
        // Dwell nullo: scatta subito, una volta.
        let (s, f) = corner_step(CornerState::default(), true, false, 5, 0);
        assert!(f);
        let (_, f) = corner_step(s, true, false, 6, 0);
        assert!(!f);
    }

    #[test]
    fn fullscreen_and_presentation_states_block_the_corner() {
        for (state, blocks) in [(1, false), (2, true), (3, true), (4, true), (5, false), (6, false), (7, false), (0, false), (-1, false), (99, false)] {
            assert_eq!(blocks_corner(state), blocks, "stato {state}");
        }
    }
}
