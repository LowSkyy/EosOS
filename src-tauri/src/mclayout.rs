//! Mission Control (fase M1): disposizione delle finestre a griglia. Solo funzioni pure, testate:
//! nessuna finestra, nessun Windows, nessun orologio.
//!
//! Le finestre si dispongono "a righe" nell'ordine dato (di lettura), tutte con la **stessa scala**
//! `s` rispetto alla loro dimensione naturale, così una finestra grande resta più grande di una
//! piccola, come in macOS. `s` è la più grande possibile (ricerca binaria) per cui le righe stanno
//! nell'area; nessuna finestra supera mai la sua dimensione naturale (`s <= 1`). Le finestre molto
//! piccole non si rimpiccioliscono sotto un lato lungo minimo, finché c'è spazio. Le righe sono
//! centrate in orizzontale e in verticale.

use crate::dockgeom::Rect;

/// Una finestra da disporre: id e dimensione naturale in pixel (> 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: u64,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Spazio fra le celle (orizzontale e verticale), in pixel.
    pub gap: i32,
    /// Lato lungo minimo di una cella, finché c'è spazio (0 = nessun minimo).
    pub min_long: i32,
}

/// Una finestra disposta: id e rettangolo della cella (pixel dello schermo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub id: u64,
    pub rect: Rect,
}

/// Dimensione di una finestra alla scala `s` con il minimo `min_long`. Pura, testata.
fn size_at(it: &Item, s: f64, min_long: i32) -> (f64, f64) {
    let (w, h) = (f64::from(it.w.max(1)), f64::from(it.h.max(1)));
    let long = w.max(h);
    let floor = if min_long > 0 { (f64::from(min_long) / long).min(1.0) } else { 0.0 };
    let f = s.max(floor).min(1.0);
    (w * f, h * f)
}

/// Righe (indici delle finestre) e altezza totale per la scala `s`, riempiendo ogni riga finché c'è
/// larghezza. `None` se una finestra da sola non sta nell'area.
fn rows_for(items: &[Item], s: f64, area: Rect, o: Options) -> Option<(Vec<Vec<usize>>, f64)> {
    let (aw, ah) = (f64::from(area.w), f64::from(area.h));
    let gap = f64::from(o.gap.max(0));
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let (mut cur, mut cur_w) = (Vec::new(), 0.0f64);
    for (i, it) in items.iter().enumerate() {
        let (w, h) = size_at(it, s, o.min_long);
        if w > aw + 0.5 || h > ah + 0.5 {
            return None;
        }
        if !cur.is_empty() && cur_w + gap + w > aw + 0.5 {
            rows.push(std::mem::take(&mut cur));
            cur_w = 0.0;
        }
        cur_w += if cur.is_empty() { w } else { gap + w };
        cur.push(i);
    }
    if !cur.is_empty() {
        rows.push(cur);
    }
    let total: f64 = rows
        .iter()
        .map(|r| r.iter().map(|&i| size_at(&items[i], s, o.min_long).1).fold(0.0, f64::max))
        .sum::<f64>()
        + gap * (rows.len().saturating_sub(1)) as f64;
    (total <= ah + 0.5).then_some((rows, total))
}

