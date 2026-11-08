# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 147 files · ~191,307 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 3118 nodes · 6636 edges · 192 communities (130 shown, 62 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 410 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f2f6947d`
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
- menumodel.rs
- EosOS
- Checklist di test manuali
- dock.rs
- dockstore.rs
- launchercalc.rs
- mchook.rs
- dockgeom.rs
- mclayout.rs
- test-dock-scales.ps1
- AppHandle
- launcheract.rs
- dockwin.rs
- collect_window
- indicators.rs
- Dock.tsx
- launcherrank.rs
- launcherwin.rs
- menubar.ts
- indicatorstore.rs
- launcherclip.rs
- menuread.rs
- mc.rs
- NetKind
- vdint.rs
- test-mc-vdesk.ps1
- launcher.rs
- autostart.rs
- compose
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
- Config
- start
- test-mc-trigger.ps1
- Value
- notifprobe.rs
- test-mc-windows.ps1
- launcherstore.rs
- Readability
- .new
- InvokeErr
- mcstore.rs
- test-mc-keys.ps1
- show
- mcwin.rs
- Read
- Menu globale per app — specifica
- Indicators
- toggle_excluded
- test-menubar.ps1
- test-menusettings.ps1
- Window
- run_search
- RawItem
- KindOut
- test-settingswin.ps1
- Settings
- HWND
- icon_data_url
- Indicatori di sistema — specifica
- rank_hits
- AtomicBool
- LauncherItem
- fold
- super
- battery_icon
- scripts
- test-notif-probe.ps1
- MenuProbeRow
- MenuAction
- lib.rs
- merge
- test-settings.ps1
- Pinned
- enable
- Cmd
- settings_set
- test-settings-s4.ps1
- run
- Kind
- settings_all
- AppMenus.tsx
- validate
- test-settings-s6.ps1
- handle
- merge
- run_child
- Cmd
- Dir
- best_bar
- Inflight
- Held
- Centro notifiche e calendario — specifica
- win_ctrl
- Phase
- mc_state
- dock_items
- dock_settings

