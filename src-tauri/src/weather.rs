//! Widget «Oggi» (fase O2): il meteo. **Spento di default**: finché l'utente non lo accende (`oggi.weather`) e non sceglie
//! una città (`oggi.city`) non parte nessuna richiesta di rete. Acceso, ogni 30 minuti al massimo (e solo mentre il pannello
//! si apre) EosOS chiede a Open-Meteo (servizio gratuito, senza account) dove si trova la città e le previsioni di oggi.
//! Alla rete va solo il nome della città (più l'indirizzo IP, come per qualunque richiesta); niente identificativi. Gli
//! indirizzi sono fissi qui: la pagina non può chiedere altro. La risposta si legge con `serde_json` e si scarta se è
//! troppo grande o strana. Il parsing e le descrizioni sono funzioni pure testate; la richiesta usa il client HTTP di
//! Windows (stessa rete e stesso proxy del sistema).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

/// Dopo quanto un meteo già letto si rilegge.
pub const FRESH: Duration = Duration::from_secs(30 * 60);
const TIMEOUT: Duration = Duration::from_secs(8);
const MAX_BODY: usize = 100_000;

const GEOCODE_BASE: &str = "https://geocoding-api.open-meteo.com";
const FORECAST_BASE: &str = "https://api.open-meteo.com";

/// Un luogo trovato.
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

/// Le previsioni di oggi.
#[derive(Clone, Debug, PartialEq)]
pub struct Forecast {
    pub temp: i32,
    pub min: i32,
    pub max: i32,
    pub code: u32,
}

/// Città accettabile: da 2 a 60 caratteri, lettere (anche accentate), cifre, spazi, apostrofo, trattino, punto e virgola.
/// Restituisce il testo ripulito. Pura, testata.
pub fn valid_city(s: &str) -> Option<String> {
    let t: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let n = t.chars().count();
    let ok = (2..=60).contains(&n) && t.chars().all(|c| c.is_alphanumeric() || " '’-.,".contains(c)) && t.chars().any(|c| c.is_alphabetic());
    ok.then_some(t)
}

/// Codifica un testo per un indirizzo (tutto tranne le lettere e cifre ASCII e `-._~`). Pura, testata.
pub fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn geocode_url(base: &str, city: &str) -> String {
    format!("{base}/v1/search?name={}&count=1&language=it&format=json", url_encode(city))
}

pub fn forecast_url(base: &str, p: &Place) -> String {
    format!("{base}/v1/forecast?latitude={:.4}&longitude={:.4}&current=temperature_2m,weather_code&daily=temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=1", p.lat, p.lon)
}

/// Il primo luogo trovato dalla ricerca della città. Pura, testata.
pub fn parse_geocode(json: &str) -> Option<Place> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let r = v.get("results")?.as_array()?.first()?;
    let (lat, lon) = (r.get("latitude")?.as_f64()?, r.get("longitude")?.as_f64()?);
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return None;
    }
    let name = r.get("name")?.as_str()?.chars().filter(|c| !c.is_control()).take(60).collect::<String>();
    (!name.trim().is_empty()).then_some(Place { name, lat, lon })
}

/// Temperatura attuale, minima e massima di oggi, codice WMO. Pura, testata.
pub fn parse_forecast(json: &str) -> Option<Forecast> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let cur = v.get("current")?;
    let temp = cur.get("temperature_2m")?.as_f64()?;
    let code = cur.get("weather_code")?.as_u64()? as u32;
    let d = v.get("daily")?;
    let max = d.get("temperature_2m_max")?.as_array()?.first()?.as_f64()?;
    let min = d.get("temperature_2m_min")?.as_array()?.first()?.as_f64()?;
    let ok = |t: f64| (-90.0..=70.0).contains(&t);
    (ok(temp) && ok(min) && ok(max)).then_some(Forecast { temp: temp.round() as i32, min: min.round() as i32, max: max.round() as i32, code })
}

