//! Launcher (fase L5): cronologia degli appunti, **solo in memoria**. Il testo copiato non viene
//! mai scritto su disco, nei log né mandato altrove: vive in questo processo e sparisce alla
//! chiusura. Solo testo; al massimo 50 voci, ciascuna fino a 10 000 caratteri, con scadenza di
//! 8 ore. Si registra solo se l'utente ha dato il consenso (`launcherstore`), e si ignorano le
//! copie che i gestori di password marcano con `ExcludeClipboardContentFromMonitorProcessing`
//! o con `CanIncludeInClipboardHistory = 0` (lo stesso segnale che rispetta la cronologia di
//! Windows).
//!
//! La logica (`History`, `should_record`, `preview`, `age_label`) è pura e testata; l'ascoltatore
//! (`AddClipboardFormatListener` su una finestra nascosta) e la lettura/scrittura degli appunti
//! stanno in fondo.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
    OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, TranslateMessage, HWND_MESSAGE, MSG,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLIPBOARDUPDATE, WNDCLASSW,
};

use crate::launcherrank::fold;

pub const MAX_ENTRIES: usize = 50;
pub const MAX_CHARS: usize = 10_000;
pub const TTL_MS: u64 = 8 * 60 * 60 * 1000;
const CF_UNICODETEXT: u32 = 13;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: u64,
    pub text: String,
    pub at_ms: u64,
}

/// Cronologia in memoria, dalla più recente.
#[derive(Default, Debug)]
pub struct History {
    items: VecDeque<Entry>,
    next_id: u64,
}

impl History {
    /// Aggiunge `text` all'istante `now`. Scarta il testo vuoto/solo spazi e quello oltre
    /// `MAX_CHARS`. Un testo identico a uno già presente non crea un doppione: la voce risale in
    /// cima (aggiornata). Oltre `MAX_ENTRIES` si scarta la più vecchia. Restituisce se è cambiato
    /// qualcosa. Pura, testata.
    pub fn push(&mut self, text: &str, now: u64) -> bool {
        if text.trim().is_empty() || text.chars().count() > MAX_CHARS {
            return false;
        }
        if let Some(pos) = self.items.iter().position(|e| e.text == text)
            && let Some(mut e) = self.items.remove(pos)
        {
            e.at_ms = now;
            self.items.push_front(e);
            return true;
        }
        self.next_id += 1;
        self.items.push_front(Entry { id: self.next_id, text: text.to_string(), at_ms: now });
        self.items.truncate(MAX_ENTRIES);
        true
    }

    /// Toglie le voci scadute. Restituisce quante.
    pub fn prune(&mut self, now: u64) -> usize {
        let before = self.items.len();
        self.items.retain(|e| now.saturating_sub(e.at_ms) < TTL_MS);
        before - self.items.len()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|e| e.id != id);
        self.items.len() != before
    }

    /// La voce `id`, se esiste e non è scaduta.
    pub fn get(&self, id: u64, now: u64) -> Option<&Entry> {
        self.items.iter().find(|e| e.id == id && now.saturating_sub(e.at_ms) < TTL_MS)
    }

    /// Voci non scadute che contengono ogni parola di `query` (senza badare a maiuscole e accenti),
    /// dalla più recente. Query vuota: tutte. Al massimo `limit`. Pura, testata.
    pub fn search(&self, query: &str, now: u64, limit: usize) -> Vec<&Entry> {
        let q = fold(query);
        let words: Vec<&str> = q.split_whitespace().collect();
        self.items
            .iter()
            .filter(|e| now.saturating_sub(e.at_ms) < TTL_MS)
            .filter(|e| {
                if words.is_empty() {
                    return true;
                }
                let t = fold(&e.text);
                words.iter().all(|w| t.contains(w))
            })
            .take(limit)
            .collect()
    }
}

