//! Indicatori di sistema (fase I1): batteria, rete e volume. Questo modulo non ha interfaccia: legge
//! lo stato da Windows, lo confronta con quello di prima e manda l'evento `indicators-changed`
//! (e il comando `indicators_get`) solo quando qualcosa cambia.
//!
//! Ogni indicatore arriva alla pagina **già con icona e testi calcolati qui** (parte pura e
//! testata): la pagina non decide nulla. La percentuale della batteria è sempre nel testo.
//!
//! Lettura: batteria con `GetSystemPowerStatus`, rete con WinRT `NetworkInformation`, volume con
//! Core Audio (dispositivo di uscita predefinito, riacquisito a ogni giro così un cambio di
//! dispositivo si vede subito). Un controllo ogni 1,5 s con confronto dello stato: costa pochissimo
//! e regge gli eventi mancati (sospensione, cuffie collegate, cavo staccato).
//!
//! Nelle build di prova (`test-hooks`) `EOSOS_FAKE_INDICATORS=<file.json>` sostituisce la lettura
//! vera con uno stato finto riletto a ogni giro: batteria, rete e volume non si possono né scaricare
//! né staccare né cambiare a comando sul PC dell'utente.

use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use windows::Networking::Connectivity::NetworkInformation;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

const POLL: Duration = Duration::from_millis(1500);

// ------------------------------------------------------------------ batteria

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Battery {
    pub percent: u8,
    pub charging: bool,
    /// Collegata alla rete elettrica (anche se non sta caricando perché è piena).
    pub plugged: bool,
    pub minutes_left: Option<u32>,
    pub icon: &'static str,
    /// Testo accanto all'icona: sempre la percentuale ("85%").
    pub label: String,
    /// Testo al passaggio del mouse.
    pub title: String,
}

/// "2 h 15 min", "45 min", "3 h". Pura, testata.
pub fn duration_label(minutes: u32) -> String {
    let (h, m) = (minutes / 60, minutes % 60);
    match (h, m) {
        (0, m) => format!("{m} min"),
        (h, 0) => format!("{h} h"),
        (h, m) => format!("{h} h {m} min"),
    }
}

/// Icona per livello e stato. Pura, testata.
pub fn battery_icon(percent: u8, charging: bool, plugged: bool) -> &'static str {
    if charging {
        "battery-charging"
    } else if plugged {
        "battery-plugged"
    } else {
        match percent {
            0..=10 => "battery-critical",
            11..=30 => "battery-low",
            31..=60 => "battery-half",
            61..=85 => "battery-high",
            _ => "battery-full",
        }
    }
}

impl Battery {
    /// Costruisce lo stato completo (icona, testi) dai dati grezzi. Pura, testata.
    pub fn new(percent: u8, charging: bool, plugged: bool, minutes_left: Option<u32>) -> Self {
        let percent = percent.min(100);
        let detail = if charging {
            Some("in carica".to_string())
        } else if plugged {
            Some("collegata alla rete".to_string())
        } else {
            minutes_left.map(|m| format!("{} rimanenti", duration_label(m)))
        };
        let title = match detail {
            Some(d) => format!("Batteria {percent}% · {d}"),
            None => format!("Batteria {percent}%"),
        };
        Self { percent, charging, plugged, minutes_left, icon: battery_icon(percent, charging, plugged), label: format!("{percent}%"), title }
    }
}

/// Stato della batteria dai campi di `SYSTEM_POWER_STATUS`: `None` se il PC non ha batteria o i
/// dati non sono noti (255). Pura, testata.
pub fn battery_from_status(ac_line: u8, flag: u8, percent: u8, life_secs: u32) -> Option<Battery> {
    if flag == 255 || flag & 128 != 0 || percent > 100 {
        return None;
    }
    let plugged = ac_line == 1;
    let charging = flag & 8 != 0;
    let minutes_left = (life_secs != u32::MAX && !plugged).then_some(life_secs / 60);
    Some(Battery::new(percent, charging, plugged, minutes_left))
}