/// Descrizione e simbolo di un codice meteo WMO (la tabella di Open-Meteo). Pura, testata.
pub fn describe(code: u32) -> (&'static str, &'static str) {
    match code {
        0 => ("Sereno", "☀️"),
        1 => ("Prevalentemente sereno", "🌤️"),
        2 => ("Parzialmente nuvoloso", "⛅"),
        3 => ("Nuvoloso", "☁️"),
        45 | 48 => ("Nebbia", "🌫️"),
        51 | 53 | 55 => ("Pioviggine", "🌦️"),
        56 | 57 => ("Pioggia gelata leggera", "🌧️"),
        61 | 63 | 65 => ("Pioggia", "🌧️"),
        66 | 67 => ("Pioggia gelata", "🌧️"),
        71 | 73 | 75 | 77 => ("Neve", "🌨️"),
        80..=82 => ("Rovesci", "🌦️"),
        85 | 86 => ("Rovesci di neve", "🌨️"),
        95 => ("Temporale", "⛈️"),
        96 | 99 => ("Temporale con grandine", "⛈️"),
        _ => ("Variabile", "🌡️"),
    }
}

/// Cosa mostra la pagina.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Reply {
    /// "off" (spento), "nocity" (acceso ma senza città), "ok" o "error".
    pub state: &'static str,
    pub place: String,
    pub temp: Option<i32>,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub label: String,
    pub glyph: String,
    pub message: Option<String>,
}

impl Reply {
    pub fn plain(state: &'static str, message: Option<String>) -> Reply {
        Reply { state, place: String::new(), temp: None, min: None, max: None, label: String::new(), glyph: String::new(), message }
    }
}

// ------------------------------------------------------------------ rete e cache

static CACHE: Mutex<Option<(String, Instant, Reply)>> = Mutex::new(None);

/// Un indirizzo di base diverso solo nelle build di prova (server locale).
fn bases() -> (String, String) {
    #[cfg(feature = "test-hooks")]
    if let Ok(b) = std::env::var("EOSOS_TEST_WEATHER_BASE") {
        return (b.clone(), b);
    }
    (GEOCODE_BASE.to_string(), FORECAST_BASE.to_string())
}

/// Spezza un indirizzo http(s) in (sicuro, server, porta, percorso con query). Pura, testata.
pub fn split_url(url: &str) -> Option<(bool, String, u16, String)> {
    let (secure, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else {
        (false, url.strip_prefix("http://")?)
    };
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().ok()?),
        None => (authority, if secure { 443 } else { 80 }),
    };
    (!host.is_empty() && !host.contains(['@', ' ']) && port != 0).then(|| (secure, host.to_string(), port, path.to_string()))
}