/// Si deve registrare questa copia? No se l'app ha chiesto di escluderla dal monitoraggio, o se
/// dichiara `CanIncludeInClipboardHistory = 0`. Pura, testata.
pub fn should_record(excluded: bool, can_include: Option<u32>) -> bool {
    !excluded && can_include != Some(0)
}

/// Riga da mostrare: la prima riga non vuota, con gli spazi ridotti e al più `MAX` caratteri
/// (con "…"). Pura, testata.
pub fn preview(text: &str) -> String {
    const MAX: usize = 100;
    let line = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let collapsed = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= MAX {
        collapsed
    } else {
        let head: String = collapsed.chars().take(MAX - 1).collect();
        format!("{head}…")
    }
}

/// Età leggibile di una voce. Pura, testata.
pub fn age_label(now: u64, at: u64) -> String {
    let secs = now.saturating_sub(at) / 1000;
    match secs {
        0..=44 => "adesso".into(),
        45..=3599 => format!("{} min fa", (secs / 60).max(1)),
        _ => format!("{} h fa", secs / 3600),
    }
}

// ------------------------------------------------------------------ stato globale

static HISTORY: Mutex<History> = Mutex::new(History { items: VecDeque::new(), next_id: 0 });
static ENABLED: AtomicBool = AtomicBool::new(false);
static OWNER: AtomicIsize = AtomicIsize::new(0);

fn hist() -> std::sync::MutexGuard<'static, History> {
    HISTORY.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Attiva o disattiva la registrazione. Disattivando la cronologia viene svuotata subito.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
    if !on {
        hist().clear();
    }
}

pub fn clear() {
    hist().clear();
}

/// Voci per la ricerca: (id, anteprima, età).
pub fn list(query: &str, limit: usize) -> Vec<(u64, String, String)> {
    let now = now_ms();
    let mut h = hist();
    h.prune(now);
    h.search(query, now, limit).into_iter().map(|e| (e.id, preview(&e.text), age_label(now, e.at_ms))).collect()
}

pub fn remove(id: u64) -> bool {
    hist().remove(id)
}

/// Rimette negli appunti il testo della voce `id` (se esiste ancora). Il testo non passa dalla pagina.
pub fn copy_back(id: u64) -> bool {
    let text = match hist().get(id, now_ms()) {
        Some(e) => e.text.clone(),
        None => return false,
    };
    write_text(&text)
}

// ------------------------------------------------------------------ Windows

fn format_id(name: PCWSTR) -> u32 {
    unsafe { RegisterClipboardFormatW(name) }
}

unsafe fn read_global_u32(h: HANDLE) -> Option<u32> {
    unsafe {
        let g = HGLOBAL(h.0);
        if GlobalSize(g) < 4 {
            return None;
        }
        let p = GlobalLock(g) as *const u32;
        if p.is_null() {
            return None;
        }
        let v = p.read_unaligned();
        let _ = GlobalUnlock(g);
        Some(v)
    }
}

