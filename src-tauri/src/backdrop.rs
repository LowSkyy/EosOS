//! Sfondo sfocato del Dock.
//!
//! Nessun effetto di sfocatura di Windows (acrilico, blur, sfondo di sistema)
//! rispetta una forma arrotondata: riempiono sempre il rettangolo della finestra.
//! Qui si fa il contrario: si cattura ciò che sta dietro al Dock, lo si manda alla
//! pagina e lo sfoca il CSS, ritagliato dalla capsula. La forma è perfetta e con i
//! bordi lisci, perché la disegna il browser.
//!
//! La finestra del Dock è esclusa dalla cattura (`WDA_EXCLUDEFROMCAPTURE`), altrimenti
//! il Dock sfocherebbe se stesso. Effetto collaterale: finché la sfocatura è attiva
//! gli screenshot e le registrazioni schermo non includono il Dock. Per questo è
//! un'impostazione.
//!
//! Il frontend richiede i fotogrammi con `dock_backdrop`; la cattura gira in un
//! thread a ~25 Hz solo quando il Dock è visibile e la sfocatura è attiva, e un
//! fotogramma identico al precedente non viene ripubblicato.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HGDIOBJ,
    SRCCOPY,
};

use crate::dockgeom::Rect;

/// Si riduce la risoluzione di questo fattore prima di inviare: la sfocatura non
/// ne ha bisogno e i dati diventano un quarto.
pub const FACTOR: usize = 2;
/// Lunghezza dell'intestazione di un fotogramma serializzato.
pub const HEADER_LEN: usize = 28;
const INTERVAL: Duration = Duration::from_millis(40);

/// Un fotogramma già ridotto, in RGBA opaco.
struct Frame {
    seq: u64,
    /// Origine della cattura rispetto all'angolo della finestra del Dock (px fisici).
    ox: i32,
    oy: i32,
    /// Dimensioni in pixel del fotogramma ridotto.
    w: u32,
    h: u32,
    rgba: Vec<u8>,
}

/// Chi usa lo sfondo: il Dock, il Launcher e il pannello degli indicatori hanno ciascuno il proprio fotogramma (sono visibili
/// insieme, in posti diversi dello schermo).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Channel {
    Dock = 0,
    Launcher = 1,
    Panel = 2,
}

const CHANNELS: [Channel; 3] = [Channel::Dock, Channel::Launcher, Channel::Panel];

static FRAMES: [Mutex<Option<Frame>>; 3] = [const { Mutex::new(None) }; 3];
static SEQ: AtomicU64 = AtomicU64::new(0);

/// Riduce di `f` un'immagine BGRA e la converte in RGBA opaco, mediando ogni
/// blocco `f`×`f`. Pura, testata.
pub fn downscale(bgra: &[u8], w: usize, h: usize, f: usize) -> (usize, usize, Vec<u8>) {
    if f == 0 || w < f || h < f || bgra.len() < w * h * 4 {
        return (0, 0, Vec::new());
    }
    let (dw, dh) = (w / f, h / f);
    let mut out = Vec::with_capacity(dw * dh * 4);
    let n = (f * f) as u32;
    for y in 0..dh {
        for x in 0..dw {
            let (mut b, mut g, mut r) = (0u32, 0u32, 0u32);
            for yy in 0..f {
                let row = ((y * f + yy) * w + x * f) * 4;
                for xx in 0..f {
                    let p = row + xx * 4;
                    b += bgra[p] as u32;
                    g += bgra[p + 1] as u32;
                    r += bgra[p + 2] as u32;
                }
            }
            out.extend_from_slice(&[(r / n) as u8, (g / n) as u8, (b / n) as u8, 255]);
        }
    }
    (dw, dh, out)
}