## God Nodes (most connected - your core abstractions)
1. `Rect` - 65 edges
2. `Pinned` - 25 edges
3. `st()` - 25 edges
4. `layout()` - 23 edges
5. `Value` - 22 edges
6. `Dock()` - 19 edges
7. `Settings` - 18 edges
8. `Profile` - 18 edges
9. `slot()` - 17 edges
10. `selftest()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `G3 — La barra: FATTA (8 nov 2026)` --references--> `acceptRow()`  [INFERRED]
  docs/MENU-GLOBALE.md → src/lib/menubar.ts
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `S3 — Le altre sezioni: FATTA (8 nov 2026)` --references--> `comboFromEvent()`  [INFERRED]
  docs/IMPOSTAZIONI.md → src/lib/settings.ts
- `Fasi` --references--> `DockSettings`  [INFERRED]
  docs/DOCK.md → src/lib/dock.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (192 total, 62 thin omitted)

### Community 0 - "EosOS: Piano di lavoro"
Cohesion: 0.25
Nodes (7): Dettagli sul ripristino (P4), EosOS: Piano di lavoro, LEGENDA dei passaggi, Principi di sicurezza (valgono per tutti i passaggi), Roadmap (FUORI SCOPE per ora), Stato finale (7 nov 2026), Traguardo della sessione

### Community 4 - "compilerOptions"
Cohesion: 0.10
Nodes (19): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+11 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 6 - "panelwin.rs"
Cohesion: 0.07
Nodes (57): SEQ, SCALE_BITS, SHOWN_AT_MS, anchor(), anchor_to_physical(), busy_for(), BUSY_UNTIL_MS, clamp_height() (+49 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.05
Nodes (15): CACHE_FOUND_TTL, CACHE_MISSING_TTL, image_path(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES (+7 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.07
Nodes (23): capture_bytes(), Channel, Dock, Launcher, Panel, CHANNELS, downscale(), downscale_averages_blocks_and_swaps_to_rgba() (+15 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.11
Nodes (17): on_ctrl(), appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), find(), hide_matching() (+9 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.08
Nodes (40): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+32 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (27): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+19 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.12
Nodes (19): only_a_single_version_folder_counts_as_the_same_app(), same_app(), activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch() (+11 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (27): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+19 more)

### Community 25 - "menubar.rs"
Cohesion: 0.09
Nodes (29): ICON_LOCK, RUN_ORDER, an_app_gets_the_hybrid_row(), APP, build_row(), Cache, CACHE, cache_fresh() (+21 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.08
Nodes (21): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, is_known(), key(), KNOWN, KNOWN_CAP, known_folder() (+13 more)

### Community 28 - "menumodel.rs"
Cohesion: 0.14
Nodes (26): app_menu(), app_menu_names_the_app_and_never_is_blank(), Item, item_at(), items_are_found_by_their_model_path(), labels_lose_accelerators_and_split_the_shortcut(), lazy_items_get_paths_from_their_source_menu(), limit_title() (+18 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.05
Nodes (19): ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), APPS_FOLDER, find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), ICON_SIZE, ICONS, is_launcher() (+11 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.10
Nodes (30): a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips(), clamp_glass(), dedup() (+22 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.07
Nodes (18): HOTKEY_THREAD, C_CFG, C_STATE, C_SWALLOWED, C_TX, CFG, CHILD_IN, CREATE_NO_WINDOW (+10 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.10
Nodes (33): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), capture_adds_context_and_stays_on_the_monitor(), capture_margin() (+25 more)

### Community 42 - "mclayout.rs"
Cohesion: 0.13
Nodes (42): Rect, inside(), thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), AREA, best_scale() (+34 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.05
Nodes (56): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+48 more)

### Community 44 - "AppHandle"
Cohesion: 0.10
Nodes (20): dock_fit(), launcher_fit(), launcher_hide(), mc_activate(), mc_close(), mc_close_window(), mc_move_window(), mc_new_desktop() (+12 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.08
Nodes (37): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+29 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (29): slide_covers_the_whole_travel_with_ease_out(), slide_y(), set_autohide(), set_reserve(), autohide(), blur_enabled(), build_region(), capture_info() (+21 more)

### Community 47 - "collect_window"
Cohesion: 0.09
Nodes (28): app_display_name(), aumid_from_store(), aumid_path(), cached_shell_name(), collect_window(), ComScope, identity(), is_app_window() (+20 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (14): Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume, only_known_indicators_open_a_settings_page(), open_settings() (+6 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.05
Nodes (74): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026) (+66 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.16
Nodes (28): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), frecency(), frecency_breaks_ties_but_cannot_beat_a_much_better_match() (+20 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.06
Nodes (41): arrow_keys_work_in_hotkeys(), Change, change_hotkey(), clamp_height(), clamp_height_limits_and_survives_garbage(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY (+33 more)

### Community 54 - "menubar.ts"
Cohesion: 0.25
Nodes (12): G4 — La tendina: FATTA (8 nov 2026), MenuPanel, firstSelectable(), flatRows(), MbAction, MbItem, MbMenu, MbRow (+4 more)

### Community 55 - "indicatorstore.rs"
Cohesion: 0.11
Nodes (32): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+24 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (38): LAST_HWND, age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history() (+30 more)

### Community 60 - "menuread.rs"
Cohesion: 0.06
Nodes (7): INFLIGHT, LAZY_TIMEOUT, MAX_INFLIGHT, MIN_UIA_MENU_ITEMS, READ_DEPTH, READ_ITEMS, UIA_TIMEOUT

### Community 61 - "mc.rs"
Cohesion: 0.08
Nodes (41): a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP, geometry() (+33 more)

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
Cohesion: 0.10
Nodes (32): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+24 more)

### Community 67 - "autostart.rs"
Cohesion: 0.09
Nodes (11): APPROVED, current_target(), exe_of(), NAME, open(), read(), RUN, run_value() (+3 more)

### Community 68 - "compose"
Cohesion: 0.26
Nodes (13): menu(), an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), compose(), is_window_menu(), it(), lazy_menus_stay_in_the_bar_even_when_empty(), levels_and_counts_are_limited(), normalize() (+5 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.07
Nodes (14): proc(), detect_restart(), hide(), restore(), explorer_restart_detection(), FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip() (+6 more)

### Community 72 - "indicators.ts"
Cohesion: 0.19
Nodes (25): I3 — Pannello: FATTA (7 nov 2026), @tauri-apps/api, Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails(), batteryFill() (+17 more)

### Community 73 - "Indicators"
Cohesion: 0.28
Nodes (8): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), duration_label(), duration_labels(), read_battery(), serialized_state_is_flat_for_the_page()

### Community 74 - "settingswin.rs"
Cohesion: 0.09
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
Cohesion: 0.20
Nodes (22): alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle(), cycling_starts_from_the_second_window_and_wraps(), disabled_shortcuts_are_never_touched(), escape_cancels_and_a_later_alt_release_does_not_commit(), is_alt(), Key (+14 more)

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
Cohesion: 0.12
Nodes (30): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+22 more)

### Community 89 - "settingsreg.rs"
Cohesion: 0.12
Nodes (27): a_failing_backend_reports_the_error_and_changes_nothing(), Applied, choices_accept_only_their_options(), fold(), folding_removes_accents_and_case(), GLASS, hotkeys_are_validated_and_written_in_a_canonical_form(), l() (+19 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Settings.tsx"
Cohesion: 0.06
Nodes (51): Come si apre, Contenuto proposto (sezioni), Cosa c'è oggi (inventario), Cosa fa macOS (da replicare) e da dove viene, Decisioni (prese dall'utente, 8 nov 2026), Fasi, Fuori scopo, Impostazioni di EosOS — specifica (+43 more)

### Community 92 - "test-menuinvoke.ps1"
Cohesion: 0.24
Nodes (14): Bar-Click(), Close-Panel(), Focus(), Inv(), Js(), Log-Count(), Log-Has(), Log-Lines() (+6 more)

### Community 94 - "Config"
Cohesion: 0.31
Nodes (10): cfg(), config_line(), config_lines_round_trip_and_garbage_is_ignored(), Diag, diag(), parse_config_line(), send_cfg(), set_config() (+2 more)

### Community 95 - "start"
Cohesion: 0.22
Nodes (7): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next(), load()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "Value"
Cohesion: 0.16
Nodes (15): real_keys(), b(), launcher_hotkey_text(), Live, want_bool(), want_list(), want_text(), Backend (+7 more)

### Community 98 - "notifprobe.rs"
Cohesion: 0.09
Nodes (26): notif_count_for(), an_allowed_listener_is_usable_and_other_states_need_consent(), an_unexplained_error_is_never_called_an_identity_problem(), build_number(), classify(), count_for_app(), count_toasts(), E_ILLEGAL_METHOD_CALL (+18 more)

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.10
Nodes (41): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+33 more)

### Community 101 - "Readability"
Cohesion: 0.22
Nodes (10): classification_covers_every_case(), classify(), p(), Probe, Readability, Both, HMenu, None (+2 more)

### Community 102 - ".new"
Cohesion: 0.33
Nodes (4): MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip()

### Community 103 - "InvokeErr"
Cohesion: 0.21
Nodes (12): a_shown_title_must_still_be_the_one_in_the_menu(), invoke_hmenu(), invoke_item(), invoke_uia(), InvokeErr, Changed, Failed, Gone (+4 more)

### Community 104 - "mcstore.rs"
Cohesion: 0.13
Nodes (30): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+22 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "show"
Cohesion: 0.19
Nodes (14): apply_styles(), capture_info(), create(), fit(), hide(), is_visible(), on_window_event(), show() (+6 more)

### Community 107 - "mcwin.rs"
Cohesion: 0.06
Nodes (57): activate(), all_labels(), alt_commit(), alt_step(), Anim, APP, apply_keys(), backdrop() (+49 more)

### Community 108 - "Read"
Cohesion: 0.19
Nodes (13): decide(), Decision, BackupAndSeed, KeepLast, Seed, Use, only_a_failed_read_is_unreliable(), Read (+5 more)

### Community 109 - "Menu globale per app — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, G1 — Misura in sola lettura: FATTA (8 nov 2026), G2 — Modello e disposizione (puri): FATTA (8 nov 2026), G3 — La barra: FATTA (8 nov 2026) (+5 more)

### Community 110 - "Indicators"
Cohesion: 0.24
Nodes (8): current(), Indicators, read_fake(), read_once(), read_real(), STATE, WAKE, indicators_get()

### Community 111 - "toggle_excluded"
Cohesion: 0.25
Nodes (11): current(), refresh_current(), reload_settings(), set_excluded(), set_menu_off(), toggle_current_excluded(), toggle_menu_off(), exclusions_ignore_case_and_toggle() (+3 more)

### Community 114 - "test-menubar.ps1"
Cohesion: 0.31
Nodes (5): BarTitles(), Focus(), Js(), Row(), Wait-Until()

### Community 116 - "test-menusettings.ps1"
Cohesion: 0.29
Nodes (6): Act(), Count-Menus(), Focus(), Js(), Row(), Wait-Until()

### Community 117 - "Window"
Cohesion: 0.33
Nodes (6): dock_menu(), dock_settings_menu(), indicators_menu(), menubar_hover(), menubar_open(), panel_toggle()

### Community 119 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 123 - "RawItem"
Cohesion: 0.29
Nodes (9): RawItem, RawMenu, lazy_uia(), probe(), read_hmenu(), read_hmenu_tree(), read_lazy_items(), read_menus() (+1 more)

### Community 124 - "KindOut"
Cohesion: 0.11
Nodes (19): Entry, kind_out(), KindOut, Action, Bool, Choice, Folders, Hotkey (+11 more)

### Community 125 - "test-settingswin.ps1"
Cohesion: 0.22
Nodes (11): Click-Nav(), Click-Switch(), Heading(), Js(), Labels(), Navs(), Row-Js(), Switch-On() (+3 more)

### Community 126 - "Settings"
Cohesion: 0.36
Nodes (9): default_settings(), Entry, File, last_settings(), parse_full(), remember_settings(), Settings, settings() (+1 more)

### Community 127 - "HWND"
Cohesion: 0.29
Nodes (6): choose(), exe_stem_of(), forget_cache(), quit_app(), run_cmd(), open_soon()

### Community 131 - "icon_data_url"
Cohesion: 0.29
Nodes (6): bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), icon_data_url(), icon_of_notepad_is_a_valid_png()

### Community 132 - "Indicatori di sistema — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti (+4 more)

### Community 137 - "rank_hits"
Cohesion: 0.53
Nodes (6): hit(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders(), rank_hits_respects_limit_and_empty()

### Community 138 - "AtomicBool"
Cohesion: 0.20
Nodes (9): AUTOHIDE, BLUR, MENU_OPEN, RESERVE, ENABLED, CORNER, EXCLUDED, VISIBLE (+1 more)

### Community 140 - "fold"
Cohesion: 0.25
Nodes (9): files(), short_dir(), short_dir_uses_home_and_elides_long_paths(), fold(), fold_lowercases_and_strips_common_accents(), initials(), initials_cover_words_and_inner_capitals(), match_tiers() (+1 more)

### Community 141 - "super"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 142 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 143 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 145 - "MenuProbeRow"
Cohesion: 0.50
Nodes (3): menu_probe_all(), MenuProbeRow, notif_probe()

### Community 146 - "MenuAction"
Cohesion: 0.22
Nodes (9): MenuAction, Open, Pin, Quit, SetGlass, ToggleAutoHide, ToggleBlur, ToggleReserve (+1 more)

### Community 147 - "lib.rs"
Cohesion: 0.10
Nodes (5): dock_backdrop(), launcher_backdrop(), mc_backdrop(), menubar_current(), panel_backdrop()

### Community 148 - "merge"
Cohesion: 0.17
Nodes (18): apply_run_order(), cached_icon(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows() (+10 more)

### Community 149 - "test-settings.ps1"
Cohesion: 0.36
Nodes (4): All-S(), Js(), Set-S(), Val()

### Community 150 - "Pinned"
Cohesion: 0.24
Nodes (13): Pinned, last_list(), load(), load_full(), Loaded, modify(), move_to(), remember() (+5 more)

### Community 151 - "enable"
Cohesion: 0.29
Nodes (14): a_missing_or_relative_exe_is_refused_and_nothing_is_written(), a_task_manager_block_counts_as_off_and_turning_on_clears_it(), blocked_by_user(), clean(), delete_value(), disable(), enable(), enable_and_disable_touch_only_our_value() (+6 more)

### Community 152 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 153 - "settings_set"
Cohesion: 0.67
Nodes (3): SetReply, settings_action(), settings_set()

### Community 158 - "test-settings-s4.ps1"
Cohesion: 0.24
Nodes (4): Inv(), Js(), Val(), Win()

### Community 159 - "run"
Cohesion: 0.60
Nodes (5): cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), run()

### Community 164 - "Kind"
Cohesion: 0.25
Nodes (8): Kind, Action, Bool, Choice, Folders, Hotkey, Names, Pinned

### Community 166 - "AppMenus.tsx"
Cohesion: 0.31
Nodes (8): App(), AppMenus(), rectArgs(), Clock(), formatClock(), msToNextMinute(), acceptRow(), isOpenable()

### Community 167 - "validate"
Cohesion: 0.32
Nodes (7): clean_list(), folders_must_be_absolute_and_safe(), name_lists_are_cleaned(), valid_folder(), validate(), List, Text

### Community 169 - "test-settings-s6.ps1"
Cohesion: 0.39
Nodes (5): Active(), Heading(), Js(), Key(), Win()

### Community 170 - "handle"
Cohesion: 0.33
Nodes (7): c_cfg(), c_send(), handle(), Msg, Release, Run, Swallowed

### Community 172 - "merge"
Cohesion: 0.50
Nodes (8): background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), pin(), run(), Running, versioned_subfolder_matches_the_pinned_launcher()

### Community 173 - "run_child"
Cohesion: 0.29
Nodes (7): down(), is_extended(), release_after_neutral(), run_child(), say(), a_lost_alt_release_is_noticed_by_the_safety_net(), stuck()

### Community 174 - "Cmd"
Cohesion: 0.16
Nodes (13): Action, App, Eos, action_paths_and_flags_survive_normalization(), Cmd, Close, Maximize, Minimize (+5 more)

### Community 178 - "Dir"
Cohesion: 0.33
Nodes (6): Dir, Left, Right, plan_switch(), switch_to(), switching_plans_the_right_number_of_steps()

### Community 180 - "best_bar"
Cohesion: 0.40
Nodes (3): best_bar(), is_system_bar(), the_title_bar_system_menu_is_not_the_app_menu()

### Community 182 - "Inflight"
Cohesion: 0.40
Nodes (3): abandoned_reads_are_capped(), Inflight, may_start_read()

### Community 184 - "Held"
Cohesion: 0.50
Nodes (4): Held, Alt, None, Win

### Community 186 - "Centro notifiche e calendario — specifica"
Cohesion: 0.20
Nodes (9): Centro notifiche e calendario — specifica, Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisioni che servono da te, Fasi, Fuori scopo, N1 — Prova di fattibilità: FATTA (8 nov 2026) — **le notifiche si possono leggere**, Principi (gli stessi di sempre) (+1 more)

### Community 187 - "win_ctrl"
Cohesion: 0.38
Nodes (5): create(), key_event(), key_events_are_well_formed(), remove_current(), win_ctrl()

### Community 189 - "Phase"
Cohesion: 0.29
Nodes (7): Diag, diag(), Phase, Closed, Closing, Open, Opening

## Knowledge Gaps
- **113 isolated node(s):** `eosos`, `Error`, `Usable`, `NeedsConsent`, `NeedsPackageIdentity` (+108 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 922 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **62 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockact.rs` to `dock.rs`, `dockstore.rs`, `merge`, `collect_window`, `merge`, `Pinned`, `package.json`?**
  _High betweenness centrality (0.174) - this node is a cross-community bridge._
- **What connects `eosos`, `Error`, `Usable` to the rest of the system?**
  _113 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockact.rs`?**
  _High betweenness centrality (0.172) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `Dock.tsx` to `AppMenus.tsx`, `indicators.ts`, `Settings.tsx`, `menubar.ts`, `package.json`, `tokens.test.ts`, `Mc.tsx`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06504494976203067 - nodes in this community are weakly interconnected._