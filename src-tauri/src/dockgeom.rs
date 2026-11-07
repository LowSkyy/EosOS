//! Geometria e animazione del Dock (fase D4): funzioni pure, testate.
//!
//! Il Dock è una finestra fissa più grande della capsula: ha margine sopra (per
//! le icone ingrandite e il rimbalzo) e ai lati (la capsula si allarga con
//! l'ingrandimento). Tutte le misure qui sono in pixel fisici, salvo dove detto.

use serde::Deserialize;

/// Distanza (px logici) tra il Dock e il bordo inferiore dell'area di lavoro.
pub const BOTTOM_MARGIN: f64 = 8.0;
/// Durata dello scorrimento di comparsa/scomparsa.
pub const SLIDE_SECONDS: f64 = 0.28;
/// Quanto il cursore deve restare fuori dal Dock prima che si nasconda.
pub const HIDE_DELAY_SECONDS: f64 = 0.5;
/// Dopo l'uscita del cursore, per quanto la zona cliccabile resta estesa: la
/// capsula impiega un attimo a rimpicciolirsi e non va tagliata di colpo.
pub const SHRINK_GRACE_SECONDS: f64 = 0.18;

/// Rettangolo in pixel fisici.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn contains(&self, p: (i32, i32)) -> bool {
        p.0 >= self.x && p.0 < self.x + self.w && p.1 >= self.y && p.1 < self.y + self.h
    }
}

/// Misure in pixel CSS (logici), come le vede il frontend: la zona è la finestra
/// intera e la capsula è il rettangolo a riposo al suo interno.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub zone_w: f64,
    pub zone_h: f64,
    pub cap_x: f64,
    pub cap_y: f64,
    pub cap_w: f64,
    pub cap_h: f64,
    pub radius: f64,
}

/// Argomenti non verificati che arrivano dal frontend.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FitArgs {
    pub zone_w: f64,
    pub zone_h: f64,
    pub cap_x: f64,
    pub cap_y: f64,
    pub cap_w: f64,
    pub cap_h: f64,
    pub radius: f64,
}

/// I valori che arrivano dal frontend si controllano: niente NaN, misure assurde
/// o capsula fuori dalla zona. Pura, testata.
pub fn sanitize(a: FitArgs) -> Option<Fit> {
    let ok = |v: f64, max: f64| v.is_finite() && v > 0.0 && v <= max;
    let off = |v: f64| v.is_finite() && (0.0..=8000.0).contains(&v);
    let valid = ok(a.zone_w, 8000.0)
        && ok(a.zone_h, 2000.0)
        && ok(a.cap_w, 8000.0)
        && ok(a.cap_h, 2000.0)
        && off(a.cap_x)
        && off(a.cap_y)
        && a.radius.is_finite()
        && (0.0..=500.0).contains(&a.radius)
        && a.cap_x + a.cap_w <= a.zone_w + 0.5
        && a.cap_y + a.cap_h <= a.zone_h + 0.5;
    valid.then_some(Fit {
        zone_w: a.zone_w,
        zone_h: a.zone_h,
        cap_x: a.cap_x,
        cap_y: a.cap_y,
        cap_w: a.cap_w,
        cap_h: a.cap_h,
        radius: a.radius,
    })
}

/// Posizione "mostrato" della finestra: centrata nell'area di lavoro, con la
/// capsula appoggiata in basso a `BOTTOM_MARGIN` dal bordo. Pura, testata.
pub fn window_rect(work: Rect, scale: f64, fit: &Fit) -> Rect {
    let w = ((fit.zone_w * scale).ceil() as i32).clamp(1, work.w.max(1));
    let h = ((fit.zone_h * scale).ceil() as i32).clamp(1, work.h.max(1));
    let margin = (BOTTOM_MARGIN * scale).round() as i32;
    Rect { x: work.x + (work.w - w) / 2, y: work.y + work.h - h - margin, w, h }
}

/// Capsula a riposo, in coordinate relative alla finestra. Pura, testata.
pub fn capsule_rect(scale: f64, fit: &Fit) -> Rect {
    let r = |v: f64| (v * scale).round() as i32;
    Rect { x: r(fit.cap_x), y: r(fit.cap_y), w: r(fit.cap_w), h: r(fit.cap_h) }
}

/// Il cursore è "sul Dock"? Si entra toccando la capsula a riposo; una volta
/// dentro si resta finché il cursore è nella finestra intera (c'è lo spazio per
/// le icone ingrandite). L'isteresi evita sfarfallii. Pura, testata.
pub fn is_active(cursor: (i32, i32), window: Rect, capsule_abs: Rect, was_active: bool) -> bool {
    if was_active { window.contains(cursor) } else { capsule_abs.contains(cursor) }
}

/// Il cursore tocca il bordo inferiore del monitor (rivela il Dock nascosto)?
/// Pura, testata.
pub fn at_bottom_edge(cursor: (i32, i32), monitor: Rect) -> bool {
    cursor.0 >= monitor.x && cursor.0 < monitor.x + monitor.w && cursor.1 >= monitor.y + monitor.h - 1
}

/// Fa avanzare la visibilità `p` (0 = nascosto, 1 = mostrato) verso l'obiettivo.
/// Pura, testata.
pub fn step_visibility(p: f64, show: bool, dt: f64) -> f64 {
    let step = dt / SLIDE_SECONDS;
    if show { (p + step).min(1.0) } else { (p - step).max(0.0) }
}

