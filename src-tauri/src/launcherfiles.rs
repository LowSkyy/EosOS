//! Launcher (fase L4): ricerca dei file. Si interroga l'indice di Windows Search (API WinRT
//! `QueryOptions` con solo-indice) sulle cartelle note dell'utente, senza scansionare il disco; se
//! l'indice non dà nulla (servizio spento, cartella non indicizzata) si ripiega su una visita
//! breve e limitata nel tempo di quelle stesse cartelle. Mai sul thread dell'interfaccia.
//!
//! La pagina vede solo il percorso di un risultato che Rust le ha dato: per aprirlo o mostrarlo in
//! Esplora risorse Rust controlla che il percorso sia tra i risultati recenti (`is_known`).

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::core::{w, GUID, HSTRING};
use windows::Storage::Search::{FolderDepth, IndexerOption, QueryOptions};
use windows::Storage::StorageFolder;
use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_Music, FOLDERID_Pictures, FOLDERID_Videos,
    SHGetKnownFolderPath, ShellExecuteW, KF_FLAG_DEFAULT,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::launcherrank::{fold, match_score};

/// Risultati al massimo per una ricerca.
pub const LIMIT: usize = 8;
/// Quanti ne chiede a ogni cartella dell'indice prima di filtrare per nome.
const PER_FOLDER: u32 = 40;
/// Il ripiego non visita più di tanti elementi né oltre questo tempo.
const FALLBACK_MAX_ENTRIES: usize = 20_000;
const FALLBACK_BUDGET: Duration = Duration::from_millis(400);
const FALLBACK_DEPTH: usize = 4;
/// Risultati ricordati per validare le richieste di apertura.
const KNOWN_CAP: usize = 256;

/// Un file trovato.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub path: PathBuf,
    pub name: String,
}

// ------------------------------------------------------------------ pure

/// Ripulisce il testo per la query dell'indice: toglie virgolette, parentesi e due punti (la
/// sintassi di ricerca di Windows) e gli spazi di troppo. `None` se restano meno di 2 caratteri
/// (con una lettera sola ogni ricerca darebbe rumore e costo). Pura, testata.
pub fn sanitize(query: &str) -> Option<String> {
    let cleaned: String = query
        .chars()
        .map(|c| if matches!(c, '"' | '(' | ')' | ':' | '<' | '>' | '*' | '?' | '\\' | '/') { ' ' } else { c })
        .collect();
    let joined = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    (joined.chars().count() >= 2).then_some(joined)
}

/// Quanto `name` corrisponde a `query`: prima la query intera; altrimenti, se ha più parole,
/// ognuna deve corrispondere (come "relazione finale" per "relazione-finale-v2.docx"), con
/// il punteggio della peggiore meno un piccolo scarto. Pura, testata.
pub fn name_score(query: &str, name: &str) -> Option<i32> {
    if let Some(s) = match_score(query, name) {
        return Some(s);
    }
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }
    let mut worst = i32::MAX;
    for w in words {
        worst = worst.min(match_score(w, name)?);
    }
    Some(worst - 20)
}

/// Tiene i risultati il cui NOME corrisponde a `query` (l'indice cerca anche nel contenuto: un file
/// il cui nome non c'entra non è quello che l'utente cerca), senza doppioni di percorso, ordinati
/// per punteggio, poi per nome più corto, poi per percorso. Al massimo `limit`. Pura, testata.
pub fn rank_hits(query: &str, hits: Vec<Hit>, limit: usize) -> Vec<Hit> {
    let mut seen = HashSet::new();
    let mut scored: Vec<(i32, Hit)> = hits
        .into_iter()
        .filter(|h| seen.insert(h.path.to_string_lossy().to_lowercase()))
        .filter_map(|h| {
            // Si confronta anche il nome senza estensione: "relazione" deve trovare "relazione.docx".
            let stem = Path::new(&h.name).file_stem().map(|s| s.to_string_lossy().to_string());
            let s = [name_score(query, &h.name), stem.and_then(|s| name_score(query, &s))].into_iter().flatten().max()?;
            Some((s, h))
        })
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.name.chars().count().cmp(&b.1.name.chars().count()))
            .then_with(|| fold(&a.1.path.to_string_lossy()).cmp(&fold(&b.1.path.to_string_lossy())))
    });
    scored.truncate(limit);
    scored.into_iter().map(|(_, h)| h).collect()
}

