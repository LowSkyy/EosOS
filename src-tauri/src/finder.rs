//! Il Finder di EosOS: una finestra propria per sfogliare cartelle e file, con l'aspetto del Finder di macOS (semafori a sinistra,
//! barra laterale, barra degli strumenti, viste a icone e a elenco, barra del percorso). L'interno di Esplora file lo disegna
//! Windows e non si puo' ridisegnare: per averlo come su macOS serve una finestra nostra.
//!
//! Qui stanno le finestre (una per `apri`, etichette `finder-N`) e le operazioni sui file: elenco, apertura, rinomina, cestino,
//! nuova cartella, copia/sposta (con le finestre di avanzamento e l'annulla di Windows). La pagina e' `src/components/Finder.tsx`.

use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use windows::core::{w, GUID, HSTRING, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{GetLogicalDrives, GetVolumeInformationW};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
    SHFileOperationW, SHGetKnownFolderPath, ShellExecuteW, FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_Music, FOLDERID_Pictures,
    FOLDERID_Profile, FOLDERID_Videos, KF_FLAG_DEFAULT, SHFILEOPSTRUCTW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

pub const PREFIX: &str = "finder-";
/// Il «percorso» del Cestino nella barra laterale (non e' una cartella vera: lo elenca `finder2::trash_list`).
pub const TRASH: &str = "eosos:trash";
static NEXT: AtomicU32 = AtomicU32::new(1);

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;

pub fn is_label(label: &str) -> bool {
    label.starts_with(PREFIX)
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub dir: bool,
    pub size: u64,
    /// Ultima modifica, secondi dal 1970.
    pub modified: i64,
    /// Estensione minuscola senza punto (vuota per le cartelle e i file senza estensione).
    pub ext: String,
    pub hidden: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct Listing {
    pub path: String,
    pub name: String,
    pub parent: Option<String>,
    pub entries: Vec<Entry>,
    pub error: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Place {
    pub name: String,
    pub path: String,
    /// home, desktop, documents, downloads, pictures, music, videos, cloud, trash, drive.
    pub kind: &'static str,
}

/// Estensione minuscola senza punto. Pura, testata.
pub fn ext_of(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, e)) if !stem.is_empty() && !e.is_empty() && e.len() <= 10 && !e.contains(' ') => e.to_lowercase(),
        _ => String::new(),
    }
}

/// Il nome da mostrare per una cartella: l'ultimo pezzo del percorso, o l'unita' («C:»). Pura, testata.
pub fn display_name(path: &str) -> String {
    let p = path.trim_end_matches(['\\', '/']);
    match p.rsplit(['\\', '/']).next() {
        Some(last) if !last.is_empty() => last.to_string(),
        _ => p.to_string(),
    }
}

/// La cartella che contiene `path`, o niente per la radice di un'unita'. Pura, testata.
pub fn parent_of(path: &str) -> Option<String> {
    let p = path.trim_end_matches(['\\', '/']);
    let (head, _) = p.rsplit_once(['\\', '/'])?;
    if head.is_empty() {
        return None;
    }
    // «C:» da solo e' la cartella corrente dell'unita', non la radice: ci vuole la barra.
    Some(if head.ends_with(':') { format!("{head}\\") } else { head.to_string() })
}

/// Un nome di file accettabile per Windows? Pura, testata.
pub fn valid_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.len() <= 240 && !n.ends_with('.') && !n.chars().any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control())
}

/// Il primo nome libero fra «base», «base 2», «base 3»... Pura rispetto a `taken`, testata.
pub fn free_name(base: &str, taken: &dyn Fn(&str) -> bool) -> String {
    if !taken(base) {
        return base.to_string();
    }
    (2..10_000).map(|n| format!("{base} {n}")).find(|n| !taken(n)).unwrap_or_else(|| base.to_string())
}

/// Il contenuto di una cartella (cartelle prima, poi per nome senza distinguere le maiuscole).
pub fn list(path: &str) -> Listing {
    let dir = Path::new(path);
    let mut out = Listing { path: path.to_string(), name: display_name(path), parent: parent_of(path), entries: Vec::new(), error: None };
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            out.error = Some(match e.kind() {
                std::io::ErrorKind::PermissionDenied => "Non hai il permesso di aprire questa cartella.".to_string(),
                std::io::ErrorKind::NotFound => "La cartella non esiste piu'.".to_string(),
                _ => format!("Non riesco ad aprire la cartella: {e}"),
            });
            return out;
        }
    };
    for item in rd.flatten() {
        let Ok(md) = item.metadata() else { continue };
        let name = item.file_name().to_string_lossy().to_string();
        let attrs = md.file_attributes();
        let modified = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs() as i64);
        out.entries.push(Entry {
            path: item.path().to_string_lossy().to_string(),
            dir: md.is_dir(),
            size: if md.is_dir() { 0 } else { md.len() },
            modified,
            ext: if md.is_dir() { String::new() } else { ext_of(&name) },
            hidden: attrs & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0 || name.starts_with('.'),
            name,
        });
    }
    out.entries.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

