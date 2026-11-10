# Graph Report - EosOS  (2026-11-10)

## Corpus Check
- 254 files · ~285,937 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 4439 nodes · 9468 edges · 291 communities (200 shown, 91 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 661 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `47936ad0`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- EosOS: Piano di lavoro
- CLAUDE.md
- compilerOptions
- tauri.conf.json
- panelwin.rs
- compilerOptions
- default.json
- watchdog.rs
- backdrop.rs
- eosos
- taskbar.rs
- test-taskbar.ps1
- test-workarea.ps1
- appbar.rs
- foreground.rs
- test-multimonitor.ps1
- dockact.rs
- package.json
- menubar.rs
- tokens.test.ts
- launcherfiles.rs
- glass.rs
- HWND
- EosOS
- Checklist di test manuali
- dock.rs
- Read
- launchercalc.rs
- mchook.rs
- dockgeom.rs
- HWND
- lib-dpi.ps1
- guard!
- launcheract.rs
- dockwin.rs
- weather.rs
- indicators.rs
- launcher.ts
- Dock.tsx
- launcherrank.rs
- launcherwin.rs
- indicatorstore.rs
- Window
- Pinned
- winsys.rs
- launcherclip.rs
- menuread.rs
- mc.rs
- NetKind
- vdint.rs
- test-mc-vdesk.ps1
- launcher.rs
- autostart.rs
- capture.rs
- test-launcher.ps1
- tests.rs
- indicators.ts
- dockkeys.rs
- settingswin.rs
- vdesk.rs
- test-panel.ps1
- test-mc-overlay.ps1
- test-settings-s5.ps1
- folderpick.rs
- test-indicators.ps1
- mckeys.rs
- test-indicators-settings.ps1
- test-mc-desktops.ps1
- test-settings-s3.ps1
- test-menudrop.ps1
- mctrigger.rs
- read_volume
- Mc.tsx
- settingsreg.rs
- test-vdesk-internal.ps1
- Settings.tsx
- test-menuinvoke.ps1
- test-notif-dnd.ps1
- start
- test-mc-trigger.ps1
- Value
- notifprobe.rs
- test-mc-windows.ps1
- launcherstore.rs
- Readability
- it
- serialize
- menubar.ts
- test-mc-keys.ps1
- calendar.ts
- open
- trayapps.rs
- stacks.rs
- Mutex
- same_app
- control.rs
- test-menubar.ps1
- launchpadfilter.rs
- test-menusettings.ps1
- notifread.rs
- duration
- notifmodel.rs
- tick
- ensure
- Menu globale per app — specifica
- Layout
- KindOut
- test-settingswin.ps1
- test-winsnap.ps1
- test-eosmenu.ps1
- super
- lib.rs
- Mission Control — specifica
- measure-idle.ps1
- test-menu-view.ps1
- Channel
- chromeguard.rs
- test-update.ps1
- test-calendar.ps1
- compose
- test-notif-panel.ps1
- test-capture.ps1
- show
- weather-mock-server.py
- test-notif-probe.ps1
- Hotkey
- test-mc-dpi.ps1
- rank
- set_value
- test-settings.ps1
- @tauri-apps/api
- mcstore.rs
- Aggiornamenti automatici
- test-stacks.ps1
- test-settings-s4.ps1
- test-oggi.ps1
- cmdpolicy.rs
- Kind
- test-launchpad.ps1
- Indicators
- MenuAction
- test-settings-s6.ps1
- test-launcher-dpi.ps1
- InvokeErr
- test-dock-mixed-dpi.ps1
- Cmd
- Indicatori di sistema — specifica
- test-calendar-n6.ps1
- test-indicators-dpi.ps1
- HWND
- test-dock-scales.ps1
- ControlPanel.tsx
- test-menu-dpi.ps1
- test-dock-multi.ps1
- test-calendar-dpi.ps1
- react
- notif_list
- test-stacks-dpi.ps1
- test-winsettings.ps1
- test-notif-model.ps1
- sandbox-install.ps1
- Prove a mano (quelle che l'assistente non può fare)
- Passata finale (8 nov 2026)
- menumodel.rs
- test-cmd-policy.ps1
- pathbuf
- stacks.ts
- Get-Monitors
- KindOut
- sandbox-update.ps1
- Dock — specifica
- Widget «Oggi»
- test-bar-glass.ps1
- Cattura dello schermo
- Rifiniture dopo le prove a mano (9 nov 2026)
- act
- run_menu_action
- best_bar
- ui
- set_low
- Outcome
- test-dock-keys.ps1
- test-dock-badges.ps1
- test-control.ps1
- Cmd
- test-dock-remove.ps1
- .new
- settings_all
- update_reminders
- AtomicU64
- Msg
- measure-open.ps1
- decide
- launcher_files
- test-dock-novita.ps1
- mcwin.rs
- winsnap.rs
- control_state
- Gestione delle finestre (snap)
- winsys_drives
- Toggle
- winsys_power
- pathbuf
- validate
- AppHandle
- run_search
- stack_list
- Badge delle notifiche sul Dock
- battery_icon
- tray_apps
- Centro di controllo
- Cmd
- test-winsys.ps1
- Correzioni e novità del 10 novembre 2026 (dopo le prove a mano)
- downscale
- winsys_startup
- Inflight
- start
- remember
- Pile nel Dock
- winsys_theme
- Held
- State
- encode
- reconcile_one
- MtaScope
- Source

## God Nodes (most connected - your core abstractions)
1. `Window` - 86 edges
2. `guard!` - 62 edges
3. `Settings` - 38 edges
4. `Rect` - 34 edges
5. `update()` - 32 edges
6. `@tauri-apps/api` - 29 edges
7. `react` - 28 edges
8. `Dock()` - 27 edges
9. `Pinned` - 25 edges
10. `st()` - 25 edges

## Surprising Connections (you probably didn't know these)
- `Consumi a riposo con i due Dock (9 nov 2026, dopo le fasi nuove)` --references--> `nextPoll()`  [INFERRED]
  docs/RIFINITURE.md → src/lib/backdrop.ts
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `G3 — La barra: FATTA (8 nov 2026)` --references--> `acceptRow()`  [INFERRED]
  docs/MENU-GLOBALE.md → src/lib/menubar.ts
- `N4 — L'elenco nel pannello: FATTO (8 nov 2026)` --references--> `isLatest()`  [INFERRED]
  docs/NOTIFICHE.md → src/lib/notifications.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (291 total, 91 thin omitted)

### Community 0 - "EosOS: Piano di lavoro"
Cohesion: 0.25
Nodes (7): Dettagli sul ripristino (P4), EosOS: Piano di lavoro, LEGENDA dei passaggi, Principi di sicurezza (valgono per tutti i passaggi), Roadmap (FUORI SCOPE per ora), Stato finale (7 nov 2026), Traguardo della sessione

### Community 4 - "compilerOptions"
Cohesion: 0.10
Nodes (19): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+11 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.08
Nodes (25): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+17 more)

### Community 6 - "panelwin.rs"
Cohesion: 0.07
Nodes (33): a_panel_above_the_icon_keeps_its_bottom_when_it_grows(), anchor(), anchor_to_physical(), busy_for(), BUSY_UNTIL_MS, clamp_height(), EDGE, EXCLUDED (+25 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.08
Nodes (12): CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name(), READY_TIMEOUT (+4 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.07
Nodes (11): BAR_DEEP_INTERVAL, BAR_INTERVAL, DEEP_IDLE_INTERVAL, DEEP_QUIET_AFTER, FACTOR, Frame, FRAMES, HEADER_LEN (+3 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.10
Nodes (21): appbar_state(), autohide_marker(), collect(), Ctx, detect_restart(), enable_autohide(), find(), hide() (+13 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (47): APP_HANDLE, APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_index(), bar_rect() (+39 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (30): ActiveApp, APP, app_from_hwnd(), cached_description(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop() (+22 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.13
Nodes (16): activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id() (+8 more)

### Community 24 - "package.json"
Cohesion: 0.08
Nodes (31): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+23 more)

### Community 25 - "menubar.rs"
Cohesion: 0.10
Nodes (29): APP, Cache, CACHE, cache_fresh(), current(), EVENT_NAME, EXCLUDED, LAST (+21 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.08
Nodes (18): bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), downloads_folder(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, KNOWN, KNOWN_CAP (+10 more)

### Community 28 - "glass.rs"
Cohesion: 0.12
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 29 - "HWND"
Cohesion: 0.24
Nodes (10): collect_own(), reload(), with_ui_visible(), guard_proc(), read_current(), wndproc(), handle(), mouse_proc() (+2 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.04
Nodes (36): a_shared_list_is_reused_only_while_recent_and_unchanged(), ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), cached_shell_name() (+28 more)

### Community 38 - "Read"
Cohesion: 0.22
Nodes (10): decide(), Decision, BackupAndSeed, KeepLast, Seed, Use, Read, Failed (+2 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.06
Nodes (37): ENABLED, C_CFG, c_cfg(), C_STATE, cfg(), CFG, CHILD_IN, child_lines_round_trip_and_garbage_is_ignored() (+29 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.09
Nodes (35): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), capture_adds_context_and_stays_on_the_monitor(), capture_margin() (+27 more)

### Community 42 - "HWND"
Cohesion: 0.19
Nodes (16): aumid_from_store(), aumid_path(), collect_window(), identity(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized() (+8 more)

### Community 43 - "lib-dpi.ps1"
Cohesion: 0.26
Nodes (11): Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale(), Check() (+3 more)

### Community 44 - "guard!"
Cohesion: 0.07
Nodes (26): capture_run(), control_set(), dock_activate(), dock_badges(), dock_reorder(), dock_stacks(), dock_tray_count(), indicator_open() (+18 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.10
Nodes (15): STARTED, CONFIRM_WINDOW, EOS_SETTINGS, MC_CORNER, MC_KEYS, MC_OPEN, MENU_ACT, names() (+7 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.07
Nodes (22): all_screens(), all_screens_wanted(), autohide(), AUTOHIDE, BLUR, CHROME_STYLES, dock_index(), docks_are_ordered_primary_first() (+14 more)

### Community 47 - "weather.rs"
Cohesion: 0.12
Nodes (29): bases(), city_names_are_checked(), current(), describe(), FC, fetch(), Forecast, FORECAST_BASE (+21 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (16): duration_label(), duration_labels(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume (+8 more)

### Community 49 - "launcher.ts"
Cohesion: 0.07
Nodes (53): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+45 more)

### Community 51 - "Dock.tsx"
Cohesion: 0.14
Nodes (33): Fasi, cssPx(), Dock(), Ghost(), Icon(), LaunchpadIcon(), StackIcon(), TrayIcon() (+25 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.15
Nodes (24): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, fold(), fold_lowercases_and_strips_common_accents(), frecency(), frecency_grows_slowly_with_use_and_decays_with_age() (+16 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.06
Nodes (49): capture_info(), Change, clamp_height(), clamp_height_limits_and_survives_garbage(), create(), EXCLUDED, fit(), FOCUS_CHECK (+41 more)

### Community 54 - "indicatorstore.rs"
Cohesion: 0.11
Nodes (34): strip_bom(), a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+26 more)

### Community 55 - "Window"
Cohesion: 0.12
Nodes (32): Window, capture_cancel(), capture_region(), dock_fit(), dock_launchpad(), dock_menu(), dock_settings_menu(), eosmenu_run() (+24 more)

### Community 57 - "Pinned"
Cohesion: 0.09
Nodes (50): Pinned, a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), ANIMS, autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips() (+42 more)

### Community 58 - "winsys.rs"
Cohesion: 0.08
Nodes (47): APPROVED, approved_hex(), broadcast_theme_change(), CREATE_NO_WINDOW, Display, displays(), Drive, drives() (+39 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (32): age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history(), enabled() (+24 more)

### Community 60 - "menuread.rs"
Cohesion: 0.05
Nodes (8): INFLIGHT, LAZY_TIMEOUT, MAX_INFLIGHT, MIN_UIA_MENU_ITEMS, READ_DEPTH, read_hmenu_tree(), READ_ITEMS, UIA_TIMEOUT

### Community 61 - "mc.rs"
Cohesion: 0.06
Nodes (80): app_display_name(), raw_app_windows(), Rect, a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD (+72 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.06
Nodes (74): mc_vd_report(), a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), build_info() (+66 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 66 - "launcher.rs"
Cohesion: 0.08
Nodes (39): all_page(), alphabetical(), AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates() (+31 more)

### Community 67 - "autostart.rs"
Cohesion: 0.12
Nodes (25): a_missing_or_relative_exe_is_refused_and_nothing_is_written(), a_task_manager_block_counts_as_off_and_turning_on_clears_it(), APPROVED, blocked_by_user(), clean(), current_target(), delete_value(), disable() (+17 more)

### Community 68 - "capture.rs"
Cohesion: 0.06
Nodes (48): a_selection_is_converted_to_physical_pixels_inside_the_monitor(), a_taken_name_gets_a_counter(), a_window_partly_off_screen_is_cut_to_the_screen(), APP, BASE_ID, clamp_to(), current_mode(), cursor_monitor() (+40 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.07
Nodes (13): THREAD_ID, THREAD_ID, C_SWALLOWED, SWALLOWED, COUNTER, FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip() (+5 more)

### Community 72 - "indicators.ts"
Cohesion: 0.15
Nodes (30): I1 — Dati: FATTA (7 nov 2026), I3 — Pannello: FATTA (7 nov 2026), Glyph(), Indicators(), Panel(), TrayApp, TrayPanel, useIndicators() (+22 more)

### Community 73 - "dockkeys.rs"
Cohesion: 0.09
Nodes (19): REGISTERED, BASE_ID, current_mode(), digits_map_to_the_number_row(), each_mode_has_its_modifiers(), effective_mode(), key_of(), MODES (+11 more)

### Community 74 - "settingswin.rs"
Cohesion: 0.07
Nodes (25): LABEL, MONITOR, VISIBLE, a_small_monitor_shrinks_the_window_but_never_exceeds_it(), APP, create(), cursor_monitor(), HEIGHT (+17 more)

### Community 75 - "vdesk.rs"
Cohesion: 0.07
Nodes (31): mc_desktops(), an_unknown_or_missing_current_desktop_falls_back_to_the_first(), BYTES, create(), current_index(), Desktop, desktops_from(), Dir (+23 more)

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "test-mc-overlay.ps1"
Cohesion: 0.20
Nodes (13): Cell-Info(), CellJs(), Esc-If-Open(), Inv(), IsOpen(), Js(), Mc(), McIdx() (+5 more)

### Community 78 - "test-settings-s5.ps1"
Cohesion: 0.24
Nodes (14): Choose-Settings-In-Panel(), Close-Panel(), Close-Settings(), Eos-Pids(), Focus(), Inv(), Js(), Native-Last() (+6 more)

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - "mckeys.rs"
Cohesion: 0.15
Nodes (31): a_lone_win_tap_opens_the_launcher_and_hides_the_release_from_start(), a_lost_alt_release_is_noticed_by_the_safety_net(), a_lost_win_release_is_forgotten_so_the_next_tap_still_works(), a_modifier_held_before_win_or_the_setting_off_means_no_tap(), a_mouse_click_or_wheel_with_win_held_is_not_a_tap(), alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle() (+23 more)

### Community 82 - "test-indicators-settings.ps1"
Cohesion: 0.20
Nodes (8): BarWin(), Choose(), Keys(), Label(), Menu-Element(), Menu-Items(), Open-Menu(), PJs()

### Community 83 - "test-mc-desktops.ps1"
Cohesion: 0.43
Nodes (5): Click-Chip(), Desktops(), Inv(), Js(), McIdx()

### Community 84 - "test-settings-s3.ps1"
Cohesion: 0.25
Nodes (10): Click-Nav(), Close-Mc(), Focus-Settings(), Inv(), Js(), Mc-Phase(), Press(), Val() (+2 more)

### Community 85 - "test-menudrop.ps1"
Cohesion: 0.26
Nodes (13): Bar-Click(), Bar-Hover(), Close-Panel(), Focus(), Inv(), Js(), Open-Menu(), Panel-Click() (+5 more)

### Community 86 - "mctrigger.rs"
Cohesion: 0.12
Nodes (26): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+18 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "Mc.tsx"
Cohesion: 0.27
Nodes (17): Mc(), advancedHint(), boxStyle(), canRemove(), cardLetter(), cellTitle(), cleanDesktopName(), desktopAction() (+9 more)

### Community 89 - "settingsreg.rs"
Cohesion: 0.10
Nodes (22): buttons_run_actions_but_have_no_value(), CAPTURE_KEYS, DOCK_ANIM, DOCK_KEYS, DOCK_SIZE, GLASS, LANGUAGES, LAUNCHER_HOTKEYS (+14 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Settings.tsx"
Cohesion: 0.05
Nodes (68): Come si apre, Contenuto proposto (sezioni), Cosa c'è oggi (inventario), Cosa fa macOS (da replicare) e da dove viene, Decisioni (prese dall'utente, 8 nov 2026), Fasi, Fuori scopo, Impostazioni di EosOS — specifica (+60 more)

### Community 92 - "test-menuinvoke.ps1"
Cohesion: 0.24
Nodes (14): Bar-Click(), Close-Panel(), Focus(), Inv(), Js(), Log-Count(), Log-Has(), Log-Lines() (+6 more)

### Community 94 - "test-notif-dnd.ps1"
Cohesion: 0.30
Nodes (8): Ensure-Panel(), Js(), My-Titles(), Open-Panel(), Panel-Open(), Pn(), Setting(), Wait-Until()

### Community 95 - "start"
Cohesion: 0.25
Nodes (6): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "Value"
Cohesion: 0.17
Nodes (12): b(), launcher_hotkey_text(), Live, want_bool(), want_list(), want_text(), Backend, Fake (+4 more)

### Community 98 - "notifprobe.rs"
Cohesion: 0.11
Nodes (24): an_allowed_listener_is_usable_and_other_states_need_consent(), an_unexplained_error_is_never_called_an_identity_problem(), build_number(), classify(), count_toasts(), E_ILLEGAL_METHOD_CALL, err(), HR_NO_PACKAGE (+16 more)

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.10
Nodes (41): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+33 more)

### Community 101 - "Readability"
Cohesion: 0.33
Nodes (6): Readability, Both, HMenu, None, Uia, Unknown

### Community 102 - "it"
Cohesion: 0.23
Nodes (13): Action, App, action_paths_and_flags_survive_normalization(), it(), Item, item_at(), items_are_found_by_their_model_path(), levels_and_counts_are_limited() (+5 more)

### Community 103 - "serialize"
Cohesion: 0.24
Nodes (16): add(), add_assigns_new_ids_and_enforces_limits(), clean_text(), clean_time(), clear_done(), display_order(), MAX_REMINDERS, MAX_TEXT (+8 more)

### Community 104 - "menubar.ts"
Cohesion: 0.19
Nodes (17): G3 — La barra: FATTA (8 nov 2026), G4 — La tendina: FATTA (8 nov 2026), AppMenus(), rectArgs(), MenuPanel, acceptRow(), firstSelectable(), flatRows() (+9 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "calendar.ts"
Cohesion: 0.06
Nodes (61): Centro notifiche e calendario — specifica, Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisioni che servono da te, Fasi, Fuori scopo, N1 — Prova di fattibilità: FATTA (8 nov 2026) — **le notifiche si possono leggere**, N2 — Il calendario: FATTO (8 nov 2026) (+53 more)

### Community 107 - "open"
Cohesion: 0.24
Nodes (13): apply_styles(), McMonitor, create_one(), cursor_monitor(), fit_exact(), hwnd_of(), icons_for(), label() (+5 more)

### Community 108 - "trayapps.rs"
Cohesion: 0.10
Nodes (22): open(), apps(), CACHE, compute(), count(), current(), Found, is_candidate() (+14 more)

### Community 109 - "stacks.rs"
Cohesion: 0.13
Nodes (25): e(), Entry, EntryOut, folders(), id_of(), ids_round_trip_and_reject_garbage(), index_of(), is_junk() (+17 more)

### Community 111 - "same_app"
Cohesion: 0.13
Nodes (29): name, apply_run_order(), background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), DockItem, dump_dock(), find(), find_with_windows(), items() (+21 more)

### Community 112 - "control.rs"
Cohesion: 0.16
Nodes (18): AWAKE, ControlState, FAKE, fake_on(), is_toggle(), keep_awake_starts_off_and_toggles(), only_known_toggles_are_accepted(), Radio (+10 more)

### Community 114 - "test-menubar.ps1"
Cohesion: 0.31
Nodes (5): BarTitles(), Focus(), Js(), Row(), Wait-Until()

### Community 115 - "launchpadfilter.rs"
Cohesion: 0.10
Nodes (20): app(), apps_only(), CACHE, decide(), exe_path_of(), Facts, is_app(), known_folder() (+12 more)

### Community 116 - "test-menusettings.ps1"
Cohesion: 0.29
Nodes (6): Act(), Count-Menus(), Focus(), Js(), Row(), Wait-Until()

### Community 117 - "notifread.rs"
Cohesion: 0.18
Nodes (5): EPOCH_DIFF_MS, in_mta(), LAST_OPENED, read_all(), remove()

### Community 118 - "duration"
Cohesion: 0.16
Nodes (20): AUTO_DELAY, check(), CHECK_TIMEOUT, clean_notes(), current_version(), describe(), each_state_has_an_italian_message(), idle() (+12 more)

### Community 119 - "notifmodel.rs"
Cohesion: 0.07
Nodes (59): a_notification_matches_by_app_name_or_by_a_token_of_its_identifier(), Badge, badges(), CACHE, CACHE_SLOT, CAP, counts(), counts_go_to_one_icon_each_and_only_icons_with_notifications_appear() (+51 more)

### Community 120 - "tick"
Cohesion: 0.13
Nodes (16): blur_enabled(), build_region(), capture_info(), client_in_window(), client_rect(), cursor(), enforce_styles(), Geometry (+8 more)

### Community 121 - "ensure"
Cohesion: 0.60
Nodes (4): end(), ensure(), finish(), win_of()

### Community 122 - "Menu globale per app — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, G1 — Misura in sola lettura: FATTA (8 nov 2026), G2 — Modello e disposizione (puri): FATTA (8 nov 2026), G5 — Invocazione: FATTA (8 nov 2026) (+4 more)

### Community 123 - "Layout"
Cohesion: 0.14
Nodes (14): Layout, Bottom, BottomLeft, BottomRight, Center, CenterThird, Left, LeftThird (+6 more)

### Community 124 - "KindOut"
Cohesion: 0.15
Nodes (13): Entry, kind_out(), Section, Appearance, Dock, General, Launcher, MenuBar (+5 more)

### Community 125 - "test-settingswin.ps1"
Cohesion: 0.22
Nodes (11): Click-Nav(), Click-Switch(), Heading(), Js(), Labels(), Navs(), Row-Js(), Switch-On() (+3 more)

### Community 126 - "test-winsnap.ps1"
Cohesion: 0.22
Nodes (7): Fire(), Js(), Open-Win(), Registered(), Set-S(), Val(), Wait-Until()

### Community 127 - "test-eosmenu.ps1"
Cohesion: 0.42
Nodes (6): Click-Item(), Close-Panel(), Js(), Open-Menu(), Pn(), Wait-Until()

### Community 129 - "super"
Cohesion: 0.22
Nodes (8): entries(), Entry, find(), only_known_ids_open_and_entries_hide_the_addresses(), open(), p(), Page, PAGES

### Community 130 - "lib.rs"
Cohesion: 0.06
Nodes (29): About, bar_backdrop(), cleanup(), dock_backdrop(), dock_items(), enable_taskbar_hiding(), eosmenu_about(), hide_requested() (+21 more)

### Community 131 - "Mission Control — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+5 more)

### Community 133 - "test-menu-view.ps1"
Cohesion: 0.36
Nodes (4): Js(), Names(), Set-S(), Setting()

### Community 134 - "Channel"
Cohesion: 0.15
Nodes (12): Channel, Bar0, Bar1, Bar2, Bar3, Dock, Dock1, Dock2 (+4 more)

### Community 135 - "chromeguard.rs"
Cohesion: 0.07
Nodes (12): a_requested_style_never_keeps_the_frame_bits(), CHROME, only_a_frame_is_removed_and_other_bits_stay(), protect(), protect_hwnd(), start(), strip(), style_allowed() (+4 more)

### Community 136 - "test-update.ps1"
Cohesion: 0.31
Nodes (5): Check-Now(), Cleanup-Eos(), Js(), Start-Eos(), Status-Now()

### Community 137 - "test-calendar.ps1"
Cohesion: 0.25
Nodes (5): Cal(), Click-Btn(), Js(), Key(), Title()

### Community 138 - "compose"
Cohesion: 0.18
Nodes (21): an_app_gets_the_hybrid_row(), build_row(), state(), the_desktop_shows_nothing_by_default_because_the_name_is_hidden(), Eos, an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), app_menu(), app_menu_names_the_app_and_never_is_blank() (+13 more)

### Community 139 - "test-notif-panel.ps1"
Cohesion: 0.33
Nodes (6): Js(), My-Titles(), Open-Panel(), Panel-Open(), Pn(), Wait-Until()

### Community 141 - "test-capture.ps1"
Cohesion: 0.27
Nodes (5): Cleanup-Eos(), Fire(), Js(), Set-S(), Start-Eos()

### Community 142 - "show"
Cohesion: 0.20
Nodes (16): create(), fit(), hide(), is_busy(), is_visible(), lock(), MON, monitor_at() (+8 more)

### Community 145 - "Hotkey"
Cohesion: 0.21
Nodes (16): arrow_keys_work_in_hotkeys(), change_hotkey(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey (+8 more)

### Community 146 - "test-mc-dpi.ps1"
Cohesion: 0.27
Nodes (10): Avg-Color(), CellJs(), Check(), Inv(), IsOpen(), Js(), Overlays(), PageFor() (+2 more)

### Community 147 - "rank"
Cohesion: 0.42
Nodes (10): empty_query_lists_only_used_apps_by_frecency(), entries(), frecency_breaks_ties_but_cannot_beat_a_much_better_match(), heavy_use_lifts_an_app_within_the_same_tier(), limit_and_empty_input_are_respected(), names(), rank(), rank_orders_by_match_then_frecency_then_length() (+2 more)

### Community 148 - "set_value"
Cohesion: 0.20
Nodes (15): a_failing_backend_reports_the_error_and_changes_nothing(), Applied, choices_accept_only_their_options(), folders_must_be_absolute_and_safe(), hotkeys_are_validated_and_written_in_a_canonical_form(), l(), name_lists_are_cleaned(), pinned_apps_can_be_reordered_and_removed_but_not_invented() (+7 more)

### Community 149 - "test-settings.ps1"
Cohesion: 0.36
Nodes (4): All-S(), Js(), Set-S(), Val()

### Community 150 - "@tauri-apps/api"
Cohesion: 0.15
Nodes (15): Capture(), Box, clampPoint(), isSelection(), MIN_SIDE, Point, selection(), sizeLabel() (+7 more)

### Community 151 - "mcstore.rs"
Cohesion: 0.13
Nodes (40): last(), LAST, load(), load_full(), LOCK, lock(), read_store(), remember() (+32 more)

### Community 152 - "Aggiornamenti automatici"
Cohesion: 0.40
Nodes (4): Aggiornamenti automatici, Come funziona, Cosa serve a te (decisioni e passaggi che non posso fare io), Verifica

### Community 153 - "test-stacks.ps1"
Cohesion: 0.35
Nodes (9): Close-Panel(), Js(), Open-Stack(), Panel-Open(), Pn(), Set-Stacks(), Stack-Names(), Stack-Titles() (+1 more)

### Community 158 - "test-settings-s4.ps1"
Cohesion: 0.24
Nodes (4): Inv(), Js(), Val(), Win()

### Community 160 - "test-oggi.ps1"
Cohesion: 0.28
Nodes (13): Add-Rem(), Cleanup-Eos(), Close-Panel(), Items(), Js(), Open-Panel(), Panel-Open(), Pn() (+5 more)

### Community 162 - "cmdpolicy.rs"
Cohesion: 0.23
Nodes (16): allowed(), background_apps_are_counted_by_the_dock_and_opened_from_the_panel(), dock_badges_belong_to_the_dock_windows(), dock_panel_and_mission_control_actions_have_one_owner(), guard(), notifications_and_launcher_commands_have_one_owner(), read_only_and_opening_commands_are_not_restricted_here(), screen_capture_commands_have_their_own_windows() (+8 more)

### Community 164 - "Kind"
Cohesion: 0.22
Nodes (9): Kind, Action, Bool, Choice, Folders, Hotkey, Names, Pinned (+1 more)

### Community 165 - "test-launchpad.ps1"
Cohesion: 0.32
Nodes (3): Apps(), Js(), Type-Search()

### Community 166 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 168 - "MenuAction"
Cohesion: 0.18
Nodes (11): MenuAction, Open, Pin, Quit, SetGlass, SetSize, ToggleAllScreens, ToggleAutoHide (+3 more)

### Community 169 - "test-settings-s6.ps1"
Cohesion: 0.39
Nodes (5): Active(), Heading(), Js(), Key(), Win()

### Community 170 - "test-launcher-dpi.ps1"
Cohesion: 0.35
Nodes (8): Check(), Close-Launcher(), Focused(), IsOpen(), Js(), Open-On(), Run-Checks(), TypeChars()

### Community 171 - "InvokeErr"
Cohesion: 0.21
Nodes (12): a_shown_title_must_still_be_the_one_in_the_menu(), invoke_hmenu(), invoke_item(), invoke_uia(), InvokeErr, Changed, Failed, Gone (+4 more)

### Community 173 - "test-dock-mixed-dpi.ps1"
Cohesion: 0.36
Nodes (6): Cdp(), Check(), Rect(), Run-Checks(), Stop-Stripes(), Wait-Dock()

### Community 176 - "Cmd"
Cohesion: 0.31
Nodes (7): choose(), exe_stem_of(), forget_cache(), menu(), on_window(), quit_app(), run_cmd()

### Community 177 - "Indicatori di sistema — specifica"
Cohesion: 0.22
Nodes (8): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti, Scelte tecniche (proposte)

### Community 178 - "test-calendar-n6.ps1"
Cohesion: 0.40
Nodes (7): Close-Panel(), Heads(), Js(), Open-Panel(), Panel-Open(), Pn(), Wait-Until()

### Community 179 - "test-indicators-dpi.ps1"
Cohesion: 0.38
Nodes (7): Check(), PageFor(), PanelOpen(), PanelWin(), PJs(), Run-Checks(), Wait-Until()

### Community 180 - "HWND"
Cohesion: 0.36
Nodes (8): classification_covers_every_case(), classify(), p(), Probe, probe(), read_hmenu(), read_menus(), read_uia()

### Community 182 - "test-dock-scales.ps1"
Cohesion: 0.36
Nodes (6): Cdp(), Check(), Rect(), Run-Checks(), Stop-Stripes(), Wait-Dock()

### Community 184 - "ControlPanel.tsx"
Cohesion: 0.19
Nodes (15): vitest@^5.1.0, vitest, ControlPanel, SettingRow, About, EosMenuPanel, ControlState, Tile (+7 more)

### Community 186 - "test-menu-dpi.ps1"
Cohesion: 0.47
Nodes (7): Check(), Close-Panel(), Focus(), JsB(), Panel-Open(), Test-Scale(), Wait-Until()

### Community 188 - "test-calendar-dpi.ps1"
Cohesion: 0.54
Nodes (6): Check(), Close-Panel(), JsB(), Panel-Open(), Test-Scale(), Wait-Until()

### Community 189 - "react"
Cohesion: 0.16
Nodes (22): react, @tauri-apps/api, App(), Clock(), ControlButton(), EosButton(), useBackdrop(), barColors() (+14 more)

### Community 190 - "notif_list"
Cohesion: 0.33
Nodes (7): notif_allowed(), notif_clear_app(), notif_dismiss(), notif_list(), notif_open(), NotifReply, now_ms()

### Community 191 - "test-stacks-dpi.ps1"
Cohesion: 0.54
Nodes (6): Check(), Close-Panel(), Js(), Panel-Open(), Test-Scale(), Wait-Until()

### Community 192 - "test-winsettings.ps1"
Cohesion: 0.33
Nodes (5): Click-Tile(), Js(), Last-Open(), Type-Search(), Win()

### Community 196 - "Prove a mano (quelle che l'assistente non può fare)"
Cohesion: 0.12
Nodes (15): 10. Cattura dello schermo (nuovo), 10. Cattura dello schermo (nuovo), 11. Correzioni del 10 novembre (da provare a mano), 1. Tasto Windows → Launcher (nuovo), 2. Mission Control e tastiera, 3. Dock e monitor, 4. Barra dei menu, 5. Sistema (+7 more)

### Community 197 - "Passata finale (8 nov 2026)"
Cohesion: 0.40
Nodes (4): Altro emerso, Consumi a riposo (`scripts/measure-idle.ps1`, build release ottimizzata, 30 s, nessuna interazione), Passata finale (8 nov 2026), Sicurezza (revisione statica di un agente, poi corretta)

### Community 198 - "menumodel.rs"
Cohesion: 0.13
Nodes (26): apps_with_their_own_in_window_menu_are_never_read(), browsers_and_chats_are_never_queried(), exclusions_ignore_case_and_toggle(), is_excluded(), labels_lose_accelerators_and_split_the_shortcut(), lazy_items_get_paths_from_their_source_menu(), limit_title(), long_titles_are_cut_without_breaking_characters() (+18 more)

### Community 201 - "test-cmd-policy.ps1"
Cohesion: 0.43
Nodes (5): Call(), Cleanup-Eos(), Js(), Setting(), Start-Eos()

### Community 202 - "pathbuf"
Cohesion: 0.15
Nodes (8): cached_icon(), ICONS, SHELL_NAMES, CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh(), TimedCache

### Community 203 - "stacks.ts"
Cohesion: 0.31
Nodes (8): StackPanel, isLatest(), sameStacks(), stackAria(), StackEntry, StackItem, StackReply, stackSummary()

### Community 204 - "Get-Monitors"
Cohesion: 0.36
Nodes (6): Get-Monitors(), Check(), Dock-Rects(), JsD(), Run-Checks(), Wait-Until()

### Community 205 - "KindOut"
Cohesion: 0.22
Nodes (9): KindOut, Action, Bool, Choice, Folders, Hotkey, Names, Pinned (+1 more)

### Community 208 - "Dock — specifica"
Cohesion: 0.33
Nodes (5): Comportamento (da replicare), Dock — specifica, Fuori scopo, Rischi aperti, Scelte tecniche

### Community 209 - "Widget «Oggi»"
Cohesion: 0.33
Nodes (5): Come è fatto, Cosa fa, Privacy e rete, Verifica, Widget «Oggi»

### Community 212 - "Cattura dello schermo"
Cohesion: 0.33
Nodes (5): Cattura dello schermo, Che cosa c'è nell'immagine, Come funziona, Prove, Selezione

### Community 214 - "Rifiniture dopo le prove a mano (9 nov 2026)"
Cohesion: 0.17
Nodes (11): Barra dei menu, Consumi a riposo con i due Dock (9 nov 2026, dopo le fasi nuove), Dock, Il tasto Windows apre il Launcher (9 nov 2026), Menu EosOS e Launchpad (9 nov 2026) — il «menu mela» e la griglia delle app, Prove, Reattività, Rifiniture dopo le prove a mano (9 nov 2026) (+3 more)

### Community 215 - "act"
Cohesion: 0.39
Nodes (8): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), commands_are_fixed_and_exact(), confirmation_expires_and_does_not_transfer_between_actions(), dangerous_actions_need_a_second_press_on_the_same_action(), plain_actions_run_immediately(), step()

### Community 216 - "run_menu_action"
Cohesion: 0.33
Nodes (7): run_menu_action(), clamp_glass(), glass_defaults_and_levels(), glass_value(), nearest_glass_id(), set_glass(), unpin()

### Community 217 - "best_bar"
Cohesion: 0.29
Nodes (5): best_bar(), is_system_bar(), lazy_uia(), read_lazy_items(), the_title_bar_system_menu_is_not_the_app_menu()

### Community 218 - "ui"
Cohesion: 0.29
Nodes (7): anim_id(), dock_sizes_are_known_and_default_to_medium(), size_factor(), the_app_animation_defaults_to_the_bounce(), Ui, ui(), dock_settings()

### Community 220 - "set_low"
Cohesion: 0.80
Nodes (3): hidden(), set_low(), shown()

### Community 221 - "Outcome"
Cohesion: 0.14
Nodes (18): Action, ACTIONS, execute(), find(), native_confirm(), now_ms(), Outcome, Confirm (+10 more)

### Community 222 - "test-dock-keys.ps1"
Cohesion: 0.31
Nodes (5): Cleanup-Eos(), Js(), Registered(), Set-S(), Start-Eos()

### Community 223 - "test-dock-badges.ps1"
Cohesion: 0.25
Nodes (5): Badge-Aria(), Badge-Text(), Has-Icon(), Js(), Set-S()

### Community 224 - "test-control.ps1"
Cohesion: 0.35
Nodes (8): Click-Tile(), Close-Panel(), Js(), Open-Control(), Pn(), State(), Tile(), Wait-Until()

### Community 225 - "Cmd"
Cohesion: 0.29
Nodes (7): Cmd, Cancel, Commit, Launcher, Next, Prev, Toggle

### Community 230 - ".new"
Cohesion: 0.33
Nodes (4): MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip()

### Community 232 - "update_reminders"
Cohesion: 0.33
Nodes (5): oggi_add(), oggi_clear_done(), oggi_reminders(), oggi_remove(), oggi_toggle()

### Community 234 - "Msg"
Cohesion: 0.33
Nodes (6): c_send(), C_TX, Msg, Release, Run, Swallowed

### Community 235 - "measure-open.ps1"
Cohesion: 0.83
Nodes (3): Cleanup-Eos(), Js(), Run()

### Community 236 - "decide"
Cohesion: 0.20
Nodes (10): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, parse() (+2 more)

### Community 241 - "test-dock-novita.ps1"
Cohesion: 0.36
Nodes (4): Attr(), Entry(), Js(), Set-S()

### Community 242 - "mcwin.rs"
Cohesion: 0.05
Nodes (33): Anim, APP, apply_keys(), Change, CORNER, DEFAULT_HOTKEY, Diag, diag() (+25 more)

### Community 244 - "winsnap.rs"
Cohesion: 0.08
Nodes (48): Action, action_named(), Restore, Snap, apply(), apply_foreground(), BASE_ID, center_keeps_the_size_and_never_exceeds_the_work_area() (+40 more)

### Community 250 - "Gestione delle finestre (snap)"
Cohesion: 0.50
Nodes (3): Come funziona, Gestione delle finestre (snap), Prove

### Community 252 - "Toggle"
Cohesion: 0.33
Nodes (6): click_decisions(), decide(), Toggle, Hide, Ignore, Show

### Community 256 - "validate"
Cohesion: 0.40
Nodes (6): clean_list(), valid_folder(), validate(), at_most_five_stacks(), effective(), the_stack_list_is_clean_and_defaults_to_downloads()

### Community 258 - "AppHandle"
Cohesion: 0.18
Nodes (25): activate(), all_labels(), alt_commit(), alt_step(), backdrop(), close(), close_animated(), close_window() (+17 more)

### Community 259 - "run_search"
Cohesion: 0.30
Nodes (12): bounded_walk(), Hit, hit(), index_query(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders() (+4 more)

### Community 261 - "Badge delle notifiche sul Dock"
Cohesion: 0.33
Nodes (5): Badge delle notifiche sul Dock, Come si abbina una notifica a un'icona (prudente: meglio nessun badge che quello sull'icona sbagliata), Dove sta il codice, Prove, Quando compare

### Community 262 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 265 - "Centro di controllo"
Cohesion: 0.50
Nodes (3): Centro di controllo, Come funziona, Prove

### Community 266 - "Cmd"
Cohesion: 0.29
Nodes (7): Cmd, Close, Maximize, Minimize, OpenSettings, Quit, Restore

### Community 268 - "Correzioni e novità del 10 novembre 2026 (dopo le prove a mano)"
Cohesion: 0.40
Nodes (4): Correzioni e novità del 10 novembre 2026 (dopo le prove a mano), Cosa NON è replicabile (e quindi resta la finestra di Windows), La barra azzurra «EosOS Dock» sopra il Dock (risolta il 10 nov), Prove

### Community 269 - "downscale"
Cohesion: 0.28
Nodes (8): capture_bytes(), downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input(), downscale_keeps_channel_order_per_block(), grab(), grab_reads_real_pixels_from_the_screen(), step()

### Community 273 - "Inflight"
Cohesion: 0.40
Nodes (3): abandoned_reads_are_capped(), Inflight, may_start_read()

### Community 274 - "start"
Cohesion: 0.25
Nodes (8): bar_interval(), capture_slows_down_only_after_a_quiet_second(), is_bar(), next_interval(), only_bar_channels_are_bars(), start(), the_bars_are_captured_less_often_when_everything_is_still(), the_capture_slows_down_in_steps_while_the_background_is_still()

### Community 275 - "remember"
Cohesion: 0.21
Nodes (12): is_known(), is_network_path(), is_risky_file(), key(), known_results_are_capped(), merge_roots(), merge_roots_drops_missing_duplicates_and_nested_folders(), network_paths_and_risky_files_are_recognised() (+4 more)

### Community 276 - "Pile nel Dock"
Cohesion: 0.40
Nodes (4): Come è fatto, Cosa fa, Pile nel Dock, Verifica

### Community 282 - "Held"
Cohesion: 0.50
Nodes (4): Held, Alt, None, Win

### Community 283 - "State"
Cohesion: 0.50
Nodes (3): START, State, STATE

### Community 284 - "encode"
Cohesion: 0.50
Nodes (4): encode(), frame_header_layout_is_stable(), latest(), latest_returns_nothing_until_there_is_a_newer_frame()

### Community 286 - "reconcile_one"
Cohesion: 0.33
Nodes (12): create(), dock_labels_follow_the_monitors(), dock_monitors(), fit(), is_dock_label(), label_for_monitor(), lock(), reconcile_all() (+4 more)

### Community 293 - "Source"
Cohesion: 0.67
Nodes (3): Source, Fallback, Index

## Knowledge Gaps
- **146 isolated node(s):** `BackupAndFresh`, `Fresh`, `Use`, `Both`, `HMenu` (+141 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1264 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **91 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `same_app` to `Pinned`, `trayapps.rs`, `dock.rs`, `dockact.rs`?**
  _High betweenness centrality (0.169) - this node is a cross-community bridge._
- **Are the 85 inferred relationships involving `Window` (e.g. with `on_window_event()` and `bar_backdrop()`) actually correct?**
  _`Window` has 85 INFERRED edges - model-reasoned connections that need verification._
- **What connects `BackupAndFresh`, `Fresh`, `Use` to the rest of the system?**
  _146 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `same_app` to `package.json`?**
  _High betweenness centrality (0.168) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.07692307692307693 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `ControlPanel.tsx` to `indicators.ts`, `menubar.ts`, `calendar.ts`, `Settings.tsx`, `stacks.ts`, `launcher.ts`, `Dock.tsx`, `@tauri-apps/api`, `package.json`, `tokens.test.ts`, `Mc.tsx`, `react`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._