// ------------------------------------------------------------------ rete

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetKind {
    Wifi,
    Ethernet,
    Cellular,
    None,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Online {
    /// Internet raggiungibile.
    Internet,
    /// Collegata ma senza Internet (o accesso limitato, per esempio una pagina di accesso).
    Limited,
    None,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Network {
    pub kind: NetKind,
    pub online: Online,
    pub name: Option<String>,
    /// Barre del segnale Wi-Fi, 0-5.
    pub bars: Option<u8>,
    pub icon: &'static str,
    pub title: String,
}

/// Un collegamento letto da Windows, prima della scelta. `level`: 0 nessuno, 1 locale, 2 limitato,
/// 3 Internet (i valori di `NetworkConnectivityLevel`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub level: i32,
    pub wlan: bool,
    pub wwan: bool,
    pub name: Option<String>,
    pub bars: Option<u8>,
}

/// Livello di Windows → raggiungibilità. Pura, testata.
pub fn online_from_level(level: i32) -> Online {
    match level {
        3 => Online::Internet,
        1 | 2 => Online::Limited,
        _ => Online::None,
    }
}

/// Icona della rete. Pura, testata.
pub fn network_icon(kind: NetKind, online: Online, bars: Option<u8>) -> &'static str {
    match (kind, online) {
        (_, Online::None) | (NetKind::None, _) => "net-off",
        (NetKind::Wifi, Online::Limited) => "wifi-alert",
        (NetKind::Wifi, Online::Internet) => match bars {
            Some(0..=1) => "wifi-1",
            Some(2..=3) => "wifi-2",
            _ => "wifi-3",
        },
        (NetKind::Ethernet, Online::Limited) => "ethernet-alert",
        (NetKind::Ethernet, Online::Internet) => "ethernet",
        (NetKind::Cellular, _) => "cellular",
    }
}

impl Network {
    /// Costruisce lo stato completo dai dati grezzi. Pura, testata.
    pub fn new(kind: NetKind, online: Online, name: Option<String>, bars: Option<u8>) -> Self {
        let bars = bars.map(|b| b.min(5));
        let what = match kind {
            NetKind::Wifi => "Wi-Fi",
            NetKind::Ethernet => "Cavo di rete",
            NetKind::Cellular => "Rete mobile",
            NetKind::None => "",
        };
        let title = if kind == NetKind::None || online == Online::None {
            "Nessuna connessione".to_string()
        } else {
            let base = match name.as_deref().filter(|n| !n.trim().is_empty()) {
                Some(n) if kind == NetKind::Wifi => format!("{what}: {n}"),
                Some(n) => format!("{what} ({n})"),
                None => what.to_string(),
            };
            if online == Online::Limited { format!("{base} · senza Internet") } else { base }
        };
        Self { kind, online, name, bars, icon: network_icon(kind, online, bars), title }
    }

    pub fn disconnected() -> Self {
        Self::new(NetKind::None, Online::None, None, None)
    }
}

/// Sceglie tra i collegamenti quello da mostrare: il livello più alto (almeno "locale"); a pari
/// livello cavo prima del Wi-Fi e Wi-Fi prima della rete mobile; poi il primo. Nessuno → scollegato.
/// Pura, testata.
pub fn pick_network(links: &[Link]) -> Network {
    let rank = |l: &Link| (l.level, if !l.wlan && !l.wwan { 2 } else if l.wlan { 1 } else { 0 });
    let best = links.iter().filter(|l| l.level > 0).fold(None::<&Link>, |best, l| match best {
        Some(b) if rank(b) >= rank(l) => Some(b),
        _ => Some(l),
    });
    match best {
        None => Network::disconnected(),
        Some(l) => {
            let kind = if l.wlan {
                NetKind::Wifi
            } else if l.wwan {
                NetKind::Cellular
            } else {
                NetKind::Ethernet
            };
            let bars = if kind == NetKind::Wifi { l.bars } else { None };
            Network::new(kind, online_from_level(l.level), l.name.clone(), bars)
        }
    }
}

// ------------------------------------------------------------------ volume

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Volume {
    pub percent: u8,
    pub muted: bool,
    pub icon: &'static str,
    pub title: String,
}

/// Icona del volume. Pura, testata.
pub fn volume_icon(percent: u8, muted: bool) -> &'static str {
    if muted {
        "volume-muted"
    } else {
        match percent {
            0 => "volume-0",
            1..=33 => "volume-1",
            34..=66 => "volume-2",
            _ => "volume-3",
        }
    }
}

