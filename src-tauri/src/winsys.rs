//! Impostazioni di Windows dentro EosOS: le pagine che si possono fare davvero (informazioni sul sistema, tema, alimentazione, app all'avvio,
//! archiviazione, schermi) con le API pubbliche di Windows, senza aprire l'app Impostazioni.
//!
//! Le altre pagine di Windows (Bluetooth e dispositivi, App predefinite, Account, Privacy…) non hanno API pubbliche per essere replicate: per
//! quelle resta l'elenco di `winsettings.rs` che apre la pagina vera. Qui la logica pura è testata; sotto c'è ciò che legge e scrive.
//!
//! Il registro si tocca con `reg.exe` (senza finestra) solo su valori noti e fissi dell'utente (HKCU); un nome che arriva dalla pagina si usa soltanto
//! se è uno di quelli appena letti.

use std::os::windows::process::CommandExt;
use std::process::Command;

use serde::Serialize;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const PERSONALIZE: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const RUN: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const NT: &str = r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion";

// ------------------------------------------------------------------ logica pura

/// «1,5 TB», «931 GB», «512 MB»: unità decimali come le mostra Windows, una cifra dopo la virgola sopra i 10 GB scarta il decimale. Pura, testata.
pub fn format_size(bytes: u64) -> String {
    const GB: f64 = 1e9;
    let b = bytes as f64;
    if b >= 1e12 {
        format!("{:.1} TB", b / 1e12).replace('.', ",")
    } else if b >= 10.0 * GB {
        format!("{:.0} GB", b / GB)
    } else if b >= GB {
        format!("{:.1} GB", b / GB).replace('.', ",")
    } else {
        format!("{:.0} MB", b / 1e6)
    }
}

/// «Windows 11 Pro 25H2 (build 26300.1234)». Pura, testata.
pub fn os_label(product: &str, display_version: &str, build: &str, ubr: Option<u32>) -> String {
    let mut s = product.trim().to_string();
    if s.is_empty() {
        s = "Windows".to_string();
    }
    // Il registro dice «Windows 10…» anche su Windows 11 (dalla build 22000 in poi).
    if build.trim().parse::<u32>().is_ok_and(|b| b >= 22000) && s.starts_with("Windows 10") {
        s = s.replacen("Windows 10", "Windows 11", 1);
    }
    if !display_version.trim().is_empty() {
        s.push(' ');
        s.push_str(display_version.trim());
    }
    if !build.trim().is_empty() {
        s.push_str(&format!(" (build {}", build.trim()));
        if let Some(u) = ubr {
            s.push_str(&format!(".{u}"));
        }
        s.push(')');
    }
    s
}

/// Le righe di `reg query`: «    Nome    REG_SZ    dati» → (nome, tipo, dati). Si ignorano le righe che non hanno quella forma. Pura, testata.
pub fn parse_reg_query(out: &str) -> Vec<(String, String, String)> {
    let mut v = Vec::new();
    for line in out.lines() {
        if !line.starts_with("    ") {
            continue;
        }
        let t = line.trim_start();
        // Nome e tipo sono separati da 4 spazi dal resto; il nome può contenere spazi singoli.
        let Some(idx) = t.find("    REG_") else { continue };
        let name = t[..idx].trim_end().to_string();
        let rest = t[idx..].trim_start();
        let (ty, data) = match rest.split_once("    ") {
            Some((a, b)) => (a.trim().to_string(), b.trim().to_string()),
            None => (rest.trim().to_string(), String::new()),
        };
        if !name.is_empty() {
            v.push((name, ty, data));
        }
    }
    v
}

/// «0x1» o «1» → 1. Pura, testata.
pub fn parse_dword(s: &str) -> Option<u32> {
    let t = s.trim();
    match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16).ok(),
        None => t.parse().ok(),
    }
}

/// Il tema com'è oggi.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    /// Le app usano il tema chiaro.
    pub apps_light: bool,
    /// Windows (barra, menu Start) usa il tema chiaro.
    pub system_light: bool,
    /// Gli effetti di trasparenza sono accesi.
    pub transparency: bool,
}

/// Dai valori del registro (assenti = come di base: chiaro e trasparenza accesa). Pura, testata.
pub fn theme_from(apps: Option<u32>, system: Option<u32>, transparency: Option<u32>) -> Theme {
    Theme { apps_light: apps != Some(0), system_light: system != Some(0), transparency: transparency != Some(0) }
}