/// Scarica un indirizzo come testo con WinHTTP (la rete e il proxy di Windows, la stessa verifica dei certificati di
/// PowerShell e di Edge), con un tempo massimo e una dimensione massima. Bloccante: fuori dal thread UI.
///
/// Non si usa `Windows.Web.Http`: sul PC di prova falliva con 0x80072F8F (certificato) verso siti che PowerShell raggiunge.
fn get(url: &str) -> Result<String, String> {
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinHttp::{
        WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetTimeouts,
        WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE, WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
    };
    let (secure, host, port, path) = split_url(url).ok_or_else(|| "Indirizzo non valido.".to_string())?;
    let ms = TIMEOUT.as_millis() as i32;
    unsafe {
        let session = WinHttpOpen(w!("EosOS"), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, None, None, 0);
        if session.is_null() {
            return Err("errore di rete (sessione)".to_string());
        }
        let result = (|| -> Result<String, String> {
            let _ = WinHttpSetTimeouts(session, ms, ms, ms, ms);
            let conn = WinHttpConnect(session, &HSTRING::from(host), port, 0);
            if conn.is_null() {
                return Err("errore di rete (connessione)".to_string());
            }
            let inner = (|| -> Result<String, String> {
                let flags = if secure { WINHTTP_FLAG_SECURE } else { WINHTTP_OPEN_REQUEST_FLAGS(0) };
                let req = WinHttpOpenRequest(conn, w!("GET"), &HSTRING::from(path), None, None, std::ptr::null(), flags);
                if req.is_null() {
                    return Err("errore di rete (richiesta)".to_string());
                }
                let body = (|| -> Result<String, String> {
                    let net = |e: windows::core::Error| format!("errore di rete {:#010X}", e.code().0);
                    WinHttpSendRequest(req, None, None, 0, 0, 0).map_err(net)?;
                    WinHttpReceiveResponse(req, std::ptr::null_mut()).map_err(net)?;
                    let mut status = 0u32;
                    let mut len = std::mem::size_of::<u32>() as u32;
                    WinHttpQueryHeaders(req, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, None, Some(&mut status as *mut u32 as *mut _), &mut len, std::ptr::null_mut()).map_err(net)?;
                    if status != 200 {
                        return Err(format!("risposta {status} del servizio"));
                    }
                    let mut out: Vec<u8> = Vec::new();
                    let mut buf = [0u8; 8192];
                    loop {
                        let mut n = 0u32;
                        WinHttpReadData(req, buf.as_mut_ptr() as *mut _, buf.len() as u32, &mut n).map_err(net)?;
                        if n == 0 {
                            break;
                        }
                        out.extend_from_slice(&buf[..n as usize]);
                        if out.len() > MAX_BODY {
                            return Err("Risposta troppo grande.".to_string());
                        }
                    }
                    String::from_utf8(out).map_err(|_| "Risposta non valida.".to_string())
                })();
                let _ = WinHttpCloseHandle(req);
                body
            })();
            let _ = WinHttpCloseHandle(conn);
            inner
        })();
        let _ = WinHttpCloseHandle(session);
        result
    }
}

fn fetch(city: &str) -> Reply {
    let (gbase, fbase) = bases();
    let place = match get(&geocode_url(&gbase, city)).map(|t| parse_geocode(&t)) {
        Ok(Some(p)) => p,
        Ok(None) => return Reply::plain("error", Some(format!("Non trovo la città «{city}»."))),
        Err(e) => return Reply::plain("error", Some(format!("Meteo non disponibile: {e}"))),
    };
    match get(&forecast_url(&fbase, &place)).map(|t| parse_forecast(&t)) {
        Ok(Some(f)) => {
            let (label, glyph) = describe(f.code);
            Reply { state: "ok", place: place.name, temp: Some(f.temp), min: Some(f.min), max: Some(f.max), label: label.to_string(), glyph: glyph.to_string(), message: None }
        }
        Ok(None) => Reply::plain("error", Some("Risposta del servizio meteo non valida.".to_string())),
        Err(e) => Reply::plain("error", Some(format!("Meteo non disponibile: {e}"))),
    }
}