impl Volume {
    pub fn new(percent: u8, muted: bool) -> Self {
        let percent = percent.min(100);
        let title = if muted { "Audio disattivato".to_string() } else { format!("Volume {percent}%") };
        Self { percent, muted, icon: volume_icon(percent, muted), title }
    }
}

/// Dal valore 0.0-1.0 di Core Audio alla percentuale intera (NaN e valori fuori scala compresi).
/// Pura, testata.
pub fn volume_from_scalar(scalar: f32, muted: bool) -> Volume {
    let p = if scalar.is_finite() { (scalar.clamp(0.0, 1.0) * 100.0).round() as u8 } else { 0 };
    Volume::new(p, muted)
}

// ------------------------------------------------------------------ insieme

/// Tutti gli indicatori. Un campo `None` = quell'hardware non c'è (desktop senza batteria, nessuna
/// uscita audio): la pagina non lo mostra.
#[derive(Serialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Indicators {
    pub battery: Option<Battery>,
    pub network: Option<Network>,
    pub volume: Option<Volume>,
}

/// Ultimo stato letto, per `indicators_get` e per confrontare.
static STATE: Mutex<Option<Indicators>> = Mutex::new(None);

pub fn current() -> Indicators {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_default()
}

/// Memorizza `new` e dice se è diverso da prima (il primo stato letto conta come cambio). Pura
/// rispetto al sistema, testata.
pub fn update_state(slot: &mut Option<Indicators>, new: Indicators) -> bool {
    if slot.as_ref() == Some(&new) {
        return false;
    }
    *slot = Some(new);
    true
}

// ------------------------------------------------------------------ lettura vera

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

fn read_battery() -> Option<Battery> {
    let mut s = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut s).ok()? };
    battery_from_status(s.ACLineStatus, s.BatteryFlag, s.BatteryLifePercent, s.BatteryLifeTime)
}

fn read_network() -> Network {
    let Ok(profiles) = NetworkInformation::GetConnectionProfiles() else { return Network::disconnected() };
    let links: Vec<Link> = profiles
        .into_iter()
        .filter_map(|p| {
            let level = p.GetNetworkConnectivityLevel().ok()?.0;
            Some(Link {
                level,
                wlan: p.IsWlanConnectionProfile().unwrap_or(false),
                wwan: p.IsWwanConnectionProfile().unwrap_or(false),
                name: p.ProfileName().ok().map(|n| n.to_string()),
                bars: p.GetSignalBars().ok().and_then(|r| r.Value().ok()),
            })
        })
        .collect();
    pick_network(&links)
}

fn read_volume() -> Option<Volume> {
    unsafe {
        let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let dev = en.GetDefaultAudioEndpoint(eRender, eMultimedia).ok()?;
        let vol: IAudioEndpointVolume = dev.Activate(CLSCTX_ALL, None).ok()?;
        let scalar = vol.GetMasterVolumeLevelScalar().ok()?;
        let muted = vol.GetMute().ok()?.as_bool();
        Some(volume_from_scalar(scalar, muted))
    }
}

/// Legge lo stato vero del sistema. Chi chiama deve aver inizializzato COM (apartment MTA) sul
/// thread e **tenerlo inizializzato per tutta la vita del thread**: le fabbriche WinRT restano in
/// cache e uscire dall'apartment tra una lettura e l'altra le lascerebbe puntare a oggetti
/// distrutti (verificato: 20 letture di fila con scope per lettura danno ACCESS_VIOLATION).
pub fn read_real() -> Indicators {
    Indicators { battery: read_battery(), network: Some(read_network()), volume: read_volume() }
}

// ------------------------------------------------------------------ sorgente finta (solo prove)

/// Stato finto da file JSON (build di prova): stessi dati grezzi che arrivano da Windows.
#[cfg(feature = "test-hooks")]
#[derive(Deserialize, Default)]
struct Fake {
    battery: Option<FakeBattery>,
    network: Option<FakeNetwork>,
    volume: Option<FakeVolume>,
}
#[cfg(feature = "test-hooks")]
#[derive(Deserialize)]
struct FakeBattery {
    percent: u8,
    #[serde(default)]
    charging: bool,
    #[serde(default)]
    plugged: bool,
    minutes_left: Option<u32>,
}
#[cfg(feature = "test-hooks")]
#[derive(Deserialize)]
struct FakeNetwork {
    kind: NetKind,
    online: Online,
    name: Option<String>,
    bars: Option<u8>,
}
#[cfg(feature = "test-hooks")]
#[derive(Deserialize)]
struct FakeVolume {
    percent: u8,
    #[serde(default)]
    muted: bool,
}