/// Il nome del valore del tema che si può cambiare da qui, dall'id che manda la pagina; sconosciuto = `None`. Pura, testata.
pub fn theme_value_name(which: &str) -> Option<&'static str> {
    match which {
        "apps" => Some("AppsUseLightTheme"),
        "system" => Some("SystemUsesLightTheme"),
        "transparency" => Some("EnableTransparency"),
        _ => None,
    }
}

/// Le scelte dei tempi di spegnimento dello schermo e di sospensione, in minuti (0 = mai). Pura.
pub const TIMEOUTS: &[u32] = &[0, 1, 2, 3, 5, 10, 15, 20, 25, 30, 45, 60, 120, 180, 240, 300];

/// Il minuti scelto più vicino fra `TIMEOUTS`, da secondi letti da Windows (0 = mai). Pura, testata.
pub fn minutes_from_seconds(secs: u32) -> u32 {
    if secs == 0 {
        return 0;
    }
    let m = secs.div_ceil(60);
    *TIMEOUTS.iter().skip(1).min_by_key(|t| t.abs_diff(m)).unwrap_or(&0)
}

/// Il comando di `powercfg /change` per un id della pagina («screen-ac», «sleep-dc»…); sconosciuto = `None`. Pura, testata.
pub fn power_setting(which: &str) -> Option<&'static str> {
    match which {
        "screen-ac" => Some("monitor-timeout-ac"),
        "screen-dc" => Some("monitor-timeout-dc"),
        "sleep-ac" => Some("standby-timeout-ac"),
        "sleep-dc" => Some("standby-timeout-dc"),
        _ => None,
    }
}

/// I 12 byte di `StartupApproved`: 02 = attivata, 03 = disattivata (gli altri 8 sono la data). Pura, testata.
pub fn approved_hex(enabled: bool) -> &'static str {
    if enabled { "020000000000000000000000" } else { "030000000000000000000000" }
}

/// «02 00 00…» o «0200…» come lo scrive `reg query` → byte. Pura, testata.
pub fn hex_bytes(s: &str) -> Vec<u8> {
    let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    clean.as_bytes().chunks(2).filter_map(|p| std::str::from_utf8(p).ok().and_then(|h| u8::from_str_radix(h, 16).ok())).collect()
}

// ------------------------------------------------------------------ lettura e scrittura

fn reg_query(key: &str, value: Option<&str>) -> String {
    let mut c = Command::new("reg");
    c.arg("query").arg(key);
    if let Some(v) = value {
        c.args(["/v", v]);
    }
    c.creation_flags(CREATE_NO_WINDOW).output().map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
}

fn reg_value(key: &str, name: &str) -> Option<String> {
    parse_reg_query(&reg_query(key, Some(name))).into_iter().find(|(n, _, _)| n.eq_ignore_ascii_case(name)).map(|(_, _, d)| d)
}

fn reg_add(key: &str, name: &str, ty: &str, data: &str) -> bool {
    Command::new("reg").args(["add", key, "/v", name, "/t", ty, "/d", data, "/f"]).creation_flags(CREATE_NO_WINDOW).output().is_ok_and(|o| o.status.success())
}

/// Informazioni sul sistema.
#[derive(Serialize, Clone, Debug)]
pub struct Info {
    pub device: String,
    pub user: String,
    pub os: String,
    pub cpu: String,
    pub memory: String,
    pub uptime: String,
}

fn uptime_label(ms: u64) -> String {
    let mins = ms / 60_000;
    let (d, h, m) = (mins / 1440, (mins / 60) % 24, mins % 60);
    if d > 0 { format!("{d} g {h} h") } else if h > 0 { format!("{h} h {m} min") } else { format!("{m} min") }
}