/// Serializza un fotogramma: intestazione di `HEADER_LEN` byte (little endian:
/// seq u64, ox i32, oy i32, w u32, h u32, fattore u32) seguita dai pixel RGBA.
/// Pura, testata.
pub fn encode(seq: u64, ox: i32, oy: i32, w: u32, h: u32, factor: u32, rgba: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(HEADER_LEN + rgba.len());
    v.extend_from_slice(&seq.to_le_bytes());
    v.extend_from_slice(&ox.to_le_bytes());
    v.extend_from_slice(&oy.to_le_bytes());
    v.extend_from_slice(&w.to_le_bytes());
    v.extend_from_slice(&h.to_le_bytes());
    v.extend_from_slice(&factor.to_le_bytes());
    v.extend_from_slice(rgba);
    v
}

/// Il fotogramma più recente del canale, se è più nuovo di `since`; altrimenti un vettore
/// vuoto (il frontend non ridisegna).
pub fn latest(ch: Channel, since: u64) -> Vec<u8> {
    let f = FRAMES[ch as usize].lock().unwrap_or_else(|e| e.into_inner());
    match f.as_ref() {
        Some(fr) if fr.seq > since => encode(fr.seq, fr.ox, fr.oy, fr.w, fr.h, FACTOR as u32, &fr.rgba),
        _ => Vec::new(),
    }
}

/// Cattura il rettangolo `r` dello schermo (BGRA, righe dall'alto, `w*h*4` byte).
fn grab(r: Rect) -> Option<Vec<u8>> {
    if r.w <= 0 || r.h <= 0 {
        return None;
    }
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return None;
        }
        let mem = CreateCompatibleDC(Some(screen));
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: r.w,
                biHeight: -r.h, // righe dall'alto verso il basso
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let out = CreateDIBSection(Some(screen), &info, DIB_RGB_COLORS, &mut bits, None, 0)
            .ok()
            .and_then(|bmp| {
                let old = SelectObject(mem, HGDIOBJ(bmp.0));
                let ok = BitBlt(mem, 0, 0, r.w, r.h, Some(screen), r.x, r.y, SRCCOPY | CAPTUREBLT).is_ok();
                let data = (ok && !bits.is_null())
                    .then(|| std::slice::from_raw_parts(bits as *const u8, (r.w * r.h * 4) as usize).to_vec());
                SelectObject(mem, old);
                let _ = DeleteObject(HGDIOBJ(bmp.0));
                data
            });
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        out
    }
}

/// Scatta una volta sola il rettangolo `r` dello schermo e lo restituisce già ridotto e serializzato
/// (come un fotogramma di `latest`). Lo usa Mission Control per lo sfondo statico, scattato prima di
/// mostrare la panoramica. `None` se la cattura non riesce.
pub fn capture_bytes(r: Rect) -> Option<Vec<u8>> {
    let bgra = grab(r)?;
    let (w, h, rgba) = downscale(&bgra, r.w as usize, r.h as usize, FACTOR);
    (w > 0).then(|| encode(1, 0, 0, w as u32, h as u32, FACTOR as u32, &rgba))
}

/// Avvia il thread di cattura (un giro per canale, solo se ha qualcosa da catturare).
pub fn start() {
    std::thread::spawn(|| {
        let mut last: [Vec<u8>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        loop {
            std::thread::sleep(INTERVAL);
            for ch in CHANNELS {
                // Un panic nella cattura non deve fermare né il Dock né il Launcher.
                let l = &mut last[ch as usize];
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| step(ch, l)));
            }
        }
    });
}

