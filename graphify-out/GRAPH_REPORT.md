# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 119 files · ~148,766 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 2521 nodes · 5409 edges · 141 communities (103 shown, 38 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 341 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `62c435db`
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
- menumodel.rs
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
- collect_window
- indicators.rs
- launcher.ts
- launcherrank.rs
- launcherwin.rs
- launcher.rs
- reconcile
- launcherclip.rs
- menuread.rs
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
- Dock.tsx
- vdesk.rs
- test-panel.ps1
- test-mc-overlay.ps1
- indicatorstore.rs
- util.rs
- test-indicators.ps1
- .new
- test-indicators-settings.ps1
- test-mc-desktops.ps1
- Indicatori di sistema — specifica
- AppHandle
- mctrigger.rs
- read_volume
- key
- test-vdesk-internal.ps1
- Mission Control — specifica
- AppHandle
- Mc.tsx
- start
- test-mc-trigger.ps1
- show
- capture_rect
- test-mc-windows.ps1
- launcherstore.rs
- Hotkey
- place
- on_window_event
- mclayout.rs
- test-mc-keys.ps1
- battery_icon
- mcwin.rs
- Phase
- Clock.tsx
- Mutex
- Cmd
- AtomicU64
- History
- Rect
- run_search
- pathbuf
- Menu globale per app — specifica
- MenuAction
- LPARAM
- Config
- snapshot
- useBackdrop.ts
- handle
- run_child
- Dock — specifica
- rank_hits
- TimedCache
- Held
- files

