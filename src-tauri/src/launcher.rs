//! Launcher (fase L1): indice delle app e ricerca. Le app vengono da `shell:AppsFolder` (programmi
//! classici e app dello Store insieme), la stessa fonte che il Dock usa per nomi, icone e avvio.
//! La classifica sta in `launcherrank` e l'uso in `launcherstore`; qui c'è solo ciò che parla con
//! Windows e la colla. Il frontend vede solo `id` e nome; ciò che si esegue lo ricava Rust.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use windows::core::{w, HSTRING};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
    BHID_EnumItems, IEnumShellItems, IShellItem, SHCreateItemFromParsingName, ShellExecuteW, SIGDN,
    SIGDN_NORMALDISPLAY, SIGDN_PARENTRELATIVEPARSING,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::dock::{aumid_path, cached_icon, ComScope};
use crate::launcheract;
use crate::launcherclip;
use crate::launchercalc;
use crate::launcherfiles;
use crate::launcherrank::{fold, rank};
use crate::launcherstore;

/// Risultati mostrati al massimo.
pub const LIMIT: usize = 8;
/// Ogni quanto si rilegge l'elenco delle app (installazioni e rimozioni).
const INDEX_TTL: Duration = Duration::from_secs(300);

/// Un'app avviabile: `id` è il nome di parsing relativo a `shell:AppsFolder`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
}

/// Un risultato per il frontend.
#[derive(Serialize, Clone, Debug)]
pub struct LauncherItem {
    pub id: String,
    pub name: String,
    /// "app", "action", "calc", "file", "clip" (appunti) o "setting" (richiesta di consenso).
    pub kind: &'static str,
    /// Testo secondario (per il calcolo: l'espressione).
    pub detail: Option<String>,
    /// Data URL dell'icona, solo se il frontend non l'ha già.
    pub icon: Option<String>,
}

/// Voce di servizio che non è un'app da lanciare: collegamenti a siti web, manuali, licenze e
/// documenti (`.url`, `.chm`, `.pdf`, `.txt`, `.html`...) e i disinstallatori. Si giudica dall'id
/// (la destinazione), non dal nome, per non nascondere app vere; l'unica eccezione per nome è il
/// prefisso "Uninstall"/"Disinstalla". Pura, testata.
pub fn is_noise(id: &str, name: &str) -> bool {
    let lid = id.trim().to_lowercase();
    if ["http://", "https://", "ftp://"].iter().any(|p| lid.starts_with(p)) {
        return true;
    }
    const DOC_EXT: [&str; 14] =
        ["url", "chm", "pdf", "txt", "rtf", "htm", "html", "md", "doc", "docx", "xml", "ini", "log", "hlp"];
    if let Some(ext) = std::path::Path::new(&lid).extension().and_then(|e| e.to_str())
        && DOC_EXT.contains(&ext)
    {
        return true;
    }
    let lname = name.trim().to_lowercase();
    lname.starts_with("uninstall ") || lname.starts_with("disinstalla ")
}

/// Toglie le voci senza id o senza nome, quelle di servizio (`is_noise`) e i doppioni (stesso id, senza badare alle maiuscole),
/// tenendo la prima. Pura, testata.
pub fn clean(raw: Vec<AppEntry>) -> Vec<AppEntry> {
    let mut seen = HashSet::new();
    raw.into_iter()
        .filter(|e| !e.id.trim().is_empty() && !e.name.trim().is_empty() && !is_noise(&e.id, &e.name))
        .filter(|e| seen.insert(e.id.to_lowercase()))
        .collect()
}

/// La voce dell'indice con questo id. Il frontend può chiedere di avviare solo ciò che è qui.
/// Pura, testata.
pub fn resolve<'a>(index: &'a [AppEntry], id: &str) -> Option<&'a AppEntry> {
    index.iter().find(|e| e.id.eq_ignore_ascii_case(id))
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn display(item: &IShellItem, kind: SIGDN) -> Option<String> {
    unsafe {
        let p = item.GetDisplayName(kind).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s
    }
}