/// Accorcia il percorso della cartella per mostrarlo: sostituisce la cartella personale con `~`
/// e, se è ancora lungo, tiene l'inizio e la fine. Pura, testata.
pub fn short_dir(dir: &str, home: Option<&str>, max: usize) -> String {
    let mut s = dir.to_string();
    if let Some(h) = home
        && !h.is_empty()
        && s.to_lowercase().starts_with(&h.to_lowercase())
    {
        s = format!("~{}", &s[h.len()..]);
    }
    let n = s.chars().count();
    if n <= max || max < 8 {
        return s;
    }
    let head: String = s.chars().take(max / 3).collect();
    let tail: String = s.chars().skip(n - (max - max / 3 - 1)).collect();
    format!("{head}…{tail}")
}

/// Visita in ampiezza `root` fino a `depth` livelli, raccogliendo i file il cui nome corrisponde a
/// `query`, ma fermandosi dopo `max_entries` elementi visti o allo scadere di `deadline`. Salta le
/// cartelle nascoste e quelle di sistema/sviluppo troppo rumorose. Testata su cartelle temporanee.
pub fn bounded_walk(root: &Path, query: &str, depth: usize, max_entries: usize, deadline: Instant) -> Vec<Hit> {
    let mut out = Vec::new();
    let mut seen = 0usize;
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::from([(root.to_path_buf(), 0)]);
    while let Some((dir, level)) = queue.pop_front() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            seen += 1;
            if seen > max_entries || Instant::now() >= deadline {
                return out;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.eq_ignore_ascii_case("node_modules") || name.eq_ignore_ascii_case("$recycle.bin") {
                continue;
            }
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                if level < depth && !ft.is_symlink() {
                    queue.push_back((entry.path(), level + 1));
                }
            } else if name_score(query, &name).is_some() {
                out.push(Hit { path: entry.path(), name });
            }
        }
    }
    out
}

// ------------------------------------------------------------------ risultati noti

static KNOWN: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

fn key(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\").to_lowercase()
}

/// Ricorda i percorsi mostrati, perché solo quelli si possano aprire. Tiene gli ultimi `KNOWN_CAP`.
pub fn remember(hits: &[Hit]) {
    let mut k = KNOWN.lock().unwrap_or_else(|e| e.into_inner());
    for h in hits {
        let p = key(&h.path);
        if !k.contains(&p) {
            k.push_back(p);
        }
    }
    while k.len() > KNOWN_CAP {
        k.pop_front();
    }
}

/// Percorso di rete (UNC o `//server`). Pura, testata.
pub fn is_network_path(p: &str) -> bool {
    let p = p.trim_start();
    p.starts_with(r"\\") || p.starts_with("//")
}

/// Tipi di file che eseguono codice: dal Launcher non si aprono (si possono solo mostrare in Esplora risorse), così una
/// ricerca non lancia mai un programma o uno script. Pura, testata.
pub fn is_risky_file(p: &Path) -> bool {
    const RISKY: &[&str] = &[
        "exe", "com", "scr", "pif", "bat", "cmd", "ps1", "psm1", "vbs", "vbe", "js", "jse", "wsf", "wsh", "hta", "msi", "msp", "lnk", "url", "reg", "cpl", "jar", "dll", "appx", "msix", "application", "gadget",
    ];
    p.extension().and_then(|e| e.to_str()).is_some_and(|e| RISKY.contains(&e.to_ascii_lowercase().as_str()))
}