pub fn info() -> Info {
    use windows::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut m = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    let mem = if unsafe { GlobalMemoryStatusEx(&mut m) }.is_ok() { format_size(m.ullTotalPhys) } else { String::new() };
    let nt = |n: &str| reg_value(NT, n).unwrap_or_default();
    let ubr = reg_value(NT, "UBR").and_then(|s| parse_dword(&s));
    let display = {
        let d = nt("DisplayVersion");
        if d.is_empty() { nt("ReleaseId") } else { d }
    };
    Info {
        device: std::env::var("COMPUTERNAME").unwrap_or_default(),
        user: std::env::var("USERNAME").unwrap_or_default(),
        os: os_label(&nt("ProductName"), &display, &nt("CurrentBuild"), ubr),
        cpu: reg_value(r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0", "ProcessorNameString").unwrap_or_default().split_whitespace().collect::<Vec<_>>().join(" "),
        memory: mem,
        uptime: uptime_label(unsafe { GetTickCount64() }),
    }
}

/// Un disco.
#[derive(Serialize, Clone, Debug)]
pub struct Drive {
    pub letter: String,
    pub label: String,
    pub total: String,
    pub free: String,
    /// Quanto è pieno, in percentuale (0..100).
    pub used_percent: u32,
}

/// Percentuale usata, limitata a 0..100. Pura, testata.
pub fn used_percent(total: u64, free: u64) -> u32 {
    if total == 0 {
        return 0;
    }
    (((total.saturating_sub(free)) as f64 / total as f64) * 100.0).round().clamp(0.0, 100.0) as u32
}

pub fn drives() -> Vec<Drive> {
    use windows::core::HSTRING;
    use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW};
    const DRIVE_FIXED: u32 = 3;
    let mask = unsafe { GetLogicalDrives() };
    let mut out = Vec::new();
    for i in 0..26u32 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let letter = (b'A' + i as u8) as char;
        let root = HSTRING::from(format!("{letter}:\\"));
        if unsafe { GetDriveTypeW(&root) } != DRIVE_FIXED {
            continue;
        }
        let (mut total, mut free) = (0u64, 0u64);
        if unsafe { GetDiskFreeSpaceExW(&root, None, Some(&mut total), Some(&mut free)) }.is_err() {
            continue;
        }
        let mut label = [0u16; 64];
        let _ = unsafe { GetVolumeInformationW(&root, Some(&mut label), None, None, None, None) };
        let len = label.iter().position(|&c| c == 0).unwrap_or(0);
        out.push(Drive { letter: format!("{letter}:"), label: String::from_utf16_lossy(&label[..len]), total: format_size(total), free: format_size(free), used_percent: used_percent(total, free) });
    }
    out
}

pub fn theme() -> Theme {
    let get = |n: &str| reg_value(PERSONALIZE, n).and_then(|s| parse_dword(&s));
    theme_from(get("AppsUseLightTheme"), get("SystemUsesLightTheme"), get("EnableTransparency"))
}

/// Cambia un valore del tema (`which`: «apps», «system», «transparency») e avvisa le finestre perché si ridisegnino.
pub fn set_theme(which: &str, light_or_on: bool) -> bool {
    let Some(name) = theme_value_name(which) else { return false };
    if !reg_add(PERSONALIZE, name, "REG_DWORD", if light_or_on { "1" } else { "0" }) {
        return false;
    }
    broadcast_theme_change();
    true
}

fn broadcast_theme_change() {
    use windows::core::w;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
    unsafe {
        let _ = SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, WPARAM(0), LPARAM(w!("ImmersiveColorSet").as_ptr() as isize), SMTO_ABORTIFHUNG, 1000, None);
        let _ = HWND::default();
    }
}

/// Alimentazione: minuti (0 = mai) con rete elettrica (ac) e a batteria (dc).
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Power {
    pub screen_ac: u32,
    pub screen_dc: u32,
    pub sleep_ac: u32,
    pub sleep_dc: u32,
    pub has_battery: bool,
}

pub fn power() -> Power {
    use windows::core::GUID;
    use windows::Win32::System::Power::{GetSystemPowerStatus, PowerGetActiveScheme, PowerReadACValueIndex, PowerReadDCValueIndex, SYSTEM_POWER_STATUS};
    // GUID dei sottogruppi e delle impostazioni (Schermo → Spegni dopo; Sospensione → Sospendi dopo).
    const SUB_VIDEO: GUID = GUID::from_u128(0x7516b95f_f776_4464_8c53_06167f40cc99);
    const VIDEOIDLE: GUID = GUID::from_u128(0x3c0bc021_c8a8_4e07_a973_6b14cbcb2b7e);
    const SUB_SLEEP: GUID = GUID::from_u128(0x238c9fa8_0aad_41ed_83f4_97be242c8f20);
    const STANDBYIDLE: GUID = GUID::from_u128(0x29f6c1db_86da_48c5_9fdb_f2b67b1f44da);
    let mut p = Power { screen_ac: 0, screen_dc: 0, sleep_ac: 0, sleep_dc: 0, has_battery: false };
    unsafe {
        let mut sps = SYSTEM_POWER_STATUS::default();
        if GetSystemPowerStatus(&mut sps).is_ok() {
            p.has_battery = sps.BatteryFlag != 128 && sps.BatteryFlag != 255;
        }
        let mut scheme: *mut GUID = std::ptr::null_mut();
        if PowerGetActiveScheme(None, &mut scheme).0 != 0 || scheme.is_null() {
            return p;
        }
        let read = |ac: bool, sub: &GUID, set: &GUID| -> u32 {
            let mut v = 0u32;
            let code = if ac { PowerReadACValueIndex(None, Some(scheme), Some(sub), Some(set), &mut v).0 } else { PowerReadDCValueIndex(None, Some(scheme), Some(sub), Some(set), &mut v) };
            if code == 0 { minutes_from_seconds(v) } else { 0 }
        };
        p.screen_ac = read(true, &SUB_VIDEO, &VIDEOIDLE);
        p.screen_dc = read(false, &SUB_VIDEO, &VIDEOIDLE);
        p.sleep_ac = read(true, &SUB_SLEEP, &STANDBYIDLE);
        p.sleep_dc = read(false, &SUB_SLEEP, &STANDBYIDLE);
        windows::Win32::Foundation::LocalFree(Some(windows::Win32::Foundation::HLOCAL(scheme as *mut _)));
    }
    p
}

