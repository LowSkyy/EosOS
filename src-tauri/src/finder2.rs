//! Finder di EosOS, fase 2: appunti condivisi con Esplora file, trascinamento dei file (dentro, fuori e fra le finestre), ricerca
//! nelle sottocartelle, Cestino (elenco, ripristino, eliminazione), etichette colorate, informazioni e anteprima rapida.
//! Le finestre e le operazioni di base sono in `finder.rs`.

use std::collections::BTreeMap;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::Shell::{DragQueryFileW, SHEmptyRecycleBinW, HDROP, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};

use crate::finder::{ext_of, Entry};

const CF_HDROP: u32 = 15;
const DROPEFFECT_COPY: u32 = 1;
const DROPEFFECT_MOVE: u32 = 2;
const DROPEFFECT_LINK: u32 = 4;

// ------------------------------------------------------------------ appunti di Windows (gli stessi di Esplora file)

/// Il blocco `DROPFILES` con l'elenco dei file (UTF-16, separati da zero, chiuso da due zeri). Pura, testata.
pub fn dropfiles(paths: &[String]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend(20u32.to_le_bytes()); // pFiles: subito dopo l'intestazione
    out.extend(0i32.to_le_bytes()); // pt.x
    out.extend(0i32.to_le_bytes()); // pt.y
    out.extend(0u32.to_le_bytes()); // fNC
    out.extend(1u32.to_le_bytes()); // fWide
    for p in paths {
        for u in p.encode_utf16() {
            out.extend(u.to_le_bytes());
        }
        out.extend(0u16.to_le_bytes());
    }
    out.extend(0u16.to_le_bytes());
    out
}

fn global(bytes: &[u8]) -> Option<HGLOBAL> {
    unsafe {
        let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len()).ok()?;
        let p = GlobalLock(h) as *mut u8;
        if p.is_null() {
            return None;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
        let _ = GlobalUnlock(h);
        Some(h)
    }
}

fn effect_format() -> u32 {
    unsafe { RegisterClipboardFormatW(&HSTRING::from("Preferred DropEffect")) }
}

/// Apre gli appunti, riprovando per un attimo se un'altra app li tiene aperti.
fn open_clipboard() -> bool {
    for _ in 0..10 {
        if unsafe { OpenClipboard(None) }.is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    false
}

/// Copia (o taglia) i file negli appunti di Windows: si incollano anche in Esplora file, nelle email, nelle chat.
pub fn clip_set(paths: &[String], cut: bool) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    if !open_clipboard() {
        return Err("Gli appunti sono occupati da un'altra app.".into());
    }
    let res = unsafe {
        let _ = EmptyClipboard();
        let files = global(&dropfiles(paths)).ok_or("Memoria non disponibile.")?;
        let ok = SetClipboardData(CF_HDROP, Some(HANDLE(files.0))).is_ok();
        let effect = if cut { DROPEFFECT_MOVE } else { DROPEFFECT_COPY | DROPEFFECT_LINK };
        if let Some(e) = global(&effect.to_le_bytes()) {
            let _ = SetClipboardData(effect_format(), Some(HANDLE(e.0)));
        }
        if ok { Ok(()) } else { Err("Non riesco a scrivere negli appunti.".into()) }
    };
    unsafe {
        let _ = CloseClipboard();
    }
    res
}