/// Il meteo per la pagina. Spento o senza città: nessuna richiesta di rete. Altrimenti usa la copia in memoria se è
/// fresca (e per la stessa città), altrimenti scarica. Un errore non si ricorda: al prossimo giro si riprova.
pub fn current(on: bool, city: Option<&str>) -> Reply {
    if !on {
        return Reply::plain("off", None);
    }
    let Some(city) = city.and_then(valid_city) else { return Reply::plain("nocity", None) };
    if let Some((c, at, r)) = CACHE.lock().unwrap_or_else(|e| e.into_inner()).clone()
        && c == city
        && at.elapsed() < FRESH
    {
        return r;
    }
    let r = fetch(&city);
    if r.state == "ok" {
        *CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some((city, Instant::now(), r.clone()));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEO: &str = r#"{"results":[{"id":1,"name":"Roma","latitude":41.89193,"longitude":12.51133,"country":"Italia"}],"generationtime_ms":0.1}"#;
    const FC: &str = r#"{"current":{"time":"2026-11-09T00:15","temperature_2m":10.1,"weather_code":3},"daily":{"time":["2026-11-09"],"temperature_2m_max":[17.3],"temperature_2m_min":[9.8]}}"#;

    #[test]
    fn urls_are_split() {
        assert_eq!(split_url("https://api.open-meteo.com/v1/forecast?x=1"), Some((true, "api.open-meteo.com".into(), 443, "/v1/forecast?x=1".into())));
        assert_eq!(split_url("http://127.0.0.1:19000/v1/search?name=Roma"), Some((false, "127.0.0.1".into(), 19000, "/v1/search?name=Roma".into())));
        assert_eq!(split_url("http://host"), Some((false, "host".into(), 80, "/".into())));
        for bad in ["", "ftp://x/y", "https://", "https://:80/x", "http://a b/x", "http://u@h/x", "http://h:0/x", "http://h:99999/x", "localhost/x"] {
            assert_eq!(split_url(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn city_names_are_checked() {
        assert_eq!(valid_city("  Roma "), Some("Roma".into()));
        assert_eq!(valid_city("San  Giovanni   in Fiore"), Some("San Giovanni in Fiore".into()));
        assert_eq!(valid_city("L'Aquila"), Some("L'Aquila".into()));
        assert_eq!(valid_city("Saint-Étienne"), Some("Saint-Étienne".into()));
        assert_eq!(valid_city("Forlì, IT"), Some("Forlì, IT".into()));
        for bad in ["", " ", "R", "12", "Roma&x=1", "Roma?", "a/b", "<b>", "Roma\u{7}", "http://x.it", &"a".repeat(61)] {
            assert_eq!(valid_city(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn urls_are_encoded() {
        assert_eq!(url_encode("Roma"), "Roma");
        assert_eq!(url_encode("San Remo"), "San%20Remo");
        assert_eq!(url_encode("Forlì"), "Forl%C3%AC");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(geocode_url("https://x", "L'Aquila"), "https://x/v1/search?name=L%27Aquila&count=1&language=it&format=json");
        let u = forecast_url("https://x", &Place { name: "R".into(), lat: 41.89193, lon: 12.51133 });
        assert!(u.starts_with("https://x/v1/forecast?latitude=41.8919&longitude=12.5113&current="));
    }

    #[test]
    fn real_shaped_responses_are_read() {
        assert_eq!(parse_geocode(GEO), Some(Place { name: "Roma".into(), lat: 41.89193, lon: 12.51133 }));
        assert_eq!(parse_forecast(FC), Some(Forecast { temp: 10, min: 10, max: 17, code: 3 }));
    }

    #[test]
    fn strange_responses_are_refused() {
        for bad in ["", "{}", "[]", "non json", r#"{"results":[]}"#, r#"{"results":[{"name":"X","latitude":999,"longitude":0}]}"#, r#"{"results":[{"name":"","latitude":1,"longitude":1}]}"#, r#"{"results":[{"latitude":1,"longitude":1}]}"#] {
            assert_eq!(parse_geocode(bad), None, "{bad}");
        }
        for bad in ["", "{}", r#"{"current":{"temperature_2m":10}}"#, r#"{"current":{"temperature_2m":1000,"weather_code":0},"daily":{"temperature_2m_max":[1],"temperature_2m_min":[1]}}"#, r#"{"current":{"temperature_2m":1,"weather_code":0},"daily":{"temperature_2m_max":[],"temperature_2m_min":[1]}}"#] {
            assert_eq!(parse_forecast(bad), None, "{bad}");
        }
    }

    #[test]
    fn weather_codes_have_descriptions() {
        assert_eq!(describe(0).0, "Sereno");
        assert_eq!(describe(3).0, "Nuvoloso");
        assert_eq!(describe(61).0, "Pioggia");
        assert_eq!(describe(95).0, "Temporale");
        assert_eq!(describe(81).0, "Rovesci");
        assert_eq!(describe(12345).0, "Variabile");
        for c in 0..=99 {
            assert!(!describe(c).0.is_empty() && !describe(c).1.is_empty());
        }
    }

    #[test]
    fn off_or_without_city_makes_no_request() {
        // Se partisse una richiesta, in questo test fallirebbe o durerebbe 8 secondi.
        assert_eq!(current(false, Some("Roma")).state, "off");
        assert_eq!(current(true, None).state, "nocity");
        assert_eq!(current(true, Some("")).state, "nocity");
        assert_eq!(current(true, Some("<script>")).state, "nocity");
    }
}
