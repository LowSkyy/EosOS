//! Centro notifiche, fase N3: la lettura vera delle notifiche di Windows (`UserNotificationListener`) e la rimozione di
//! una singola notifica. Tocca testi privati dell'utente: si usa SOLO se l'utente ha acceso l'interruttore «Mostra le
//! notifiche» (`notif.read`, spento di default), i testi restano in memoria (mai nei log né su disco) e questa
//! funzione non scrive niente da nessuna parte. La parte di logica (gruppi, ordine, limiti) è in `notifmodel.rs`.

use windows::UI::Notifications::Management::UserNotificationListener;
use windows::UI::Notifications::NotificationKinds;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

use crate::notifmodel::RawNotif;

/// Differenza fra il 1601 (origine dei `DateTime` di Windows) e il 1970, in millisecondi.
const EPOCH_DIFF_MS: i64 = 11_644_473_600_000;

fn in_mta<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    std::thread::spawn(move || unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let r = f();
        if init {
            CoUninitialize();
        }
        r
    })
    .join()
    .ok()
}

/// Tutte le notifiche toast adesso presenti. `Err` con un messaggio breve (senza testi dell'utente) se non si riesce.
pub fn read_all() -> Result<Vec<RawNotif>, String> {
    in_mta(|| -> Result<Vec<RawNotif>, String> {
        let one = || -> windows::core::Result<Vec<RawNotif>> {
            let listener = UserNotificationListener::Current()?;
            let list = listener.GetNotificationsAsync(NotificationKinds::Toast)?.join()?;
            let mut out = Vec::new();
            for n in list {
                let app = n.AppInfo()?;
                let mut lines = Vec::new();
                if let Ok(visual) = n.Notification().and_then(|x| x.Visual()) {
                    for binding in visual.Bindings()? {
                        for t in binding.GetTextElements()? {
                            lines.push(t.Text()?.to_string());
                        }
                    }
                }
                out.push(RawNotif {
                    id: n.Id()?,
                    app_id: app.AppUserModelId().map(|s| s.to_string()).unwrap_or_default(),
                    app_name: app.DisplayInfo().and_then(|d| d.DisplayName()).map(|s| s.to_string()).unwrap_or_default(),
                    lines,
                    time_ms: n.CreationTime()?.UniversalTime / 10_000 - EPOCH_DIFF_MS,
                });
            }
            Ok(out)
        };
        one().map_err(|e| format!("Windows non ha dato le notifiche (codice {:#010x}).", e.code().0))
    })
    .unwrap_or_else(|| Err("Lettura interrotta.".into()))
}

/// Toglie la notifica `id` dal Centro notifiche di Windows. Chi chiama ha già controllato che l'id sia fra quelli
/// mostrati.
pub fn remove(id: u32) -> Result<(), String> {
    in_mta(move || -> Result<(), String> {
        (|| -> windows::core::Result<()> { UserNotificationListener::Current()?.RemoveNotification(id) })()
            .map_err(|e| format!("Non riesco a togliere la notifica (codice {:#010x}).", e.code().0))
    })
    .unwrap_or_else(|| Err("Operazione interrotta.".into()))
}

/// Apre l'app che ha mandato una notifica, come farebbe un clic sulla notifica: `shell:AppsFolder\<identificatore>`
/// (funziona per le app dello Store e per quelle che Windows conosce con quell'identificatore). `false` se Windows non
/// la conosce. Nelle build di prova `EOSOS_TEST_NO_OPEN` non apre niente e ricorda l'identificatore (`last_opened`).
pub fn open_app(app_id: &str) -> bool {
    if !crate::notifmodel::valid_app_id(app_id) {
        return false;
    }
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("EOSOS_TEST_NO_OPEN").is_some() {
        *LAST_OPENED.lock().unwrap_or_else(|e| e.into_inner()) = Some(app_id.to_string());
        return true;
    }
    use windows::core::{w, HSTRING};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let target = HSTRING::from(format!("shell:AppsFolder\\{}", app_id.trim()));
    let r = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    r.0 as usize > 32
}

#[cfg(feature = "test-hooks")]
static LAST_OPENED: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// L'ultimo identificatore «aperto» in una build di prova con `EOSOS_TEST_NO_OPEN`.
#[cfg(feature = "test-hooks")]
pub fn last_opened() -> Option<String> {
    LAST_OPENED.lock().unwrap_or_else(|e| e.into_inner()).clone()
}
