//! Profili dei desktop virtuali (Mission Control, fase M5). Solo funzioni pure, testate: nessun COM.
//!
//! Le interfacce non documentate di Windows per i desktop virtuali cambiano fra le versioni: identificatori
//! (IID) e posizione dei metodi nella tabella virtuale. Per non dipendere da una build precisa il loro
//! layout sta in **profili guidati da dati**: uno incorporato nel programma (`vdesk-profiles.json`) e uno
//! opzionale in `%LOCALAPPDATA%\EosOS\vdesk-profiles.json` che ha la precedenza. Un cambio di
//! identificatori o di indici si risolve aggiornando il file, senza ricompilare; un cambio di *firma* dei
//! metodi (parametri diversi) richiede codice nuovo (qui c'è solo il flag `monitorArgs`).
//!
//! Un profilo non è una garanzia: scegliere quello giusto per la build è solo il primo passo, poi la
//! verifica di coerenza in sola lettura (`vdint`) decide se si può usare davvero.

use serde::Deserialize;
use windows::core::GUID;

/// Indici dei metodi di `IVirtualDesktopManagerInternal` nella tabella virtuale (0-2 sono IUnknown).
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagerMethods {
    pub get_count: u16,
    pub move_view: u16,
    pub can_move_view: u16,
    pub get_current: u16,
    pub get_desktops: u16,
    pub get_adjacent: u16,
    pub switch_to: u16,
    pub create: u16,
    pub remove: u16,
    pub find: u16,
    pub include_exclude: u16,
    pub set_name: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Manager {
    pub service: String,
    pub iid: String,
    pub methods: ManagerMethods,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DesktopMethods {
    pub get_id: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct DesktopIface {
    pub iid: String,
    pub methods: DesktopMethods,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ViewMethods {
    pub get_view_for_hwnd: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Views {
    pub service: String,
    pub iid: String,
    pub methods: ViewMethods,
}

/// Un profilo: per quali build vale e come sono fatte le interfacce.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    #[serde(default)]
    pub description: String,
    pub min_build: u32,
    pub max_build: u32,
    /// Revisione minima (la parte dopo il punto: 26100.**2605**), se serve restringere.
    #[serde(default)]
    pub min_ubr: u32,
    /// I metodi del gestore che lavorano per monitor prendono un `HMONITOR` come primo argomento
    /// (Windows 11 prima della 24H2).
    #[serde(default)]
    pub monitor_args: bool,
    pub manager: Manager,
    pub desktop: DesktopIface,
    pub views: Views,
}

#[derive(Debug, Deserialize)]
struct File {
    version: u32,
    profiles: Vec<Profile>,
}

/// Il profilo incorporato nel programma.
pub const BUILTIN: &str = include_str!("vdesk-profiles.json");

/// Legge un GUID scritto come "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}" (graffe facoltative, maiuscole o
/// minuscole). `None` se il formato non è questo. Pura, testata.
pub fn parse_guid(text: &str) -> Option<GUID> {
    let t = text.trim().trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = t.split('-').collect();
    if parts.len() != 5 || [8, 4, 4, 4, 12].iter().zip(&parts).any(|(n, p)| p.len() != *n) {
        return None;
    }
    let hex = |s: &str| u64::from_str_radix(s, 16).ok();
    let d1 = hex(parts[0])? as u32;
    let d2 = hex(parts[1])? as u16;
    let d3 = hex(parts[2])? as u16;
    let tail = hex(&format!("{}{}", parts[3], parts[4]))?;
    let mut d4 = [0u8; 8];
    d4.copy_from_slice(&tail.to_be_bytes());
    Some(GUID { data1: d1, data2: d2, data3: d3, data4: d4 })
}

/// Il GUID come testo con le graffe, in maiuscolo. Pura, testata.
pub fn guid_text(g: &GUID) -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        g.data1, g.data2, g.data3, g.data4[0], g.data4[1], g.data4[2], g.data4[3], g.data4[4], g.data4[5], g.data4[6], g.data4[7]
    )
}

/// Indici accettabili per un metodo: dopo i tre di IUnknown e in un intervallo ragionevole.
const MIN_INDEX: u16 = 3;
const MAX_INDEX: u16 = 80;

impl Profile {
    /// Controlla che il profilo sia sensato: GUID validi, indici nell'intervallo, nessun indice ripetuto
    /// nel gestore, intervallo di build coerente. Restituisce il primo problema trovato. Pura, testata.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("profilo senza id".into());
        }
        if self.min_build > self.max_build {
            return Err(format!("{}: intervallo di build rovesciato", self.id));
        }
        for (what, g) in [
            ("manager.service", &self.manager.service),
            ("manager.iid", &self.manager.iid),
            ("desktop.iid", &self.desktop.iid),
            ("views.service", &self.views.service),
            ("views.iid", &self.views.iid),
        ] {
            if parse_guid(g).is_none() {
                return Err(format!("{}: {what} non e' un GUID valido ({g})", self.id));
            }
        }
        let m = &self.manager.methods;
        let manager = [
            m.get_count, m.move_view, m.can_move_view, m.get_current, m.get_desktops, m.get_adjacent, m.switch_to, m.create,
            m.remove, m.find, m.include_exclude, m.set_name,
        ];
        let all = manager.iter().chain([&self.desktop.methods.get_id, &self.views.methods.get_view_for_hwnd]);
        if let Some(bad) = all.clone().find(|i| !(MIN_INDEX..=MAX_INDEX).contains(*i)) {
            return Err(format!("{}: indice di metodo {bad} fuori da {MIN_INDEX}..={MAX_INDEX}", self.id));
        }
        let mut sorted = manager.to_vec();
        sorted.sort_unstable();
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return Err(format!("{}: due metodi del gestore con lo stesso indice", self.id));
        }
        Ok(())
    }

    /// Il profilo vale per questa build? Pura, testata.
    pub fn matches(&self, build: u32, ubr: u32) -> bool {
        build >= self.min_build && build <= self.max_build && (build > self.min_build || ubr >= self.min_ubr)
    }
}