/// Legge il file finto; `None` se manca o non è JSON valido (si tiene lo stato di prima).
#[cfg(feature = "test-hooks")]
fn read_fake(path: &str) -> Option<Indicators> {
    let f: Fake = serde_json::from_str(crate::dockstore::strip_bom(&std::fs::read_to_string(path).ok()?)).ok()?;
    Some(Indicators {
        battery: f.battery.map(|b| Battery::new(b.percent, b.charging, b.plugged, b.minutes_left)),
        network: f.network.map(|n| Network::new(n.kind, n.online, n.name, n.bars)),
        volume: f.volume.map(|v| Volume::new(v.percent, v.muted)),
    })
}

fn read_once() -> Option<Indicators> {
    #[cfg(feature = "test-hooks")]
    if let Some(path) = std::env::var_os("EOSOS_FAKE_INDICATORS") {
        return read_fake(&path.to_string_lossy());
    }
    Some(read_real())
}

// ------------------------------------------------------------------ clic

/// Pagina di Impostazioni per un indicatore. La pagina manda solo l'id; l'indirizzo lo sceglie
/// Rust da questa tabella fissa (un id sconosciuto non apre nulla). Pura, testata.
pub fn settings_uri(id: &str) -> Option<&'static str> {
    match id {
        "battery" => Some("ms-settings:batterysaver"),
        "network" => Some("ms-settings:network"),
        "sound" => Some("ms-settings:sound"),
        _ => None,
    }
}

/// Apre la pagina di Impostazioni dell'indicatore `id`. `false` se l'id è sconosciuto o l'apertura fallisce.
pub fn open_settings(id: &str) -> bool {
    use windows::core::{w, HSTRING};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let Some(uri) = settings_uri(id) else { return false };
    let target = HSTRING::from(uri);
    let r = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    r.0 as usize > 32
}

// ------------------------------------------------------------------ servizio