/// Legge il testo corrente degli appunti rispettando le esclusioni. `None` se non c'è testo, se la
/// copia è esclusa, se è troppo grande o se gli appunti restano occupati da un altro programma.
fn read_current(owner: HWND) -> Option<String> {
    unsafe {
        let mut opened = false;
        for _ in 0..8 {
            if OpenClipboard(Some(owner)).is_ok() {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if !opened {
            return None;
        }
        let result = (|| {
            let excluded = IsClipboardFormatAvailable(format_id(w!("ExcludeClipboardContentFromMonitorProcessing"))).is_ok();
            let can_include = GetClipboardData(format_id(w!("CanIncludeInClipboardHistory")))
                .ok()
                .and_then(|h| read_global_u32(h));
            if !should_record(excluded, can_include) || IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
                return None;
            }
            let h = GetClipboardData(CF_UNICODETEXT).ok()?;
            let g = HGLOBAL(h.0);
            let bytes = GlobalSize(g);
            if !(2..=(MAX_CHARS + 1) * 4).contains(&bytes) {
                return None; // vuoto o enorme: non si copia in memoria
            }
            let p = GlobalLock(g) as *const u16;
            if p.is_null() {
                return None;
            }
            let units = bytes / 2;
            let slice = std::slice::from_raw_parts(p, units);
            let len = slice.iter().position(|&c| c == 0).unwrap_or(units);
            let text = String::from_utf16_lossy(&slice[..len]);
            let _ = GlobalUnlock(g);
            Some(text)
        })();
        let _ = CloseClipboard();
        result
    }
}

/// Scrive `text` negli appunti come testo Unicode.
fn write_text(text: &str) -> bool {
    let owner = HWND(OWNER.load(Ordering::Relaxed) as *mut _);
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let mut opened = false;
        for _ in 0..8 {
            if OpenClipboard(Some(owner)).is_ok() {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if !opened {
            return false;
        }
        let ok = (|| {
            EmptyClipboard().ok()?;
            let g = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2).ok()?;
            let p = GlobalLock(g) as *mut u16;
            if p.is_null() {
                return None;
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), p, wide.len());
            let _ = GlobalUnlock(g);
            SetClipboardData(CF_UNICODETEXT, Some(HANDLE(g.0))).ok()?;
            Some(())
        })()
        .is_some();
        let _ = CloseClipboard();
        ok
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE && enabled()
        && let Some(text) = read_current(hwnd)
    {
        hist().push(&text, now_ms());
    }
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}

/// Avvia l'ascoltatore degli appunti (finestra nascosta solo-messaggi) e legge il consenso salvato.
pub fn start() {
    ENABLED.store(crate::launcherstore::settings().clipboard == Some(true), Ordering::Relaxed);
    std::thread::spawn(|| unsafe {
        let Ok(hi) = GetModuleHandleW(None) else { return };
        let class = w!("EosOSClipboardListener");
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hi.into(), lpszClassName: class, ..Default::default() };
        RegisterClassW(&wc);
        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            w!(""),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(hi.into()),
            None,
        ) else {
            return;
        };
        OWNER.store(hwnd.0 as isize, Ordering::Relaxed);
        if AddClipboardFormatListener(hwnd).is_err() {
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: u64 = 1_000_000_000;

    #[test]
    fn push_ignores_blank_and_oversized_text() {
        let mut h = History::default();
        assert!(!h.push("", T0));
        assert!(!h.push("   \n\t ", T0));
        assert!(!h.push(&"x".repeat(MAX_CHARS + 1), T0));
        assert!(h.push(&"x".repeat(MAX_CHARS), T0));
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn newest_first_and_duplicates_move_up_without_growing() {
        let mut h = History::default();
        h.push("uno", T0);
        h.push("due", T0 + 1);
        h.push("tre", T0 + 2);
        h.push("uno", T0 + 3);
        let texts: Vec<_> = h.search("", T0 + 4, 10).iter().map(|e| e.text.clone()).collect();
        assert_eq!(texts, ["uno", "tre", "due"]);
        assert_eq!(h.len(), 3);
        // L'id della voce ripetuta resta lo stesso (la pagina puo' averlo in mano).
        let id = h.search("uno", T0 + 4, 1)[0].id;
        h.push("uno", T0 + 5);
        assert_eq!(h.search("uno", T0 + 6, 1)[0].id, id);
    }

    #[test]
    fn capped_at_max_entries_dropping_the_oldest() {
        let mut h = History::default();
        for i in 0..(MAX_ENTRIES + 10) {
            h.push(&format!("voce {i}"), T0 + i as u64);
        }
        assert_eq!(h.len(), MAX_ENTRIES);
        let all = h.search("", T0 + 1000, 1000);
        assert_eq!(all[0].text, format!("voce {}", MAX_ENTRIES + 9));
        assert!(!all.iter().any(|e| e.text == "voce 0"));
    }

    #[test]
    fn entries_expire_after_the_ttl() {
        let mut h = History::default();
        h.push("vecchia", T0);
        h.push("recente", T0 + TTL_MS - 1000);
        let now = T0 + TTL_MS + 1;
        assert_eq!(h.search("", now, 10).len(), 1, "la scaduta non compare nemmeno prima di prune");
        assert_eq!(h.prune(now), 1);
        assert_eq!(h.len(), 1);
        let id = h.search("", now, 1)[0].id;
        assert!(h.get(id, now).is_some());
        assert!(h.get(id, T0 + 2 * TTL_MS + 10).is_none(), "get rispetta la scadenza");
        // Un orologio che torna indietro non scade nulla.
        assert_eq!(h.prune(T0 - 5000), 0);
    }

    #[test]
    fn search_requires_every_word_ignoring_case_and_accents() {
        let mut h = History::default();
        h.push("Il caffè è pronto", T0);
        h.push("riunione alle 15", T0 + 1);
        assert_eq!(h.search("CAFFE", T0 + 2, 10).len(), 1);
        assert_eq!(h.search("pronto caffe", T0 + 2, 10).len(), 1, "ordine libero");
        assert_eq!(h.search("caffe riunione", T0 + 2, 10).len(), 0);
        assert_eq!(h.search("zzz", T0 + 2, 10).len(), 0);
        assert_eq!(h.search("  ", T0 + 2, 10).len(), 2, "query vuota: tutte");
        assert_eq!(h.search("", T0 + 2, 1).len(), 1, "limite");
    }

    #[test]
    fn remove_and_clear() {
        let mut h = History::default();
        h.push("a", T0);
        h.push("b", T0 + 1);
        let id = h.search("a", T0 + 2, 1)[0].id;
        assert!(h.remove(id));
        assert!(!h.remove(id));
        assert_eq!(h.len(), 1);
        h.clear();
        assert_eq!(h.len(), 0);
        assert!(h.get(id, T0).is_none());
    }

    #[test]
    fn password_manager_copies_are_not_recorded() {
        assert!(should_record(false, None), "copia normale");
        assert!(should_record(false, Some(1)), "esplicitamente ammessa");
        assert!(!should_record(true, None), "ExcludeClipboardContentFromMonitorProcessing");
        assert!(!should_record(false, Some(0)), "CanIncludeInClipboardHistory = 0");
        assert!(!should_record(true, Some(0)));
        assert!(should_record(false, Some(7)), "qualsiasi valore diverso da 0 ammette");
    }

    #[test]
    fn preview_takes_the_first_meaningful_line_and_truncates() {
        assert_eq!(preview("  \n\n  ciao   mondo \nsecondo"), "ciao mondo");
        assert_eq!(preview(""), "");
        assert_eq!(preview("\n \n"), "");
        let long = "a".repeat(300);
        let p = preview(&long);
        assert_eq!(p.chars().count(), 100);
        assert!(p.ends_with('…'));
        assert_eq!(preview(&"é".repeat(100)).chars().count(), 100, "100 esatti: intero");
    }

    #[test]
    fn age_labels() {
        assert_eq!(age_label(T0, T0), "adesso");
        assert_eq!(age_label(T0 + 44_000, T0), "adesso");
        assert_eq!(age_label(T0 + 60_000, T0), "1 min fa");
        assert_eq!(age_label(T0 + 59 * 60_000, T0), "59 min fa");
        assert_eq!(age_label(T0 + 3 * 3_600_000, T0), "3 h fa");
        assert_eq!(age_label(T0, T0 + 5000), "adesso", "orologio indietro");
    }

    #[test]
    fn disabling_clears_the_global_history() {
        // Stato globale: usa un testo unico per non interferire con altri test.
        let marker = format!("marker-{}", std::process::id());
        hist().push(&marker, now_ms());
        assert!(!list(&marker, 5).is_empty());
        set_enabled(false);
        assert!(list(&marker, 5).is_empty());
        assert!(!enabled());
    }
}