/// Legge le app da `shell:AppsFolder`. Vuota se la shell non risponde.
fn enumerate() -> Vec<AppEntry> {
    let _com = ComScope::new();
    let mut raw = Vec::new();
    unsafe {
        let Ok(folder) = SHCreateItemFromParsingName::<_, _, IShellItem>(w!("shell:AppsFolder"), None) else {
            return raw;
        };
        let Ok(en) = folder.BindToHandler::<_, IEnumShellItems>(None, &BHID_EnumItems) else {
            return raw;
        };
        loop {
            let mut slot: [Option<IShellItem>; 1] = [None];
            let mut got = 0u32;
            let _ = en.Next(&mut slot, Some(&mut got));
            let Some(item) = slot[0].take().filter(|_| got == 1) else { break };
            let (Some(id), Some(name)) =
                (display(&item, SIGDN_PARENTRELATIVEPARSING), display(&item, SIGDN_NORMALDISPLAY))
            else {
                continue;
            };
            raw.push(AppEntry { id, name });
        }
    }
    clean(raw)
}

static INDEX: Mutex<Option<(Instant, Vec<AppEntry>)>> = Mutex::new(None);
static REBUILD: Mutex<()> = Mutex::new(());

fn cached() -> Option<(Instant, Vec<AppEntry>)> {
    INDEX.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// L'indice delle app, riletto se vecchio. Una rilettura vuota non cancella un indice buono
/// (la shell può non rispondere per un attimo, per esempio dopo il riavvio di Explorer).
pub fn apps() -> Vec<AppEntry> {
    if let Some((at, v)) = cached()
        && at.elapsed() < INDEX_TTL
    {
        return v;
    }
    let _one = REBUILD.lock().unwrap_or_else(|e| e.into_inner());
    // Un altro thread può aver finito mentre si aspettava.
    let old = match cached() {
        Some((at, v)) if at.elapsed() < INDEX_TTL => return v,
        Some((_, v)) => v,
        None => Vec::new(),
    };
    let fresh = enumerate();
    let chosen = if fresh.is_empty() { old } else { fresh };
    *INDEX.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), chosen.clone()));
    chosen
}

/// Prepara l'indice in un thread, così la prima ricerca non aspetta la shell.
pub fn warm_up() {
    std::thread::spawn(|| {
        let _ = apps();
    });
}

/// Cerca `query` tra calcolo, app e azioni. `have`: gli id di cui il frontend ha già l'icona.
/// `filter`: solo "app" o solo "action" (le schede); `None` = tutto. Il risultato del calcolo, se
/// c'è, sta sempre in cima. Qualunque ricerca annulla una conferma in sospeso.
pub fn search(query: &str, have: &[String], filter: Option<&str>) -> Vec<LauncherItem> {
    launcheract::cancel_pending();
    // Gli appunti si vedono SOLO nella loro scheda: mai tra i risultati generali (potrebbero
    // contenere cose private che non si vogliono mostrare a ogni tasto).
    if filter == Some("clip") {
        return clips(query);
    }
    let (want_apps, want_actions, want_calc) = match filter {
        None => (true, true, true),
        Some("app") => (true, false, false),
        Some("action") => (false, true, false),
        Some(_) => (false, false, false),
    };
    let mut out: Vec<LauncherItem> = Vec::new();
    if want_calc
        && let Some(v) = launchercalc::evaluate(query)
    {
        out.push(LauncherItem {
            id: "calc".into(),
            name: launchercalc::format(v),
            kind: "calc",
            detail: Some(query.trim().to_string()),
            icon: None,
        });
    }
    let mut scored: Vec<(i32, LauncherItem)> = Vec::new();
    if want_apps {
        let index = apps();
        let usage = launcherstore::load();
        let entries: Vec<(String, String)> = index.iter().map(|e| (e.id.clone(), e.name.clone())).collect();
        let have: HashSet<String> = have.iter().map(|h| fold(h)).collect();
        for r in rank(&entries, &usage, query, now_secs(), LIMIT) {
            let e = &index[r.index];
            let icon = (!have.contains(&fold(&e.id))).then(|| cached_icon(&aumid_path(&e.id))).flatten();
            scored.push((r.score, LauncherItem { id: e.id.clone(), name: e.name.clone(), kind: "app", detail: None, icon }));
        }
    }
    if want_actions {
        for (a, score) in launcheract::search(query) {
            scored.push((score, LauncherItem { id: a.id.into(), name: a.name.into(), kind: "action", detail: None, icon: None }));
        }
    }
    // Ordine stabile: a pari punteggio restano prima le app (inserite per prime).
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    // Un'azione con lo stesso nome di un'app (Impostazioni) non si mostra due volte: vince l'app.
    let app_names: HashSet<String> = scored.iter().filter(|(_, i)| i.kind == "app").map(|(_, i)| fold(&i.name)).collect();
    scored.retain(|(_, i)| i.kind != "action" || !app_names.contains(&fold(&i.name)));
    out.extend(scored.into_iter().map(|(_, it)| it));
    out.truncate(LIMIT);
    out
}