/// I file negli appunti di Windows (da EosOS o da Esplora file) e se vanno spostati (tagliati) invece che copiati.
pub fn clip_get() -> (Vec<String>, bool) {
    unsafe {
        if IsClipboardFormatAvailable(CF_HDROP).is_err() || !open_clipboard() {
            return (Vec::new(), false);
        }
        let mut paths = Vec::new();
        if let Ok(h) = GetClipboardData(CF_HDROP) {
            let drop = HDROP(h.0);
            let n = DragQueryFileW(drop, u32::MAX, None);
            for i in 0..n {
                let len = DragQueryFileW(drop, i, None) as usize;
                let mut buf = vec![0u16; len + 1];
                DragQueryFileW(drop, i, Some(&mut buf));
                paths.push(String::from_utf16_lossy(&buf[..len]));
            }
        }
        let mut cut = false;
        if let Ok(h) = GetClipboardData(effect_format()) {
            let g = HGLOBAL(h.0);
            let p = GlobalLock(g) as *const u32;
            if !p.is_null() && GlobalSize(g) >= 4 {
                cut = *p & DROPEFFECT_MOVE != 0 && *p & DROPEFFECT_COPY == 0;
            }
            let _ = GlobalUnlock(g);
        }
        let _ = CloseClipboard();
        (paths, cut)
    }
}

/// Dopo uno spostamento da «taglia» gli appunti si svuotano (come fa Esplora file).
pub fn clip_clear() {
    if open_clipboard() {
        unsafe {
            let _ = EmptyClipboard();
            let _ = CloseClipboard();
        }
    }
}

// ------------------------------------------------------------------ trascinamento

/// Lascia dei file su una cartella: stessa unita' = sposta, altra unita' (o Ctrl premuto) = copia, come Esplora file. I file gia'
/// in quella cartella, o la cartella dentro se stessa, si saltano.
pub fn drop_into(paths: &[String], dest: &str, force_copy: bool) -> Result<(), String> {
    let dest_l = dest.trim_end_matches('\\').to_lowercase();
    let list: Vec<String> = paths
        .iter()
        .filter(|p| {
            let pl = p.trim_end_matches('\\').to_lowercase();
            let parent = Path::new(p).parent().map(|x| x.to_string_lossy().trim_end_matches('\\').to_lowercase());
            parent.as_deref() != Some(dest_l.as_str()) && !(dest_l == pl || dest_l.starts_with(&format!("{pl}\\")))
        })
        .cloned()
        .collect();
    if list.is_empty() {
        return Ok(());
    }
    let same = |p: &String| p.get(..2).map(str::to_lowercase) == dest.get(..2).map(str::to_lowercase);
    let mv = !force_copy && list.iter().all(same);
    crate::finder::transfer(&list, dest, mv)
}

/// Comincia il trascinamento dei file verso un'altra app, Esplora file o un'altra finestra del Finder. Va chiamata dal thread
/// principale, mentre il tasto del mouse e' premuto: il ciclo del trascinamento e' quello di Windows.
pub fn drag_out(paths: &[String]) {
    use windows::Win32::System::Com::IDataObject;
    use windows::Win32::UI::Shell::Common::ITEMIDLIST;
    use windows::Win32::UI::Shell::{BHID_DataObject, IShellItemArray, ILFree, SHCreateShellItemArrayFromIDLists, SHDoDragDrop, SHParseDisplayName};
    use windows::Win32::System::Ole::DROPEFFECT;
    unsafe {
        let mut pidls: Vec<*mut ITEMIDLIST> = Vec::new();
        for p in paths {
            let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
            if SHParseDisplayName(&HSTRING::from(p.as_str()), None, &mut pidl, 0, None).is_ok() && !pidl.is_null() {
                pidls.push(pidl);
            }
        }
        if !pidls.is_empty() {
            let consts: Vec<*const ITEMIDLIST> = pidls.iter().map(|p| *p as *const _).collect();
            if let Ok(arr) = SHCreateShellItemArrayFromIDLists(&consts) {
                let arr: IShellItemArray = arr;
                if let Ok(obj) = arr.BindToHandler::<_, IDataObject>(None::<&windows::Win32::System::Com::IBindCtx>, &BHID_DataObject) {
                    let _ = SHDoDragDrop(None::<HWND>, &obj, None, DROPEFFECT(DROPEFFECT_COPY | DROPEFFECT_MOVE | DROPEFFECT_LINK));
                }
            }
        }
        for p in pidls {
            ILFree(Some(p));
        }
    }
}