/// Il percorso è tra quelli mostrati di recente? Pura rispetto al disco; testata.
pub fn is_known(path: &Path) -> bool {
    KNOWN.lock().unwrap_or_else(|e| e.into_inner()).contains(&key(path))
}

// ------------------------------------------------------------------ Windows

fn known_folder(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(PathBuf::from)
    }
}

/// La cartella Download dell'utente (percorso vero, anche se spostata).
pub fn downloads_folder() -> Option<PathBuf> {
    known_folder(&FOLDERID_Downloads)
}

/// Cartelle personali dove si cerca (percorsi veri, anche se spostate su OneDrive), più quelle
/// aggiunte dall'utente in `launcher.json` (`searchFolders`) che esistono davvero. Senza doppioni.
pub fn search_roots() -> Vec<PathBuf> {
    let known = [&FOLDERID_Documents, &FOLDERID_Desktop, &FOLDERID_Downloads, &FOLDERID_Pictures, &FOLDERID_Music, &FOLDERID_Videos]
        .into_iter()
        .filter_map(known_folder);
    // Mai percorsi di rete: enumerarli manda le credenziali a quel server.
    let extra = crate::launcherstore::settings().search_folders.into_iter().filter(|p| !is_network_path(p)).map(PathBuf::from);
    merge_roots(known.chain(extra).collect())
}

/// Tiene le cartelle che esistono, senza doppioni (maiuscole ignorate) e senza cartelle già
/// contenute in un'altra della lista (si cercherebbero due volte). Pura sui percorsi, testata.
pub fn merge_roots(candidates: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for c in candidates.into_iter().filter(|p| p.is_dir()) {
        let k = key(&c);
        let inside = |a: &str, b: &str| a != b && a.starts_with(&format!("{}\\", b.trim_end_matches('\\')));
        if out.iter().any(|o| key(o) == k || inside(&k, &key(o))) {
            continue;
        }
        out.retain(|o| !inside(&key(o), &k));
        out.push(c);
    }
    out
}

struct MtaScope(bool);
impl MtaScope {
    fn new() -> Self {
        Self(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok())
    }
}
impl Drop for MtaScope {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

/// Interroga l'indice di Windows Search su una cartella. `Err` se l'API fallisce (indice non
/// disponibile); `Ok(vuoto)` se non c'è nulla.
fn index_query(folder: &Path, text: &str) -> windows::core::Result<Vec<Hit>> {
    let f = StorageFolder::GetFolderFromPathAsync(&HSTRING::from(folder.to_string_lossy().as_ref()))?.join()?;
    let opts = QueryOptions::new()?;
    opts.SetFolderDepth(FolderDepth::Deep)?;
    opts.SetIndexerOption(IndexerOption::OnlyUseIndexer)?;
    opts.SetUserSearchFilter(&HSTRING::from(text))?;
    let q = f.CreateFileQueryWithOptions(&opts)?;
    let files = q.GetFilesAsync(0, PER_FOLDER)?.join()?;
    let mut out = Vec::new();
    for file in files {
        if let (Ok(path), Ok(name)) = (file.Path(), file.Name()) {
            out.push(Hit { path: PathBuf::from(path.to_string()), name: name.to_string() });
        }
    }
    Ok(out)
}

/// Esito di una ricerca, con l'origine (utile alle prove e ai messaggi).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Source {
    Index,
    Fallback,
}

fn run_search(text: &str, query: &str, use_index: bool) -> (Vec<Hit>, Source) {
    let _mta = MtaScope::new();
    let roots = search_roots();
    let mut hits = Vec::new();
    if use_index {
        for r in &roots {
            if let Ok(h) = index_query(r, text) {
                hits.extend(h);
            }
        }
    }
    let ranked = rank_hits(query, hits, LIMIT);
    if !ranked.is_empty() {
        return (ranked, Source::Index);
    }
    // Ripiego: indice spento, cartella non indicizzata o nessuna corrispondenza per nome.
    let deadline = Instant::now() + FALLBACK_BUDGET;
    let mut walked = Vec::new();
    for r in &roots {
        walked.extend(bounded_walk(r, query, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, deadline));
        if Instant::now() >= deadline {
            break;
        }
    }
    (rank_hits(query, walked, LIMIT), Source::Fallback)
}

