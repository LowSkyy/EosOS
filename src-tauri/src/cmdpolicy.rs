//! Quale finestra può chiamare quale comando. Le pagine (barra, Dock, pannello, Launcher, impostazioni...) sono tutte
//! locali e con una CSP rigida, ma se una venisse compromessa non deve poter arrivare ai comandi delle altre: per
//! esempio dalla barra non si devono poter accendere la lettura delle notifiche o la cronologia degli appunti. La
//! regola sta qui, pura e testata; `guard` la applica all'inizio dei comandi sensibili.

/// Può la finestra `label` chiamare il comando `cmd` (con la chiave `key`, per `settings_set`)? Pura, testata.
pub fn allowed(label: &str, cmd: &str, key: Option<&str>) -> bool {
    match cmd {
        "settings_open" | "settings_pick_folder" | "settings_close" | "settings_action" | "settings_search" => label == "settings",
        "settings_all" => label == "settings" || label == "panel",
        // Il pannello sotto l'orologio può solo accendere/spegnere la lettura delle notifiche e «Non disturbare».
        "settings_set" => label == "settings" || (label == "panel" && matches!(key, Some("notif.read" | "notif.dnd"))),
        c if c.starts_with("notif_") => label == "panel",
        c if c.starts_with("launcher_") => label == "launcher",
        _ => true,
    }
}

/// Applica `allowed`. Nelle build di prova (`test-hooks`) i test usano una finestra qualunque come canale: a meno
/// che `EOSOS_TEST_STRICT` sia impostata, lì tutto è permesso.
pub fn guard(label: &str, cmd: &str, key: Option<&str>) -> Result<(), String> {
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("EOSOS_TEST_STRICT").is_none() {
        return Ok(());
    }
    if allowed(label, cmd, key) {
        Ok(())
    } else {
        Err("Comando non permesso da questa finestra.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_belong_to_the_settings_window() {
        for c in ["settings_open", "settings_pick_folder", "settings_close", "settings_action", "settings_search"] {
            assert!(allowed("settings", c, None), "{c}");
            for other in ["panel", "launcher", "dock", "bar-0", "main", "mc-0"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
        assert!(allowed("settings", "settings_set", Some("dock.pinned")));
        assert!(allowed("settings", "settings_all", None) && allowed("panel", "settings_all", None));
        assert!(!allowed("launcher", "settings_all", None) && !allowed("bar-0", "settings_all", None));
    }

    #[test]
    fn the_panel_can_only_flip_the_two_notification_switches() {
        assert!(allowed("panel", "settings_set", Some("notif.read")));
        assert!(allowed("panel", "settings_set", Some("notif.dnd")));
        for k in ["launcher.clipboard", "dock.pinned", "launcher.folders", "general.autostart", "mc.hotkey"] {
            assert!(!allowed("panel", "settings_set", Some(k)), "{k}");
        }
        assert!(!allowed("panel", "settings_set", None));
        for w in ["launcher", "dock", "bar-0", "bar-1", "main", "mc-0"] {
            assert!(!allowed(w, "settings_set", Some("notif.read")), "{w}");
        }
    }

    #[test]
    fn notifications_and_launcher_commands_have_one_owner() {
        for c in ["notif_list", "notif_dismiss", "notif_open", "notif_clear_app"] {
            assert!(allowed("panel", c, None) && !allowed("launcher", c, None) && !allowed("bar-0", c, None), "{c}");
        }
        for c in ["launcher_search", "launcher_files", "launcher_open_file", "launcher_clip_set", "launcher_clip_use", "launcher_action", "launcher_launch", "launcher_hide"] {
            assert!(allowed("launcher", c, None) && !allowed("panel", c, None) && !allowed("dock", c, None), "{c}");
        }
    }

    #[test]
    fn other_commands_are_not_restricted_here() {
        assert!(allowed("dock", "dock_activate", None) && allowed("bar-0", "panel_toggle", None));
    }
}