// ------------------------------------------------------------------ ricerca nelle sottocartelle

static SEARCH: AtomicU64 = AtomicU64::new(0);

/// Il nome contiene tutte le parole cercate (senza maiuscole)? Pura, testata.
pub fn matches(name: &str, query: &str) -> bool {
    let n = name.to_lowercase();
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    !words.is_empty() && words.iter().all(|w| n.contains(w.as_str()))
}

/// Cerca per nome in `root` e nelle sottocartelle (al massimo 500 risultati e 6 secondi). Una nuova ricerca ferma la precedente.
pub fn search(root: &str, query: &str) -> Vec<Entry> {
    let me = SEARCH.fetch_add(1, Ordering::SeqCst) + 1;
    let start = Instant::now();
    let mut out = Vec::new();
    let mut queue = std::collections::VecDeque::from([PathBuf::from(root)]);
    while let Some(dir) = queue.pop_front() {
        if SEARCH.load(Ordering::SeqCst) != me || start.elapsed() > Duration::from_secs(6) || out.len() >= 500 {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for item in rd.flatten() {
            let Ok(md) = item.metadata() else { continue };
            let name = item.file_name().to_string_lossy().to_string();
            let attrs = md.file_attributes();
            let hidden = attrs & 0x6 != 0 || name.starts_with('.');
            // Le cartelle di sistema nascoste e i collegamenti di cartella (giunzioni) non si visitano.
            if md.is_dir() && !hidden && attrs & 0x400 == 0 {
                queue.push_back(item.path());
            }
            if matches(&name, query) && !hidden {
                out.push(entry_of(&item.path(), &md, name));
            }
        }
    }
    out
}

fn entry_of(path: &Path, md: &std::fs::Metadata, name: String) -> Entry {
    let modified = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs() as i64);
    Entry {
        path: path.to_string_lossy().to_string(),
        dir: md.is_dir(),
        size: if md.is_dir() { 0 } else { md.len() },
        modified,
        ext: if md.is_dir() { String::new() } else { ext_of(&name) },
        hidden: md.file_attributes() & 0x6 != 0 || name.starts_with('.'),
        name,
    }
}

/// Le voci di un elenco di percorsi (per le etichette): quelle che non esistono piu' si saltano.
pub fn entries(paths: &[String]) -> Vec<Entry> {
    paths
        .iter()
        .filter_map(|p| {
            let path = Path::new(p);
            let md = std::fs::metadata(path).ok()?;
            let name = path.file_name().map_or_else(|| p.clone(), |n| n.to_string_lossy().to_string());
            Some(entry_of(path, &md, name))
        })
        .collect()
}

// ------------------------------------------------------------------ Cestino

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct TrashItem {
    /// Il file nel Cestino (`$R...`): identifica la voce.
    pub id: String,
    pub name: String,
    /// Dove stava.
    pub origin: String,
    pub dir: bool,
    pub size: u64,
    /// Quando e' stato eliminato, secondi dal 1970.
    pub deleted: i64,
    pub ext: String,
}

/// Legge un file `$I` del Cestino (versione 1 o 2): dimensione, quando, percorso originale. Pura, testata.
pub fn parse_info(bytes: &[u8]) -> Option<(u64, i64, String)> {
    let u64_at = |o: usize| bytes.get(o..o + 8).map(|b| u64::from_le_bytes(b.try_into().unwrap_or([0; 8])));
    let version = u64_at(0)?;
    let size = u64_at(8)?;
    let ft = u64_at(16)?;
    let deleted = (ft / 10_000_000) as i64 - 11_644_473_600;
    let raw: &[u8] = match version {
        2 => {
            let n = u32::from_le_bytes(bytes.get(24..28)?.try_into().ok()?) as usize;
            bytes.get(28..28 + n * 2)?
        }
        1 => bytes.get(24..24 + 520)?,
        _ => return None,
    };
    let units: Vec<u16> = raw.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).take_while(|u| *u != 0).collect();
    let path = String::from_utf16_lossy(&units);
    (!path.is_empty()).then_some((size, deleted, path))
}