/// Avvia il controllo periodico: a ogni cambio salva lo stato e manda `indicators-changed` a tutte
/// le finestre.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        // COM inizializzato una volta sola per tutta la vita del thread (vedi `read_real`).
        let _mta = MtaScope::new();
        loop {
            // Un panic nella lettura non deve fermare il servizio.
            let read = std::panic::catch_unwind(read_once).ok().flatten();
            if let Some(new) = read {
                let changed = update_state(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()), new.clone());
                if changed {
                    let _ = app.emit("indicators-changed", &new);
                }
            }
            std::thread::sleep(POLL);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(level: i32, wlan: bool, wwan: bool, name: &str, bars: Option<u8>) -> Link {
        Link { level, wlan, wwan, name: Some(name.into()), bars }
    }

    /// La pagina disegna un'icona per ognuno di questi id (`KNOWN_ICONS` in src/lib/indicators.ts):
    /// se Rust ne produce uno nuovo o ne perde uno, questo test e quello della pagina si rompono.
    #[test]
    fn every_icon_rust_can_emit_is_one_the_page_knows() {
        const KNOWN: [&str; 20] = [
            "battery-critical", "battery-low", "battery-half", "battery-high", "battery-full", "battery-charging",
            "battery-plugged", "wifi-1", "wifi-2", "wifi-3", "wifi-alert", "ethernet", "ethernet-alert", "cellular",
            "net-off", "volume-0", "volume-1", "volume-2", "volume-3", "volume-muted",
        ];
        let mut seen = std::collections::BTreeSet::new();
        for p in 0..=100u8 {
            for (c, pl) in [(false, false), (true, true), (false, true), (true, false)] {
                seen.insert(battery_icon(p, c, pl));
            }
            for m in [false, true] {
                seen.insert(volume_icon(p, m));
            }
        }
        for kind in [NetKind::Wifi, NetKind::Ethernet, NetKind::Cellular, NetKind::None] {
            for online in [Online::Internet, Online::Limited, Online::None] {
                for bars in [None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(5)] {
                    seen.insert(network_icon(kind, online, bars));
                }
            }
        }
        let mut expected: Vec<&str> = KNOWN.to_vec();
        expected.sort_unstable();
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn only_known_indicators_open_a_settings_page() {
        assert_eq!(settings_uri("battery"), Some("ms-settings:batterysaver"));
        assert_eq!(settings_uri("network"), Some("ms-settings:network"));
        assert_eq!(settings_uri("sound"), Some("ms-settings:sound"));
        for bad in ["", "volume", "Battery", "ms-settings:privacy", "calc.exe", r"C:\Windows\System32\cmd.exe", "battery "] {
            assert_eq!(settings_uri(bad), None, "{bad:?}");
            assert!(!open_settings(bad));
        }
    }

    #[test]
    fn duration_labels() {
        assert_eq!(duration_label(0), "0 min");
        assert_eq!(duration_label(45), "45 min");
        assert_eq!(duration_label(60), "1 h");
        assert_eq!(duration_label(135), "2 h 15 min");
        assert_eq!(duration_label(600), "10 h");
    }

    #[test]
    fn battery_icon_levels_and_states() {
        assert_eq!(battery_icon(5, false, false), "battery-critical");
        assert_eq!(battery_icon(10, false, false), "battery-critical");
        assert_eq!(battery_icon(11, false, false), "battery-low");
        assert_eq!(battery_icon(30, false, false), "battery-low");
        assert_eq!(battery_icon(31, false, false), "battery-half");
        assert_eq!(battery_icon(60, false, false), "battery-half");
        assert_eq!(battery_icon(85, false, false), "battery-high");
        assert_eq!(battery_icon(86, false, false), "battery-full");
        assert_eq!(battery_icon(100, false, false), "battery-full");
        assert_eq!(battery_icon(5, true, true), "battery-charging", "in carica vince sul livello");
        assert_eq!(battery_icon(100, false, true), "battery-plugged");
    }

    #[test]
    fn battery_texts_always_show_the_percentage() {
        let b = Battery::new(85, false, false, Some(135));
        assert_eq!(b.label, "85%");
        assert_eq!(b.title, "Batteria 85% · 2 h 15 min rimanenti");
        assert_eq!(Battery::new(40, true, true, None).title, "Batteria 40% · in carica");
        assert_eq!(Battery::new(100, false, true, None).title, "Batteria 100% · collegata alla rete");
        assert_eq!(Battery::new(70, false, false, None).title, "Batteria 70%");
        assert_eq!(Battery::new(200, false, false, None).percent, 100, "fuori scala: tetto");
        for p in [0u8, 1, 9, 50, 99, 100] {
            assert_eq!(Battery::new(p, false, false, None).label, format!("{p}%"));
        }
    }

    #[test]
    fn battery_from_windows_status() {
        // Su batteria, 80%, 90 minuti.
        let b = battery_from_status(0, 0, 80, 90 * 60).unwrap();
        assert_eq!((b.percent, b.charging, b.plugged, b.minutes_left), (80, false, false, Some(90)));
        // In carica (flag 8), collegata: nessun tempo residuo.
        let c = battery_from_status(1, 8, 55, u32::MAX).unwrap();
        assert!(c.charging && c.plugged && c.minutes_left.is_none());
        // Collegata e piena (non in carica): il tempo non conta.
        let f = battery_from_status(1, 0, 100, 4000).unwrap();
        assert!(f.plugged && !f.charging && f.minutes_left.is_none());
        // Tempo sconosciuto.
        assert_eq!(battery_from_status(0, 0, 50, u32::MAX).unwrap().minutes_left, None);
        // Nessuna batteria (flag 128), dati sconosciuti (255), percentuale sconosciuta (255).
        assert!(battery_from_status(1, 128, 0, 0).is_none());
        assert!(battery_from_status(1, 255, 100, 0).is_none());
        assert!(battery_from_status(1, 0, 255, 0).is_none());
        // Flag combinati: carica + alta (4 | 8) resta in carica.
        assert!(battery_from_status(1, 8 | 1, 90, u32::MAX).unwrap().charging);
    }

    #[test]
    fn online_levels() {
        assert_eq!(online_from_level(3), Online::Internet);
        assert_eq!(online_from_level(2), Online::Limited);
        assert_eq!(online_from_level(1), Online::Limited);
        assert_eq!(online_from_level(0), Online::None);
        assert_eq!(online_from_level(-1), Online::None);
        assert_eq!(online_from_level(99), Online::None);
    }

    #[test]
    fn network_icons() {
        assert_eq!(network_icon(NetKind::Wifi, Online::Internet, Some(0)), "wifi-1");
        assert_eq!(network_icon(NetKind::Wifi, Online::Internet, Some(1)), "wifi-1");
        assert_eq!(network_icon(NetKind::Wifi, Online::Internet, Some(3)), "wifi-2");
        assert_eq!(network_icon(NetKind::Wifi, Online::Internet, Some(5)), "wifi-3");
        assert_eq!(network_icon(NetKind::Wifi, Online::Internet, None), "wifi-3");
        assert_eq!(network_icon(NetKind::Wifi, Online::Limited, Some(4)), "wifi-alert");
        assert_eq!(network_icon(NetKind::Ethernet, Online::Internet, None), "ethernet");
        assert_eq!(network_icon(NetKind::Ethernet, Online::Limited, None), "ethernet-alert");
        assert_eq!(network_icon(NetKind::Cellular, Online::Internet, None), "cellular");
        assert_eq!(network_icon(NetKind::None, Online::None, None), "net-off");
        assert_eq!(network_icon(NetKind::Wifi, Online::None, Some(5)), "net-off");
    }

    #[test]
    fn network_titles() {
        assert_eq!(Network::new(NetKind::Wifi, Online::Internet, Some("Casa".into()), Some(4)).title, "Wi-Fi: Casa");
        assert_eq!(Network::new(NetKind::Wifi, Online::Limited, Some("Bar".into()), Some(2)).title, "Wi-Fi: Bar · senza Internet");
        assert_eq!(Network::new(NetKind::Ethernet, Online::Internet, Some("Rete 2".into()), None).title, "Cavo di rete (Rete 2)");
        assert_eq!(Network::new(NetKind::Ethernet, Online::Internet, None, None).title, "Cavo di rete");
        assert_eq!(Network::new(NetKind::Wifi, Online::Internet, Some("  ".into()), None).title, "Wi-Fi", "nome vuoto");
        assert_eq!(Network::disconnected().title, "Nessuna connessione");
        assert_eq!(Network::new(NetKind::Wifi, Online::Internet, None, Some(9)).bars, Some(5), "tetto delle barre");
    }

    #[test]
    fn pick_network_prefers_internet_then_cable() {
        assert_eq!(pick_network(&[]).kind, NetKind::None);
        assert_eq!(pick_network(&[link(0, true, false, "x", Some(3))]).kind, NetKind::None, "livello 0 non conta");
        // Internet batte accesso locale, qualunque sia il tipo.
        let n = pick_network(&[link(1, false, false, "Cavo senza Internet", None), link(3, true, false, "Wifi", Some(4))]);
        assert_eq!((n.kind, n.online), (NetKind::Wifi, Online::Internet));
        // A pari livello: cavo prima del Wi-Fi, Wi-Fi prima della rete mobile.
        let both = pick_network(&[link(3, true, false, "Wifi", Some(5)), link(3, false, false, "Ethernet", None)]);
        assert_eq!(both.kind, NetKind::Ethernet);
        let w = pick_network(&[link(3, false, true, "SIM", None), link(3, true, false, "Wifi", Some(2))]);
        assert_eq!(w.kind, NetKind::Wifi);
        assert_eq!(w.bars, Some(2));
        // Solo accesso locale: collegata ma senza Internet.
        let l = pick_network(&[link(1, true, false, "Ospiti", Some(3))]);
        assert_eq!((l.kind, l.online, l.icon), (NetKind::Wifi, Online::Limited, "wifi-alert"));
        // Le barre non si portano dietro per cavo o rete mobile.
        assert_eq!(pick_network(&[link(3, false, false, "E", Some(4))]).bars, None);
    }

    #[test]
    fn volume_icons_and_titles() {
        assert_eq!(volume_icon(0, false), "volume-0");
        assert_eq!(volume_icon(1, false), "volume-1");
        assert_eq!(volume_icon(33, false), "volume-1");
        assert_eq!(volume_icon(34, false), "volume-2");
        assert_eq!(volume_icon(66, false), "volume-2");
        assert_eq!(volume_icon(67, false), "volume-3");
        assert_eq!(volume_icon(100, false), "volume-3");
        assert_eq!(volume_icon(80, true), "volume-muted", "il silenzio vince sul livello");
        assert_eq!(Volume::new(40, false).title, "Volume 40%");
        assert_eq!(Volume::new(40, true).title, "Audio disattivato");
        assert_eq!(Volume::new(250, false).percent, 100);
    }

    #[test]
    fn volume_from_scalar_is_robust() {
        assert_eq!(volume_from_scalar(0.0, false).percent, 0);
        assert_eq!(volume_from_scalar(0.5, false).percent, 50);
        assert_eq!(volume_from_scalar(0.495, false).percent, 50, "arrotonda");
        assert_eq!(volume_from_scalar(1.0, false).percent, 100);
        assert_eq!(volume_from_scalar(1.7, false).percent, 100);
        assert_eq!(volume_from_scalar(-0.3, false).percent, 0);
        assert_eq!(volume_from_scalar(f32::NAN, false).percent, 0);
        assert_eq!(volume_from_scalar(f32::INFINITY, true).percent, 0);
        assert!(volume_from_scalar(0.3, true).muted);
    }

    #[test]
    fn update_state_reports_only_real_changes() {
        let mut slot: Option<Indicators> = None;
        let a = Indicators { volume: Some(Volume::new(30, false)), ..Default::default() };
        assert!(update_state(&mut slot, a.clone()), "il primo stato e' un cambio");
        assert!(!update_state(&mut slot, a.clone()), "identico: nessun evento");
        let b = Indicators { volume: Some(Volume::new(31, false)), ..Default::default() };
        assert!(update_state(&mut slot, b.clone()));
        assert_eq!(slot, Some(b));
        // Anche la scomparsa di un indicatore (cuffie scollegate, batteria rimossa) e' un cambio.
        assert!(update_state(&mut slot, Indicators::default()));
    }

    #[test]
    fn serialized_state_is_flat_for_the_page() {
        let st = Indicators {
            battery: Some(Battery::new(85, true, true, None)),
            network: Some(Network::new(NetKind::Wifi, Online::Internet, Some("Casa".into()), Some(4))),
            volume: Some(Volume::new(40, true)),
        };
        let v: serde_json::Value = serde_json::to_value(&st).unwrap();
        assert_eq!(v["battery"]["label"], "85%");
        assert_eq!(v["battery"]["icon"], "battery-charging");
        assert_eq!(v["network"]["kind"], "wifi");
        assert_eq!(v["network"]["online"], "internet");
        assert_eq!(v["network"]["icon"], "wifi-3");
        assert_eq!(v["volume"]["icon"], "volume-muted");
        let none = serde_json::to_value(Indicators::default()).unwrap();
        assert!(none["battery"].is_null() && none["network"].is_null() && none["volume"].is_null());
    }

    /// Lettura vera, in sola lettura: stampa lo stato del PC e controlla solo la coerenza. Un solo
    /// thread tiene COM inizializzato per tutto il test, come fa il servizio (vedi `read_real`).
    #[test]
    fn real_read_smoke() {
        let (st, first, avg) = std::thread::spawn(|| {
            let _mta = MtaScope::new();
            let t = std::time::Instant::now();
            let st = read_real();
            let first = t.elapsed();
            // Costo a regime: 20 letture di fila (il controllo vero ne fa una ogni 1,5 s).
            let t = std::time::Instant::now();
            for _ in 0..20 {
                let _ = read_real();
            }
            (st, first, t.elapsed() / 20)
        })
        .join()
        .unwrap();
        println!("batteria: {:?}", st.battery.as_ref().map(|b| (&b.label, b.icon, &b.title)));
        println!("rete: {:?}", st.network.as_ref().map(|n| (n.icon, &n.title)));
        println!("volume: {:?}", st.volume.as_ref().map(|v| (v.percent, v.muted, v.icon)));
        println!("prima lettura {first:?}, media a regime {avg:?}");
        assert!(avg < Duration::from_millis(50), "lettura troppo costosa: {avg:?}");
        if let Some(b) = &st.battery {
            assert!(b.percent <= 100 && b.label.ends_with('%'));
        }
        let n = st.network.expect("la rete e' sempre presente (anche scollegata)");
        assert!(!n.title.is_empty());
        if let Some(v) = &st.volume {
            assert!(v.percent <= 100);
        }
    }
}
