//! Centro notifiche, fase N1: la prova di fattibilità, in SOLA LETTURA. Chiede a Windows se questo exe (EosOS così
//! com'è, non impacchettato) può usare `UserNotificationListener`, l'interfaccia che permette di leggere le notifiche
//! delle altre app. Si fanno solo due domande che non cambiano niente e non mostrano finestre: «ho un'identità di
//! pacchetto?» e «che stato di accesso ha il listener?». Non si chiede MAI l'accesso (`RequestAccessAsync`) e non si
//! legge MAI una notifica: serve solo a sapere che cosa dice Windows. Il risultato contiene codici e parole, mai testi
//! dell'utente.

use serde::Serialize;
use windows::UI::Notifications::Management::{UserNotificationListener, UserNotificationListenerAccessStatus};
use windows::UI::Notifications::NotificationKinds;
use windows::Win32::Foundation::{APPMODEL_ERROR_NO_PACKAGE, ERROR_INSUFFICIENT_BUFFER, WIN32_ERROR};
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

/// Cosa ha risposto il listener.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "result", content = "value", rename_all = "snake_case")]
pub enum ListenerResult {
    /// Windows ha dato uno stato di accesso: "allowed", "denied" o "unspecified".
    Status(String),
    /// La chiamata è fallita con questo codice (HRESULT) e questo messaggio.
    Error { code: i32, message: String },
}

/// L'esito della prova.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    /// Se l'accesso è consentito: quante notifiche toast ci sono ADESSO (solo il numero, nessun contenuto).
    pub toast_count: Option<u32>,
    /// Se contarle è fallito: il codice dell'errore.
    pub count_error: Option<i32>,
    /// Il processo ha un'identità di pacchetto (è impacchettato)?
    pub has_package_identity: bool,
    /// Il codice con cui Windows ha risposto alla domanda sull'identità.
    pub package_code: u32,
    pub listener: ListenerResult,
    /// La versione di Windows (build), per sapere se «pacchetto con posizione esterna» esiste (serve la 19041).
    pub build: u32,
    pub verdict: Verdict,
}

/// La conclusione, in una parola.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Il listener risponde ed è consentito: si possono leggere le notifiche.
    Usable,
    /// Il listener risponde ma l'accesso non è ancora concesso (va chiesto o dato dalle Impostazioni di Windows).
    NeedsConsent,
    /// Il listener non è utilizzabile da un'app senza identità di pacchetto.
    NeedsPackageIdentity,
    /// Un errore che non so spiegare.
    Unknown,
}

/// `E_ILLEGAL_METHOD_CALL`, il codice con cui Windows rifiuta le API che richiedono l'identità di pacchetto.
const E_ILLEGAL_METHOD_CALL: i32 = 0x8000_000Eu32 as i32;
/// `HRESULT_FROM_WIN32(APPMODEL_ERROR_NO_PACKAGE)`.
const HR_NO_PACKAGE: i32 = 0x8007_3D54u32 as i32;

/// Dalla risposta del listener (e dal fatto di avere o no un'identità) alla conclusione. Pura, testata.
pub fn classify(listener: &ListenerResult) -> Verdict {
    match listener {
        ListenerResult::Status(s) if s == "allowed" => Verdict::Usable,
        ListenerResult::Status(_) => Verdict::NeedsConsent,
        ListenerResult::Error { code, .. } if *code == E_ILLEGAL_METHOD_CALL || *code == HR_NO_PACKAGE => Verdict::NeedsPackageIdentity,
        // Un errore qualunque, anche in un processo senza identità, non si spaccia per un problema di identità.
        ListenerResult::Error { .. } => Verdict::Unknown,
    }
}

/// Il nome dello stato di accesso come testo fisso. Pura, testata.
pub fn status_name(s: UserNotificationListenerAccessStatus) -> &'static str {
    match s {
        UserNotificationListenerAccessStatus::Allowed => "allowed",
        UserNotificationListenerAccessStatus::Denied => "denied",
        UserNotificationListenerAccessStatus::Unspecified => "unspecified",
        _ => "unknown",
    }
}

fn package_code() -> u32 {
    let mut len = 0u32;
    unsafe { GetCurrentPackageFullName(&mut len, None) }.0
}

fn build_number() -> u32 {
    // Il numero di build dalla chiave di Windows (sola lettura), come per i desktop virtuali.
    crate::vdint::build_info().0
}

/// Quante notifiche toast ci sono adesso, e un eventuale errore. Solo il numero.
fn count_toasts() -> (Option<u32>, Option<i32>) {
    let r = (|| -> windows::core::Result<u32> {
        let l = UserNotificationListener::Current()?;
        let list = l.GetNotificationsAsync(NotificationKinds::Toast)?.join()?;
        list.Size()
    })();
    match r {
        Ok(n) => (Some(n), None),
        Err(e) => (None, Some(e.code().0)),
    }
}