fn known(id: &GUID) -> Option<String> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, Some(HANDLE::default())).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s
    }
}

/// La cartella dell'utente (dove si apre il Finder).
pub fn home() -> String {
    known(&FOLDERID_Profile).or_else(|| std::env::var("USERPROFILE").ok()).unwrap_or_else(|| "C:\\".to_string())
}

/// La barra laterale: le cartelle dell'utente e le unita'.
pub fn places() -> Vec<Place> {
    let mut out = Vec::new();
    let favs: [(&GUID, &str, &'static str); 7] = [
        (&FOLDERID_Profile, "", "home"),
        (&FOLDERID_Desktop, "Scrivania", "desktop"),
        (&FOLDERID_Documents, "Documenti", "documents"),
        (&FOLDERID_Downloads, "Download", "downloads"),
        (&FOLDERID_Pictures, "Immagini", "pictures"),
        (&FOLDERID_Music, "Musica", "music"),
        (&FOLDERID_Videos, "Filmati", "videos"),
    ];
    for (id, name, kind) in favs {
        if let Some(path) = known(id) {
            let name = if name.is_empty() { display_name(&path) } else { name.to_string() };
            out.push(Place { name, path, kind });
        }
    }
    if let Some(path) = known(&windows::Win32::UI::Shell::FOLDERID_SkyDrive).filter(|p| Path::new(p).is_dir()) {
        out.push(Place { name: "OneDrive".into(), path, kind: "cloud" });
    }
    out.push(Place { name: "Cestino".into(), path: TRASH.into(), kind: "trash" });
    let mask = unsafe { GetLogicalDrives() };
    for i in 0..26u32 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let letter = (b'A' + i as u8) as char;
        let root = format!("{letter}:\\");
        let mut label = [0u16; 64];
        let ok = unsafe { GetVolumeInformationW(&HSTRING::from(root.as_str()), Some(&mut label), None, None, None, None) }.is_ok();
        if !ok {
            continue; // unita' senza disco (lettore vuoto)
        }
        let text = String::from_utf16_lossy(&label).trim_end_matches('\0').to_string();
        let name = if text.is_empty() { format!("Disco ({letter}:)") } else { format!("{text} ({letter}:)") };
        out.push(Place { name, path: root, kind: "drive" });
    }
    out
}

/// Apre un file (o un collegamento, un programma) con l'app predefinita di Windows.
pub fn open_file(path: &str) {
    unsafe {
        ShellExecuteW(None, w!("open"), &HSTRING::from(path), None, None, SW_SHOWNORMAL);
    }
}

/// Mostra l'elemento in Esplora file (selezionato).
pub fn reveal(path: &str) {
    let _ = std::process::Command::new("explorer.exe").arg(format!("/select,{path}")).spawn();
}

/// Rinomina un elemento restando nella stessa cartella. Risponde con il nuovo percorso.
pub fn rename(path: &str, new_name: &str) -> Result<String, String> {
    let new_name = new_name.trim();
    if !valid_name(new_name) {
        return Err("Questo nome non e' valido per Windows.".into());
    }
    let from = Path::new(path);
    let to: PathBuf = from.parent().ok_or("Questo elemento non si puo' rinominare.")?.join(new_name);
    if to.exists() && !to.to_string_lossy().eq_ignore_ascii_case(path) {
        return Err("Esiste gia' un elemento con questo nome.".into());
    }
    std::fs::rename(from, &to).map_err(|e| format!("Non riesco a rinominare: {e}"))?;
    let to = to.to_string_lossy().to_string();
    crate::finder2::tag_moved(path, &to);
    Ok(to)
}