/// Posizione verticale della finestra per la visibilità `p`, con uscita
/// morbida (ease-out cubica). Pura, testata.
pub fn slide_y(shown_y: i32, hidden_y: i32, p: f64) -> i32 {
    let p = p.clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - p).powi(3);
    hidden_y + ((shown_y - hidden_y) as f64 * eased).round() as i32
}

/// Altezza della finestra ancora dentro lo schermo (il resto va ritagliato, o
/// comparirebbe su un monitor posto sotto). Pura, testata.
pub fn visible_height(window_y: i32, window_h: i32, screen_bottom: i32) -> i32 {
    (screen_bottom - window_y).clamp(0, window_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fit() -> Fit {
        // Zona 600x150 css; capsula 400x78 css, 100px dai lati e in basso.
        Fit { zone_w: 600.0, zone_h: 150.0, cap_x: 100.0, cap_y: 72.0, cap_w: 400.0, cap_h: 78.0, radius: 26.0 }
    }

    fn args() -> FitArgs {
        FitArgs { zone_w: 600.0, zone_h: 150.0, cap_x: 100.0, cap_y: 72.0, cap_w: 400.0, cap_h: 78.0, radius: 26.0 }
    }

    #[test]
    fn window_is_centered_with_the_capsule_near_the_bottom() {
        let work = Rect { x: 0, y: 28, w: 1920, h: 1020 };
        let r = window_rect(work, 1.0, &fit());
        assert_eq!((r.w, r.h), (600, 150));
        assert_eq!(r.x, 660);
        assert_eq!(r.y, 28 + 1020 - 150 - 8);
    }

    #[test]
    fn scale_and_negative_origin_are_respected() {
        let work = Rect { x: -2560, y: 0, w: 2560, h: 1400 };
        let r = window_rect(work, 1.5, &fit());
        assert_eq!((r.w, r.h), (900, 225));
        assert_eq!(r.x, -2560 + (2560 - 900) / 2);
        assert_eq!(r.y, 1400 - 225 - 12);
        let c = capsule_rect(1.5, &fit());
        assert_eq!((c.x, c.y, c.w, c.h), (150, 108, 600, 117));
    }

    #[test]
    fn never_wider_than_the_work_area() {
        let r = window_rect(Rect { x: 0, y: 0, w: 300, h: 600 }, 1.0, &fit());
        assert_eq!((r.x, r.w), (0, 300));
    }

    #[test]
    fn sanitize_accepts_valid_and_rejects_nonsense() {
        assert!(sanitize(args()).is_some());
        assert!(sanitize(FitArgs { zone_w: f64::NAN, ..args() }).is_none());
        assert!(sanitize(FitArgs { zone_w: 0.0, ..args() }).is_none());
        assert!(sanitize(FitArgs { cap_h: -1.0, ..args() }).is_none());
        assert!(sanitize(FitArgs { radius: f64::INFINITY, ..args() }).is_none());
        assert!(sanitize(FitArgs { zone_w: 1e9, ..args() }).is_none());
        // Capsula che esce dalla zona.
        assert!(sanitize(FitArgs { cap_x: 300.0, ..args() }).is_none());
        assert!(sanitize(FitArgs { cap_y: 100.0, ..args() }).is_none());
    }

    #[test]
    fn hysteresis_enters_on_the_capsule_and_stays_in_the_whole_window() {
        let win = Rect { x: 0, y: 0, w: 600, h: 150 };
        let cap = Rect { x: 100, y: 72, w: 400, h: 78 };
        // Sopra la capsula, ma dentro la finestra: non basta per entrare...
        assert!(!is_active((300, 20), win, cap, false));
        // ...ma una volta dentro si resta.
        assert!(is_active((300, 20), win, cap, true));
        assert!(is_active((300, 100), win, cap, false));
        // Fuori dalla finestra si esce in ogni caso.
        assert!(!is_active((300, 200), win, cap, true));
        assert!(!is_active((-1, 100), win, cap, true));
    }

    #[test]
    fn bottom_edge_is_the_last_pixel_row_of_the_monitor() {
        let mon = Rect { x: 0, y: 0, w: 1920, h: 1080 };
        assert!(at_bottom_edge((500, 1079), mon));
        assert!(!at_bottom_edge((500, 1078), mon));
        assert!(!at_bottom_edge((1920, 1079), mon));
        assert!(!at_bottom_edge((-1, 1079), mon));
    }

    #[test]
    fn visibility_moves_at_a_constant_rate_and_saturates() {
        let p = step_visibility(0.0, true, SLIDE_SECONDS / 2.0);
        assert!((p - 0.5).abs() < 1e-9);
        assert_eq!(step_visibility(0.9, true, 1.0), 1.0);
        assert_eq!(step_visibility(0.1, false, 1.0), 0.0);
    }

    #[test]
    fn slide_covers_the_whole_travel_with_ease_out() {
        assert_eq!(slide_y(900, 1080, 1.0), 900);
        assert_eq!(slide_y(900, 1080, 0.0), 1080);
        // Ease-out: a metà tempo è già oltre metà strada.
        assert!(slide_y(900, 1080, 0.5) < 990);
        assert_eq!(slide_y(900, 1080, 2.0), 900, "p fuori scala viene limitato");
    }

    #[test]
    fn only_the_part_inside_the_screen_is_visible() {
        assert_eq!(visible_height(900, 150, 1080), 150);
        assert_eq!(visible_height(1000, 150, 1080), 80);
        assert_eq!(visible_height(1080, 150, 1080), 0);
        assert_eq!(visible_height(1200, 150, 1080), 0);
    }
}