/// Quante notifiche toast di una app PRECISA (per identificatore di app) ci sono adesso. Serve alle prove, che
/// mandano notifiche proprie e contano solo quelle: non si leggono mai titoli né testi, né notifiche di altre app.
#[cfg(feature = "test-hooks")]
pub fn count_for_app(aumid: &str) -> Option<u32> {
    let aumid = aumid.to_string();
    std::thread::spawn(move || unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let r = (|| -> windows::core::Result<u32> {
            let l = UserNotificationListener::Current()?;
            let list = l.GetNotificationsAsync(NotificationKinds::Toast)?.join()?;
            let mut n = 0;
            for i in 0..list.Size()? {
                let id = list.GetAt(i)?.AppInfo()?.AppUserModelId()?.to_string();
                if id.eq_ignore_ascii_case(&aumid) {
                    n += 1;
                }
            }
            Ok(n)
        })();
        if init {
            CoUninitialize();
        }
        r.ok()
    })
    .join()
    .ok()
    .flatten()
}

fn listener_result() -> ListenerResult {
    unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let r = (|| -> windows::core::Result<String> {
            let l = UserNotificationListener::Current()?;
            Ok(status_name(l.GetAccessStatus()?).to_string())
        })();
        if init {
            CoUninitialize();
        }
        match r {
            Ok(s) => ListenerResult::Status(s),
            Err(e) => ListenerResult::Error { code: e.code().0, message: e.message().to_string() },
        }
    }
}

/// Esegue la prova (in un thread a parte con il suo COM).
pub fn probe() -> Probe {
    let listener = std::thread::spawn(listener_result).join().unwrap_or(ListenerResult::Error { code: 0, message: "thread terminato".into() });
    let (toast_count, count_error) = if listener == ListenerResult::Status("allowed".into()) {
        std::thread::spawn(|| unsafe {
            let init = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
            let r = count_toasts();
            if init {
                CoUninitialize();
            }
            r
        })
        .join()
        .unwrap_or((None, None))
    } else {
        (None, None)
    };
    let code = package_code();
    // Con un buffer vuoto Windows risponde «buffer troppo piccolo» se il pacchetto c'è, «nessun pacchetto» se non c'è.
    let has_identity = WIN32_ERROR(code) == ERROR_INSUFFICIENT_BUFFER && WIN32_ERROR(code) != APPMODEL_ERROR_NO_PACKAGE;
    Probe { toast_count, count_error, has_package_identity: has_identity, package_code: code, verdict: classify(&listener), listener, build: build_number() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(code: i32) -> ListenerResult {
        ListenerResult::Error { code, message: String::new() }
    }

    #[test]
    fn an_allowed_listener_is_usable_and_other_states_need_consent() {
        assert_eq!(classify(&ListenerResult::Status("allowed".into())), Verdict::Usable);
        assert_eq!(classify(&ListenerResult::Status("unspecified".into())), Verdict::NeedsConsent);
        assert_eq!(classify(&ListenerResult::Status("denied".into())), Verdict::NeedsConsent);
        assert_eq!(classify(&ListenerResult::Status("unknown".into())), Verdict::NeedsConsent);
    }

    #[test]
    fn the_package_identity_errors_are_recognized() {
        assert_eq!(classify(&err(0x8000_000Eu32 as i32)), Verdict::NeedsPackageIdentity);
        assert_eq!(classify(&err(0x8007_3D54u32 as i32)), Verdict::NeedsPackageIdentity);
    }

    #[test]
    fn an_unexplained_error_is_never_called_an_identity_problem() {
        assert_eq!(classify(&err(0x8007_0005u32 as i32)), Verdict::Unknown, "accesso negato non e' un problema di identita'");
        assert_eq!(classify(&err(1)), Verdict::Unknown);
    }

    #[test]
    fn status_names_are_fixed_words() {
        assert_eq!(status_name(UserNotificationListenerAccessStatus::Allowed), "allowed");
        assert_eq!(status_name(UserNotificationListenerAccessStatus::Denied), "denied");
        assert_eq!(status_name(UserNotificationListenerAccessStatus::Unspecified), "unspecified");
    }

    #[test]
    fn the_result_serializes_without_free_text_of_the_user() {
        let p = Probe { toast_count: None, count_error: Some(5), has_package_identity: false, package_code: 15700, listener: err(5), build: 26300, verdict: Verdict::Unknown };
        let j = serde_json::to_string(&p).unwrap();
        assert!(j.contains("\"verdict\":\"unknown\"") && j.contains("\"result\":\"error\""), "{j}");
    }
}