/// Crea «Nuova cartella» (o «Nuova cartella 2»...) dentro `dir`. Risponde con il suo percorso.
pub fn new_folder(dir: &str) -> Result<String, String> {
    let base = Path::new(dir);
    let name = free_name("Nuova cartella", &|n| base.join(n).exists());
    let path = base.join(name);
    std::fs::create_dir(&path).map_err(|e| format!("Non riesco a creare la cartella: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

/// Elenco di percorsi nel formato delle operazioni della shell: separati da zero, chiusi da due zeri.
fn multi(paths: &[String]) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();
    for p in paths {
        out.extend(p.encode_utf16());
        out.push(0);
    }
    out.push(0);
    out
}

fn shell_op(func: u32, from: &[String], to: Option<&str>, flags: u16) -> Result<(), String> {
    if from.is_empty() {
        return Ok(());
    }
    let src = multi(from);
    let dst = to.map(|t| multi(&[t.to_string()]));
    let mut op = SHFILEOPSTRUCTW { wFunc: func, pFrom: PCWSTR(src.as_ptr()), pTo: dst.as_ref().map_or(PCWSTR::null(), |d| PCWSTR(d.as_ptr())), fFlags: flags, ..Default::default() };
    let code = unsafe { SHFileOperationW(&mut op) };
    if code == 0 && !op.fAnyOperationsAborted.as_bool() { Ok(()) } else { Err("Operazione non completata.".into()) }
}

const FO_MOVE: u32 = 1;
const FO_COPY: u32 = 2;
const FO_DELETE: u32 = 3;
const FOF_RENAMEONCOLLISION: u16 = 0x8;
const FOF_NOCONFIRMATION: u16 = 0x10;
const FOF_ALLOWUNDO: u16 = 0x40;

/// Sposta nel Cestino (si puo' ripristinare da li').
pub fn trash(paths: &[String]) -> Result<(), String> {
    shell_op(FO_DELETE, paths, None, FOF_ALLOWUNDO | FOF_NOCONFIRMATION)
}

/// Copia o sposta dei file in `dest` (con l'avanzamento, le domande e l'annulla di Windows; nella stessa cartella fa una copia).
pub fn transfer(paths: &[String], dest: &str, mv: bool) -> Result<(), String> {
    shell_op(if mv { FO_MOVE } else { FO_COPY }, paths, Some(dest), FOF_ALLOWUNDO | FOF_RENAMEONCOLLISION)
}

/// Incolla in `dest` i file degli appunti di Windows (copiati qui o in Esplora file).
pub fn paste(dest: &str) -> Result<(), String> {
    let (paths, cut) = crate::finder2::clip_get();
    if paths.is_empty() {
        return Err("Negli appunti non ci sono file.".into());
    }
    transfer(&paths, dest, cut)?;
    if cut {
        crate::finder2::clip_clear();
    }
    Ok(())
}

/// Icona (o, per immagini e video, anteprima) dell'elemento, come data URL PNG.
pub fn image(path: &str, size: i32, thumb: bool) -> Option<String> {
    crate::dock::shell_image_data_url(Path::new(path), size.clamp(16, 256), thumb)
}

/// Apre una finestra del Finder sulla cartella indicata (o sulla cartella dell'utente). Dal thread chiamante mai quello principale:
/// creare una finestra WebView2 dal thread principale puo' bloccare tutto.
pub fn open(app: &AppHandle, path: Option<String>) {
    if crate::lockscreen::is_locked() {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let path = path.filter(|p| Path::new(p).is_dir()).unwrap_or_else(home);
        let label = format!("{PREFIX}{}", NEXT.fetch_add(1, Ordering::SeqCst));
        let url = format!("index.html?view=finder&path={}", urlencode(&path));
        let made = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(url.into()))
            .title("Finder")
            .decorations(false)
            .resizable(true)
            .shadow(true)
            .inner_size(980.0, 620.0)
            .min_inner_size(620.0, 380.0)
            .center()
            .visible(false)
            .build();
        match made {
            Ok(win) => {
                // Stessa protezione delle altre finestre di EosOS: nessuna area non client, quindi niente cornice ne' filo chiaro di
                // Windows attorno (la guardia delle cornici toglie la barra del titolo e Windows disegnava al suo posto il bordo sottile).
                crate::chromeguard::protect(&win);
                // Come le finestre di macOS: angoli arrotondati e nessun filo bianco attorno. Windows lo rimette quando la finestra
                // si mostra, si attiva o cambia dimensione: si riapplica in quei momenti (vedi `on_window_event`).
                frame(&win.as_ref().window());
                let w2 = win.clone();
                std::thread::spawn(move || {
                    for ms in [80u64, 300, 900] {
                        std::thread::sleep(std::time::Duration::from_millis(ms));
                        frame(&w2.as_ref().window());
                    }
                });
                let _ = win.show();
                let _ = win.set_focus();
            }
            Err(e) => eprintln!("finder: finestra non creata: {e}"),
        }
    });
}