/// Una pagina dell'elenco di TUTTE le app (per il Launchpad): quante ce ne sono in tutto e le voci di questa pagina.
#[derive(Serialize, Clone, Debug)]
pub struct GridPage {
    pub total: usize,
    pub items: Vec<LauncherItem>,
}

/// Quante app al massimo in una pagina del Launchpad.
pub const GRID_PAGE_MAX: usize = 64;

/// Gli indici delle app in ordine alfabetico (senza badare ad accenti e maiuscole), a parità l'ordine dell'indice. Pura, testata.
pub fn alphabetical(entries: &[(String, String)]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..entries.len()).collect();
    idx.sort_by_key(|&i| fold(&entries[i].1));
    idx
}

/// Tutte le app, in ordine alfabetico, a pagine: `offset` e `limit` (al massimo `GRID_PAGE_MAX`). `have`: gli id di cui la
/// pagina ha già l'icona. Il Launchpad le chiede a pezzi, così le prime compaiono subito e le icone si caricano man mano.
pub fn all_page(offset: usize, limit: usize, have: &[String]) -> GridPage {
    let index = apps();
    let entries: Vec<(String, String)> = index.iter().map(|e| (e.id.clone(), e.name.clone())).collect();
    let order = alphabetical(&entries);
    let have: HashSet<String> = have.iter().map(|h| fold(h)).collect();
    let items = order
        .iter()
        .skip(offset)
        .take(limit.min(GRID_PAGE_MAX))
        .map(|&i| {
            let e = &index[i];
            let icon = (!have.contains(&fold(&e.id))).then(|| cached_icon(&aumid_path(&e.id))).flatten();
            LauncherItem { id: e.id.clone(), name: e.name.clone(), kind: "app", detail: None, icon }
        })
        .collect();
    GridPage { total: index.len(), items }
}

/// Cerca `query` tra i file (indice di Windows Search, con ripiego). Lento rispetto al resto: la
/// pagina lo chiama a parte. L'`id` di un file è il suo percorso; il nome è il nome del file e il
/// testo secondario la cartella accorciata.
pub fn files(query: &str, have: &[String]) -> Vec<LauncherItem> {
    let home = std::env::var("USERPROFILE").ok();
    let have: HashSet<String> = have.iter().map(|h| fold(h)).collect();
    launcherfiles::search(query)
        .into_iter()
        .map(|h| {
            let id = h.path.to_string_lossy().to_string();
            let dir = h.path.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default();
            let icon = (!have.contains(&fold(&id))).then(|| cached_icon(&h.path)).flatten();
            LauncherItem {
                kind: "file",
                detail: Some(launcherfiles::short_dir(&dir, home.as_deref(), 48)),
                name: h.name,
                id,
                icon,
            }
        })
        .collect()
}