/// Cambia un tempo (`which` come «screen-ac»; `minutes` fra `TIMEOUTS`) con `powercfg /change`.
pub fn set_power(which: &str, minutes: u32) -> bool {
    let Some(setting) = power_setting(which) else { return false };
    if !TIMEOUTS.contains(&minutes) {
        return false;
    }
    Command::new("powercfg").args(["/change", setting, &minutes.to_string()]).creation_flags(CREATE_NO_WINDOW).output().is_ok_and(|o| o.status.success())
}

/// Un'app che parte con Windows (voce dell'utente in `Run`).
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct StartupApp {
    pub name: String,
    pub command: String,
    pub enabled: bool,
}

pub fn startup_apps() -> Vec<StartupApp> {
    let approved: Vec<(String, Vec<u8>)> = parse_reg_query(&reg_query(APPROVED, None)).into_iter().map(|(n, _, d)| (n, hex_bytes(&d))).collect();
    let mut v: Vec<StartupApp> = parse_reg_query(&reg_query(RUN, None))
        .into_iter()
        .filter(|(_, ty, _)| ty == "REG_SZ" || ty == "REG_EXPAND_SZ")
        .map(|(name, _, command)| {
            let blocked = approved.iter().find(|(n, _)| n.eq_ignore_ascii_case(&name)).map(|(_, b)| crate::autostart::blocked_by_user(Some(b))).unwrap_or(false);
            StartupApp { name, command, enabled: !blocked }
        })
        .collect();
    v.sort_by_key(|a| a.name.to_lowercase());
    v
}

/// Accende o spegne un'app all'avvio, come fa Gestione attività (`StartupApproved`). Solo un nome fra quelli appena letti.
pub fn set_startup(name: &str, enabled: bool) -> bool {
    if !startup_apps().iter().any(|a| a.name == name) {
        return false;
    }
    reg_add(APPROVED, name, "REG_BINARY", approved_hex(enabled))
}

/// Uno schermo.
#[derive(Serialize, Clone, Debug)]
pub struct Display {
    pub name: String,
    pub primary: bool,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub x: i32,
    pub y: i32,
}