/// Le cartelle del Cestino dell'utente, una per unita' (`X:\$Recycle.Bin\<SID>`): quelle che si possono leggere sono sue.
fn trash_dirs() -> Vec<PathBuf> {
    let mask = unsafe { windows::Win32::Storage::FileSystem::GetLogicalDrives() };
    let mut out = Vec::new();
    for i in 0..26u32 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let root = PathBuf::from(format!("{}:\\$Recycle.Bin", (b'A' + i as u8) as char));
        let Ok(rd) = std::fs::read_dir(&root) else { continue };
        for d in rd.flatten() {
            if d.file_name().to_string_lossy().starts_with("S-1-5-21") && std::fs::read_dir(d.path()).is_ok() {
                out.push(d.path());
            }
        }
    }
    out
}

/// Cosa c'e' nel Cestino, dal piu' recente.
pub fn trash_list() -> Vec<TrashItem> {
    let mut out = Vec::new();
    for dir in trash_dirs() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for f in rd.flatten() {
            let name = f.file_name().to_string_lossy().to_string();
            let Some(rest) = name.strip_prefix("$I") else { continue };
            let data = dir.join(format!("$R{rest}"));
            let Ok(md) = std::fs::metadata(&data) else { continue };
            let Some((size, deleted, origin)) = std::fs::read(f.path()).ok().and_then(|b| parse_info(&b)) else { continue };
            let shown = Path::new(&origin).file_name().map_or_else(|| origin.clone(), |n| n.to_string_lossy().to_string());
            out.push(TrashItem { id: data.to_string_lossy().to_string(), ext: if md.is_dir() { String::new() } else { ext_of(&shown) }, name: shown, origin, dir: md.is_dir(), size, deleted });
        }
    }
    out.sort_by_key(|t| std::cmp::Reverse(t.deleted));
    out
}

fn info_of(id: &str) -> Option<PathBuf> {
    let p = Path::new(id);
    let name = p.file_name()?.to_string_lossy().to_string();
    Some(p.with_file_name(format!("$I{}", name.strip_prefix("$R")?)))
}

/// Rimette gli elementi dove stavano (con un nome nuovo se nel frattempo li' c'e' gia' qualcosa). Risponde con quanti.
pub fn trash_restore(ids: &[String]) -> Result<usize, String> {
    let mut n = 0;
    for id in ids {
        let Some(info) = info_of(id) else { continue };
        let Some((_, _, origin)) = std::fs::read(&info).ok().and_then(|b| parse_info(&b)) else { continue };
        let target = Path::new(&origin);
        let parent = target.parent().ok_or("Percorso originale non valido.")?;
        std::fs::create_dir_all(parent).map_err(|e| format!("Non riesco a ricreare la cartella di origine: {e}"))?;
        let stem = target.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext = target.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let free = crate::finder::free_name(&stem, &|s| parent.join(format!("{s}{ext}")).exists());
        std::fs::rename(id, parent.join(format!("{free}{ext}"))).map_err(|e| format!("Non riesco a ripristinare «{}»: {e}", target.display()))?;
        let _ = std::fs::remove_file(&info);
        n += 1;
    }
    notify_trash();
    Ok(n)
}

/// Elimina per sempre alcuni elementi del Cestino.
pub fn trash_delete(ids: &[String]) -> Result<usize, String> {
    let mut n = 0;
    for id in ids {
        let Some(info) = info_of(id) else { continue };
        let p = Path::new(id);
        let res = if p.is_dir() { std::fs::remove_dir_all(p) } else { std::fs::remove_file(p) };
        res.map_err(|e| format!("Non riesco a eliminare: {e}"))?;
        let _ = std::fs::remove_file(info);
        n += 1;
    }
    notify_trash();
    Ok(n)
}

