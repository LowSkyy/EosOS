//! Mission Control (fase M1): elenco delle finestre per monitor e loro disposizione. Per ora nessuna
//! finestra e nessuna miniatura: il comando `mc_windows` restituisce i dati che la panoramica
//! (M2) disegnerà. Le finestre vengono dal filtro del Dock (`dock::raw_app_windows`); la
//! disposizione è `mclayout`.
//!
//! Le finestre minimizzate non hanno un'immagine viva: compaiono come schede di dimensione fissa
//! (con l'icona dell'app e il titolo, un po' attenuate) e un clic le ripristina. Si prende la
//! dimensione di ripristino solo per sapere su quale monitor stanno.
//!
//! La pagina vedrà solo l'`id` di ciascuna finestra (l'indirizzo della finestra come numero): chi
//! agisce su una finestra (attivarla, chiuderla) è Rust, che prima ricontrolla che l'id sia ancora
//! tra le finestre elencate.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;
use tauri::AppHandle;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, GetWindowTextW, IsIconic};

use crate::dock::{app_display_name, raw_app_windows, ComScope};
use crate::dockgeom::Rect;
use crate::mclayout::{fit_in, layout, monitor_of, reading_order, thumb_box, to_css, Item, Options, Placed};

/// Misure della panoramica in pixel CSS: moltiplicate per la scala del monitor.
const MARGIN: i32 = 60;
/// Spazio in alto riservato alla fascia dei desktop (M4).
const TOP_RESERVED: i32 = 140;
const GAP: i32 = 24;
const MIN_LONG: i32 = 220;
/// Dimensione naturale della scheda di una finestra minimizzata.
const CARD: (i32, i32) = (320, 200);
/// Altezza delle fasce dell'ordine di lettura.
const BAND: i32 = 240;
const MAX_TITLE: usize = 200;

/// Una finestra elencata per la panoramica.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct McWindow {
    /// Indirizzo della finestra come numero (solo un identificatore: Rust lo ricontrolla).
    pub id: u64,
    pub title: String,
    pub app: String,
    pub minimized: bool,
    /// Rettangolo reale (o di ripristino se minimizzata) in pixel dello schermo: x, y, larghezza, altezza.
    pub rect: [i32; 4],
    /// Cella nella griglia della panoramica, in pixel dello schermo.
    pub cell: [i32; 4],
    /// Identità dell'app (eseguibile o identificatore dello Store): serve a trovare l'icona. Non va alla pagina.
    #[serde(skip)]
    pub identity: PathBuf,
}

/// Un monitor con le sue finestre già disposte.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct McMonitor {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scale: f64,
    pub windows: Vec<McWindow>,
}

/// Un monitor come servono qui: rettangolo e scala.
#[derive(Clone, Copy, Debug)]
pub struct Mon {
    pub rect: Rect,
    pub scale: f64,
}

/// La finestra grezza letta da Windows, prima della disposizione.
#[derive(Clone, Debug)]
pub struct Raw {
    pub id: u64,
    pub title: String,
    pub app: String,
    pub minimized: bool,
    pub rect: Rect,
    /// Indice del monitor se già noto (finestre minimizzate); altrimenti si ricava dal rettangolo.
    pub monitor: Option<usize>,
    pub identity: PathBuf,
}

/// Area in cui disporre le finestre di un monitor, in pixel dello schermo: il monitor meno i margini
/// e lo spazio della fascia dei desktop. Pura, testata.
pub fn area_for(m: Mon) -> Rect {
    let k = if m.scale.is_finite() && m.scale > 0.0 { m.scale } else { 1.0 };
    let (margin, top) = ((f64::from(MARGIN) * k).round() as i32, (f64::from(TOP_RESERVED) * k).round() as i32);
    Rect { x: m.rect.x + margin, y: m.rect.y + top, w: (m.rect.w - 2 * margin).max(0), h: (m.rect.h - top - margin).max(0) }
}