/// Voci della scheda Appunti. Senza consenso (mai chiesto o rifiutato) c'è una sola voce che lo
/// chiede: la cronologia non si registra finché l'utente non la attiva.
pub fn clips(query: &str) -> Vec<LauncherItem> {
    if !launcherclip::enabled() {
        return vec![LauncherItem {
            id: "clip:enable".into(),
            name: "Attiva la cronologia degli appunti".into(),
            kind: "setting",
            detail: Some("Solo in memoria, mai su disco; ignora le password".into()),
            icon: None,
        }];
    }
    launcherclip::list(query, LIMIT)
        .into_iter()
        .map(|(id, preview, age)| LauncherItem {
            id: format!("clip:{id}"),
            name: preview,
            kind: "clip",
            detail: Some(age),
            icon: None,
        })
        .collect()
}

/// L'utente accetta o rifiuta la cronologia degli appunti: la scelta si salva (solo il sì/no) e
/// rifiutare svuota subito la memoria.
pub fn set_clipboard(on: bool) {
    launcherstore::set_clipboard_consent(on);
    launcherclip::set_enabled(on);
}

fn clip_id(id: &str) -> Option<u64> {
    id.strip_prefix("clip:")?.parse().ok()
}

/// Rimette negli appunti la voce `id` (Rust ha il testo; la pagina non lo vede).
pub fn clip_use(id: &str) -> bool {
    clip_id(id).is_some_and(launcherclip::copy_back)
}

/// Toglie una voce dalla cronologia.
pub fn clip_delete(id: &str) -> bool {
    clip_id(id).is_some_and(launcherclip::remove)
}