/// Svuota il Cestino (la conferma la chiede la pagina).
pub fn trash_empty() -> Result<(), String> {
    unsafe { SHEmptyRecycleBinW(None, PCWSTR::null(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND) }.map_err(|e| format!("Non riesco a svuotare il Cestino: {e}"))
}

fn notify_trash() {
    use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_UPDATEDIR, SHCNF_PATHW};
    for d in trash_dirs() {
        let w = HSTRING::from(d.to_string_lossy().as_ref());
        unsafe { SHChangeNotify(SHCNE_UPDATEDIR, SHCNF_PATHW, Some(w.as_ptr() as *const _), None) };
    }
}

// ------------------------------------------------------------------ etichette

/// Le etichette colorate di macOS: l'id e il nome.
pub const TAGS: &[(&str, &str)] = &[("red", "Rosso"), ("orange", "Arancione"), ("yellow", "Giallo"), ("green", "Verde"), ("blue", "Blu"), ("purple", "Viola"), ("gray", "Grigio")];

static TAG_LOCK: Mutex<()> = Mutex::new(());

fn tags_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\tags.json"))
}

/// Tutte le etichette: percorso -> colori.
pub fn tags() -> BTreeMap<String, Vec<String>> {
    tags_path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save_tags(t: &BTreeMap<String, Vec<String>>) {
    if let Some(p) = tags_path() {
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Ok(s) = serde_json::to_string_pretty(t) {
            let _ = std::fs::write(p, s);
        }
    }
}

/// Mette o toglie un'etichetta. Pura rispetto alla mappa, testata.
pub fn toggle_tag(map: &mut BTreeMap<String, Vec<String>>, paths: &[String], color: &str, on: bool) {
    for p in paths {
        let list = map.entry(p.clone()).or_default();
        list.retain(|c| c != color);
        if on {
            list.push(color.to_string());
        }
    }
    map.retain(|_, v| !v.is_empty());
}

pub fn tag(paths: &[String], color: &str, on: bool) -> BTreeMap<String, Vec<String>> {
    let _g = TAG_LOCK.lock();
    let mut t = tags();
    if TAGS.iter().any(|(id, _)| *id == color) {
        toggle_tag(&mut t, paths, color, on);
        save_tags(&t);
    }
    t
}

/// Un elemento rinominato o spostato si porta dietro le sue etichette.
pub fn tag_moved(from: &str, to: &str) {
    let _g = TAG_LOCK.lock();
    let mut t = tags();
    if let Some(v) = t.remove(from) {
        t.insert(to.to_string(), v);
        save_tags(&t);
    }
}

// ------------------------------------------------------------------ informazioni e anteprima

#[derive(Serialize, Clone, Debug, Default)]
pub struct Info {
    pub size: u64,
    /// Per le cartelle: quanti elementi contiene (in tutto, sottocartelle comprese).
    pub items: u64,
    /// Il conteggio si e' fermato prima della fine (cartelle enormi).
    pub partial: bool,
    pub created: i64,
    pub modified: i64,
    pub accessed: i64,
    pub readonly: bool,
    pub hidden: bool,
}

fn secs(t: std::io::Result<std::time::SystemTime>) -> i64 {
    t.ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs() as i64)
}

/// Le informazioni su un elemento; per le cartelle si sommano i file (al massimo 5 secondi).
pub fn info(path: &str) -> Option<Info> {
    let md = std::fs::metadata(path).ok()?;
    let attrs = md.file_attributes();
    let mut out = Info { size: md.len(), created: secs(md.created()), modified: secs(md.modified()), accessed: secs(md.accessed()), readonly: attrs & 1 != 0, hidden: attrs & 2 != 0, ..Default::default() };
    if md.is_dir() {
        out.size = 0;
        let start = Instant::now();
        let mut stack = vec![PathBuf::from(path)];
        while let Some(d) = stack.pop() {
            if start.elapsed() > Duration::from_secs(5) {
                out.partial = true;
                break;
            }
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for item in rd.flatten() {
                let Ok(m) = item.metadata() else { continue };
                out.items += 1;
                if m.is_dir() {
                    if m.file_attributes() & 0x400 == 0 {
                        stack.push(item.path());
                    }
                } else {
                    out.size += m.len();
                }
            }
        }
    }
    Some(out)
}

