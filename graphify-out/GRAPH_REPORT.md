# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 115 files · ~143,363 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 2432 nodes · 5270 edges · 139 communities (103 shown, 36 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 332 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0bc2d74e`
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
- launcherstore.rs
- reconcile
- launcherclip.rs
- fold
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
- .new
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
- Dock.tsx
- Hotkey
- mclayout.rs
- Rect
- test-mc-keys.ps1
- rank_hits
- mcwin.rs
- Phase
- @tauri-apps/api
- Indicators
- Cmd
- wndproc
- MenuAction
- reconcile
- pathbuf
- place
- on_window_event
- AtomicU64
- run
- useBackdrop.ts
- hist
- snapshot
- Dock — specifica
- Response
- list
- battery_icon
- AtomicIsize
- Toggle

## God Nodes (most connected - your core abstractions)
1. `Rect` - 61 edges
2. `st()` - 25 edges
3. `Pinned` - 25 edges
4. `layout()` - 23 edges
5. `Dock()` - 19 edges
6. `open()` - 18 edges
7. `Profile` - 18 edges
8. `same_app()` - 17 edges
9. `compilerOptions` - 17 edges
10. `select()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts
- `Fasi` --references--> `DockSettings`  [INFERRED]
  docs/DOCK.md → src/lib/dock.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (139 total, 36 thin omitted)

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
Nodes (42): anchor(), clamp_height(), click_decisions(), create(), decide(), EDGE, EXCLUDED, fit() (+34 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.07
Nodes (12): spawn_child(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name() (+4 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.07
Nodes (23): capture_bytes(), Channel, Dock, Launcher, Panel, CHANNELS, downscale(), downscale_averages_blocks_and_swaps_to_rgba() (+15 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.12
Nodes (16): appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), find(), hide_matching(), hide_real() (+8 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (41): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+33 more)

### Community 20 - "foreground.rs"
Cohesion: 0.09
Nodes (24): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+16 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.12
Nodes (15): activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id_roundtrip_with_colons_in_the_id(), minimizes_only_when_the_app_is_already_in_front() (+7 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (26): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+18 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.07
Nodes (21): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, is_known(), key(), KNOWN, KNOWN_CAP, known_folder() (+13 more)

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
Cohesion: 0.05
Nodes (19): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent() (+11 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (62): Pinned, pinned_dir(), read_pinned(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files() (+54 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.06
Nodes (34): C_CFG, c_cfg(), c_send(), C_STATE, C_TX, cfg(), CFG, CHILD_IN (+26 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (28): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+20 more)

### Community 42 - "items_basic"
Cohesion: 0.15
Nodes (17): app_display_name(), apply_run_order(), cached_shell_name(), ComScope, DockItem, dump_dock(), find(), find_with_windows() (+9 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.07
Nodes (45): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+37 more)

### Community 44 - "lib.rs"
Cohesion: 0.07
Nodes (26): menu_id(), dock_fit(), dock_items(), dock_menu(), dock_settings(), dock_settings_menu(), get_active_app(), indicators_menu() (+18 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.08
Nodes (37): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, cancel_pending(), commands_are_fixed_and_exact(), CONFIRM_WINDOW (+29 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (14): AUTOHIDE, BLUR, CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES, RECONCILE_EVERY (+6 more)

### Community 47 - "HWND"
Cohesion: 0.13
Nodes (22): aumid_from_store(), aumid_path(), collect_window(), identity(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized() (+14 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (14): Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume, only_known_indicators_open_a_settings_page(), open_settings() (+6 more)

### Community 49 - "launcher.ts"
Cohesion: 0.09
Nodes (39): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+31 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.16
Nodes (28): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), frecency(), frecency_breaks_ties_but_cannot_beat_a_much_better_match() (+20 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (20): Change, EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, HOTKEY_ID, LABEL, MAX_HEIGHT, MIN_HEIGHT (+12 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.05
Nodes (71): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+63 more)

### Community 55 - "reconcile"
Cohesion: 0.18
Nodes (10): slide_covers_the_whole_travel_with_ease_out(), slide_y(), set_autohide(), autohide(), build_region(), cursor(), enforce_styles(), hwnd_of() (+2 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.06
Nodes (6): CF_UNICODETEXT, MAX_CHARS, MAX_ENTRIES, start(), T0, TTL_MS

### Community 60 - "fold"
Cohesion: 0.25
Nodes (9): files(), short_dir(), short_dir_uses_home_and_elides_long_paths(), fold(), fold_lowercases_and_strips_common_accents(), initials(), initials_cover_words_and_inner_capitals(), match_tiers() (+1 more)

### Community 61 - "mc.rs"
Cohesion: 0.09
Nodes (34): a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP, geometry() (+26 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.05
Nodes (73): mc_vd_report(), a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), capabilities_follow_the_selection_all_or_nothing() (+65 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 66 - "mcstore.rs"
Cohesion: 0.13
Nodes (29): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+21 more)

### Community 67 - "main.tsx"
Cohesion: 0.13
Nodes (15): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+7 more)

### Community 68 - "mckeys.rs"
Cohesion: 0.15
Nodes (28): a_lost_alt_release_is_noticed_by_the_safety_net(), alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle(), cycling_starts_from_the_second_window_and_wraps(), disabled_shortcuts_are_never_touched(), escape_cancels_and_a_later_alt_release_does_not_commit(), Held (+20 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.08
Nodes (18): HOTKEY_THREAD, C_SWALLOWED, proc(), SWALLOWED, detect_restart(), hide(), restore(), COUNTER (+10 more)

### Community 72 - "indicators.ts"
Cohesion: 0.20
Nodes (24): I3 — Pannello: FATTA (7 nov 2026), Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails(), batteryFill(), DetailRow (+16 more)

### Community 73 - "Indicators"
Cohesion: 0.28
Nodes (8): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), duration_label(), duration_labels(), read_battery(), serialized_state_is_flat_for_the_page()

### Community 74 - "History"
Cohesion: 0.19
Nodes (8): capped_at_max_entries_dropping_the_oldest(), entries_expire_after_the_ttl(), Entry, History, HISTORY, newest_first_and_duplicates_move_up_without_growing(), remove(), search_requires_every_word_ignoring_case_and_accents()

### Community 75 - "vdesk.rs"
Cohesion: 0.06
Nodes (32): mc_desktops(), an_unknown_or_missing_current_desktop_falls_back_to_the_first(), BYTES, create(), current_index(), Desktop, desktops_from(), Dir (+24 more)

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "test-mc-overlay.ps1"
Cohesion: 0.20
Nodes (13): Cell-Info(), CellJs(), Esc-If-Open(), Inv(), IsOpen(), Js(), Mc(), McIdx() (+5 more)

### Community 78 - "indicatorstore.rs"
Cohesion: 0.10
Nodes (36): Read, Failed, Missing, a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh (+28 more)

### Community 79 - "util.rs"
Cohesion: 0.14
Nodes (8): cached_icon(), ICONS, SHELL_NAMES, CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh(), TimedCache

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - ".new"
Cohesion: 0.40
Nodes (3): MtaScope, real_set_volume(), real_volume_roundtrip()

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
Cohesion: 0.18
Nodes (16): mc_windows(), McMonitor, McWindow, create_one(), fit_exact(), hwnd_of(), icons_for(), label() (+8 more)

### Community 86 - "mctrigger.rs"
Cohesion: 0.13
Nodes (24): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+16 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "same_app"
Cohesion: 0.24
Nodes (11): name, app_windows_for(), is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer(), may_inherit(), only_a_single_version_folder_counts_as_the_same_app() (+3 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Mission Control — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+5 more)

### Community 92 - "AppHandle"
Cohesion: 0.20
Nodes (22): activate(), all_labels(), alt_commit(), alt_step(), close(), close_animated(), close_window(), hwnd_from() (+14 more)

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
Cohesion: 0.26
Nodes (12): apply_styles(), capture_info(), create(), fit(), hide(), is_visible(), show(), start() (+4 more)

### Community 98 - "capture_rect"
Cohesion: 0.25
Nodes (9): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), set_blur(), blur_enabled(), capture_info(), toggle_blur() (+1 more)

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "Dock.tsx"
Cohesion: 0.21
Nodes (23): Fasi, cssPx(), Dock(), Ghost(), Icon(), applyMove(), clampGlass(), DockItem (+15 more)

### Community 101 - "Hotkey"
Cohesion: 0.23
Nodes (15): arrow_keys_work_in_hotkeys(), change_hotkey(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey (+7 more)

### Community 102 - "mclayout.rs"
Cohesion: 0.23
Nodes (26): a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), best_scale(), check_invariants(), empty_and_degenerate_input(), gaps_are_respected_when_there_is_room(), hit_test(), hit_testing_finds_the_cell_under_the_pointer() (+18 more)

### Community 103 - "Rect"
Cohesion: 0.12
Nodes (18): Rect, inside(), thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), AREA, css_conversion_is_relative_to_the_monitor_and_scaled(), fit_in(), monitor_of() (+10 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "rank_hits"
Cohesion: 0.53
Nodes (6): hit(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders(), rank_hits_respects_limit_and_empty()

### Community 107 - "mcwin.rs"
Cohesion: 0.07
Nodes (13): Anim, APP, CORNER, DEFAULT_HOTKEY, FRAME_STYLES, HOTKEY_ID, PREFIX, recent_order() (+5 more)

### Community 108 - "Phase"
Cohesion: 0.29
Nodes (7): Diag, diag(), Phase, Closed, Closing, Open, Opening

### Community 109 - "@tauri-apps/api"
Cohesion: 0.30
Nodes (7): @tauri-apps/api, App(), ActiveApp, ActiveAppName(), Clock(), formatClock(), msToNextMinute()

### Community 110 - "Indicators"
Cohesion: 0.22
Nodes (9): current(), Indicators, read_fake(), read_once(), read_real(), real_read_smoke(), STATE, WAKE (+1 more)

### Community 111 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 114 - "wndproc"
Cohesion: 0.22
Nodes (8): copy_back(), format_id(), password_manager_copies_are_not_recorded(), read_current(), read_global_u32(), should_record(), wndproc(), write_text()

### Community 117 - "MenuAction"
Cohesion: 0.22
Nodes (9): MenuAction, Open, Pin, Quit, SetGlass, ToggleAutoHide, ToggleBlur, ToggleReserve (+1 more)

### Community 119 - "reconcile"
Cohesion: 0.27
Nodes (9): set_reserve(), create(), fit(), lock(), reconcile(), reserve_allowed(), reserve_enabled(), start() (+1 more)

### Community 124 - "place"
Cohesion: 0.33
Nodes (6): clamp_height(), clamp_height_limits_and_survives_garbage(), place(), place_centers_horizontally_and_sits_a_fifth_down(), place_never_leaves_the_work_area(), place_scales_with_dpi_and_follows_the_monitor()

### Community 125 - "on_window_event"
Cohesion: 0.33
Nodes (4): focus_loss_counts(), focus_loss_is_ignored_only_right_after_opening(), on_window_event(), since_start()

### Community 126 - "AtomicU64"
Cohesion: 0.50
Nodes (3): SEQ, SCALE_BITS, SHOWN_AT_MS

### Community 127 - "run"
Cohesion: 0.24
Nodes (9): set_mc_corner(), set_mc_keys(), set_mc_open(), cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), on_ctrl() (+1 more)

### Community 128 - "useBackdrop.ts"
Cohesion: 0.44
Nodes (5): useBackdrop(), Frame, FRAME_HEADER, parseFrame(), sourceRect()

### Community 129 - "hist"
Cohesion: 0.32
Nodes (6): clear(), disabling_clears_the_global_history(), enabled(), hist(), remove_and_clear(), set_enabled()

### Community 130 - "snapshot"
Cohesion: 0.32
Nodes (6): monitors(), snapshot(), button_down(), corner_thread(), keys_config(), start()

### Community 131 - "Dock — specifica"
Cohesion: 0.33
Nodes (5): Comportamento (da replicare), Dock — specifica, Fuori scopo, Rischi aperti, Scelte tecniche

### Community 132 - "Response"
Cohesion: 0.33
Nodes (5): dock_backdrop(), launcher_backdrop(), mc_backdrop(), panel_backdrop(), backdrop()

### Community 133 - "list"
Cohesion: 0.33
Nodes (6): age_label(), age_labels(), list(), now_ms(), preview(), preview_takes_the_first_meaningful_line_and_truncates()

### Community 134 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 135 - "AtomicIsize"
Cohesion: 0.50
Nodes (3): LAST_HWND, OWNER, HWND_RAW

### Community 136 - "Toggle"
Cohesion: 0.50
Nodes (4): Toggle, Hide, Ignore, Show

## Knowledge Gaps
- **72 isolated node(s):** `Open`, `Quit`, `Pin`, `Unpin`, `ToggleAutoHide` (+67 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 690 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **36 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `same_app` to `dock.rs`, `dockstore.rs`, `items_basic`, `HWND`, `dockact.rs`, `merge`?**
  _High betweenness centrality (0.142) - this node is a cross-community bridge._
- **What connects `Open`, `Quit`, `Pin` to the rest of the system?**
  _72 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `same_app` to `package.json`?**
  _High betweenness centrality (0.140) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `@tauri-apps/api` connect `@tauri-apps/api` to `useBackdrop.ts`, `main.tsx`, `Dock.tsx`, `indicators.ts`, `launcher.ts`, `package.json`, `Mc.tsx`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06818181818181818 - nodes in this community are weakly interconnected._