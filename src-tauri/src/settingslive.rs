//! Impostazioni di EosOS, fase S1: il collegamento ai depositi veri. Ogni chiave dell'elenco (`settingsreg`) passa
//! per le stesse funzioni che oggi usano i menu contestuali e le voci del Launcher, così la finestra delle
//! impostazioni e quei menu restano sempre d'accordo e una modifica ha subito effetto dove ne può avere.
//! Non ha test propri (tocca file e finestre): la logica è in `settingsreg`, la verifica è dal vivo.

use tauri::{AppHandle, Emitter};

use crate::settingsreg::{Backend, SettingsError, Value};

pub struct Live {
    app: AppHandle,
}

impl Live {
    pub fn new(app: AppHandle) -> Self {
        Live { app }
    }
}

fn b(v: bool) -> Option<Value> {
    Some(Value::Bool(v))
}

fn want_bool(v: &Value) -> Result<bool, SettingsError> {
    match v {
        Value::Bool(x) => Ok(*x),
        _ => Err(SettingsError("Serve un sì o un no.".into())),
    }
}

fn want_text(v: &Value) -> Result<&str, SettingsError> {
    match v {
        Value::Text(x) => Ok(x),
        _ => Err(SettingsError("Serve un testo.".into())),
    }
}

fn want_list(v: &Value) -> Result<Vec<String>, SettingsError> {
    match v {
        Value::List(x) => Ok(x.clone()),
        _ => Err(SettingsError("Serve un elenco.".into())),
    }
}

/// La scorciatoia del Launcher come testo canonico (la predefinita se non è stata scelta).
fn launcher_hotkey_text() -> String {
    crate::launcherstore::settings()
        .hotkey
        .and_then(|t| crate::launcherwin::parse_hotkey(&t))
        .map_or_else(|| crate::launcherwin::format_hotkey(crate::launcherwin::DEFAULT_HOTKEY), crate::launcherwin::format_hotkey)
}

impl Backend for Live {
    fn get(&self, key: &str) -> Option<Value> {
        let ind = crate::indicatorstore::load;
        let mc = crate::mcstore::load;
        match key {
            "general.autostart" => b(crate::autostart::is_enabled(&crate::autostart::real_keys(), crate::autostart::NAME)),
            "general.language" => Some(Value::Text("it".into())),
            "notif.read" => b(mc().notif_read),
            "oggi.weather" => b(mc().weather_on),
            "oggi.city" => Some(Value::Text(mc().weather_city.unwrap_or_default())),
            "general.update_auto" => b(mc().update_auto),
            "notif.dnd" => b(mc().notif_dnd),
            "calendar.week_start" => Some(Value::Text(if mc().calendar_sunday { "sun" } else { "mon" }.to_string())),
            "dock.autohide" => b(crate::dockwin::autohide()),
            "dock.reserve" => b(crate::dockwin::reserve_enabled()),
            "dock.size" => Some(Value::Text(match mc().dock_size.as_str() { "" => "medium".to_string(), s => s.to_string() })),
            "dock.blur" => b(crate::dockwin::blur_enabled()),
            "dock.stacks" => Some(Value::List(crate::stacks::folders())),
            "dock.pinned" => Some(Value::List(crate::dockstore::load().into_iter().map(|p| p.name).collect())),
            "dock.glass" => Some(Value::Text(crate::dockstore::nearest_glass_id(crate::dockstore::settings().glass).to_string())),
            "bar.battery" => b(ind().battery),
            "bar.percent" => b(ind().percent),
            "bar.network" => b(ind().network),
            "bar.volume" => b(ind().volume),
            "menu.enabled" => b(!mc().menu_off),
            "menu.excluded" => Some(Value::List(mc().menu_excluded)),
            "general.hide_taskbar" => b(!mc().keep_taskbar),
            "menu.show_name" => b(!mc().menu_hide_name),
            "menu.show_window" => b(mc().menu_show_window),
            "launcher.hotkey" => Some(Value::Text(launcher_hotkey_text())),
            "launcher.clipboard" => b(crate::launcherstore::settings().clipboard == Some(true)),
            "launcher.folders" => Some(Value::List(crate::launcherstore::settings().search_folders)),
            "mc.hotkey" => Some(Value::Text(mc().hotkey.unwrap_or_else(|| crate::mcwin::DEFAULT_HOTKEY.to_string()))),
            "mc.corner" => b(mc().hot_corner),
            "mc.alttab" => b(!mc().windows_alt_tab),
            "mc.wintab" => b(!mc().windows_win_tab),
            _ => None,
        }
    }