/// Estensioni che l'anteprima mostra come testo.
const TEXTS: &[&str] = &["txt", "log", "md", "ini", "cfg", "csv", "json", "xml", "html", "css", "js", "ts", "tsx", "jsx", "py", "rs", "cs", "cpp", "c", "h", "java", "sh", "ps1", "bat", "yml", "yaml", "toml", "sql", "reg"];

#[derive(Serialize, Clone, Debug, Default)]
pub struct Peek {
    /// Le prime righe (testi e codice).
    pub text: Option<String>,
    /// Un'immagine grande (anteprima di Windows), come data URL.
    pub image: Option<String>,
}

/// L'anteprima rapida (barra spaziatrice): testo per i testi, immagine grande per il resto.
pub fn peek(path: &str) -> Peek {
    let ext = ext_of(path);
    if TEXTS.contains(&ext.as_str()) {
        let mut buf = vec![0u8; 64 * 1024];
        let n = std::fs::File::open(path).and_then(|mut f| std::io::Read::read(&mut f, &mut buf)).unwrap_or(0);
        buf.truncate(n);
        return Peek { text: Some(String::from_utf8_lossy(&buf).to_string()), image: None };
    }
    Peek { text: None, image: crate::dock::shell_image_data_url(Path::new(path), 768, true) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropfiles_has_the_header_and_double_zero() {
        let b = dropfiles(&["C:\\a".into(), "D:\\b".into()]);
        assert_eq!(&b[..4], &20u32.to_le_bytes());
        assert_eq!(&b[16..20], &1u32.to_le_bytes());
        let tail: Vec<u16> = b[20..].as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
        assert_eq!(String::from_utf16_lossy(&tail), "C:\\a\0D:\\b\0\0");
    }

    #[test]
    fn search_words_must_all_appear() {
        assert!(matches("Relazione Finale 2025.docx", "finale rel"));
        assert!(!matches("Relazione.docx", "finale"));
        assert!(!matches("x", "  "));
    }

    #[test]
    fn recycle_bin_info_files_are_read() {
        let path = "C:\\Dati\\nota.txt";
        let units: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let mut b = Vec::new();
        b.extend(2u64.to_le_bytes());
        b.extend(1234u64.to_le_bytes());
        b.extend(((1_700_000_000u64 + 11_644_473_600) * 10_000_000).to_le_bytes());
        b.extend((units.len() as u32).to_le_bytes());
        for u in units {
            b.extend(u.to_le_bytes());
        }
        assert_eq!(parse_info(&b), Some((1234, 1_700_000_000, path.to_string())));
        assert_eq!(parse_info(&[0; 10]), None);
    }

    #[test]
    fn tags_toggle_and_disappear_when_empty() {
        let mut m = BTreeMap::new();
        toggle_tag(&mut m, &["a".into(), "b".into()], "red", true);
        toggle_tag(&mut m, &["a".into()], "blue", true);
        toggle_tag(&mut m, &["a".into()], "red", true);
        assert_eq!(m["a"], vec!["blue".to_string(), "red".to_string()]);
        toggle_tag(&mut m, &["b".into()], "red", false);
        assert!(!m.contains_key("b"));
    }

    #[test]
    fn dropping_into_the_same_folder_or_into_itself_does_nothing() {
        assert!(drop_into(&["C:\\x\\a.txt".into()], "C:\\x", false).is_ok());
        assert!(drop_into(&["C:\\x".into()], "C:\\x\\sotto", false).is_ok());
    }
}