/// La scala più grande (0..=1) per cui tutto sta nell'area, con la ricerca binaria. Se nemmeno la
/// scala minima basta con il minimo di lato lungo, lo dimezza finché non basta (o lo toglie).
fn best_scale(items: &[Item], area: Rect, o: Options) -> (f64, Options) {
    let mut opts = o;
    loop {
        if rows_for(items, 1e-4, area, opts).is_some() {
            let (mut lo, mut hi) = (1e-4f64, 1.0f64);
            if rows_for(items, 1.0, area, opts).is_some() {
                return (1.0, opts);
            }
            for _ in 0..48 {
                let mid = (lo + hi) / 2.0;
                if rows_for(items, mid, area, opts).is_some() {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            return (lo, opts);
        }
        if opts.min_long <= 0 {
            return (1e-4, opts); // area minuscola: si va comunque, con celle minime
        }
        opts.min_long = if opts.min_long < 8 { 0 } else { opts.min_long / 2 };
    }
}

/// Dispone `items` (nell'ordine dato) nell'`area`. Ogni cella ha le proporzioni della finestra,
/// non esce dall'area e non si sovrappone alle altre. Vuoto se non ci sono finestre o l'area non
/// ha dimensioni. Pura, testata.
pub fn layout(items: &[Item], area: Rect, o: Options) -> Vec<Placed> {
    if items.is_empty() || area.w <= 0 || area.h <= 0 {
        return Vec::new();
    }
    let (s, opts) = best_scale(items, area, o);
    let Some((rows, total_h)) = rows_for(items, s, area, opts).or_else(|| rows_for(items, 1e-4, area, opts)) else {
        // Neppure la scala minima entra: caso degenerato (area di pochi pixel). Celle da 1 px in fila.
        return items
            .iter()
            .enumerate()
            .map(|(i, it)| Placed { id: it.id, rect: Rect { x: area.x + (i as i32).min(area.w - 1), y: area.y, w: 1, h: 1 } })
            .collect();
    };
    let gap = f64::from(opts.gap.max(0));
    let mut out = Vec::with_capacity(items.len());
    let mut y = f64::from(area.y) + (f64::from(area.h) - total_h) / 2.0;
    for row in &rows {
        let sizes: Vec<(f64, f64)> = row.iter().map(|&i| size_at(&items[i], s, opts.min_long)).collect();
        let row_h = sizes.iter().map(|z| z.1).fold(0.0, f64::max);
        let row_w: f64 = sizes.iter().map(|z| z.0).sum::<f64>() + gap * (row.len() - 1) as f64;
        let mut x = f64::from(area.x) + (f64::from(area.w) - row_w) / 2.0;
        for (&i, &(w, h)) in row.iter().zip(&sizes) {
            let (cw, ch) = ((w.round() as i32).max(1), (h.round() as i32).max(1));
            let cx = (x.round() as i32).clamp(area.x, (area.x + area.w - cw).max(area.x));
            let cy = ((y + (row_h - h) / 2.0).round() as i32).clamp(area.y, (area.y + area.h - ch).max(area.y));
            out.push(Placed { id: items[i].id, rect: Rect { x: cx, y: cy, w: cw, h: ch } });
            x += w + gap;
        }
        y += row_h + gap;
    }
    out
}

/// Adatta un'immagine `src_w` x `src_h` dentro `bounds` mantenendo le proporzioni, senza ingrandirla
/// oltre la sua dimensione, centrata. `None` se la sorgente o il riquadro non hanno dimensioni.
/// Serve alla miniatura di una finestra dentro la sua cella (Windows la stira per riempire il
/// rettangolo di destinazione, quindi le proporzioni le deve dare chi chiama). Pura, testata.
pub fn fit_in(src_w: i32, src_h: i32, bounds: Rect) -> Option<Rect> {
    if src_w <= 0 || src_h <= 0 || bounds.w <= 0 || bounds.h <= 0 {
        return None;
    }
    let s = (f64::from(bounds.w) / f64::from(src_w)).min(f64::from(bounds.h) / f64::from(src_h)).min(1.0);
    let (w, h) = (((f64::from(src_w) * s).round() as i32).max(1), ((f64::from(src_h) * s).round() as i32).max(1));
    Some(Rect { x: bounds.x + (bounds.w - w) / 2, y: bounds.y + (bounds.h - h) / 2, w, h })
}

/// Il riquadro della cella dove sta la miniatura: la cella meno il margine interno `pad` su tutti i
/// lati e la fascia dell'etichetta `label_h` in basso. `None` se non resta spazio. Pura, testata.
pub fn thumb_box(cell: Rect, pad: i32, label_h: i32) -> Option<Rect> {
    let (w, h) = (cell.w - 2 * pad, cell.h - 2 * pad - label_h);
    (w > 0 && h > 0).then_some(Rect { x: cell.x + pad, y: cell.y + pad, w, h })
}

/// Rettangolo in pixel dello schermo → pixel CSS relativi all'origine del monitor (quello che
/// disegna la pagina della panoramica a schermo intero su quel monitor). Pura, testata.
pub fn to_css(r: Rect, origin: (i32, i32), scale: f64) -> [f64; 4] {
    let k = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    [f64::from(r.x - origin.0) / k, f64::from(r.y - origin.1) / k, f64::from(r.w) / k, f64::from(r.h) / k]
}

/// La finestra sotto il punto, se c'è. Le celle non si sovrappongono, quindi al massimo una. Pura, testata.
#[cfg_attr(not(test), allow(dead_code))] // la pagina fa il suo hit-test sugli elementi; resta per le prove
pub fn hit_test(placed: &[Placed], x: i32, y: i32) -> Option<u64> {
    placed.iter().find(|p| p.rect.contains((x, y))).map(|p| p.id)
}

/// Mette le finestre in ordine di lettura (dall'alto in basso a righe, poi da sinistra a destra):
/// il centro di ciascuna si assegna a una fascia orizzontale di altezza `band` e dentro la fascia si
/// ordina per x; a parità, per id. Così una finestra resta vicina a dove sta davvero e l'ordine è
/// stabile. Pura, testata.
pub fn reading_order(windows: &mut [(u64, Rect)], band: i32) {
    let band = band.max(1);
    windows.sort_by_key(|(id, r)| ((r.y + r.h / 2).div_euclid(band), r.x + r.w / 2, *id));
}

/// L'indice del monitor che contiene la maggior parte del rettangolo; se non ne tocca nessuno,
/// quello il cui centro è più vicino. `None` solo senza monitor. Pura, testata.
pub fn monitor_of(rect: Rect, monitors: &[Rect]) -> Option<usize> {
    let overlap = |m: &Rect| {
        let w = (rect.x + rect.w).min(m.x + m.w) - rect.x.max(m.x);
        let h = (rect.y + rect.h).min(m.y + m.h) - rect.y.max(m.y);
        if w > 0 && h > 0 { i64::from(w) * i64::from(h) } else { 0 }
    };
    let best = monitors.iter().enumerate().map(|(i, m)| (overlap(m), i)).max_by_key(|&(a, i)| (a, std::cmp::Reverse(i)))?;
    if best.0 > 0 {
        return Some(best.1);
    }
    let (cx, cy) = (i64::from(rect.x) + i64::from(rect.w) / 2, i64::from(rect.y) + i64::from(rect.h) / 2);
    monitors
        .iter()
        .enumerate()
        .min_by_key(|(i, m)| {
            let (dx, dy) = (cx - (i64::from(m.x) + i64::from(m.w) / 2), cy - (i64::from(m.y) + i64::from(m.h) / 2));
            (dx * dx + dy * dy, *i)
        })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect { x: 100, y: 200, w: 1720, h: 800 };
    const OPT: Options = Options { gap: 20, min_long: 200 };

    fn item(id: u64, w: i32, h: i32) -> Item {
        Item { id, w, h }
    }

    fn inside(r: Rect, a: Rect) -> bool {
        r.x >= a.x && r.y >= a.y && r.x + r.w <= a.x + a.w && r.y + r.h <= a.y + a.h
    }

    fn overlaps(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    /// Controlli validi per qualunque disposizione: tutto dentro, niente sovrapposizioni, ids intatti.
    fn check_invariants(items: &[Item], placed: &[Placed], area: Rect) {
        assert_eq!(placed.len(), items.len());
        for (it, p) in items.iter().zip(placed) {
            assert_eq!(it.id, p.id, "ordine e id conservati");
            assert!(inside(p.rect, area), "{:?} fuori da {area:?}", p.rect);
            assert!(p.rect.w >= 1 && p.rect.h >= 1);
        }
        for (i, a) in placed.iter().enumerate() {
            for b in &placed[i + 1..] {
                assert!(!overlaps(a.rect, b.rect), "sovrapposte: {:?} e {:?}", a.rect, b.rect);
            }
        }
    }

    #[test]
    fn empty_and_degenerate_input() {
        assert!(layout(&[], AREA, OPT).is_empty());
        assert!(layout(&[item(1, 100, 100)], Rect { x: 0, y: 0, w: 0, h: 500 }, OPT).is_empty());
        assert!(layout(&[item(1, 100, 100)], Rect { x: 0, y: 0, w: 500, h: -3 }, OPT).is_empty());
    }

    #[test]
    fn a_single_small_window_keeps_its_natural_size_and_is_centered() {
        let p = layout(&[item(7, 400, 300)], AREA, OPT);
        assert_eq!(p.len(), 1);
        assert_eq!((p[0].rect.w, p[0].rect.h), (400, 300), "mai ingrandita oltre la sua dimensione");
        assert_eq!(p[0].rect.x, AREA.x + (AREA.w - 400) / 2);
        assert_eq!(p[0].rect.y, AREA.y + (AREA.h - 300) / 2);
    }

    #[test]
    fn a_huge_window_is_scaled_down_keeping_its_proportions() {
        let p = layout(&[item(1, 3840, 2160)], AREA, OPT);
        let r = p[0].rect;
        assert!(inside(r, AREA));
        assert!(r.w == AREA.w || r.h == AREA.h, "usa tutto lo spazio disponibile: {r:?}");
        let ratio = f64::from(r.w) / f64::from(r.h);
        assert!((ratio - 3840.0 / 2160.0).abs() < 0.02, "proporzioni: {ratio}");
    }

    #[test]
    fn invariants_hold_for_many_counts_and_shapes() {
        let shapes = [(1920, 1080), (800, 600), (1280, 720), (400, 900), (3000, 400), (300, 300), (640, 480)];
        for n in [1usize, 2, 3, 4, 5, 7, 12, 20, 33, 60, 120] {
            let items: Vec<Item> = (0..n).map(|i| item(i as u64 + 1, shapes[i % shapes.len()].0, shapes[i % shapes.len()].1)).collect();
            check_invariants(&items, &layout(&items, AREA, OPT), AREA);
        }
    }

    #[test]
    fn proportions_are_preserved_within_rounding() {
        let items = vec![item(1, 1920, 1080), item(2, 500, 800), item(3, 1000, 1000), item(4, 2400, 600)];
        for (it, p) in items.iter().zip(layout(&items, AREA, Options { gap: 20, min_long: 0 })) {
            let (a, b) = (f64::from(it.w) / f64::from(it.h), f64::from(p.rect.w) / f64::from(p.rect.h));
            assert!((a - b).abs() / a < 0.03, "{}: {a} vs {b}", it.id);
        }
    }

    #[test]
    fn the_same_scale_keeps_relative_sizes() {
        // Senza minimo di lato, una finestra due volte piu' grande resta due volte piu' grande.
        let items = vec![item(1, 1600, 900), item(2, 800, 450), item(3, 1600, 900), item(4, 800, 450)];
        let p = layout(&items, AREA, Options { gap: 20, min_long: 0 });
        let ratio = f64::from(p[0].rect.w) / f64::from(p[1].rect.w);
        assert!((ratio - 2.0).abs() < 0.05, "{ratio}");
    }

    #[test]
    fn tiny_windows_are_not_shrunk_below_the_minimum_when_there_is_room() {
        let items = vec![item(1, 1920, 1080), item(2, 120, 80), item(3, 1920, 1080)];
        let p = layout(&items, AREA, OPT);
        let small = p[1].rect;
        assert!(small.w.max(small.h) >= 120, "non piu' piccola del naturale: {small:?}");
        check_invariants(&items, &p, AREA);
        // Una finestra naturalmente piu' piccola del minimo resta alla sua dimensione naturale (non si ingrandisce).
        assert!(small.w <= 120 && small.h <= 80);
    }

    #[test]
    fn the_minimum_is_dropped_when_there_are_too_many_windows() {
        let items: Vec<Item> = (0..150).map(|i| item(i + 1, 1280, 720)).collect();
        let p = layout(&items, AREA, Options { gap: 10, min_long: 400 });
        check_invariants(&items, &p, AREA);
    }

    #[test]
    fn is_deterministic_and_order_is_kept() {
        let items: Vec<Item> = (0..9).map(|i| item(100 - i, 1000 + i as i32 * 50, 700)).collect();
        let a = layout(&items, AREA, OPT);
        let b = layout(&items, AREA, OPT);
        assert_eq!(a, b);
        assert_eq!(a.iter().map(|p| p.id).collect::<Vec<_>>(), items.iter().map(|i| i.id).collect::<Vec<_>>());
    }

    #[test]
    fn tiny_areas_never_panic_or_overflow() {
        let items: Vec<Item> = (0..40).map(|i| item(i + 1, 1920, 1080)).collect();
        for (w, h) in [(1, 1), (5, 3), (30, 20), (100, 40), (7, 700)] {
            let area = Rect { x: -50, y: 10, w, h };
            let p = layout(&items, area, OPT);
            assert_eq!(p.len(), items.len());
            for q in &p {
                assert!(q.rect.w >= 1 && q.rect.h >= 1);
                assert!(inside(q.rect, area) || area.w < 2 || area.h < 2, "{:?} in {area:?}", q.rect);
            }
        }
    }

    #[test]
    fn gaps_are_respected_when_there_is_room() {
        let items: Vec<Item> = (0..6).map(|i| item(i + 1, 900, 600)).collect();
        let p = layout(&items, AREA, OPT);
        for (i, a) in p.iter().enumerate() {
            for b in &p[i + 1..] {
                let dx = (a.rect.x - (b.rect.x + b.rect.w)).max(b.rect.x - (a.rect.x + a.rect.w));
                let dy = (a.rect.y - (b.rect.y + b.rect.h)).max(b.rect.y - (a.rect.y + a.rect.h));
                assert!(dx.max(dy) >= OPT.gap - 2, "troppo vicine: {:?} {:?}", a.rect, b.rect);
            }
        }
    }

    #[test]
    fn negative_origin_areas_work_for_monitors_left_of_the_primary() {
        let area = Rect { x: -1920, y: 100, w: 1800, h: 700 };
        let items: Vec<Item> = (0..8).map(|i| item(i + 1, 1280, 720)).collect();
        check_invariants(&items, &layout(&items, area, OPT), area);
    }

    #[test]
    fn thumbnails_fit_their_box_keeping_proportions_and_never_grow() {
        let b = Rect { x: 100, y: 50, w: 400, h: 300 };
        // Piu' larga del riquadro: si riduce alla larghezza e si centra in verticale.
        let r = fit_in(1600, 800, b).unwrap();
        assert_eq!((r.w, r.h), (400, 200));
        assert_eq!((r.x, r.y), (100, 50 + 50));
        // Piu' alta: si riduce all'altezza e si centra in orizzontale.
        let r = fit_in(300, 900, b).unwrap();
        assert_eq!((r.w, r.h), (100, 300));
        assert_eq!((r.x, r.y), (100 + 150, 50));
        // Piu' piccola del riquadro: non si ingrandisce, si centra.
        let r = fit_in(200, 100, b).unwrap();
        assert_eq!((r.w, r.h), (200, 100));
        assert_eq!((r.x, r.y), (100 + 100, 50 + 100));
        assert!(fit_in(0, 100, b).is_none() && fit_in(100, -1, b).is_none());
        assert!(fit_in(100, 100, Rect { x: 0, y: 0, w: 0, h: 50 }).is_none());
        // Proporzioni estreme: mai zero pixel.
        let r = fit_in(5000, 3, b).unwrap();
        assert!(r.w >= 1 && r.h >= 1 && r.w <= 400);
        // Dentro il riquadro, sempre.
        for (w, h) in [(1920, 1080), (1, 1), (777, 13), (13, 777)] {
            let r = fit_in(w, h, b).unwrap();
            assert!(r.x >= b.x && r.y >= b.y && r.x + r.w <= b.x + b.w && r.y + r.h <= b.y + b.h);
        }
    }

    #[test]
    fn the_thumbnail_box_leaves_room_for_padding_and_the_label() {
        let cell = Rect { x: 10, y: 20, w: 500, h: 400 };
        assert_eq!(thumb_box(cell, 8, 34), Some(Rect { x: 18, y: 28, w: 484, h: 350 }));
        assert_eq!(thumb_box(cell, 0, 0), Some(cell));
        assert!(thumb_box(Rect { x: 0, y: 0, w: 100, h: 40 }, 8, 34).is_none(), "nessuno spazio per la miniatura");
        assert!(thumb_box(Rect { x: 0, y: 0, w: 10, h: 400 }, 8, 0).is_none());
    }

    #[test]
    fn css_conversion_is_relative_to_the_monitor_and_scaled() {
        let r = Rect { x: -1800, y: 300, w: 600, h: 300 };
        assert_eq!(to_css(r, (-1920, 0), 1.0), [120.0, 300.0, 600.0, 300.0]);
        assert_eq!(to_css(Rect { x: 300, y: 150, w: 600, h: 300 }, (0, 0), 1.5), [200.0, 100.0, 400.0, 200.0]);
        assert_eq!(to_css(r, (-1920, 0), f64::NAN), to_css(r, (-1920, 0), 1.0), "scala assurda: 1");
        assert_eq!(to_css(r, (-1920, 0), 0.0), to_css(r, (-1920, 0), 1.0));
    }

    #[test]
    fn hit_testing_finds_the_cell_under_the_pointer() {
        let items = vec![item(1, 800, 500), item(2, 800, 500)];
        let p = layout(&items, AREA, OPT);
        for q in &p {
            assert_eq!(hit_test(&p, q.rect.x + q.rect.w / 2, q.rect.y + q.rect.h / 2), Some(q.id));
        }
        assert_eq!(hit_test(&p, AREA.x - 5, AREA.y - 5), None);
        let a = p[0].rect;
        let b = p[1].rect;
        let gap_x = if a.x + a.w <= b.x { (a.x + a.w + b.x) / 2 } else { (b.x + b.w + a.x) / 2 };
        let gap_y = a.y + a.h / 2;
        if a.y == b.y {
            assert_eq!(hit_test(&p, gap_x, gap_y), None, "lo spazio fra le celle non e' una finestra");
        }
        assert_eq!(hit_test(&[], 5, 5), None);
    }

    #[test]
    fn reading_order_goes_row_by_row_and_is_stable() {
        let r = |x, y| Rect { x, y, w: 200, h: 100 };
        let mut v = vec![(1u64, r(900, 50)), (2, r(100, 500)), (3, r(100, 60)), (4, r(500, 40)), (5, r(100, 55))];
        reading_order(&mut v, 200);
        // Prima fascia (y centro 100..160): x crescente; poi la seconda fascia.
        assert_eq!(v.iter().map(|x| x.0).collect::<Vec<_>>(), vec![3, 5, 4, 1, 2]);
        let before = v.clone();
        reading_order(&mut v, 200);
        assert_eq!(v, before, "idempotente");
        let mut tie = vec![(9u64, r(100, 50)), (3, r(100, 50))];
        reading_order(&mut tie, 0); // fascia assurda: trattata come 1
        assert_eq!(tie[0].0, 3, "a parita' vince l'id minore");
    }

    #[test]
    fn windows_belong_to_the_monitor_that_holds_most_of_them() {
        let mons = [Rect { x: -1920, y: 0, w: 1920, h: 1080 }, Rect { x: 0, y: 0, w: 1920, h: 1080 }];
        let m = |x, y, w, h| monitor_of(Rect { x, y, w, h }, &mons);
        assert_eq!(m(100, 100, 800, 600), Some(1));
        assert_eq!(m(-1000, 100, 800, 600), Some(0));
        assert_eq!(m(-300, 100, 800, 600), Some(1), "a cavallo: vince chi ne contiene di piu'");
        assert_eq!(m(-500, 100, 800, 600), Some(0));
        assert_eq!(m(-960, 100, 1920, 600), Some(0), "meta e meta: vince il primo");
        // Fuori da ogni monitor: il piu' vicino.
        assert_eq!(m(-5000, 200, 100, 100), Some(0));
        assert_eq!(m(4000, 200, 100, 100), Some(1));
        assert_eq!(monitor_of(Rect { x: 0, y: 0, w: 10, h: 10 }, &[]), None);
    }
}