/// Dispone le finestre sui monitor: ogni finestra va al monitor che ne contiene di più (o a quello
/// già indicato), in ordine di lettura, con la scala del monitor; le minimizzate sono schede di
/// misura fissa. Pura rispetto al sistema, testata.
pub fn build(raws: &[Raw], mons: &[Mon]) -> Vec<McMonitor> {
    let rects: Vec<Rect> = mons.iter().map(|m| m.rect).collect();
    let mut out: Vec<McMonitor> = mons
        .iter()
        .map(|m| McMonitor { x: m.rect.x, y: m.rect.y, w: m.rect.w, h: m.rect.h, scale: m.scale, windows: Vec::new() })
        .collect();
    let mut per: Vec<Vec<&Raw>> = vec![Vec::new(); mons.len()];
    for r in raws {
        if let Some(i) = r.monitor.filter(|&i| i < mons.len()).or_else(|| monitor_of(r.rect, &rects)) {
            per[i].push(r);
        }
    }
    for (i, list) in per.iter_mut().enumerate() {
        let k = if mons[i].scale.is_finite() && mons[i].scale > 0.0 { mons[i].scale } else { 1.0 };
        let mut order: Vec<(u64, Rect)> = list.iter().map(|r| (r.id, r.rect)).collect();
        reading_order(&mut order, (f64::from(BAND) * k).round() as i32);
        let by_id = |id: u64| list.iter().find(|r| r.id == id).copied();
        let items: Vec<Item> = order
            .iter()
            .filter_map(|(id, _)| by_id(*id))
            .map(|r| {
                let (w, h) = if r.minimized {
                    ((f64::from(CARD.0) * k).round() as i32, (f64::from(CARD.1) * k).round() as i32)
                } else {
                    (r.rect.w.max(1), r.rect.h.max(1))
                };
                Item { id: r.id, w, h }
            })
            .collect();
        let opts = Options { gap: (f64::from(GAP) * k).round() as i32, min_long: (f64::from(MIN_LONG) * k).round() as i32 };
        let placed: Vec<Placed> = layout(&items, area_for(mons[i]), opts);
        out[i].windows = placed
            .iter()
            .filter_map(|p| {
                let r = by_id(p.id)?;
                Some(McWindow {
                    id: r.id,
                    title: r.title.clone(),
                    app: r.app.clone(),
                    minimized: r.minimized,
                    rect: [r.rect.x, r.rect.y, r.rect.w, r.rect.h],
                    cell: [p.rect.x, p.rect.y, p.rect.w, p.rect.h],
                    identity: r.identity.clone(),
                })
            })
            .collect();
    }
    out
}

/// Margine interno di una cella e altezza della fascia dell'etichetta (px CSS, per la scala del monitor).
pub const CELL_PAD: i32 = 8;
pub const LABEL_H: i32 = 38;

/// Una finestra come la disegna la pagina della panoramica: tutte le misure in px CSS relativi
/// all'origine del monitor (la pagina occupa il monitor intero).
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ViewWindow {
    pub id: u64,
    pub title: String,
    pub app: String,
    pub minimized: bool,
    /// Data URL dell'icona dell'app, se disponibile.
    pub icon: Option<String>,
    pub cell: [f64; 4],
    /// Dove sta la miniatura viva (disegnata da Windows sopra la pagina); `None` per le minimizzate e
    /// per le finestre di cui Windows non dà la miniatura: la pagina disegna una scheda con l'icona.
    pub thumb: Option<[f64; 4]>,
}

/// La panoramica di un monitor, per la pagina.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct View {
    pub index: usize,
    pub scale: f64,
    pub width: f64,
    pub height: f64,
    pub windows: Vec<ViewWindow>,
    /// I desktop virtuali per la fascia in alto (uguali su ogni monitor); li riempie chi apre la panoramica.
    pub desktops: Vec<crate::vdesk::Desktop>,
}

/// Dove va la miniatura di una finestra nella sua cella: il riquadro libero della cella (senza
/// margine e senza fascia dell'etichetta) con le proporzioni della finestra sorgente, in pixel
/// dello schermo. `None` se non c'è spazio o la sorgente non ha dimensioni. Pura, testata.
pub fn thumb_rect(cell: Rect, src: (i32, i32), scale: f64) -> Option<Rect> {
    let k = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    let b = thumb_box(cell, (f64::from(CELL_PAD) * k).round() as i32, (f64::from(LABEL_H) * k).round() as i32)?;
    fit_in(src.0, src.1, b)
}

/// Costruisce la vista di un monitor. `thumbs`: miniature effettivamente registrate (id → rettangolo in
/// pixel dello schermo); `icons`: icone per identità. Pura, testata.
pub fn view_of(index: usize, m: &McMonitor, thumbs: &HashMap<u64, Rect>, icons: &HashMap<PathBuf, String>) -> View {
    let origin = (m.x, m.y);
    let windows = m
        .windows
        .iter()
        .map(|w| ViewWindow {
            id: w.id,
            title: w.title.clone(),
            app: w.app.clone(),
            minimized: w.minimized,
            icon: icons.get(&w.identity).cloned(),
            cell: to_css(Rect { x: w.cell[0], y: w.cell[1], w: w.cell[2], h: w.cell[3] }, origin, m.scale),
            thumb: if w.minimized { None } else { thumbs.get(&w.id).map(|r| to_css(*r, origin, m.scale)) },
        })
        .collect();
    let k = if m.scale.is_finite() && m.scale > 0.0 { m.scale } else { 1.0 };
    View { index, scale: m.scale, width: f64::from(m.w) / k, height: f64::from(m.h) / k, windows, desktops: Vec::new() }
}

