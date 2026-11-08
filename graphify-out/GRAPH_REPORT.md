# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 116 files · ~144,333 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 2440 nodes · 5277 edges · 136 communities (99 shown, 37 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 332 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2f13e65c`
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
- glass.rs
- tokens.test.ts
- launcherfiles.rs
- merge
- EosOS
- Checklist di test manuali
- dock.rs
- dockstore.rs
- launchercalc.rs
- mchook.rs
- dockgeom.rs
- items_basic
- test-dock-scales.ps1
- lib.rs
- launcheract.rs
- dockwin.rs
- HWND
- indicators.rs
- launcher.ts
- launcherrank.rs
- launcherwin.rs
- launcher.rs
- reconcile
- launcherclip.rs
- files
- mc.rs
- NetKind
- vdint.rs
- test-mc-vdesk.ps1
- mcstore.rs
- main.tsx
- mckeys.rs
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
- History
- vdesk.rs
- test-panel.ps1
- test-mc-overlay.ps1
- indicatorstore.rs
- util.rs
- test-indicators.ps1
- set_volume
- test-indicators-settings.ps1
- test-mc-desktops.ps1
- Indicatori di sistema — specifica
- AppHandle
- mctrigger.rs
- read_volume
- same_app
- test-vdesk-internal.ps1
- Mission Control — specifica
- AppHandle
- run_search
- Mc.tsx
- start
- test-mc-trigger.ps1
- show
- capture_rect
- test-mc-windows.ps1
- launcherstore.rs
- Hotkey
- State
- show
- test-mc-keys.ps1
- rank_hits
- mcwin.rs
- Phase
- Clock.tsx
- Mutex
- Cmd
- rank
- MenuAction
- theme.ts
- pathbuf
- Menu globale per app — specifica
- Config
- handle
- run
- run_child
- Outcome
- start
- State
- Run
- Held