fn step(ch: Channel, last: &mut Vec<u8>) {
    let info = match ch {
        Channel::Dock => crate::dockwin::capture_info(),
        Channel::Launcher => crate::launcherwin::capture_info(),
        Channel::Panel => crate::panelwin::capture_info(),
    };
    let Some((window, capture)) = info else {
        last.clear(); // al prossimo giro si ripubblica: lo sfondo può essere cambiato
        // Niente da catturare (finestra nascosta): il fotogramma vecchio non deve ricomparire
        // alla riapertura, magari in un altro punto dello schermo.
        *FRAMES[ch as usize].lock().unwrap_or_else(|e| e.into_inner()) = None;
        return;
    };
    let Some(bgra) = grab(capture) else { return };
    let (w, h, rgba) = downscale(&bgra, capture.w as usize, capture.h as usize, FACTOR);
    if w == 0 || *last == rgba {
        return;
    }
    last.clone_from(&rgba);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    *FRAMES[ch as usize].lock().unwrap_or_else(|e| e.into_inner()) = Some(Frame {
        seq,
        ox: capture.x - window.x,
        oy: capture.y - window.y,
        w: w as u32,
        h: h as u32,
        rgba,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downscale_averages_blocks_and_swaps_to_rgba() {
        // 2x2 pixel BGRA: blu, rosso, verde, bianco -> un solo pixel medio.
        let src = [
            255, 0, 0, 0, // blu
            0, 0, 255, 0, // rosso
            0, 255, 0, 0, // verde
            255, 255, 255, 0, // bianco
        ];
        let (w, h, out) = downscale(&src, 2, 2, 2);
        assert_eq!((w, h), (1, 1));
        // medie: R=(0+255+0+255)/4=127, G=(0+0+255+255)/4=127, B=(255+0+0+255)/4=127
        assert_eq!(out, vec![127, 127, 127, 255]);
    }

    #[test]
    fn downscale_keeps_channel_order_per_block() {
        // Quattro pixel rossi (BGRA) -> un pixel rosso (RGBA), alfa sempre opaco.
        let src = [0, 0, 200, 0].repeat(4);
        assert_eq!(downscale(&src, 2, 2, 2).2, vec![200, 0, 0, 255]);
    }

    #[test]
    fn downscale_drops_the_odd_edge_and_rejects_bad_input() {
        let src = vec![9u8; 5 * 3 * 4];
        let (w, h, out) = downscale(&src, 5, 3, 2);
        assert_eq!((w, h, out.len()), (2, 1, 2 * 4));
        assert_eq!(downscale(&src, 5, 3, 0).2.len(), 0);
        assert_eq!(downscale(&src[..10], 5, 3, 2).2.len(), 0, "buffer troppo corto");
        assert_eq!(downscale(&src, 1, 1, 2).2.len(), 0, "piu' piccolo del fattore");
    }

    #[test]
    fn frame_header_layout_is_stable() {
        let v = encode(0x0102_0304_0506_0708, -40, 12, 580, 84, 2, &[1, 2, 3, 4]);
        assert_eq!(v.len(), HEADER_LEN + 4);
        assert_eq!(&v[0..8], &0x0102_0304_0506_0708u64.to_le_bytes());
        assert_eq!(i32::from_le_bytes(v[8..12].try_into().unwrap()), -40);
        assert_eq!(i32::from_le_bytes(v[12..16].try_into().unwrap()), 12);
        assert_eq!(u32::from_le_bytes(v[16..20].try_into().unwrap()), 580);
        assert_eq!(u32::from_le_bytes(v[20..24].try_into().unwrap()), 84);
        assert_eq!(u32::from_le_bytes(v[24..28].try_into().unwrap()), 2);
        assert_eq!(&v[28..], &[1, 2, 3, 4]);
    }

    #[test]
    fn latest_returns_nothing_until_there_is_a_newer_frame() {
        // Nessun fotogramma pubblicato in questo test: vuoto per qualunque `since`.
        assert!(latest(Channel::Dock, u64::MAX).is_empty());
        assert!(latest(Channel::Launcher, u64::MAX).is_empty());
        assert!(latest(Channel::Panel, u64::MAX).is_empty());
    }

    #[test]
    fn grab_reads_real_pixels_from_the_screen() {
        // Cattura un rettangolo piccolo in alto a sinistra: deve dare w*h*4 byte.
        let g = grab(Rect { x: 0, y: 0, w: 16, h: 8 });
        // Senza sessione desktop (CI) può non esserci: in quel caso il test non dice nulla.
        if let Some(px) = g {
            assert_eq!(px.len(), 16 * 8 * 4);
        }
        assert!(grab(Rect { x: 0, y: 0, w: 0, h: 5 }).is_none());
    }
}