fn title_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let n = unsafe { GetWindowTextW(hwnd, &mut buf) };
    let t = String::from_utf16_lossy(&buf[..n.max(0) as usize]);
    t.chars().take(MAX_TITLE).collect()
}

/// Rettangolo e stato di una finestra: reale se è normale, di ripristino se è minimizzata (la sua
/// posizione da icona è fuori schermo). Per le minimizzate si ricava anche il monitor con l'API di
/// Windows, che conosce quello di ripristino.
fn geometry(hwnd: HWND, mons: &[Mon]) -> Option<(Rect, bool, Option<usize>)> {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let mut wp = windows::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT {
                length: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };
            windows::Win32::UI::WindowsAndMessaging::GetWindowPlacement(hwnd, &mut wp).ok()?;
            let n = wp.rcNormalPosition;
            let rect = Rect { x: n.left, y: n.top, w: (n.right - n.left).max(1), h: (n.bottom - n.top).max(1) };
            let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
            let mon = if GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut mi).as_bool() {
                let r = mi.rcMonitor;
                mons.iter().position(|m| m.rect.x == r.left && m.rect.y == r.top)
            } else {
                None
            };
            return Some((rect, true, mon));
        }
        let mut r = RECT::default();
        GetWindowRect(hwnd, &mut r).ok()?;
        let rect = Rect { x: r.left, y: r.top, w: r.right - r.left, h: r.bottom - r.top };
        (rect.w > 0 && rect.h > 0).then_some((rect, false, None))
    }
}

