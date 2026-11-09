//! Impostazioni di Windows nella finestra delle Impostazioni di EosOS. Le pagine di Windows non si possono incorporare:
//! EosOS ne mostra l'elenco (raggruppato come in Windows 11, con ricerca) e con un clic apre la pagina vera. L'elenco è una
//! TABELLA FISSA qui: la pagina manda solo l'id di una voce e l'indirizzo `ms-settings:` lo sceglie Rust, quindi non
//! si può far aprire altro. Funzioni pure e testate; l'apertura usa la shell di Windows.

use serde::Serialize;

/// Una pagina delle Impostazioni di Windows.
#[derive(Clone, Copy, Debug)]
pub struct Page {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    /// Una riga che dice cosa c'è (e aiuta la ricerca).
    pub hint: &'static str,
    /// Indirizzo `ms-settings:` della pagina.
    pub uri: &'static str,
}

const fn p(id: &'static str, group: &'static str, label: &'static str, hint: &'static str, uri: &'static str) -> Page {
    Page { id, group, label, hint, uri }
}

/// Le pagine, nell'ordine dei gruppi di Windows 11.
pub const PAGES: &[Page] = &[
    p("home", "Generale", "Pagina iniziale", "Tutte le Impostazioni di Windows", "ms-settings:"),
    p("display", "Sistema", "Schermo", "Luminosità, scala, risoluzione, più monitor, HDR", "ms-settings:display"),
    p("sound", "Sistema", "Audio", "Uscita, ingresso, volume, dispositivi", "ms-settings:sound"),
    p("notifications", "Sistema", "Notifiche", "Quali app possono avvisarti, non disturbare", "ms-settings:notifications"),
    p("focus", "Sistema", "Assistente notifiche", "Non disturbare e sessioni di concentrazione", "ms-settings:quietmomentshome"),
    p("power", "Sistema", "Alimentazione e batteria", "Risparmio energia, sospensione, schermo spento", "ms-settings:powersleep"),
    p("storage", "Sistema", "Archiviazione", "Spazio sul disco, pulizia, Sensore di memoria", "ms-settings:storagesense"),
    p("multitasking", "Sistema", "Multitasking", "Snap delle finestre, desktop virtuali, Alt+Tab", "ms-settings:multitasking"),
    p("clipboard", "Sistema", "Appunti", "Cronologia degli appunti e sincronizzazione", "ms-settings:clipboard"),
    p("about", "Sistema", "Informazioni", "Nome del PC, versione di Windows, specifiche", "ms-settings:about"),
    p("bluetooth", "Bluetooth e dispositivi", "Bluetooth e dispositivi", "Aggiungi e gestisci dispositivi", "ms-settings:bluetooth"),
    p("printers", "Bluetooth e dispositivi", "Stampanti e scanner", "Aggiungi o rimuovi stampanti", "ms-settings:printers"),
    p("mouse", "Bluetooth e dispositivi", "Mouse", "Velocità, pulsanti, scorrimento", "ms-settings:mousetouchpad"),
    p("touchpad", "Bluetooth e dispositivi", "Touchpad", "Gesti, sensibilità, tocco", "ms-settings:devices-touchpad"),
    p("typing", "Bluetooth e dispositivi", "Digitazione", "Correzione automatica e suggerimenti", "ms-settings:typing"),
    p("autoplay", "Bluetooth e dispositivi", "Riproduzione automatica", "Cosa fare quando colleghi un dispositivo", "ms-settings:autoplay"),
    p("network", "Rete e Internet", "Rete e Internet", "Stato della connessione", "ms-settings:network"),
    p("wifi", "Rete e Internet", "Wi-Fi", "Reti wireless disponibili e note", "ms-settings:network-wifi"),
    p("ethernet", "Rete e Internet", "Ethernet", "Connessione via cavo", "ms-settings:network-ethernet"),
    p("vpn", "Rete e Internet", "VPN", "Connessioni VPN", "ms-settings:network-vpn"),
    p("proxy", "Rete e Internet", "Proxy", "Server proxy", "ms-settings:network-proxy"),
    p("airplane", "Rete e Internet", "Modalità aereo", "Spegni Wi-Fi, Bluetooth e rete mobile", "ms-settings:network-airplanemode"),
    p("personalization", "Personalizzazione", "Personalizzazione", "Sfondo, colori, temi, schermata di blocco", "ms-settings:personalization"),
    p("background", "Personalizzazione", "Sfondo", "Immagine, colore o presentazione", "ms-settings:personalization-background"),
    p("colors", "Personalizzazione", "Colori", "Tema chiaro o scuro, colore di risalto", "ms-settings:colors"),
    p("themes", "Personalizzazione", "Temi", "Insiemi di sfondo, colori e suoni", "ms-settings:themes"),
    p("lockscreen", "Personalizzazione", "Schermata di blocco", "Immagine e stato all'accesso", "ms-settings:lockscreen"),
    p("fonts", "Personalizzazione", "Caratteri", "Installa e gestisci i caratteri", "ms-settings:fonts"),
    p("taskbar", "Personalizzazione", "Barra delle applicazioni", "Impostazioni della barra di Windows", "ms-settings:taskbar"),
    p("apps", "App", "App installate", "Disinstalla e modifica le app", "ms-settings:appsfeatures"),
    p("defaultapps", "App", "App predefinite", "Quale app apre quale tipo di file", "ms-settings:defaultapps"),
    p("startupapps", "App", "Avvio automatico delle app", "App che partono con Windows", "ms-settings:startupapps"),
    p("yourinfo", "Account", "Le tue informazioni", "Profilo e foto dell'account", "ms-settings:yourinfo"),
    p("signin", "Account", "Opzioni di accesso", "PIN, password, Windows Hello", "ms-settings:signinoptions"),
    p("datetime", "Data, ora e lingua", "Data e ora", "Fuso orario, ora automatica", "ms-settings:dateandtime"),
    p("language", "Data, ora e lingua", "Lingua e area geografica", "Lingua di Windows, formato, tastiere", "ms-settings:regionlanguage"),
    p("gaming", "Giochi", "Giochi", "Game Bar, modalità gioco, acquisizioni", "ms-settings:gaming-gamebar"),
    p("access", "Accessibilità", "Accessibilità", "Testo, contrasto, Assistente vocale, sottotitoli", "ms-settings:easeofaccess"),
    p("privacy", "Privacy e sicurezza", "Privacy e sicurezza", "Permessi delle app: posizione, camera, microfono", "ms-settings:privacy"),
    p("defender", "Privacy e sicurezza", "Sicurezza di Windows", "Antivirus, firewall, protezione del dispositivo", "ms-settings:windowsdefender"),
    p("update", "Windows Update", "Windows Update", "Cerca e installa gli aggiornamenti", "ms-settings:windowsupdate"),
];