/// Porta davanti una finestra del Finder gia' aperta; `false` se non ce ne sono.
pub fn focus_existing(app: &AppHandle) -> bool {
    let Some(win) = app.webview_windows().into_iter().find(|(l, _)| is_label(l)).map(|(_, w)| w) else { return false };
    let _ = win.unminimize();
    let _ = win.show();
    let _ = win.set_focus();
    true
}

/// Percorso dentro un indirizzo: tutto cio' che non e' lettera, cifra o `-_.~` diventa `%XX`. Pura, testata.
pub fn urlencode(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

/// Chiudere una finestra del Finder chiude solo lei (le altre finestre di EosOS, chiudendosi, chiudono tutta l'app).
pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) -> bool {
    if !is_label(window.label()) {
        return false;
    }
    if matches!(event, tauri::WindowEvent::Focused(_) | tauri::WindowEvent::Resized(_) | tauri::WindowEvent::ScaleFactorChanged { .. }) {
        frame(window);
    }
    true
}

/// Angoli arrotondati e nessun bordo di Windows (il filo chiaro attorno alle finestre senza cornice).
fn frame(window: &tauri::Window) {
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE};
    let Ok(h) = window.hwnd() else { return };
    let none: u32 = 0xFFFF_FFFE; // DWMWA_COLOR_NONE
    let round: u32 = 2; // DWMWCP_ROUND
    unsafe {
        let _ = DwmSetWindowAttribute(h, DWMWA_BORDER_COLOR, &none as *const _ as *const _, 4);
        let _ = DwmSetWindowAttribute(h, DWMWA_WINDOW_CORNER_PREFERENCE, &round as *const _ as *const _, 4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_are_lowercase_and_only_real_ones() {
        assert_eq!(ext_of("Foto.JPG"), "jpg");
        assert_eq!(ext_of("archivio.tar.gz"), "gz");
        assert_eq!(ext_of(".gitignore"), "");
        assert_eq!(ext_of("README"), "");
        assert_eq!(ext_of("nota. con spazio"), "");
    }

    #[test]
    fn names_and_parents_follow_windows_paths() {
        assert_eq!(display_name(r"C:\Users\Anna"), "Anna");
        assert_eq!(display_name(r"C:\"), "C:");
        assert_eq!(parent_of(r"C:\Users\Anna"), Some(r"C:\Users".to_string()));
        assert_eq!(parent_of(r"C:\Users"), Some(r"C:\".to_string()));
        assert_eq!(parent_of(r"C:\"), None);
        assert_eq!(parent_of(r"D:\Dati\"), Some(r"D:\".to_string()));
    }

    #[test]
    fn invalid_names_are_refused() {
        assert!(valid_name("Relazione finale.docx"));
        for bad in ["", "  ", "a/b", r"a\b", "che?", "fine.", "a:b", "x*y", "<tag>"] {
            assert!(!valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn the_first_free_name_is_chosen() {
        assert_eq!(free_name("Nuova cartella", &|_| false), "Nuova cartella");
        assert_eq!(free_name("Nuova cartella", &|n| n == "Nuova cartella" || n == "Nuova cartella 2"), "Nuova cartella 3");
    }

    #[test]
    fn paths_survive_the_address() {
        assert_eq!(urlencode(r"C:\Dati miei\è"), "C%3A%5CDati%20miei%5C%C3%A8");
    }

    #[test]
    fn listing_puts_folders_first_and_reports_a_missing_folder() {
        let dir = std::env::temp_dir().join(format!("eosos-finder-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Zeta")).unwrap();
        std::fs::write(dir.join("alfa.txt"), b"ciao").unwrap();
        std::fs::write(dir.join("Beta.PNG"), b"x").unwrap();
        let l = list(&dir.to_string_lossy());
        assert!(l.error.is_none());
        let names: Vec<&str> = l.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Zeta", "alfa.txt", "Beta.PNG"]);
        assert!(l.entries[0].dir && l.entries[1].size == 4 && l.entries[2].ext == "png");
        let made = new_folder(&dir.to_string_lossy()).unwrap();
        assert!(made.ends_with("Nuova cartella"));
        let renamed = rename(&made, "Progetti").unwrap();
        assert!(Path::new(&renamed).is_dir() && rename(&renamed, "a/b").is_err());
        let _ = std::fs::remove_dir_all(&dir);
        assert!(list(&dir.to_string_lossy()).error.is_some());
    }
}