/// Legge un file di profili. Un profilo non valido si scarta, non invalida gli altri: restituisce quelli buoni
/// e i messaggi sugli scartati. `Err` solo se il testo non è un file di profili. Pura, testata.
pub fn parse_profiles(text: &str) -> Result<(Vec<Profile>, Vec<String>), String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let f: File = serde_json::from_str(text).map_err(|e| format!("file dei profili non leggibile: {e}"))?;
    if f.version != 1 {
        return Err(format!("versione {} del file dei profili non supportata", f.version));
    }
    let mut ok = Vec::new();
    let mut problems = Vec::new();
    for p in f.profiles {
        match p.validate() {
            Ok(()) => ok.push(p),
            Err(e) => problems.push(e),
        }
    }
    Ok((ok, problems))
}

/// Mette insieme i profili: quelli dell'utente per primi (hanno la precedenza), poi quelli incorporati; a
/// parità di id vince il primo. Pura, testata.
pub fn merge(user: Vec<Profile>, builtin: Vec<Profile>) -> Vec<Profile> {
    let mut out: Vec<Profile> = Vec::new();
    for p in user.into_iter().chain(builtin) {
        if !out.iter().any(|x| x.id == p.id) {
            out.push(p);
        }
    }
    out
}

/// L'ordine in cui provare i profili per una build: prima quelli che la dichiarano (nell'ordine dato), poi
/// gli altri dal più vicino in su (per una build non coperta può valere lo stesso layout di una vicina: la
/// verifica di coerenza lo dirà). Restituisce `(profilo, dichiarato)`. Pura, testata.
pub fn candidates(profiles: &[Profile], build: u32, ubr: u32) -> Vec<(&Profile, bool)> {
    let mut declared: Vec<(&Profile, bool)> = profiles.iter().filter(|p| p.matches(build, ubr)).map(|p| (p, true)).collect();
    let mut others: Vec<&Profile> = profiles.iter().filter(|p| !p.matches(build, ubr)).collect();
    others.sort_by_key(|p| {
        let dist = if build < p.min_build { p.min_build - build } else { build.saturating_sub(p.max_build) };
        (dist, p.id.clone())
    });
    declared.extend(others.into_iter().map(|p| (p, false)));
    declared
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin() -> Vec<Profile> {
        parse_profiles(BUILTIN).expect("il profilo incorporato e' leggibile").0
    }

    #[test]
    fn the_builtin_profile_file_is_valid() {
        let (ok, problems) = parse_profiles(BUILTIN).unwrap();
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(ok.len(), 1);
        let p = &ok[0];
        assert!(p.validate().is_ok());
        assert!(p.matches(26100, 2605) && p.matches(26300, 9818) && !p.matches(22631, 0));
        assert_eq!(guid_text(&parse_guid(&p.manager.iid).unwrap()), "{53F5CA0B-158F-4124-900C-057158060B27}");
        assert_eq!(p.manager.methods.get_count, 3);
        assert!(!p.monitor_args);
    }

    #[test]
    fn guids_parse_with_or_without_braces_and_roundtrip() {
        let g = parse_guid("{c5e0cdca-7b6e-41b2-9fc4-d93975cc467b}").unwrap();
        assert_eq!(g.data1, 0xC5E0CDCA);
        assert_eq!(g.data2, 0x7B6E);
        assert_eq!(g.data3, 0x41B2);
        assert_eq!(g.data4, [0x9F, 0xC4, 0xD9, 0x39, 0x75, 0xCC, 0x46, 0x7B]);
        assert_eq!(guid_text(&g), "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}");
        assert_eq!(parse_guid("C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B"), Some(g));
        assert_eq!(parse_guid("  {C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}  "), Some(g));
        for bad in ["", "{}", "non un guid", "{C5E0CDCA-7B6E-41B2-9FC4}", "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467}", "{ZZE0CDCA-7B6E-41B2-9FC4-D93975CC467B}", "{C5E0CDCA7B6E41B29FC4D93975CC467B}"] {
            assert_eq!(parse_guid(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn build_matching_respects_the_revision_only_at_the_lower_edge() {
        let mut p = builtin().remove(0);
        p.min_build = 26100;
        p.min_ubr = 2605;
        p.max_build = 26200;
        assert!(!p.matches(26100, 2604), "stessa build, revisione troppo vecchia");
        assert!(p.matches(26100, 2605));
        assert!(p.matches(26101, 0), "una build piu' nuova non guarda la revisione");
        assert!(p.matches(26200, 5));
        assert!(!p.matches(26201, 0));
        assert!(!p.matches(26099, 9999));
    }

    #[test]
    fn broken_profiles_are_rejected_one_by_one() {
        let mut p = builtin().remove(0);
        p.manager.iid = "non-un-guid".into();
        assert!(p.validate().unwrap_err().contains("manager.iid"));
        let mut p = builtin().remove(0);
        p.manager.methods.create = 2;
        assert!(p.validate().unwrap_err().contains("fuori"));
        let mut p = builtin().remove(0);
        p.manager.methods.remove = 500;
        assert!(p.validate().unwrap_err().contains("fuori"));
        let mut p = builtin().remove(0);
        p.manager.methods.remove = p.manager.methods.create;
        assert!(p.validate().unwrap_err().contains("stesso indice"));
        let mut p = builtin().remove(0);
        p.min_build = p.max_build + 1;
        assert!(p.validate().unwrap_err().contains("rovesciato"));
        let mut p = builtin().remove(0);
        p.id = "  ".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn a_bad_profile_does_not_invalidate_the_good_ones() {
        let good = serde_json::to_value(serde_json::from_str::<serde_json::Value>(BUILTIN).unwrap()["profiles"][0].clone()).unwrap();
        let mut bad = good.clone();
        bad["id"] = "rotto".into();
        bad["manager"]["iid"] = "x".into();
        let text = serde_json::json!({ "version": 1, "profiles": [bad, good] }).to_string();
        let (ok, problems) = parse_profiles(&text).unwrap();
        assert_eq!(ok.len(), 1);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("rotto"));
    }

    #[test]
    fn unreadable_or_unsupported_files_are_an_error_not_a_panic() {
        assert!(parse_profiles("").is_err());
        assert!(parse_profiles("non json").is_err());
        assert!(parse_profiles(r#"{"version":2,"profiles":[]}"#).unwrap_err().contains("versione 2"));
        assert!(parse_profiles(r#"{"version":1}"#).is_err(), "manca l'elenco");
        assert!(parse_profiles(r#"{"version":1,"profiles":[{"id":"x"}]}"#).is_err(), "profilo incompleto");
        assert_eq!(parse_profiles(r#"{"version":1,"profiles":[]}"#).unwrap().0.len(), 0);
        let bom = format!("{}{}", char::from_u32(0xfeff).unwrap(), r#"{"version":1,"profiles":[]}"#);
        assert!(parse_profiles(&bom).is_ok(), "BOM tollerato");
    }

    #[test]
    fn user_profiles_take_precedence_and_ids_do_not_repeat() {
        let b = builtin();
        let mut mine = b[0].clone();
        mine.description = "dell'utente".into();
        let merged = merge(vec![mine.clone()], b.clone());
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].description, "dell'utente");
        let mut other = b[0].clone();
        other.id = "altro".into();
        let merged = merge(vec![other], b);
        assert_eq!(merged.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["altro", "w11-24h2-and-later"]);
    }

    #[test]
    fn candidates_put_declared_profiles_first_then_the_nearest_ones() {
        let base = builtin().remove(0);
        let mk = |id: &str, min: u32, max: u32| Profile { id: id.into(), min_build: min, max_build: max, min_ubr: 0, ..base.clone() };
        let ps = vec![mk("vecchio", 19000, 19999), mk("recente", 22000, 22999), mk("nuovo", 26100, 26200), mk("lontano", 10000, 10500)];
        // Build coperta: il dichiarato per primo.
        let c = candidates(&ps, 26150, 0);
        assert_eq!((c[0].0.id.as_str(), c[0].1), ("nuovo", true));
        // Build NON coperta (26300): nessun dichiarato; si provano i piu' vicini per primi.
        let c = candidates(&ps, 26300, 0);
        assert!(c.iter().all(|(_, declared)| !declared));
        assert_eq!(c.iter().map(|(p, _)| p.id.as_str()).collect::<Vec<_>>(), ["nuovo", "recente", "vecchio", "lontano"]);
        // Nessun profilo: nessun candidato.
        assert!(candidates(&[], 26300, 0).is_empty());
    }
}