## God Nodes (most connected - your core abstractions)
1. `Rect` - 61 edges
2. `Pinned` - 25 edges
3. `st()` - 25 edges
4. `layout()` - 23 edges
5. `Dock()` - 19 edges
6. `Profile` - 18 edges
7. `open()` - 18 edges
8. `same_app()` - 17 edges
9. `compilerOptions` - 17 edges
10. `select()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts
- `Fasi` --references--> `DockSettings`  [INFERRED]
  docs/DOCK.md → src/lib/dock.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (141 total, 38 thin omitted)

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
Nodes (48): anchor(), anchor_to_physical(), clamp_height(), click_decisions(), create(), decide(), EDGE, EXCLUDED (+40 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.07
Nodes (11): CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name(), READY_TIMEOUT (+3 more)

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
Cohesion: 0.15
Nodes (10): activate(), activate_or_launch(), focus(), launch(), minimizes_only_when_the_app_is_already_in_front(), PREFIX, quit(), run_menu_action() (+2 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (27): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+19 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.07
Nodes (21): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, is_known(), key(), KNOWN, KNOWN_CAP, known_folder() (+13 more)

### Community 28 - "menumodel.rs"
Cohesion: 0.11
Nodes (37): Action, App, Eos, action_paths_and_flags_survive_normalization(), an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), app_menu(), app_menu_names_the_app_and_never_is_blank(), Cmd (+29 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.04
Nodes (20): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent() (+12 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (67): Pinned, same_app(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips() (+59 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.08
Nodes (15): HOTKEY_THREAD, C_CFG, C_STATE, C_SWALLOWED, C_TX, CFG, CHILD_IN, CREATE_NO_WINDOW (+7 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (28): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+20 more)

### Community 42 - "items_basic"
Cohesion: 0.21
Nodes (18): apply_run_order(), background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows() (+10 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.07
Nodes (45): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+37 more)

### Community 44 - "lib.rs"
Cohesion: 0.05
Nodes (36): cleanup(), dock_backdrop(), dock_fit(), dock_items(), dock_menu(), dock_settings(), dock_settings_menu(), enable_taskbar_hiding() (+28 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.08
Nodes (35): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+27 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (17): AUTOHIDE, BLUR, build_region(), CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES (+9 more)

### Community 47 - "collect_window"
Cohesion: 0.11
Nodes (26): app_windows_for(), aumid_from_store(), aumid_path(), collect_window(), ComScope, identity(), is_app_window(), is_packaged_path() (+18 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (16): duration_label(), duration_labels(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume (+8 more)

### Community 49 - "launcher.ts"
Cohesion: 0.09
Nodes (39): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+31 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.13
Nodes (34): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+26 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (20): Change, EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, HOTKEY_ID, LABEL, MAX_HEIGHT, MIN_HEIGHT (+12 more)

### Community 54 - "launcher.rs"
Cohesion: 0.09
Nodes (32): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+24 more)

### Community 55 - "reconcile"
Cohesion: 0.19
Nodes (14): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), create(), cursor(), enforce_styles(), fit(), hwnd_of() (+6 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.06
Nodes (17): age_label(), age_labels(), CF_UNICODETEXT, format_id(), HISTORY, MAX_CHARS, MAX_ENTRIES, password_manager_copies_are_not_recorded() (+9 more)

### Community 60 - "menuread.rs"
Cohesion: 0.07
Nodes (17): classification_covers_every_case(), classify(), is_system_bar(), MIN_UIA_MENU_ITEMS, p(), Probe, probe(), read_hmenu() (+9 more)

### Community 61 - "mc.rs"
Cohesion: 0.09
Nodes (34): a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP, geometry() (+26 more)

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
Cohesion: 0.11
Nodes (17): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+9 more)

### Community 68 - "mckeys.rs"
Cohesion: 0.20
Nodes (22): alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle(), cycling_starts_from_the_second_window_and_wraps(), disabled_shortcuts_are_never_touched(), escape_cancels_and_a_later_alt_release_does_not_commit(), is_alt(), Key (+14 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.08
Nodes (12): detect_restart(), hide(), restore(), explorer_restart_detection(), FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding() (+4 more)

### Community 72 - "indicators.ts"
Cohesion: 0.18
Nodes (26): I3 — Pannello: FATTA (7 nov 2026), @tauri-apps/api, App(), Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails() (+18 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "Dock.tsx"
Cohesion: 0.21
Nodes (23): Fasi, cssPx(), Dock(), Ghost(), Icon(), applyMove(), clampGlass(), DockItem (+15 more)

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
Cohesion: 0.12
Nodes (11): app_display_name(), cached_icon(), cached_shell_name(), parse_aumid(), shell_display_name(), cached_description(), file_description(), CACHE_FOUND_TTL (+3 more)

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - ".new"
Cohesion: 0.33
Nodes (4): MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip()

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
Cohesion: 0.17
Nodes (17): alt_or_win_down(), mc_windows(), McMonitor, McWindow, create_one(), fit_exact(), hwnd_of(), icons_for() (+9 more)

### Community 86 - "mctrigger.rs"
Cohesion: 0.13
Nodes (24): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+16 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "key"
Cohesion: 0.33
Nodes (7): is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon(), processes_named()

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Mission Control — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+5 more)

### Community 92 - "AppHandle"
Cohesion: 0.17
Nodes (24): activate(), all_labels(), alt_commit(), alt_step(), backdrop(), close(), close_animated(), close_window() (+16 more)

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
Cohesion: 0.27
Nodes (11): apply_styles(), create(), fit(), hide(), is_visible(), show(), start(), target_monitor() (+3 more)

### Community 98 - "capture_rect"
Cohesion: 0.28
Nodes (9): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur(), capture_info() (+1 more)

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.11
Nodes (40): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+32 more)

### Community 101 - "Hotkey"
Cohesion: 0.23
Nodes (14): arrow_keys_work_in_hotkeys(), change_hotkey(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey (+6 more)

### Community 102 - "place"
Cohesion: 0.33
Nodes (6): clamp_height(), clamp_height_limits_and_survives_garbage(), place(), place_centers_horizontally_and_sits_a_fifth_down(), place_never_leaves_the_work_area(), place_scales_with_dpi_and_follows_the_monitor()

### Community 103 - "on_window_event"
Cohesion: 0.33
Nodes (4): focus_loss_counts(), focus_loss_is_ignored_only_right_after_opening(), on_window_event(), since_start()

### Community 104 - "mclayout.rs"
Cohesion: 0.23
Nodes (26): a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), best_scale(), check_invariants(), empty_and_degenerate_input(), gaps_are_respected_when_there_is_room(), hit_test(), hit_testing_finds_the_cell_under_the_pointer() (+18 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 107 - "mcwin.rs"
Cohesion: 0.08
Nodes (12): Anim, APP, CORNER, DEFAULT_HOTKEY, FRAME_STYLES, HOTKEY_ID, PREFIX, REFRESH_AFTER (+4 more)

### Community 108 - "Phase"
Cohesion: 0.29
Nodes (7): Diag, diag(), Phase, Closed, Closing, Open, Opening

### Community 109 - "Clock.tsx"
Cohesion: 0.73
Nodes (3): Clock(), formatClock(), msToNextMinute()

### Community 111 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 114 - "AtomicU64"
Cohesion: 0.50
Nodes (3): SEQ, SCALE_BITS, SHOWN_AT_MS

### Community 116 - "History"
Cohesion: 0.15
Nodes (16): capped_at_max_entries_dropping_the_oldest(), clear(), copy_back(), disabling_clears_the_global_history(), enabled(), entries_expire_after_the_ttl(), Entry, hist() (+8 more)

### Community 117 - "Rect"
Cohesion: 0.14
Nodes (16): Rect, inside(), thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), AREA, css_conversion_is_relative_to_the_monitor_and_scaled(), fit_in(), monitor_of() (+8 more)

### Community 119 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 124 - "Menu globale per app — specifica"
Cohesion: 0.20
Nodes (9): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, G1 — Misura in sola lettura: FATTA (8 nov 2026), G2 — Modello e disposizione (puri): FATTA (8 nov 2026), Menu globale per app — specifica (+1 more)

### Community 125 - "MenuAction"
Cohesion: 0.17
Nodes (13): foreign_or_malformed_menu_ids_are_ignored(), menu_id(), menu_id_roundtrip_with_colons_in_the_id(), MenuAction, Open, Pin, Quit, SetGlass (+5 more)

### Community 126 - "LPARAM"
Cohesion: 0.31
Nodes (3): wndproc(), proc(), wndproc()

### Community 127 - "Config"
Cohesion: 0.24
Nodes (12): cfg(), config_line(), config_lines_round_trip_and_garbage_is_ignored(), Diag, diag(), may_restart(), parse_config_line(), send_cfg() (+4 more)

### Community 130 - "snapshot"
Cohesion: 0.28
Nodes (7): monitors(), snapshot(), button_down(), corner_thread(), hotkey_thread(), keys_config(), start()

### Community 131 - "useBackdrop.ts"
Cohesion: 0.44
Nodes (5): useBackdrop(), Frame, FRAME_HEADER, parseFrame(), sourceRect()

### Community 134 - "handle"
Cohesion: 0.33
Nodes (7): c_cfg(), c_send(), handle(), Msg, Release, Run, Swallowed

### Community 135 - "run_child"
Cohesion: 0.29
Nodes (7): down(), is_extended(), release_after_neutral(), run_child(), say(), a_lost_alt_release_is_noticed_by_the_safety_net(), stuck()

### Community 136 - "Dock — specifica"
Cohesion: 0.33
Nodes (5): Comportamento (da replicare), Dock — specifica, Fuori scopo, Rischi aperti, Scelte tecniche

### Community 137 - "rank_hits"
Cohesion: 0.53
Nodes (6): hit(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders(), rank_hits_respects_limit_and_empty()

### Community 138 - "TimedCache"
Cohesion: 0.67
Nodes (3): ICONS, SHELL_NAMES, TimedCache

### Community 139 - "Held"
Cohesion: 0.50
Nodes (4): Held, Alt, None, Win

### Community 140 - "files"
Cohesion: 0.67
Nodes (3): files(), short_dir(), short_dir_uses_home_and_elides_long_paths()

## Knowledge Gaps
- **83 isolated node(s):** `Quit`, `Minimize`, `Maximize`, `Restore`, `Close` (+78 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 738 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **38 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `dock.rs`, `items_basic`, `collect_window`, `dockact.rs`, `package.json`, `key`?**
  _High betweenness centrality (0.128) - this node is a cross-community bridge._
- **What connects `Quit`, `Minimize`, `Maximize` to the rest of the system?**
  _83 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.127) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `useBackdrop.ts`, `main.tsx`, `indicators.ts`, `Dock.tsx`, `Clock.tsx`, `launcher.ts`, `tokens.test.ts`, `Mc.tsx`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0597567424643046 - nodes in this community are weakly interconnected._