/// Gli schermi (dalla finestra di Tauri: posizione, misura fisica e scala).
pub fn displays(app: &tauri::AppHandle) -> Vec<Display> {
    let primary = app.primary_monitor().ok().flatten().map(|m| (m.position().x, m.position().y));
    app.available_monitors()
        .map(|list| {
            list.iter()
                .enumerate()
                .map(|(i, m)| Display {
                    name: m.name().cloned().filter(|n| !n.is_empty()).unwrap_or_else(|| format!("Schermo {}", i + 1)),
                    primary: primary == Some((m.position().x, m.position().y)),
                    width: m.size().width,
                    height: m.size().height,
                    scale: m.scale_factor(),
                    x: m.position().x,
                    y: m.position().y,
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_decimal_like_windows() {
        assert_eq!(format_size(0), "0 MB");
        assert_eq!(format_size(512_000_000), "512 MB");
        assert_eq!(format_size(1_500_000_000), "1,5 GB");
        assert_eq!(format_size(931_000_000_000), "931 GB");
        assert_eq!(format_size(2_000_000_000_000), "2,0 TB");
    }

    #[test]
    fn the_os_label_is_built_from_the_registry_values() {
        assert_eq!(os_label("Windows 11 Pro", "25H2", "26300", Some(1234)), "Windows 11 Pro 25H2 (build 26300.1234)");
        assert_eq!(os_label("Windows 10 Home", "", "19045", None), "Windows 10 Home (build 19045)");
        assert_eq!(os_label("", "", "", None), "Windows");
        assert_eq!(os_label("Windows 10 Pro", "25H2", "26300", None), "Windows 11 Pro 25H2 (build 26300)", "il registro dice 10 su Windows 11");
    }

    #[test]
    fn reg_query_output_is_parsed() {
        let out = "\nHKEY_CURRENT_USER\\Software\\X\\Run\n    OneDrive    REG_SZ    \"C:\\Program Files\\OD\\OneDrive.exe\" /background\n    Nome Con Spazi    REG_SZ    C:\\a b\\c.exe\n    Vuoto    REG_SZ    \n    Num    REG_DWORD    0x1\n\nFine\n";
        let v = parse_reg_query(out);
        assert_eq!(v.len(), 4, "{v:?}");
        assert_eq!(v[0], ("OneDrive".into(), "REG_SZ".into(), "\"C:\\Program Files\\OD\\OneDrive.exe\" /background".into()));
        assert_eq!(v[1].0, "Nome Con Spazi");
        assert_eq!(v[2].2, "");
        assert_eq!(v[3], ("Num".into(), "REG_DWORD".into(), "0x1".into()));
        assert!(parse_reg_query("").is_empty());
    }

    #[test]
    fn dwords_and_hex_are_parsed() {
        assert_eq!(parse_dword("0x1"), Some(1));
        assert_eq!(parse_dword("0x00000000"), Some(0));
        assert_eq!(parse_dword("5"), Some(5));
        assert_eq!(parse_dword("boh"), None);
        assert_eq!(hex_bytes("02 00 00 00"), vec![2, 0, 0, 0]);
        assert_eq!(hex_bytes("030000000000000000000000").len(), 12);
        assert_eq!(hex_bytes("zz"), Vec::<u8>::new());
    }

    #[test]
    fn the_theme_defaults_to_light_with_transparency() {
        assert_eq!(theme_from(None, None, None), Theme { apps_light: true, system_light: true, transparency: true });
        assert_eq!(theme_from(Some(0), Some(1), Some(0)), Theme { apps_light: false, system_light: true, transparency: false });
        assert_eq!(theme_value_name("apps"), Some("AppsUseLightTheme"));
        assert_eq!(theme_value_name("system"), Some("SystemUsesLightTheme"));
        assert_eq!(theme_value_name("transparency"), Some("EnableTransparency"));
        assert_eq!(theme_value_name("boh"), None);
        assert!(!set_theme("boh", true), "un id inventato non scrive niente");
    }

    #[test]
    fn power_minutes_snap_to_the_offered_choices() {
        assert_eq!(minutes_from_seconds(0), 0, "mai");
        assert_eq!(minutes_from_seconds(60), 1);
        assert_eq!(minutes_from_seconds(600), 10);
        assert_eq!(minutes_from_seconds(900), 15);
        assert_eq!(minutes_from_seconds(14_400), 240);
        assert_eq!(minutes_from_seconds(50 * 60), 45, "il piu vicino");
        assert_eq!(power_setting("screen-ac"), Some("monitor-timeout-ac"));
        assert_eq!(power_setting("sleep-dc"), Some("standby-timeout-dc"));
        assert_eq!(power_setting("boh"), None);
        assert!(!set_power("boh", 5) && !set_power("screen-ac", 7), "id sconosciuto o minuti non ammessi: niente");
    }

    #[test]
    fn startup_blocks_use_the_task_manager_bytes() {
        assert_eq!(approved_hex(true), "020000000000000000000000");
        assert_eq!(approved_hex(false), "030000000000000000000000");
        assert!(crate::autostart::blocked_by_user(Some(&hex_bytes(approved_hex(false)))));
        assert!(!crate::autostart::blocked_by_user(Some(&hex_bytes(approved_hex(true)))));
        assert!(!set_startup("nome-che-non-esiste-eosos-test", true), "solo nomi che ci sono");
    }

    #[test]
    fn used_percent_is_bounded() {
        assert_eq!(used_percent(0, 0), 0);
        assert_eq!(used_percent(1000, 250), 75);
        assert_eq!(used_percent(1000, 2000), 0);
        assert_eq!(used_percent(1000, 0), 100);
    }

    #[test]
    fn uptime_is_readable() {
        assert_eq!(uptime_label(5 * 60_000), "5 min");
        assert_eq!(uptime_label(125 * 60_000), "2 h 5 min");
        assert_eq!(uptime_label(3 * 86_400_000 + 4 * 3_600_000), "3 g 4 h");
    }

    #[test]
    fn the_real_machine_can_be_read() {
        let i = info();
        println!("{i:?}");
        assert!(!i.os.is_empty());
        println!("{:?}", drives());
        println!("{:?}", theme());
        println!("{:?}", power());
        println!("{:?}", startup_apps());
        assert!(!drives().is_empty());
    }
}
