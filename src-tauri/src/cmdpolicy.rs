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
        // Dock: aprire app e cambiare le app fissate.
        "dock_activate" | "dock_reorder" | "dock_stacks" | "dock_launchpad" | "dock_badges" => label == "dock" || label.starts_with("dock-"),
        // Contenuto delle pile: lo chiede solo il pannello.
        c if c.starts_with("stack_") => label == "panel",
        // Pannello degli indicatori: volume vero, apertura delle impostazioni di Windows, scelta di una voce di menu di
        // un'altra app.
        "indicator_open" | "volume_set" | "volume_mute" | "menubar_choose" => label == "panel",
        // Mission Control: attivare e chiudere finestre vere, cambiare e modificare i desktop virtuali.
        "mc_activate" | "mc_close_window" | "mc_switch_desktop" | "mc_new_desktop" | "mc_move_window" | "mc_remove_desktop" | "mc_rename_desktop" => label.starts_with("mc-"),
        // Menu EosOS (il «menu mela» a sinistra della barra): lo gestisce il pannello.
        c if c.starts_with("eosmenu_") => label == "panel",
        // Schermata di blocco: solo le sue finestre (lock-0, lock-1...).
        "lock_info" | "lock_wallpaper" | "lock_try_password" | "lock_try_blank" | "lock_try_hello" => crate::lockscreen::is_lock_label(label),
        // App in background (area di notifica): quante sono lo chiede l'icona nella barra in alto; l'elenco e l'apertura sono del pannello.
        "tray_count" => label == "main" || label.starts_with("bar-"),
        "tray_apps" | "tray_open" => label == "panel",
        // Cattura dello schermo: l'avvio dal pannello; il rettangolo scelto lo manda solo la finestra di selezione.
        "capture_run" => label == "panel",
        "capture_region" | "capture_cancel" => label == "capture",
        // Centro di controllo (Wi-Fi, Bluetooth, mantieni sveglio): lo gestisce il pannello.
        c if c.starts_with("control_") => label == "panel",
        // Impostazioni di Windows fatte da EosOS (info, tema, alimentazione, app all'avvio, dischi, schermi): solo la finestra Impostazioni.
        c if c.starts_with("winsys_") => label == "settings",
        // Impostazioni di Windows: l'elenco e l'apertura sono della finestra Impostazioni.
        c if c.starts_with("winsettings_") => label == "settings",
        // Aggiornamenti: li gestisce solo la finestra Impostazioni.
        c if c.starts_with("update_") => label == "settings",
        // Widget Oggi (promemoria e meteo): solo il pannello sotto l'orologio.
        c if c.starts_with("oggi_") => label == "panel",
        c if c.starts_with("notif_") => label == "panel",
        c if c.starts_with("launcher_") => label == "launcher",
        // Finder di EosOS: elenco, apertura e modifica dei file solo dalle sue finestre.
        c if c.starts_with("finder_") => crate::finder::is_label(label),
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
    fn the_today_widget_belongs_to_the_panel() {
        for c in ["oggi_reminders", "oggi_add", "oggi_toggle", "oggi_remove", "oggi_clear_done", "oggi_weather"] {
            assert!(allowed("panel", c, None), "{c}");
            for other in ["settings", "launcher", "dock", "bar-0", "main", "mc-0"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
        assert!(allowed("panel", "settings_set", Some("notif.read")) && !allowed("panel", "settings_set", Some("oggi.city")));
    }

    #[test]
    fn dock_badges_belong_to_the_dock_windows() {
        assert!(allowed("dock", "dock_badges", None) && allowed("dock-1", "dock_badges", None));
        for other in ["panel", "settings", "launcher", "bar-0", "main", "mc-0"] {
            assert!(!allowed(other, "dock_badges", None), "{other}");
        }
    }

    #[test]
    fn screen_capture_commands_have_their_own_windows() {
        assert!(allowed("panel", "capture_run", None));
        for other in ["capture", "settings", "launcher", "dock", "bar-0", "main", "mc-0"] {
            assert!(!allowed(other, "capture_run", None), "{other}");
        }
        for c in ["capture_region", "capture_cancel"] {
            assert!(allowed("capture", c, None), "{c}");
            for other in ["panel", "settings", "launcher", "dock", "bar-0", "main", "mc-0"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
    }

    #[test]
    fn background_apps_are_counted_by_the_bars_and_opened_from_the_panel() {
        assert!(allowed("main", "tray_count", None) && allowed("bar-0", "tray_count", None) && allowed("bar-1", "tray_count", None));
        for c in ["tray_apps", "tray_open"] {
            assert!(allowed("panel", c, None), "{c}");
            for other in ["dock", "dock-1", "settings", "launcher", "bar-0", "main", "mc-0", "capture"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
        for other in ["panel", "settings", "launcher", "dock", "dock-1", "mc-0", "capture"] {
            assert!(!allowed(other, "tray_count", None), "{other}");
        }
    }

    #[test]
    fn windows_system_settings_belong_to_the_settings_window() {
        for c in ["winsys_info", "winsys_drives", "winsys_displays", "winsys_theme", "winsys_theme_set", "winsys_power", "winsys_power_set", "winsys_startup", "winsys_startup_set"] {
            assert!(allowed("settings", c, None), "{c}");
            for other in ["panel", "launcher", "dock", "dock-1", "bar-0", "main", "mc-0", "capture"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
    }

    #[test]
    fn the_control_center_belongs_to_the_panel() {
        for c in ["control_state", "control_set"] {
            assert!(allowed("panel", c, None), "{c}");
            for other in ["settings", "launcher", "dock", "bar-0", "main", "mc-0"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
    }

    #[test]
    fn the_eos_menu_belongs_to_the_panel() {
        for c in ["eosmenu_run", "eosmenu_about"] {
            assert!(allowed("panel", c, None), "{c}");
            for other in ["settings", "launcher", "dock", "bar-0", "main", "mc-0"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
    }

    #[test]
    fn windows_settings_belong_to_the_settings_window() {
        for c in ["winsettings_list", "winsettings_open"] {
            assert!(allowed("settings", c, None), "{c}");
            for other in ["panel", "launcher", "dock", "bar-0", "main", "mc-0"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
        }
    }

    #[test]
    fn updates_belong_to_the_settings_window() {
        for c in ["update_status", "update_check", "update_install"] {
            assert!(allowed("settings", c, None), "{c}");
            for other in ["panel", "launcher", "dock", "bar-0", "main", "mc-0"] {
                assert!(!allowed(other, c, None), "{other} {c}");
            }
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
    fn dock_panel_and_mission_control_actions_have_one_owner() {
        for c in ["stack_list", "stack_open", "stack_open_folder"] {
            assert!(allowed("panel", c, None) && !allowed("dock", c, None) && !allowed("bar-0", c, None) && !allowed("launcher", c, None), "{c}");
        }
        for c in ["dock_activate", "dock_reorder", "dock_stacks", "dock_launchpad"] {
            // Il Dock di un monitor secondario (etichetta «dock-...») vale come quello del primario.
            assert!(allowed("dock-DISPLAY2", c, None) && !allowed("bar-DISPLAY2", c, None), "{c}");
            assert!(allowed("dock", c, None) && !allowed("bar-0", c, None) && !allowed("panel", c, None) && !allowed("launcher", c, None), "{c}");
        }
        for c in ["indicator_open", "volume_set", "volume_mute", "menubar_choose"] {
            assert!(allowed("panel", c, None) && !allowed("bar-0", c, None) && !allowed("dock", c, None) && !allowed("mc-0", c, None), "{c}");
        }
        for c in ["mc_activate", "mc_close_window", "mc_switch_desktop", "mc_new_desktop", "mc_move_window", "mc_remove_desktop", "mc_rename_desktop"] {
            assert!(allowed("mc-0", c, None) && allowed("mc-1", c, None) && !allowed("bar-0", c, None) && !allowed("dock", c, None) && !allowed("panel", c, None), "{c}");
        }
    }

    #[test]
    fn read_only_and_opening_commands_are_not_restricted_here() {
        assert!(allowed("bar-0", "panel_toggle", None) && allowed("dock", "dock_items", None) && allowed("bar-0", "indicators_get", None));
    }
    #[test]
    fn the_lock_commands_belong_to_the_lock_windows_only() {
        for c in ["lock_info", "lock_wallpaper", "lock_try_password", "lock_try_blank", "lock_try_hello"] {
            for ok in ["lock-0", "lock-1", "lock-12"] {
                assert!(allowed(ok, c, None), "{ok} {c}");
            }
            for other in ["main", "bar-0", "dock", "dock-1", "panel", "launcher", "settings", "mc-0", "capture", "lock", "lock-", "lock-x", ""] {
                assert!(!allowed(other, c, None), "{other:?} {c}");
            }
        }
    }
}