/// La pagina con questo id.
pub fn find(id: &str) -> Option<&'static Page> {
    PAGES.iter().find(|p| p.id == id)
}

/// Una voce dell'elenco per la pagina (senza l'indirizzo: quello resta in Rust).
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub hint: &'static str,
}

/// L'elenco per la pagina.
pub fn entries() -> Vec<Entry> {
    PAGES.iter().map(|p| Entry { id: p.id, group: p.group, label: p.label, hint: p.hint }).collect()
}

/// Apre la pagina `id` nelle Impostazioni di Windows. `false` se l'id non c'è o la shell non riesce.
/// Nelle build di prova (`EOSOS_TEST_NO_OPEN`) registra la richiesta senza aprire niente.
pub fn open(id: &str) -> bool {
    let Some(page) = find(id) else { return false };
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("EOSOS_TEST_NO_OPEN").is_some() {
        *LAST_OPEN.lock().unwrap_or_else(|e| e.into_inner()) = Some(page.uri.to_string());
        return true;
    }
    use windows::core::{w, HSTRING};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let target = HSTRING::from(page.uri);
    let r = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    r.0 as usize > 32
}

#[cfg(feature = "test-hooks")]
static LAST_OPEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// L'ultimo indirizzo «aperto» nelle build di prova.
#[cfg(feature = "test-hooks")]
pub fn last_open() -> Option<String> {
    LAST_OPEN.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_consistent() {
        let mut ids: Vec<_> = PAGES.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PAGES.len(), "id unici");
        for p in PAGES {
            assert!(p.uri.starts_with("ms-settings:"), "{} deve aprire solo le Impostazioni di Windows: {}", p.id, p.uri);
            assert!(!p.uri.contains([' ', '"', '|', '<', '>', '&']), "{}", p.uri);
            assert!(!p.label.is_empty() && !p.group.is_empty() && !p.hint.is_empty(), "{}", p.id);
        }
        assert!(PAGES.len() >= 30);
    }

    #[test]
    fn groups_are_contiguous_so_the_page_can_list_them_in_order() {
        let mut seen: Vec<&str> = Vec::new();
        for p in PAGES {
            if seen.last() != Some(&p.group) {
                assert!(!seen.contains(&p.group), "il gruppo {} e' spezzato", p.group);
                seen.push(p.group);
            }
        }
    }

    #[test]
    fn only_known_ids_open_and_entries_hide_the_addresses() {
        assert!(find("display").is_some() && find("update").is_some());
        for bad in ["", "ms-settings:display", "display ", "../x", "Display"] {
            assert!(find(bad).is_none(), "{bad:?}");
            assert!(!open(bad), "{bad:?}");
        }
        let e = entries();
        assert_eq!(e.len(), PAGES.len());
        let text = serde_json::to_string(&e).unwrap();
        assert!(!text.contains("ms-settings:"), "l'indirizzo non va alla pagina");
    }
}