/// Monitor del sistema (pixel fisici e scala).
pub(crate) fn monitors(app: &AppHandle) -> Vec<Mon> {
    app.available_monitors()
        .map(|v| {
            v.iter()
                .map(|m| Mon {
                    rect: Rect { x: m.position().x, y: m.position().y, w: m.size().width as i32, h: m.size().height as i32 },
                    scale: m.scale_factor(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Legge le finestre aperte (desktop corrente) e le dispone sui monitor.
pub fn snapshot(app: &AppHandle) -> Vec<McMonitor> {
    let mons = monitors(app);
    if mons.is_empty() {
        return Vec::new();
    }
    let _com = ComScope::new();
    let raws: Vec<Raw> = raw_app_windows()
        .into_iter()
        .filter_map(|(hwnd, _pid, identity)| {
            let (rect, minimized, monitor) = geometry(hwnd, &mons)?;
            Some(Raw {
                id: hwnd.0 as usize as u64,
                title: title_of(hwnd),
                app: app_display_name(&identity),
                minimized,
                rect,
                monitor,
                identity,
            })
        })
        .collect();
    build(&raws, &mons)
}

#[cfg(test)]
mod tests {
    use super::*;

    const M0: Mon = Mon { rect: Rect { x: -1920, y: 0, w: 1920, h: 1080 }, scale: 1.0 };
    const M1: Mon = Mon { rect: Rect { x: 0, y: 0, w: 2880, h: 1620 }, scale: 1.5 };

    fn raw(id: u64, x: i32, y: i32, w: i32, h: i32, minimized: bool) -> Raw {
        Raw { id, title: format!("Finestra {id}"), app: "App".into(), minimized, rect: Rect { x, y, w, h }, monitor: None, identity: PathBuf::from("app.exe") }
    }

    fn inside(c: [i32; 4], a: Rect) -> bool {
        c[0] >= a.x && c[1] >= a.y && c[0] + c[2] <= a.x + a.w && c[1] + c[3] <= a.y + a.h
    }

    #[test]
    fn the_area_leaves_room_for_margins_and_the_spaces_bar_scaled_by_dpi() {
        let a = area_for(M0);
        assert_eq!((a.x, a.y, a.w, a.h), (-1920 + 60, 140, 1920 - 120, 1080 - 140 - 60));
        let b = area_for(M1);
        assert_eq!((b.x, b.y), (90, 210), "60 e 140 px CSS a scala 1,5");
        assert_eq!((b.w, b.h), (2880 - 180, 1620 - 210 - 90));
        // Monitor minuscolo: mai misure negative. Scala assurda: si usa 1.
        let tiny = area_for(Mon { rect: Rect { x: 0, y: 0, w: 50, h: 50 }, scale: 1.0 });
        assert!(tiny.w >= 0 && tiny.h >= 0);
        assert_eq!(area_for(Mon { rect: M0.rect, scale: f64::NAN }), area_for(M0));
    }

    #[test]
    fn windows_go_to_the_monitor_that_holds_them_and_fit_its_area() {
        let raws = vec![
            raw(1, -1500, 100, 1000, 700, false),
            raw(2, 300, 200, 1400, 900, false),
            raw(3, 1200, 300, 800, 600, false),
            raw(4, -300, 100, 900, 600, false), // a cavallo: piu' sul monitor 1
        ];
        let out = build(&raws, &[M0, M1]);
        assert_eq!(out.len(), 2);
        let ids = |m: &McMonitor| m.windows.iter().map(|w| w.id).collect::<Vec<_>>();
        let mut a = ids(&out[0]);
        a.sort_unstable();
        let mut b = ids(&out[1]);
        b.sort_unstable();
        assert_eq!(a, vec![1]);
        assert_eq!(b, vec![2, 3, 4]);
        for (mon, m) in out.iter().zip([M0, M1]) {
            for w in &mon.windows {
                assert!(inside(w.cell, area_for(m)), "{:?} fuori da {:?}", w.cell, area_for(m));
            }
        }
    }

    #[test]
    fn minimized_windows_become_fixed_size_cards_and_use_their_known_monitor() {
        let mut min = raw(9, -32000, -32000, 1600, 900, true);
        min.monitor = Some(0);
        let out = build(&[min, raw(1, 100, 100, 1600, 900, false)], &[M0, M1]);
        let card = out[0].windows.iter().find(|w| w.id == 9).expect("scheda sul monitor 0");
        assert!(card.minimized);
        // Natura fissa (320x200 a scala 1): la scheda non dipende dalla dimensione di ripristino.
        assert_eq!((card.cell[2], card.cell[3]), (320, 200));
        assert!(out[1].windows.iter().all(|w| w.id != 9));
        // Un indice di monitor inesistente non fa perdere la finestra: si ricava dal rettangolo.
        let mut odd = raw(5, 100, 100, 800, 600, false);
        odd.monitor = Some(7);
        let out = build(&[odd], &[M0, M1]);
        assert_eq!(out[1].windows.len(), 1);
    }

    #[test]
    fn reading_order_places_windows_near_where_they_are() {
        // Tre finestre sullo stesso monitor: in alto a sinistra, in alto a destra, in basso.
        let raws = vec![raw(3, 400, 800, 600, 400, false), raw(2, 1500, 80, 600, 400, false), raw(1, 100, 100, 600, 400, false)];
        let out = build(&raws, &[M1]);
        assert_eq!(out[0].windows.iter().map(|w| w.id).collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn output_is_deterministic_and_keeps_text_fields() {
        let raws: Vec<Raw> = (0..12).map(|i| raw(i + 1, 50 * i as i32, 40 * i as i32, 900 + i as i32 * 10, 600, i % 5 == 0)).collect();
        let a = build(&raws, &[M1]);
        assert_eq!(a, build(&raws, &[M1]));
        assert_eq!(a[0].windows.len(), 12);
        assert!(a[0].windows.iter().all(|w| w.title.starts_with("Finestra ") && w.app == "App"));
    }

    #[test]
    fn no_monitors_or_no_windows_give_empty_results() {
        assert!(build(&[raw(1, 0, 0, 100, 100, false)], &[]).is_empty());
        let out = build(&[], &[M0, M1]);
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|m| m.windows.is_empty()));
    }

    fn monitor_with(windows: &[(u64, [i32; 4], bool)]) -> McMonitor {
        McMonitor {
            x: -1920,
            y: 0,
            w: 1920,
            h: 1080,
            scale: 1.5,
            windows: windows
                .iter()
                .map(|(id, cell, min)| McWindow {
                    id: *id,
                    title: format!("T{id}"),
                    app: "App".into(),
                    minimized: *min,
                    rect: [0, 0, 800, 600],
                    cell: *cell,
                    identity: PathBuf::from(format!("app{id}.exe")),
                })
                .collect(),
        }
    }

    #[test]
    fn thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions() {
        let cell = Rect { x: 0, y: 0, w: 600, h: 500 };
        let t = thumb_rect(cell, (1600, 800), 1.0).unwrap();
        // Riquadro libero: 584 x 454; sorgente 2:1 -> piu' larga: larghezza piena.
        assert_eq!((t.w, t.h), (584, 292));
        assert!(t.y + t.h <= cell.h - LABEL_H, "sopra la fascia dell'etichetta");
        assert!(t.x >= CELL_PAD && t.y >= CELL_PAD);
        // A scala 1,5 i margini crescono.
        let t15 = thumb_rect(Rect { x: 0, y: 0, w: 900, h: 750 }, (1600, 800), 1.5).unwrap();
        assert!(t15.y + t15.h <= 750 - (f64::from(LABEL_H) * 1.5) as i32 + 1);
        assert!(thumb_rect(Rect { x: 0, y: 0, w: 100, h: 50 }, (800, 600), 1.0).is_none(), "cella troppo piccola");
        assert!(thumb_rect(cell, (0, 600), 1.0).is_none());
    }

    #[test]
    fn the_view_is_in_css_pixels_relative_to_the_monitor() {
        let m = monitor_with(&[(1, [-1860, 210, 600, 450], false), (2, [-1200, 210, 480, 300], true)]);
        let mut thumbs = HashMap::new();
        thumbs.insert(1u64, Rect { x: -1850, y: 220, w: 300, h: 150 });
        thumbs.insert(2u64, Rect { x: -1190, y: 220, w: 10, h: 10 });
        let mut icons = HashMap::new();
        icons.insert(PathBuf::from("app1.exe"), "data:image/png;base64,AAA".to_string());
        let v = view_of(3, &m, &thumbs, &icons);
        assert_eq!(v.index, 3);
        assert_eq!((v.width, v.height, v.scale), (1280.0, 720.0, 1.5));
        assert_eq!(v.windows[0].cell, [40.0, 140.0, 400.0, 300.0]);
        assert_eq!(v.windows[0].thumb, Some([46.666_666_666_666_664, 146.666_666_666_666_66, 200.0, 100.0]));
        assert_eq!(v.windows[0].icon.as_deref(), Some("data:image/png;base64,AAA"));
        // Una minimizzata non ha mai miniatura viva, anche se ne risultasse una registrata; senza icona nota: None.
        assert!(v.windows[1].minimized && v.windows[1].thumb.is_none() && v.windows[1].icon.is_none());
    }

    #[test]
    fn a_window_without_a_registered_thumbnail_falls_back_to_a_card() {
        let m = monitor_with(&[(1, [-1800, 300, 500, 400], false)]);
        let v = view_of(0, &m, &HashMap::new(), &HashMap::new());
        assert!(v.windows[0].thumb.is_none(), "la pagina disegna la scheda con l'icona");
    }

    #[test]
    fn the_identity_never_reaches_the_page() {
        let m = monitor_with(&[(1, [-1800, 300, 500, 400], false)]);
        let json = serde_json::to_string(&m).unwrap();
        assert!(!json.contains("identity") && !json.contains("app1.exe"), "{json}");
        let v = serde_json::to_string(&view_of(0, &m, &HashMap::new(), &HashMap::new())).unwrap();
        assert!(!v.contains("app1.exe") && !v.contains("identity"));
    }

    #[test]
    fn serialized_form_is_flat_for_the_page() {
        let out = build(&[raw(1, 100, 100, 800, 600, false)], &[M1]);
        let v = serde_json::to_value(&out).unwrap();
        assert_eq!(v[0]["windows"][0]["id"], 1);
        assert_eq!(v[0]["windows"][0]["minimized"], false);
        assert_eq!(v[0]["windows"][0]["cell"].as_array().unwrap().len(), 4);
        assert_eq!(v[0]["scale"], 1.5);
    }

    /// Lettura vera, in sola lettura: verifica la coerenza senza stampare i titoli delle finestre
    /// dell'utente (solo quante sono).
    #[test]
    fn real_windows_are_consistent() {
        // Gli HWND non attraversano i thread: dentro il thread si riducono a (indirizzo, pid).
        let raws: Vec<(usize, u32)> = std::thread::spawn(|| {
            let _com = ComScope::new();
            raw_app_windows().into_iter().map(|(h, pid, _)| (h.0 as usize, pid)).collect()
        })
        .join()
        .unwrap();
        println!("finestre reali (desktop corrente): {}", raws.len());
        for (hwnd, pid) in &raws {
            assert_ne!(*hwnd, 0);
            assert_ne!(*pid, std::process::id(), "mai EosOS stessa");
        }
    }
}
