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
    /// Data URL dell'icona, solo se il frontend non l'ha già.
    pub icon: Option<String>,
}

/// Toglie le voci senza id o senza nome e i doppioni (stesso id, senza badare alle maiuscole),
/// tenendo la prima. Pura, testata.
pub fn clean(raw: Vec<AppEntry>) -> Vec<AppEntry> {
    let mut seen = HashSet::new();
    raw.into_iter()
        .filter(|e| !e.id.trim().is_empty() && !e.name.trim().is_empty())
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

/// Cerca `query` tra le app. `have`: gli id di cui il frontend ha già l'icona.
pub fn search(query: &str, have: &[String]) -> Vec<LauncherItem> {
    let index = apps();
    let usage = launcherstore::load();
    let entries: Vec<(String, String)> = index.iter().map(|e| (e.id.clone(), e.name.clone())).collect();
    let have: HashSet<String> = have.iter().map(|h| fold(h)).collect();
    rank(&entries, &usage, query, now_secs(), LIMIT)
        .into_iter()
        .map(|r| {
            let e = &index[r.index];
            let icon = (!have.contains(&fold(&e.id))).then(|| cached_icon(&aumid_path(&e.id))).flatten();
            LauncherItem { id: e.id.clone(), name: e.name.clone(), icon }
        })
        .collect()
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
        for q in ["calc", "cal", "note", "paint", "vsc", "chr", "terminal", "zzzzqq", ""] {
            let r = search(q, &["__nessuna__".into()]);
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
        let top = search("", &[]);
        assert_eq!(top.first().map(|x| x.id.clone()), Some(calc.id.clone()));
        println!("avviata {} e registrata", calc.name);
    }
}