static LATEST: AtomicU64 = AtomicU64::new(0);
static SEARCH_LOCK: Mutex<()> = Mutex::new(());

/// Cerca `query` tra i file. Una ricerca più recente rende inutile quella ancora in coda (la pagina
/// ne scarta comunque la risposta): ritorna subito vuota. `EOSOS_NO_INDEX=1` forza il ripiego.
pub fn search(query: &str) -> Vec<Hit> {
    let Some(text) = sanitize(query) else { return Vec::new() };
    let mine = LATEST.fetch_add(1, Ordering::SeqCst) + 1;
    let _one = SEARCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if LATEST.load(Ordering::SeqCst) != mine {
        return Vec::new();
    }
    let use_index = std::env::var_os("EOSOS_NO_INDEX").is_none();
    let (q, t) = (text.clone(), text);
    // Thread dedicato (apartment MTA): le operazioni asincrone WinRT si attendono senza rischi.
    let hits = std::thread::spawn(move || run_search(&t, &q, use_index).0).join().unwrap_or_default();
    remember(&hits);
    hits
}

/// Apre il file (o lo mostra in Esplora risorse con `reveal`), solo se è tra i risultati recenti.
pub fn open(path: &str, reveal: bool) -> bool {
    let p = PathBuf::from(path);
    if !is_known(&p) || !p.exists() || (!reveal && is_risky_file(&p)) {
        return false;
    }
    unsafe {
        let r = if reveal {
            let args = HSTRING::from(format!("/select,\"{}\"", p.to_string_lossy()));
            ShellExecuteW(None, w!("open"), w!("explorer.exe"), &args, None, SW_SHOWNORMAL)
        } else {
            let file = HSTRING::from(p.to_string_lossy().as_ref());
            let dir = p.parent().map(|d| HSTRING::from(d.to_string_lossy().as_ref()));
            match &dir {
                Some(d) => ShellExecuteW(None, w!("open"), &file, None, d, SW_SHOWNORMAL),
                None => ShellExecuteW(None, w!("open"), &file, None, None, SW_SHOWNORMAL),
            }
        };
        r.0 as usize > 32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_paths_and_risky_files_are_recognised() {
        assert!(is_network_path(r"\\evil\share") && is_network_path("//evil/share") && is_network_path(r"  \\x\y"));
        assert!(!is_network_path(r"C:\Users\x") && !is_network_path("D:/a"));
        for r in [r"C:\a\x.exe", r"C:\a\X.BAT", r"C:\a\s.ps1", r"C:\a\l.lnk", r"C:\a\m.msi", r"C:\a\h.hta", r"C:\a\a.cmd"] {
            assert!(is_risky_file(Path::new(r)), "{r}");
        }
        for ok in [r"C:\a\doc.pdf", r"C:\a\foto.jpg", r"C:\a\nota.txt", r"C:\a\senza", r"C:\a\exe"] {
            assert!(!is_risky_file(Path::new(ok)), "{ok}");
        }
    }

    fn hit(path: &str) -> Hit {
        let p = PathBuf::from(path);
        Hit { name: p.file_name().unwrap().to_string_lossy().to_string(), path: p }
    }

    #[test]
    fn sanitize_strips_query_syntax_and_short_input() {
        assert_eq!(sanitize("relazione"), Some("relazione".into()));
        assert_eq!(sanitize("  due   parole "), Some("due parole".into()));
        assert_eq!(sanitize(r#"kind:"pic" OR (x)"#), Some("kind pic OR x".into()));
        assert_eq!(sanitize(r"C:\Windows\*.exe"), Some("C Windows .exe".into()));
        assert_eq!(sanitize("a"), None);
        assert_eq!(sanitize(" a "), None);
        assert_eq!(sanitize(""), None);
        assert_eq!(sanitize(r#""":()"#), None);
    }

    #[test]
    fn rank_hits_keeps_name_matches_dedupes_and_orders() {
        let hits = vec![
            hit(r"C:\u\Documenti\relazione finale.docx"),
            hit(r"C:\u\Documenti\relazione.docx"),
            hit(r"C:\u\Documenti\RELAZIONE.DOCX"), // stesso percorso a meno delle maiuscole? no: diverso nome ma stesso lowercase
            hit(r"C:\u\Documenti\bilancio.xlsx"), // trovato dall'indice per il contenuto, nome estraneo
            hit(r"C:\u\Documenti\relazione.docx"), // doppione esatto
        ];
        let r = rank_hits("relazione", hits, 10);
        let names: Vec<_> = r.iter().map(|h| h.name.as_str()).collect();
        assert!(!names.contains(&"bilancio.xlsx"), "nome estraneo: scartato");
        assert_eq!(names.iter().filter(|n| n.eq_ignore_ascii_case("relazione.docx")).count(), 1, "nessun doppione");
        // Il nome senza estensione identico alla query vince.
        assert!(names[0].eq_ignore_ascii_case("relazione.docx"), "{names:?}");
        assert_eq!(names.last().copied(), Some("relazione finale.docx"));
    }

    #[test]
    fn multi_word_queries_match_names_with_separators() {
        assert!(name_score("eosos prova", "eosos-prova-launcher-123.txt").is_some());
        assert!(name_score("relazione finale", "relazione_finale_v2.docx").is_some());
        assert!(name_score("finale relazione", "relazione finale.docx").is_some(), "ordine libero");
        assert!(name_score("relazione bilancio", "relazione finale.docx").is_none(), "ogni parola deve esserci");
        assert!(name_score("relazione", "relazione.docx").unwrap() > name_score("relazione x", "relazione x.docx").unwrap() - 1);
        assert_eq!(name_score("zzz", "relazione.docx"), None);
        let r = rank_hits("eosos prova", vec![hit(r"C:\d\eosos-prova-1.txt"), hit(r"C:\d\eosos-altro.txt")], 8);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn rank_hits_respects_limit_and_empty() {
        let hits: Vec<_> = (0..20).map(|i| hit(&format!(r"C:\x\nota{i}.txt"))).collect();
        assert_eq!(rank_hits("nota", hits, 8).len(), 8);
        assert!(rank_hits("nota", Vec::new(), 8).is_empty());
        assert!(rank_hits("zzz", vec![hit(r"C:\x\nota.txt")], 8).is_empty());
    }

    #[test]
    fn short_dir_uses_home_and_elides_long_paths() {
        assert_eq!(short_dir(r"C:\Users\Raffaele\Documents", Some(r"C:\Users\Raffaele"), 60), r"~\Documents");
        assert_eq!(short_dir(r"c:\users\raffaele\x", Some(r"C:\Users\Raffaele"), 60), r"~\x", "senza badare alle maiuscole");
        assert_eq!(short_dir(r"D:\progetti", Some(r"C:\Users\R"), 60), r"D:\progetti");
        let long = format!(r"D:\{}", "cartella\\".repeat(20));
        let s = short_dir(&long, None, 40);
        assert!(s.chars().count() <= 40 && s.contains('…'), "{s}");
        assert_eq!(short_dir("abc", None, 2), "abc", "limite assurdo: non si tocca");
        assert_eq!(short_dir("", None, 40), "");
    }

    fn temp_tree(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("eosos-files-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("a/b/c/d/e")).unwrap();
        std::fs::create_dir_all(root.join("node_modules/x")).unwrap();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        for f in ["relazione.docx", "a/relazione bis.txt", "a/b/relazione tris.txt", "a/b/c/d/e/relazione profonda.txt", "node_modules/x/relazione.js", ".git/relazione.txt", "altro.txt"] {
            std::fs::write(root.join(f), "x").unwrap();
        }
        root
    }

    #[test]
    fn bounded_walk_finds_by_name_within_depth_and_skips_noise() {
        let root = temp_tree("walk");
        let far = Instant::now() + Duration::from_secs(5);
        let hits = bounded_walk(&root, "relazione", 3, 10_000, far);
        let names: Vec<_> = hits.iter().map(|h| h.name.clone()).collect();
        assert!(names.contains(&"relazione.docx".to_string()));
        assert!(names.contains(&"relazione bis.txt".to_string()));
        assert!(names.contains(&"relazione tris.txt".to_string()));
        assert!(!names.contains(&"relazione profonda.txt".to_string()), "oltre la profondita'");
        assert!(!names.iter().any(|n| n == "relazione.js"), "node_modules saltata");
        assert_eq!(names.iter().filter(|n| n.as_str() == "relazione.txt").count(), 0, ".git saltata");
        assert!(!names.contains(&"altro.txt".to_string()));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots() {
        let root = temp_tree("budget");
        let far = Instant::now() + Duration::from_secs(5);
        assert!(bounded_walk(&root, "relazione", 5, 2, far).len() <= 2, "tetto di elementi visti");
        assert!(bounded_walk(&root, "relazione", 5, 10_000, Instant::now()).is_empty(), "tempo scaduto");
        assert!(bounded_walk(&root.join("non-esiste"), "x", 3, 100, far).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn only_remembered_paths_can_be_opened() {
        let unique = format!(r"C:\eosos-test-{}\nota.txt", std::process::id());
        assert!(!is_known(Path::new(&unique)));
        assert!(!open(&unique, false), "percorso mai mostrato: rifiutato");
        remember(&[hit(&unique)]);
        assert!(is_known(Path::new(&unique.to_uppercase())), "senza badare alle maiuscole");
        // Noto ma inesistente: non si apre.
        assert!(!open(&unique, false));
        assert!(!open(r"C:\Windows\System32\cmd.exe", false), "un percorso arbitrario non e' apribile");
    }

    #[test]
    fn merge_roots_drops_missing_duplicates_and_nested_folders() {
        let base = std::env::temp_dir().join(format!("eosos-roots-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("a/b")).unwrap();
        std::fs::create_dir_all(base.join("c")).unwrap();
        let upper = PathBuf::from(base.join("A").to_string_lossy().to_uppercase());
        let r = merge_roots(vec![
            base.join("a/b"),         // dentro "a": va tolta
            base.join("a"),
            upper,                    // stessa cartella con altre maiuscole
            base.join("non-esiste"),  // inesistente
            base.join("c"),
        ]);
        let names: Vec<_> = r.iter().map(|p| p.file_name().unwrap().to_string_lossy().to_string()).collect();
        assert_eq!(names, vec!["a", "c"], "{names:?}");
        assert!(merge_roots(Vec::new()).is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn known_results_are_capped() {
        let tag = std::process::id();
        let many: Vec<_> = (0..(KNOWN_CAP + 50)).map(|i| hit(&format!(r"C:\cap{tag}\f{i}.txt"))).collect();
        remember(&many);
        assert!(!is_known(Path::new(&format!(r"C:\cap{tag}\f0.txt"))), "i piu' vecchi escono");
        assert!(is_known(Path::new(&format!(r"C:\cap{tag}\f{}.txt", KNOWN_CAP + 49))));
    }

    /// Prova sulla macchina vera, in sola lettura: non fallisce se l'indice è spento.
    #[test]
    fn real_search_smoke() {
        let t = Instant::now();
        let (hits, src) = std::thread::spawn(|| run_search("txt", "txt", true)).join().unwrap();
        println!("origine {src:?}, {} risultati in {:?}", hits.len(), t.elapsed());
        for h in hits.iter().take(3) {
            println!("  {}", h.name);
        }
        let (fb, _) = std::thread::spawn(|| run_search("txt", "txt", false)).join().unwrap();
        println!("ripiego: {} risultati", fb.len());
    }
}