## God Nodes (most connected - your core abstractions)
1. `Rect` - 61 edges
2. `Pinned` - 25 edges
3. `st()` - 25 edges
4. `layout()` - 23 edges
5. `Dock()` - 19 edges
6. `Profile` - 18 edges
7. `open()` - 18 edges
8. `compilerOptions` - 17 edges
9. `select()` - 17 edges
10. `selftest()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `Fasi` --references--> `DockSettings`  [INFERRED]
  docs/DOCK.md → src/lib/dock.ts
- `I1 — Dati: FATTA (7 nov 2026)` --references--> `Battery`  [INFERRED]
  docs/INDICATORI.md → src/lib/indicators.ts
- `I1 — Dati: FATTA (7 nov 2026)` --references--> `Network`  [INFERRED]
  docs/INDICATORI.md → src/lib/indicators.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (136 total, 37 thin omitted)

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
Cohesion: 0.06
Nodes (36): anchor(), anchor_to_physical(), clamp_height(), click_decisions(), decide(), EDGE, EXCLUDED, FOCUS_CHECK (+28 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.08
Nodes (9): CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name(), READY_TIMEOUT (+1 more)

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
Cohesion: 0.07
Nodes (41): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+33 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (27): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+19 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.13
Nodes (15): activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id_roundtrip_with_colons_in_the_id(), minimizes_only_when_the_app_is_already_in_front() (+7 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (25): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+17 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.07
Nodes (15): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, KNOWN, KNOWN_CAP, known_folder(), LATEST, LIMIT (+7 more)

### Community 28 - "merge"
Cohesion: 0.50
Nodes (8): background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), pin(), run(), Running, versioned_subfolder_matches_the_pinned_launcher()

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.04
Nodes (22): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, aumid_from_store(), bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor() (+14 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (67): Pinned, same_app(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips() (+59 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.06
Nodes (18): HOTKEY_THREAD, C_CFG, C_STATE, C_SWALLOWED, C_TX, CFG, CHILD_IN, CREATE_NO_WINDOW (+10 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (28): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+20 more)

### Community 42 - "items_basic"
Cohesion: 0.17
Nodes (17): app_display_name(), apply_run_order(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows() (+9 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.07
Nodes (45): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+37 more)

### Community 44 - "lib.rs"
Cohesion: 0.06
Nodes (32): menu_id(), dock_backdrop(), dock_fit(), dock_items(), dock_menu(), dock_settings(), dock_settings_menu(), get_active_app() (+24 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.12
Nodes (28): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, cancel_pending(), commands_are_fixed_and_exact(), CONFIRM_WINDOW (+20 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (14): AUTOHIDE, BLUR, build_region(), CHROME_STYLES, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES, RECONCILE_EVERY (+6 more)

### Community 47 - "HWND"
Cohesion: 0.14
Nodes (21): aumid_path(), collect_window(), identity(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized(), process_aumid() (+13 more)

### Community 48 - "indicators.rs"
Cohesion: 0.10
Nodes (15): battery_icon(), battery_icon_levels_and_states(), duration_label(), duration_labels(), every_icon_rust_can_emit_is_one_the_page_knows(), Fake, FakeBattery, FakeNetwork (+7 more)

### Community 49 - "launcher.ts"
Cohesion: 0.05
Nodes (74): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026) (+66 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.15
Nodes (24): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, fold(), fold_lowercases_and_strips_common_accents(), frecency(), frecency_grows_slowly_with_use_and_decays_with_age() (+16 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.06
Nodes (31): SEQ, Change, clamp_height(), clamp_height_limits_and_survives_garbage(), EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, focus_loss_counts() (+23 more)

### Community 54 - "launcher.rs"
Cohesion: 0.09
Nodes (30): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+22 more)

### Community 55 - "reconcile"
Cohesion: 0.19
Nodes (14): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), create(), cursor(), enforce_styles(), fit(), hwnd_of() (+6 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.06
Nodes (17): age_label(), age_labels(), CF_UNICODETEXT, format_id(), HISTORY, MAX_CHARS, MAX_ENTRIES, password_manager_copies_are_not_recorded() (+9 more)

### Community 60 - "files"
Cohesion: 0.67
Nodes (3): files(), short_dir(), short_dir_uses_home_and_elides_long_paths()

### Community 61 - "mc.rs"
Cohesion: 0.06
Nodes (75): Rect, a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP (+67 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.06
Nodes (73): a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), build_info(), capabilities_follow_the_selection_all_or_nothing() (+65 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 66 - "mcstore.rs"
Cohesion: 0.13
Nodes (29): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+21 more)

### Community 67 - "main.tsx"
Cohesion: 0.14
Nodes (16): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+8 more)

### Community 68 - "mckeys.rs"
Cohesion: 0.20
Nodes (22): alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle(), cycling_starts_from_the_second_window_and_wraps(), disabled_shortcuts_are_never_touched(), escape_cancels_and_a_later_alt_release_does_not_commit(), is_alt(), Key (+14 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.09
Nodes (14): proc(), detect_restart(), hide(), restore(), explorer_restart_detection(), FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip() (+6 more)

### Community 72 - "indicators.ts"
Cohesion: 0.21
Nodes (22): I3 — Pannello: FATTA (7 nov 2026), Glyph(), Panel(), batteryDetails(), batteryFill(), DetailRow, IndicatorItem, indicatorItems() (+14 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "History"
Cohesion: 0.15
Nodes (17): capped_at_max_entries_dropping_the_oldest(), clear(), copy_back(), disabling_clears_the_global_history(), enabled(), entries_expire_after_the_ttl(), Entry, hist() (+9 more)

### Community 75 - "vdesk.rs"
Cohesion: 0.06
Nodes (31): mc_desktops(), an_unknown_or_missing_current_desktop_falls_back_to_the_first(), BYTES, create(), current_index(), Desktop, desktops_from(), Dir (+23 more)

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "test-mc-overlay.ps1"
Cohesion: 0.20
Nodes (13): Cell-Info(), CellJs(), Esc-If-Open(), Inv(), IsOpen(), Js(), Mc(), McIdx() (+5 more)

### Community 78 - "indicatorstore.rs"
Cohesion: 0.11
Nodes (33): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+25 more)

### Community 79 - "util.rs"
Cohesion: 0.14
Nodes (8): cached_icon(), ICONS, SHELL_NAMES, CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh(), TimedCache

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - "set_volume"
Cohesion: 0.18
Nodes (10): fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip(), sanitize_volume(), set_volume() (+2 more)

### Community 82 - "test-indicators-settings.ps1"
Cohesion: 0.20
Nodes (8): BarWin(), Choose(), Keys(), Label(), Menu-Element(), Menu-Items(), Open-Menu(), PJs()

### Community 83 - "test-mc-desktops.ps1"
Cohesion: 0.43
Nodes (5): Click-Chip(), Desktops(), Inv(), Js(), McIdx()

### Community 84 - "Indicatori di sistema — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti (+4 more)

### Community 85 - "AppHandle"
Cohesion: 0.21
Nodes (14): McMonitor, monitors(), snapshot(), create_one(), cursor_monitor(), fit_exact(), hwnd_of(), icons_for() (+6 more)

### Community 86 - "mctrigger.rs"
Cohesion: 0.12
Nodes (26): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+18 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "same_app"
Cohesion: 0.18
Nodes (11): app_windows_for(), cached_shell_name(), ComScope, is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer(), may_inherit() (+3 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Mission Control — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+5 more)

### Community 92 - "AppHandle"
Cohesion: 0.19
Nodes (24): activate(), all_labels(), alt_commit(), alt_step(), backdrop(), close(), close_animated(), close_window() (+16 more)

### Community 93 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 94 - "Mc.tsx"
Cohesion: 0.27
Nodes (17): Mc(), advancedHint(), boxStyle(), canRemove(), cardLetter(), cellTitle(), cleanDesktopName(), desktopAction() (+9 more)

### Community 95 - "start"
Cohesion: 0.25
Nodes (6): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "show"
Cohesion: 0.20
Nodes (13): capture_info(), create(), fit(), hide(), is_visible(), on_window_event(), show(), since_start() (+5 more)

### Community 98 - "capture_rect"
Cohesion: 0.29
Nodes (8): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur(), capture_info()

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.16
Nodes (20): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide_never_treats_a_failed_read_as_missing(), Entry, File, hotkey_and_search_folders_roundtrip_and_default(), LAST, load(), map() (+12 more)

### Community 101 - "Hotkey"
Cohesion: 0.27
Nodes (12): arrow_keys_work_in_hotkeys(), change_hotkey(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey (+4 more)

### Community 102 - "State"
Cohesion: 0.15
Nodes (21): set_clipboard(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+13 more)

### Community 103 - "show"
Cohesion: 0.21
Nodes (14): apply_styles(), create(), fit(), hide(), is_visible(), lock(), monitor_at(), on_window_event() (+6 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "rank_hits"
Cohesion: 0.26
Nodes (12): hit(), is_known(), key(), known_results_are_capped(), multi_word_queries_match_names_with_separators(), name_score(), only_remembered_paths_can_be_opened(), open() (+4 more)

### Community 107 - "mcwin.rs"
Cohesion: 0.07
Nodes (14): Anim, APP, CORNER, DEFAULT_HOTKEY, FRAME_STYLES, HOTKEY_ID, PREFIX, REFRESH_AFTER (+6 more)

### Community 108 - "Phase"
Cohesion: 0.18
Nodes (9): diag(), Diag, diag(), on_window_event(), Phase, Closed, Closing, Open (+1 more)

### Community 109 - "Clock.tsx"
Cohesion: 0.73
Nodes (3): Clock(), formatClock(), msToNextMinute()

### Community 111 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 114 - "rank"
Cohesion: 0.42
Nodes (10): empty_query_lists_only_used_apps_by_frecency(), entries(), frecency_breaks_ties_but_cannot_beat_a_much_better_match(), heavy_use_lifts_an_app_within_the_same_tier(), limit_and_empty_input_are_respected(), names(), rank(), rank_orders_by_match_then_frecency_then_length() (+2 more)

### Community 117 - "MenuAction"
Cohesion: 0.22
Nodes (9): MenuAction, Open, Pin, Quit, SetGlass, ToggleAutoHide, ToggleBlur, ToggleReserve (+1 more)

### Community 119 - "theme.ts"
Cohesion: 0.36
Nodes (5): applyThemePref(), initTheme(), isThemePref(), readThemePref(), ThemePref

### Community 124 - "Menu globale per app — specifica"
Cohesion: 0.25
Nodes (7): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, Menu globale per app — specifica, Rischi aperti

### Community 125 - "Config"
Cohesion: 0.43
Nodes (8): cfg(), config_line(), config_lines_round_trip_and_garbage_is_ignored(), parse_config_line(), send_cfg(), set_config(), start(), Config

### Community 126 - "handle"
Cohesion: 0.33
Nodes (7): c_cfg(), c_send(), handle(), Msg, Release, Run, Swallowed

### Community 127 - "run"
Cohesion: 0.32
Nodes (8): set_mc_corner(), set_mc_keys(), set_mc_open(), cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), run()

### Community 128 - "run_child"
Cohesion: 0.29
Nodes (7): down(), is_extended(), release_after_neutral(), run_child(), say(), a_lost_alt_release_is_noticed_by_the_safety_net(), stuck()

### Community 129 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Confirm, Done, Failed, Unknown

### Community 130 - "start"
Cohesion: 0.33
Nodes (5): configured_hotkey(), hotkey_thread(), keys_config(), start(), try_register()

### Community 131 - "State"
Cohesion: 0.50
Nodes (3): Geometry, State, STATE

### Community 132 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

### Community 133 - "Held"
Cohesion: 0.50
Nodes (4): Held, Alt, None, Win

## Knowledge Gaps
- **72 isolated node(s):** `Closed`, `Closing`, `Open`, `Opening`, `Dock` (+67 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 697 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **37 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `dock.rs`, `items_basic`, `HWND`, `dockact.rs`, `same_app`, `package.json`, `merge`?**
  _High betweenness centrality (0.140) - this node is a cross-community bridge._
- **What connects `Closed`, `Closing`, `Open` to the rest of the system?**
  _72 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.137) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `launcher.ts` to `indicators.ts`, `Clock.tsx`, `theme.ts`, `package.json`, `tokens.test.ts`, `Mc.tsx`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06105457909343201 - nodes in this community are weakly interconnected._