/// Avvia l'app con questo `id`, se è nell'indice, e ne registra l'uso. `false` se l'id è
/// sconosciuto o l'avvio fallisce.
pub fn launch(id: &str) -> bool {
    let index = apps();
    let Some(entry) = resolve(&index, id) else { return false };
    let target = HSTRING::from(aumid_path(&entry.id).to_string_lossy().as_ref());
    let result = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    // ShellExecute riesce con un valore maggiore di 32.
    let started = result.0 as usize > 32;
    if started {
        launcherstore::record_launch(&entry.id, now_secs());
    }
    started
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_app_grid_is_alphabetical_ignoring_accents_and_case() {
        let e = |n: &str| (format!("id-{n}"), n.to_string());
        let entries = vec![e("Zeta"), e("àlfa"), e("beta"), e("Alfa"), e("Chrome")];
        let order: Vec<&str> = alphabetical(&entries).iter().map(|&i| entries[i].1.as_str()).collect();
        assert_eq!(order, ["àlfa", "Alfa", "beta", "Chrome", "Zeta"], "pari merito: l'ordine dell'indice");
        assert!(alphabetical(&[]).is_empty());
        assert!(GRID_PAGE_MAX >= 32);
    }

    use super::*;

    fn e(id: &str, name: &str) -> AppEntry {
        AppEntry { id: id.into(), name: name.into() }
    }

    #[test]
    fn clean_drops_empty_and_duplicates() {
        let v = clean(vec![e("A", "Alpha"), e("a", "Alpha bis"), e("", "Senza id"), e("B", "  "), e("C", "Gamma")]);
        assert_eq!(v, vec![e("A", "Alpha"), e("C", "Gamma")]);
        assert!(clean(vec![]).is_empty());
    }

    #[test]
    fn clip_ids_parse_strictly() {
        assert_eq!(clip_id("clip:12"), Some(12));
        assert_eq!(clip_id("clip:"), None);
        assert_eq!(clip_id("clip:-1"), None);
        assert_eq!(clip_id("clip:1 OR 1"), None);
        assert_eq!(clip_id("12"), None);
        assert_eq!(clip_id("clip:enable"), None);
        assert!(!clip_use("clip:999999999"), "id inesistente");
        assert!(!clip_delete("clip:999999999"));
    }

    #[test]
    fn service_entries_are_noise_but_real_apps_are_not() {
        // Dall'elenco reale: collegamenti web, manuali, licenze, documenti, disinstallatori.
        for (id, name) in [
            ("https://nodejs.org/", "Node.js website"),
            ("http://support.steampowered.com/", "Steam Support Center"),
            (r"{7C5A}\Windows Kits\10\Shortcuts\UWPAppsSamples.url", "Sample UWP Apps"),
            (r"C:\Python314\Doc\html\index.html", "Python 3.14 Manuals (64-bit)"),
            (r"{6D80}\WinRAR\WinRAR.chm", "Guida a WinRAR"),
            (r"{6D80}\PDF-XChange\PXCLicense.pdf", "PDF-XChange Editor License Agreement"),
            (r"{6D80}\WinRAR\Rar.txt", "Manuale di RAR"),
            ("Microsoft.AutoGenerated.{E5A4}", "Uninstall Node.js"),
            ("X", "Disinstalla Qualcosa"),
        ] {
            assert!(is_noise(id, name), "{name} dovrebbe essere rumore");
        }
        for (id, name) in [
            ("Microsoft.WindowsCalculator_8wekyb3d8bbwe!App", "Calcolatrice"),
            (r"C:\Python314\python.exe", "Python 3.14 (64-bit)"),
            ("::{645FF040-5081-101B-9F08-00AA002F954E}", "Cestino"),
            (r"{6D80}\WinRAR\WinRAR.exe", "WinRAR"),
            (r"{6D80}\Revo Uninstaller\RevoUnin.exe", "Revo Uninstaller"),
            (r"{1AC1}\services.msc", "Servizi"),
            ("Brave", "Brave"),
            (r"C:\xampp\htdocs", "XAMPP htdocs folder"),
        ] {
            assert!(!is_noise(id, name), "{name} e' un'app vera");
        }
    }

    #[test]
    fn clean_also_drops_noise() {
        let v = clean(vec![e("https://x.org/", "Sito web"), e("A", "Alpha"), e("a.pdf", "Manuale")]);
        assert_eq!(v, vec![e("A", "Alpha")]);
    }

    #[test]
    fn resolve_only_finds_indexed_ids() {
        let idx = vec![e("Microsoft.Calc!App", "Calcolatrice")];
        assert!(resolve(&idx, "microsoft.calc!app").is_some(), "senza badare alle maiuscole");
        assert!(resolve(&idx, r"C:\Windows\System32\cmd.exe").is_none(), "un percorso non indicizzato non si avvia");
        assert!(resolve(&idx, "").is_none());
        assert!(resolve(&[], "x").is_none());
    }

    /// Prova contro la shell vera: sola lettura, non avvia nulla.
    #[test]
    fn real_apps_folder_lists_apps() {
        let v = apps();
        println!("app trovate: {}", v.len());
        for x in v.iter().take(5) {
            println!("  {} -> {}", x.name, x.id);
        }
        assert!(v.iter().all(|x| !x.id.is_empty() && !x.name.is_empty()));
    }

    /// Ricerche vere sull'indice reale (sola lettura, nessun avvio).
    #[test]
    fn real_search_examples() {
        for q in ["calc", "cal", "note", "paint", "vsc", "chr", "terminal", "zzzzqq", "", "12*7", "20% di 340", "riavvia", "impost", "7-zip"] {
            let r = search(q, &["__nessuna__".into()], None);
            let names: Vec<_> = r.iter().map(|x| format!("{}{}", x.name, if x.icon.is_some() { "*" } else { "" })).collect();
            println!("[{q}] -> {names:?}");
        }
    }

    /// Avvia davvero la Calcolatrice (app di prova) e controlla che l'uso venga registrato.
    /// Solo a richiesta: `cargo test real_launch -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn real_launch_calculator() {
        let calc = apps().into_iter().find(|a| a.id.contains("WindowsCalculator")).expect("Calcolatrice");
        assert!(!launch(r"C:\Windows\System32\cmd.exe"), "un id non indicizzato non si avvia");
        assert!(launch(&calc.id));
        let usage = launcherstore::load();
        assert_eq!(usage.get(&calc.id).map(|u| u.count), Some(1));
        let top = search("", &[], None);
        assert_eq!(top.first().map(|x| x.id.clone()), Some(calc.id.clone()));
        println!("avviata {} e registrata", calc.name);
    }
}
