# Graph Report - EosOS  (2026-11-09)

## Corpus Check
- 195 files · ~232,591 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 3646 nodes · 7750 edges · 233 communities (155 shown, 78 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 480 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `6354b034`
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
- load
- EosOS
- Checklist di test manuali
- dock.rs
- dockstore.rs
- launchercalc.rs
- mchook.rs
- dockgeom.rs
- mclayout.rs
- lib-dpi.ps1
- Window
- launcheract.rs
- dockwin.rs
- weather.rs
- indicators.rs
- launcher.ts
- Dock.tsx
- launcherrank.rs
- launcherwin.rs
- mcstore.rs
- lib.rs
- launcherclip.rs
- menuread.rs
- mc.rs
- NetKind
- vdint.rs
- test-mc-vdesk.ps1
- launcher.rs
- autostart.rs
- items_and_windows
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
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
- .new
- InvokeErr
- menubar.ts
- test-mc-keys.ps1
- calendar.ts
- mcwin.rs
- super
- stacks.rs
- Mutex
- indicatorstore.rs
- test-menubar.ps1
- test-menusettings.ps1
- notifread.rs
- open
- notifmodel.rs
- Read
- menumodel.rs
- show
- run_search
- KindOut
- test-settingswin.ps1
- compose
- reminders.rs
- enable
- Mission Control — specifica
- measure-idle.ps1
- test-menu-view.ps1
- remember
- Launcher — specifica
- test-update.ps1
- test-calendar.ps1
- Toggle
- test-notif-panel.ps1
- stacks.ts
- super
- battery_icon
- weather-mock-server.py
- test-notif-probe.ps1
- RawItem
- test-mc-dpi.ps1
- Indicatori di sistema — specifica
- Cmd
- test-settings.ps1
- @tauri-apps/api
- Aggiornamenti automatici
- test-stacks.ps1
- decide
- test-settings-s4.ps1
- test-oggi.ps1
- reload_settings
- Kind
- rank_hits
- LPARAM
- HWND
- test-settings-s6.ps1
- test-launcher-dpi.ps1
- restore_autohide
- test-dock-mixed-dpi.ps1
- same_app
- test-calendar-n6.ps1
- test-indicators-dpi.ps1
- TopMenu
- test-dock-scales.ps1
- Phase
- test-menu-dpi.ps1
- mc_state
- test-calendar-dpi.ps1
- react
- Dock — specifica
- test-stacks-dpi.ps1
- latest
- test-notif-model.ps1
- sandbox-install.ps1
- Passata finale (8 nov 2026)
- capture_rect
- test-cmd-policy.ps1
- Pile nel Dock
- Inflight
- dock_items
- sandbox-update.ps1
- LPARAM
- pathbuf
- Widget «Oggi»
- test-bar-glass.ps1
- menubar_current
- stack_list
- notif_dismiss
- KEEP_HIDDEN
- win_ctrl
- Dir
- test-dock-remove.ps1
- update_reminders

## God Nodes (most connected - your core abstractions)
1. `Rect` - 66 edges
2. `guard!` - 40 edges
3. `Settings` - 29 edges
4. `Pinned` - 25 edges
5. `st()` - 25 edges
6. `update()` - 23 edges
7. `layout()` - 23 edges
8. `Value` - 22 edges
9. `Dock()` - 22 edges
10. `@tauri-apps/api` - 20 edges

## Surprising Connections (you probably didn't know these)
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `G3 — La barra: FATTA (8 nov 2026)` --references--> `acceptRow()`  [INFERRED]
  docs/MENU-GLOBALE.md → src/lib/menubar.ts
- `N4 — L'elenco nel pannello: FATTO (8 nov 2026)` --references--> `isLatest()`  [INFERRED]
  docs/NOTIFICHE.md → src/lib/notifications.ts
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (233 total, 78 thin omitted)

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
Cohesion: 0.06
Nodes (59): SEQ, a_panel_above_the_icon_keeps_its_bottom_when_it_grows(), anchor(), anchor_to_physical(), busy_for(), BUSY_UNTIL_MS, clamp_height(), click_decisions() (+51 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.06
Nodes (10): image_path(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, LOG_MAX_BYTES, ready_event_name(), READY_TIMEOUT, run() (+2 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.05
Nodes (35): capture_info(), BAR_INTERVAL, capture_bytes(), capture_slows_down_only_after_a_quiet_second(), Channel, Bar0, Bar1, Bar2 (+27 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.15
Nodes (15): appbar_state(), autohide_marker(), collect(), Ctx, detect_restart(), enable_autohide(), find(), hide() (+7 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (44): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_index(), bar_rect(), bar_rect_follows_monitor_with_negative_origin() (+36 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (30): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+22 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.09
Nodes (24): activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id() (+16 more)

### Community 24 - "package.json"
Cohesion: 0.07
Nodes (33): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+25 more)

### Community 25 - "menubar.rs"
Cohesion: 0.08
Nodes (32): AUTOHIDE, BLUR, MENU_OPEN, RESERVE, APP, Cache, CACHE, cache_fresh() (+24 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.08
Nodes (14): files(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, KNOWN, KNOWN_CAP, LATEST, LIMIT (+6 more)

### Community 28 - "load"
Cohesion: 0.27
Nodes (10): cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), oggi_reminders(), oggi_weather(), on_ctrl(), run() (+2 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.05
Nodes (25): ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor() (+17 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (67): Pinned, a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips(), clamp_glass() (+59 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.07
Nodes (29): C_CFG, c_cfg(), C_STATE, C_TX, cfg(), CFG, CHILD_IN, config_line() (+21 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (28): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+20 more)

### Community 42 - "mclayout.rs"
Cohesion: 0.14
Nodes (40): Rect, thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), AREA, best_scale(), check_invariants() (+32 more)

### Community 43 - "lib-dpi.ps1"
Cohesion: 0.26
Nodes (11): Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale(), Check() (+3 more)

### Community 44 - "Window"
Cohesion: 0.11
Nodes (29): dock_activate(), dock_reorder(), dock_settings_menu(), dock_stacks(), indicator_open(), indicators_menu(), launcher_action(), launcher_clip_delete() (+21 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.07
Nodes (39): APP_HANDLE, a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW (+31 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (30): slide_covers_the_whole_travel_with_ease_out(), slide_y(), set_autohide(), set_reserve(), autohide(), blur_enabled(), build_region(), capture_info() (+22 more)

### Community 47 - "weather.rs"
Cohesion: 0.07
Nodes (48): AUTO_DELAY, check(), CHECK_TIMEOUT, clean_notes(), current_version(), describe(), each_state_has_an_italian_message(), idle() (+40 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (16): duration_label(), duration_labels(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume (+8 more)

### Community 49 - "launcher.ts"
Cohesion: 0.16
Nodes (27): L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Launcher(), clampSelection(), combine(), fallbackLetter(), Filter, filterForKey(), filterLabel() (+19 more)

### Community 51 - "Dock.tsx"
Cohesion: 0.18
Nodes (27): Fasi, cssPx(), Dock(), Ghost(), Icon(), StackIcon(), applyMove(), clampGlass() (+19 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.13
Nodes (34): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+26 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.05
Nodes (47): arrow_keys_work_in_hotkeys(), Change, change_hotkey(), clamp_height(), clamp_height_limits_and_survives_garbage(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY (+39 more)

### Community 54 - "mcstore.rs"
Cohesion: 0.16
Nodes (31): last(), LAST, load_full(), lock(), parse(), read_store(), remember(), serialize() (+23 more)

### Community 55 - "lib.rs"
Cohesion: 0.10
Nodes (26): dock_fit(), dock_menu(), launcher_hide(), mc_close(), mc_move_window(), mc_new_desktop(), mc_open(), mc_remove_desktop() (+18 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (34): age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history(), enabled() (+26 more)

### Community 60 - "menuread.rs"
Cohesion: 0.05
Nodes (7): INFLIGHT, LAZY_TIMEOUT, MAX_INFLIGHT, MIN_UIA_MENU_ITEMS, READ_DEPTH, READ_ITEMS, UIA_TIMEOUT

### Community 61 - "mc.rs"
Cohesion: 0.08
Nodes (39): raw_app_windows(), a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP (+31 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.06
Nodes (73): a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), build_info(), capabilities_follow_the_selection_all_or_nothing() (+65 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 66 - "launcher.rs"
Cohesion: 0.10
Nodes (32): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+24 more)

### Community 67 - "autostart.rs"
Cohesion: 0.09
Nodes (11): APPROVED, current_target(), exe_of(), NAME, open(), read(), RUN, run_value() (+3 more)

### Community 68 - "items_and_windows"
Cohesion: 0.20
Nodes (15): app_display_name(), apply_run_order(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows() (+7 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.08
Nodes (11): C_SWALLOWED, SWALLOWED, COUNTER, FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding(), restore_is_idempotent_and_safe_when_nothing_hidden() (+3 more)

### Community 72 - "indicators.ts"
Cohesion: 0.16
Nodes (29): I1 — Dati: FATTA (7 nov 2026), I3 — Pannello: FATTA (7 nov 2026), @tauri-apps/api, Glyph(), Indicators(), Panel(), useIndicators(), Battery (+21 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "settingswin.rs"
Cohesion: 0.08
Nodes (22): a_small_monitor_shrinks_the_window_but_never_exceeds_it(), APP, create(), cursor_monitor(), HEIGHT, hide(), is_menu_id(), LABEL (+14 more)

### Community 75 - "vdesk.rs"
Cohesion: 0.09
Nodes (20): mc_desktops(), an_unknown_or_missing_current_desktop_falls_back_to_the_first(), BYTES, current_index(), Desktop, desktops_from(), guid_from_bytes(), guids_are_read_in_windows_byte_order() (+12 more)

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
Nodes (28): a_lost_alt_release_is_noticed_by_the_safety_net(), alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle(), cycling_starts_from_the_second_window_and_wraps(), disabled_shortcuts_are_never_touched(), escape_cancels_and_a_later_alt_release_does_not_commit(), Held (+20 more)

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
Cohesion: 0.13
Nodes (24): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+16 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "Mc.tsx"
Cohesion: 0.27
Nodes (17): Mc(), advancedHint(), boxStyle(), canRemove(), cardLetter(), cellTitle(), cleanDesktopName(), desktopAction() (+9 more)

### Community 89 - "settingsreg.rs"
Cohesion: 0.10
Nodes (34): a_failing_backend_reports_the_error_and_changes_nothing(), Applied, buttons_run_actions_but_have_no_value(), choices_accept_only_their_options(), DOCK_SIZE, fold(), folders_must_be_absolute_and_safe(), folding_removes_accents_and_case() (+26 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Settings.tsx"
Cohesion: 0.08
Nodes (47): Come si apre, Contenuto proposto (sezioni), Cosa c'è oggi (inventario), Cosa fa macOS (da replicare) e da dove viene, Decisioni (prese dall'utente, 8 nov 2026), Fasi, Fuori scopo, Impostazioni di EosOS — specifica (+39 more)

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
Cohesion: 0.15
Nodes (16): b(), launcher_hotkey_text(), Live, want_bool(), want_list(), want_text(), Backend, clean_list() (+8 more)

### Community 98 - "notifprobe.rs"
Cohesion: 0.11
Nodes (24): an_allowed_listener_is_usable_and_other_states_need_consent(), an_unexplained_error_is_never_called_an_identity_problem(), build_number(), classify(), count_toasts(), E_ILLEGAL_METHOD_CALL, err(), HR_NO_PACKAGE (+16 more)

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.13
Nodes (25): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide_never_treats_a_failed_read_as_missing(), Entry, File, hotkey_and_search_folders_roundtrip_and_default(), LAST, load(), lock_store() (+17 more)

### Community 101 - "Readability"
Cohesion: 0.22
Nodes (10): classification_covers_every_case(), classify(), p(), Probe, Readability, Both, HMenu, None (+2 more)

### Community 102 - ".new"
Cohesion: 0.33
Nodes (4): MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip()

### Community 103 - "InvokeErr"
Cohesion: 0.21
Nodes (12): a_shown_title_must_still_be_the_one_in_the_menu(), invoke_hmenu(), invoke_item(), invoke_uia(), InvokeErr, Changed, Failed, Gone (+4 more)

### Community 104 - "menubar.ts"
Cohesion: 0.10
Nodes (29): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, G1 — Misura in sola lettura: FATTA (8 nov 2026), G2 — Modello e disposizione (puri): FATTA (8 nov 2026), G3 — La barra: FATTA (8 nov 2026) (+21 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "calendar.ts"
Cohesion: 0.06
Nodes (61): Centro notifiche e calendario — specifica, Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisioni che servono da te, Fasi, Fuori scopo, N1 — Prova di fattibilità: FATTA (8 nov 2026) — **le notifiche si possono leggere**, N2 — Il calendario: FATTO (8 nov 2026) (+53 more)

### Community 107 - "mcwin.rs"
Cohesion: 0.05
Nodes (53): activate(), all_labels(), alt_commit(), alt_step(), Anim, APP, apply_keys(), backdrop() (+45 more)

### Community 108 - "super"
Cohesion: 0.14
Nodes (8): cached_icon(), ICONS, SHELL_NAMES, CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh(), TimedCache

### Community 109 - "stacks.rs"
Cohesion: 0.11
Nodes (30): downloads_folder(), known_folder(), at_most_five_stacks(), e(), effective(), Entry, EntryOut, folders() (+22 more)

### Community 111 - "indicatorstore.rs"
Cohesion: 0.13
Nodes (26): strip_bom(), a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+18 more)

### Community 114 - "test-menubar.ps1"
Cohesion: 0.31
Nodes (5): BarTitles(), Focus(), Js(), Row(), Wait-Until()

### Community 116 - "test-menusettings.ps1"
Cohesion: 0.29
Nodes (6): Act(), Count-Menus(), Focus(), Js(), Row(), Wait-Until()

### Community 117 - "notifread.rs"
Cohesion: 0.13
Nodes (8): app_ids_that_could_hide_a_command_are_refused(), valid_app_id(), EPOCH_DIFF_MS, in_mta(), LAST_OPENED, open_app(), read_all(), remove()

### Community 118 - "open"
Cohesion: 0.24
Nodes (13): apply_styles(), McMonitor, McWindow, create_one(), fit_exact(), hwnd_of(), icons_for(), label() (+5 more)

### Community 119 - "notifmodel.rs"
Cohesion: 0.13
Nodes (32): a_notification_with_only_a_body_or_only_a_title_is_kept(), ago(), all_the_ids_of_an_app_are_found_even_beyond_the_group_cap(), app_label(), build_groups(), clean_text(), contains_id(), date_short() (+24 more)

### Community 120 - "Read"
Cohesion: 0.17
Nodes (19): Read, Failed, Missing, decide(), Decision, BackupAndFresh, Fresh, KeepLast (+11 more)

### Community 121 - "menumodel.rs"
Cohesion: 0.13
Nodes (28): app_menu(), app_menu_names_the_app_and_never_is_blank(), apps_with_their_own_in_window_menu_are_never_read(), browsers_and_chats_are_never_queried(), exclusions_ignore_case_and_toggle(), is_excluded(), labels_lose_accelerators_and_split_the_shortcut(), lazy_items_get_paths_from_their_source_menu() (+20 more)

### Community 122 - "show"
Cohesion: 0.19
Nodes (14): create(), fit(), hide(), is_visible(), on_window_event(), show(), since_start(), start() (+6 more)

### Community 123 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 124 - "KindOut"
Cohesion: 0.10
Nodes (21): Entry, kind_out(), KindOut, Action, Bool, Choice, Folders, Hotkey (+13 more)

### Community 125 - "test-settingswin.ps1"
Cohesion: 0.22
Nodes (11): Click-Nav(), Click-Switch(), Heading(), Js(), Labels(), Navs(), Row-Js(), Switch-On() (+3 more)

### Community 126 - "compose"
Cohesion: 0.21
Nodes (17): an_app_gets_the_hybrid_row(), build_row(), state(), the_desktop_shows_only_its_name(), Eos, an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), compose(), filter_view() (+9 more)

### Community 127 - "reminders.rs"
Cohesion: 0.24
Nodes (16): add(), add_assigns_new_ids_and_enforces_limits(), clean_text(), clean_time(), clear_done(), display_order(), MAX_REMINDERS, MAX_TEXT (+8 more)

### Community 130 - "enable"
Cohesion: 0.26
Nodes (15): a_missing_or_relative_exe_is_refused_and_nothing_is_written(), a_task_manager_block_counts_as_off_and_turning_on_clears_it(), blocked_by_user(), clean(), delete_value(), disable(), enable(), enable_and_disable_touch_only_our_value() (+7 more)

### Community 131 - "Mission Control — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+5 more)

### Community 133 - "test-menu-view.ps1"
Cohesion: 0.36
Nodes (4): Js(), Names(), Set-S(), Setting()

### Community 134 - "remember"
Cohesion: 0.21
Nodes (12): is_known(), is_network_path(), is_risky_file(), key(), known_results_are_capped(), merge_roots(), merge_roots_drops_missing_duplicates_and_nested_folders(), network_paths_and_risky_files_are_recognised() (+4 more)

### Community 135 - "Launcher — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+4 more)

### Community 136 - "test-update.ps1"
Cohesion: 0.31
Nodes (5): Check-Now(), Cleanup-Eos(), Js(), Start-Eos(), Status-Now()

### Community 137 - "test-calendar.ps1"
Cohesion: 0.25
Nodes (5): Cal(), Click-Btn(), Js(), Key(), Title()

### Community 138 - "Toggle"
Cohesion: 0.29
Nodes (8): menu_id(), menu_ids_roundtrip_and_ignore_other_menus(), parse_menu_id(), Toggle, Battery, Network, Percent, Volume

### Community 139 - "test-notif-panel.ps1"
Cohesion: 0.33
Nodes (6): Js(), My-Titles(), Open-Panel(), Panel-Open(), Pn(), Wait-Until()

### Community 140 - "stacks.ts"
Cohesion: 0.31
Nodes (8): StackPanel, isLatest(), sameStacks(), stackAria(), StackEntry, StackItem, StackReply, stackSummary()

### Community 141 - "super"
Cohesion: 0.09
Nodes (21): allowed(), dock_panel_and_mission_control_actions_have_one_owner(), guard(), notifications_and_launcher_commands_have_one_owner(), read_only_and_opening_commands_are_not_restricted_here(), settings_belong_to_the_settings_window(), the_panel_can_only_flip_the_two_notification_switches(), the_today_widget_belongs_to_the_panel() (+13 more)

### Community 142 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 145 - "RawItem"
Cohesion: 0.40
Nodes (3): best_bar(), is_system_bar(), the_title_bar_system_menu_is_not_the_app_menu()

### Community 146 - "test-mc-dpi.ps1"
Cohesion: 0.27
Nodes (10): Avg-Color(), CellJs(), Check(), Inv(), IsOpen(), Js(), Overlays(), PageFor() (+2 more)

### Community 147 - "Indicatori di sistema — specifica"
Cohesion: 0.22
Nodes (8): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti, Scelte tecniche (proposte)

### Community 148 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 149 - "test-settings.ps1"
Cohesion: 0.36
Nodes (4): All-S(), Js(), Set-S(), Val()

### Community 150 - "@tauri-apps/api"
Cohesion: 0.22
Nodes (7): initGlass(), view, applyThemePref(), initTheme(), isThemePref(), readThemePref(), ThemePref

### Community 152 - "Aggiornamenti automatici"
Cohesion: 0.40
Nodes (4): Aggiornamenti automatici, Come funziona, Cosa serve a te (decisioni e passaggi che non posso fare io), Verifica

### Community 153 - "test-stacks.ps1"
Cohesion: 0.35
Nodes (9): Close-Panel(), Js(), Open-Stack(), Panel-Open(), Pn(), Set-Stacks(), Stack-Names(), Stack-Titles() (+1 more)

### Community 154 - "decide"
Cohesion: 0.29
Nodes (7): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use

### Community 158 - "test-settings-s4.ps1"
Cohesion: 0.24
Nodes (4): Inv(), Js(), Val(), Win()

### Community 160 - "test-oggi.ps1"
Cohesion: 0.28
Nodes (13): Add-Rem(), Cleanup-Eos(), Close-Panel(), Items(), Js(), Open-Panel(), Panel-Open(), Pn() (+5 more)

### Community 163 - "reload_settings"
Cohesion: 0.39
Nodes (8): current(), refresh_current(), reload_settings(), set_excluded(), set_menu_off(), set_view(), toggle_current_excluded(), toggle_menu_off()

### Community 164 - "Kind"
Cohesion: 0.22
Nodes (9): Kind, Action, Bool, Choice, Folders, Hotkey, Names, Pinned (+1 more)

### Community 165 - "rank_hits"
Cohesion: 0.53
Nodes (6): hit(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders(), rank_hits_respects_limit_and_empty()

### Community 166 - "LPARAM"
Cohesion: 0.13
Nodes (20): aumid_from_store(), aumid_path(), cached_shell_name(), collect_window(), ComScope, identity(), is_app_window(), is_packaged_path() (+12 more)

### Community 168 - "HWND"
Cohesion: 0.29
Nodes (9): RawItem, RawMenu, lazy_uia(), probe(), read_hmenu(), read_hmenu_tree(), read_lazy_items(), read_menus() (+1 more)

### Community 169 - "test-settings-s6.ps1"
Cohesion: 0.39
Nodes (5): Active(), Heading(), Js(), Key(), Win()

### Community 170 - "test-launcher-dpi.ps1"
Cohesion: 0.35
Nodes (8): Check(), Close-Launcher(), Focused(), IsOpen(), Js(), Open-On(), Run-Checks(), TypeChars()

### Community 172 - "restore_autohide"
Cohesion: 0.32
Nodes (8): restore(), restore_autohide(), restore_if_hidden(), restore_real(), log(), log_path(), restore_everything(), wait_and_restore()

### Community 173 - "test-dock-mixed-dpi.ps1"
Cohesion: 0.33
Nodes (7): Get-Monitors(), Cdp(), Check(), Rect(), Run-Checks(), Stop-Stripes(), Wait-Dock()

### Community 177 - "same_app"
Cohesion: 0.20
Nodes (17): name, background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), key(), live_process_scan_excludes_self_and_finds_explorer(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), only_a_single_version_folder_counts_as_the_same_app() (+9 more)

### Community 178 - "test-calendar-n6.ps1"
Cohesion: 0.40
Nodes (7): Close-Panel(), Heads(), Js(), Open-Panel(), Panel-Open(), Pn(), Wait-Until()

### Community 179 - "test-indicators-dpi.ps1"
Cohesion: 0.38
Nodes (7): Check(), PageFor(), PanelOpen(), PanelWin(), PJs(), Run-Checks(), Wait-Until()

### Community 180 - "TopMenu"
Cohesion: 0.13
Nodes (21): menu(), Action, App, action_paths_and_flags_survive_normalization(), Cmd, Close, Maximize, Minimize (+13 more)

### Community 182 - "test-dock-scales.ps1"
Cohesion: 0.36
Nodes (6): Cdp(), Check(), Rect(), Run-Checks(), Stop-Stripes(), Wait-Dock()

### Community 184 - "Phase"
Cohesion: 0.29
Nodes (7): Diag, diag(), Phase, Closed, Closing, Open, Opening

### Community 186 - "test-menu-dpi.ps1"
Cohesion: 0.47
Nodes (7): Check(), Close-Panel(), Focus(), JsB(), Panel-Open(), Test-Scale(), Wait-Until()

### Community 188 - "test-calendar-dpi.ps1"
Cohesion: 0.54
Nodes (6): Check(), Close-Panel(), JsB(), Panel-Open(), Test-Scale(), Wait-Until()

### Community 189 - "react"
Cohesion: 0.24
Nodes (12): react, App(), Clock(), useBackdrop(), barTint(), Frame, FRAME_HEADER, meanLuminance() (+4 more)

### Community 190 - "Dock — specifica"
Cohesion: 0.33
Nodes (5): Comportamento (da replicare), Dock — specifica, Fuori scopo, Rischi aperti, Scelte tecniche

### Community 191 - "test-stacks-dpi.ps1"
Cohesion: 0.54
Nodes (6): Check(), Close-Panel(), Js(), Panel-Open(), Test-Scale(), Wait-Until()

### Community 192 - "latest"
Cohesion: 0.33
Nodes (5): bar_backdrop(), dock_backdrop(), launcher_backdrop(), mc_backdrop(), panel_backdrop()

### Community 197 - "Passata finale (8 nov 2026)"
Cohesion: 0.40
Nodes (4): Altro emerso, Consumi a riposo (`scripts/measure-idle.ps1`, build release ottimizzata, 30 s, nessuna interazione), Passata finale (8 nov 2026), Sicurezza (revisione statica di un agente, poi corretta)

### Community 200 - "capture_rect"
Cohesion: 0.29
Nodes (7): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), capture_info(), update_affinity(), capture_info()

### Community 201 - "test-cmd-policy.ps1"
Cohesion: 0.43
Nodes (5): Call(), Cleanup-Eos(), Js(), Setting(), Start-Eos()

### Community 202 - "Pile nel Dock"
Cohesion: 0.40
Nodes (4): Come è fatto, Cosa fa, Pile nel Dock, Verifica

### Community 204 - "Inflight"
Cohesion: 0.40
Nodes (3): abandoned_reads_are_capped(), Inflight, may_start_read()

### Community 207 - "LPARAM"
Cohesion: 0.26
Nodes (9): wndproc(), c_send(), handle(), Msg, Release, Run, Swallowed, proc() (+1 more)

### Community 209 - "Widget «Oggi»"
Cohesion: 0.33
Nodes (5): Come è fatto, Cosa fa, Privacy e rete, Verifica, Widget «Oggi»

### Community 221 - "notif_dismiss"
Cohesion: 0.33
Nodes (7): notif_allowed(), notif_clear_app(), notif_dismiss(), notif_list(), notif_open(), NotifReply, now_ms()

### Community 225 - "win_ctrl"
Cohesion: 0.38
Nodes (5): create(), key_event(), key_events_are_well_formed(), remove_current(), win_ctrl()

### Community 228 - "Dir"
Cohesion: 0.33
Nodes (6): Dir, Left, Right, plan_switch(), switch_to(), switching_plans_the_right_number_of_steps()

### Community 232 - "update_reminders"
Cohesion: 0.53
Nodes (5): oggi_add(), oggi_clear_done(), oggi_remove(), oggi_toggle(), update_reminders()

## Knowledge Gaps
- **120 isolated node(s):** `Use`, `Fresh`, `BackupAndFresh`, `General`, `Appearance` (+115 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1053 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **78 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `same_app` to `items_and_windows`, `dock.rs`, `dockstore.rs`, `dockact.rs`?**
  _High betweenness centrality (0.169) - this node is a cross-community bridge._
- **What connects `Use`, `Fresh`, `BackupAndFresh` to the rest of the system?**
  _120 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `same_app` to `package.json`?**
  _High betweenness centrality (0.168) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.07692307692307693 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `indicators.ts`, `menubar.ts`, `calendar.ts`, `stacks.ts`, `launcher.ts`, `Dock.tsx`, `@tauri-apps/api`, `Mc.tsx`, `tokens.test.ts`, `Settings.tsx`, `react`?**
  _High betweenness centrality (0.084) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05754475703324808 - nodes in this community are weakly interconnected._