    fn action(&mut self, key: &str) -> Result<(), SettingsError> {
        match key {
            "general.report" => {
                if crate::vdreport::write_and_open() {
                    Ok(())
                } else {
                    Err(SettingsError("Non riesco a scrivere il rapporto.".into()))
                }
            }
            "windows.home" => {
                if crate::winsettings::open("home") {
                    Ok(())
                } else {
                    Err(SettingsError("Non riesco ad aprire le Impostazioni di Windows.".into()))
                }
            }
            _ => Err(SettingsError("Pulsante sconosciuto.".into())),
        }
    }

    fn set(&mut self, key: &str, value: &Value) -> Result<(), SettingsError> {
        use crate::indicatorstore::Toggle;
        let indicator = |t: Toggle, cur: bool, want: bool| {
            if cur != want {
                crate::indicatorstore::toggle(t);
                crate::indicators::refresh_now();
            }
        };
        match key {
            "general.autostart" => {
                let keys = crate::autostart::real_keys();
                if want_bool(value)? {
                    let exe = std::env::current_exe().map_err(|_| SettingsError("Non trovo il file di EosOS.".into()))?;
                    crate::autostart::enable(&keys, crate::autostart::NAME, &exe).map_err(SettingsError)?;
                } else {
                    crate::autostart::disable(&keys, crate::autostart::NAME).map_err(SettingsError)?;
                }
            }
            "notif.dnd" => {
                crate::mcstore::set_notif_dnd(want_bool(value)?);
            }
            "general.update_auto" => {
                crate::mcstore::set_update_auto(want_bool(value)?);
            }
            "oggi.weather" => {
                crate::mcstore::set_weather_on(want_bool(value)?);
            }
            "oggi.city" => {
                let c = want_text(value)?;
                crate::mcstore::set_weather_city((!c.is_empty()).then(|| c.to_string()));
            }
            "notif.read" => {
                crate::mcstore::set_notif_read(want_bool(value)?);
            }
            "calendar.week_start" => {
                crate::mcstore::set_calendar_sunday(want_text(value)? == "sun");
            }
            "general.language" => {
                if want_text(value)? != "it" {
                    return Err(SettingsError("Per ora c'è solo l'italiano.".into()));
                }
            }
            "dock.autohide" => {
                if crate::dockwin::autohide() != want_bool(value)? {
                    crate::dockwin::toggle_autohide();
                }
                self.dock_changed();
            }
            "dock.reserve" => {
                if crate::dockwin::reserve_enabled() != want_bool(value)? {
                    crate::dockwin::toggle_reserve();
                }
                self.dock_changed();
            }
            "dock.blur" => {
                if crate::dockwin::blur_enabled() != want_bool(value)? {
                    crate::dockwin::toggle_blur();
                }
                self.dock_changed();
            }
            "dock.size" => {
                let id = want_text(value)?;
                if !crate::dockstore::SIZES.iter().any(|s| s.0 == id) {
                    return Err(SettingsError("Dimensione non ammessa.".into()));
                }
                crate::mcstore::set_dock_size(id);
                self.dock_changed();
            }
            "dock.stacks" => {
                crate::mcstore::set_dock_stacks(Some(want_list(value)?));
                self.dock_changed();
            }
            "dock.pinned" => {
                let cur = crate::dockstore::load();
                let names: Vec<String> = cur.iter().map(|p| p.name.clone()).collect();
                let keep = crate::settingsreg::plan_pins(&names, &want_list(value)?)?;
                // Prima si tolgono quelle non volute, poi si mettono nell'ordine voluto.
                for (i, p) in cur.iter().enumerate() {
                    if !keep.contains(&i) {
                        crate::dockstore::unpin(&p.path);
                    }
                }
                for (to, i) in keep.iter().enumerate() {
                    crate::dockstore::move_to(&cur[*i].path, None, to);
                }
                self.dock_changed();
            }
            "dock.glass" => {
                let v = crate::dockstore::glass_value(want_text(value)?).ok_or_else(|| SettingsError("Livello di vetro sconosciuto.".into()))?;
                crate::dockstore::set_glass(v);
                self.dock_changed();
            }
            "bar.battery" => indicator(Toggle::Battery, crate::indicatorstore::load().battery, want_bool(value)?),
            "bar.percent" => indicator(Toggle::Percent, crate::indicatorstore::load().percent, want_bool(value)?),
            "bar.network" => indicator(Toggle::Network, crate::indicatorstore::load().network, want_bool(value)?),
            "bar.volume" => indicator(Toggle::Volume, crate::indicatorstore::load().volume, want_bool(value)?),
            "menu.enabled" => crate::menubar::set_menu_off(!want_bool(value)?),
            "menu.excluded" => crate::menubar::set_excluded(want_list(value)?),
            "general.hide_taskbar" => {
                crate::mcstore::set_hide_taskbar(want_bool(value)?);
            }
            "menu.show_name" => crate::menubar::set_view(Some(want_bool(value)?), None),
            "menu.show_window" => crate::menubar::set_view(None, Some(want_bool(value)?)),
            "launcher.hotkey" => {
                let h = crate::launcherwin::parse_hotkey(want_text(value)?).ok_or_else(|| SettingsError("Scorciatoia non valida.".into()))?;
                // Cambia subito; si salva solo se la nuova combinazione è libera.
                if !crate::launcherwin::change_hotkey(h) {
                    return Err(SettingsError("Questa scorciatoia è già usata da un altro programma.".into()));
                }
                let default = h == crate::launcherwin::DEFAULT_HOTKEY;
                crate::launcherstore::set_hotkey((!default).then(|| crate::launcherwin::format_hotkey(h)));
            }
            "launcher.clipboard" => crate::launcher::set_clipboard(want_bool(value)?),
            "launcher.folders" => {
                let list = want_list(value)?;
                if let Some(missing) = list.iter().find(|p| !std::path::Path::new(p.as_str()).is_dir()) {
                    return Err(SettingsError(format!("La cartella non esiste: {}", missing.chars().take(60).collect::<String>())));
                }
                crate::launcherstore::set_search_folders(list);
            }
            "mc.hotkey" => {
                let text = want_text(value)?.to_string();
                let h = crate::launcherwin::parse_hotkey(&text).ok_or_else(|| SettingsError("Scorciatoia non valida.".into()))?;
                // Cambia subito; si salva solo se la nuova combinazione è libera.
                if !crate::mcwin::change_hotkey(h) {
                    return Err(SettingsError("Questa scorciatoia è già usata da un altro programma (o dal Launcher).".into()));
                }
                let default = text == crate::mcwin::DEFAULT_HOTKEY;
                crate::mcstore::set_hotkey((!default).then_some(text));
            }
            "mc.corner" => crate::mcwin::set_corner(want_bool(value)?),
            "mc.alttab" => crate::mcwin::set_key("alt", want_bool(value)?),
            "mc.wintab" => crate::mcwin::set_key("win", want_bool(value)?),
            _ => return Err(SettingsError("Impostazione sconosciuta.".into())),
        }
        Ok(())
    }
}

impl Live {
    /// Il Dock aggiorna subito vetro e impostazioni, come dopo una scelta dal suo menu.
    fn dock_changed(&self) {
        let _ = self.app.emit("dock-settings-changed", crate::dockstore::ui());
    }
}
