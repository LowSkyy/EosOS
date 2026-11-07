# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 100 files · ~111,038 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1955 nodes · 4092 edges · 117 communities (82 shown, 35 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 271 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f4610e85`
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
- icon_data_url
- dockgeom.rs
- items_basic
- test-dock-scales.ps1
- lib.rs
- launcheract.rs
- dockwin.rs
- HWND
- indicators.rs
- Dock.tsx
- launcherrank.rs
- launcherwin.rs
- launcherstore.rs
- reconcile
- launcherclip.rs
- search
- mc.rs
- NetKind
- Rect
- launcher.rs
- Launcher — specifica
- main.tsx
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
- scripts
- test-panel.ps1
- test-mc-overlay.ps1
- indicatorstore.rs
- util.rs
- test-indicators.ps1
- set_volume
- test-indicators-settings.ps1
- show
- Indicatori di sistema — specifica
- McMonitor
- window_aumid
- read_volume
- app_windows_for
- @tauri-apps/api
- Mission Control — specifica
- Channel
- Hotkey
- Mc.tsx
- start
- TimedCache
- aumid_path
- capture_rect
- test-mc-windows.ps1
- place
- on_window_event
- Run
- reconcile
- run
- thumb_rect
- mcwin.rs
- PENDING_CHANGE
- dock_items
- Mutex
- AtomicU32

## God Nodes (most connected - your core abstractions)
1. `Rect` - 53 edges
2. `Pinned` - 25 edges
3. `layout()` - 23 edges
4. `Dock()` - 19 edges
5. `same_app()` - 17 edges
6. `compilerOptions` - 17 edges
7. `item()` - 16 edges
8. `Launcher()` - 16 edges
9. `items_and_windows()` - 16 edges
10. `build()` - 15 edges

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

## Communities (117 total, 35 thin omitted)

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
Cohesion: 0.05
Nodes (49): anchor(), anchor_to_physical(), clamp_height(), click_decisions(), create(), decide(), EDGE, EXCLUDED (+41 more)

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
Cohesion: 0.08
Nodes (17): capture_bytes(), downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input(), downscale_keeps_channel_order_per_block(), encode(), FACTOR, Frame (+9 more)

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
Cohesion: 0.08
Nodes (26): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+18 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.09
Nodes (23): activate(), activate_or_launch(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id(), MenuAction (+15 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (26): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+18 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.05
Nodes (42): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, Hit, hit() (+34 more)

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
Nodes (9): APPS_FOLDER, ICON_LOCK, ICON_SIZE, is_version_dir(), LAUNCHERS, only_a_single_version_folder_counts_as_the_same_app(), PKEY_APP_USER_MODEL_ID, RUN_ORDER (+1 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (67): Pinned, same_app(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips() (+59 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "icon_data_url"
Cohesion: 0.29
Nodes (6): bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), icon_data_url(), icon_of_notepad_is_a_valid_png()

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (28): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+20 more)

### Community 42 - "items_basic"
Cohesion: 0.26
Nodes (12): apply_run_order(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows(), items_basic() (+4 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.08
Nodes (35): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+27 more)

### Community 44 - "lib.rs"
Cohesion: 0.08
Nodes (16): cancel_pending(), dock_fit(), dock_menu(), dock_settings(), dock_settings_menu(), get_active_app(), indicators_menu(), launcher_files() (+8 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.09
Nodes (30): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+22 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (14): AUTOHIDE, BLUR, CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES, RECONCILE_EVERY (+6 more)

### Community 47 - "HWND"
Cohesion: 0.18
Nodes (14): collect_window(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized(), process_aumid(), raw_app_windows(), running_of() (+6 more)

### Community 48 - "indicators.rs"
Cohesion: 0.09
Nodes (15): battery_icon(), battery_icon_levels_and_states(), duration_label(), duration_labels(), every_icon_rust_can_emit_is_one_the_page_knows(), Fake, FakeBattery, FakeNetwork (+7 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.07
Nodes (62): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026), vitest@^5.1.0 (+54 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.13
Nodes (34): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+26 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (20): EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, HOTKEY_ID, HWND_RAW, LABEL, MAX_HEIGHT, MIN_HEIGHT (+12 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.11
Nodes (40): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+32 more)

### Community 55 - "reconcile"
Cohesion: 0.20
Nodes (9): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), build_region(), cursor(), enforce_styles(), hwnd_of(), tick() (+1 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (35): age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history(), enabled() (+27 more)

### Community 60 - "search"
Cohesion: 0.47
Nodes (6): cached_icon(), clips(), files(), LauncherItem, real_search_examples(), search()

### Community 61 - "mc.rs"
Cohesion: 0.11
Nodes (32): app_display_name(), area_for(), BAND, build(), CARD, CELL_PAD, GAP, geometry() (+24 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "Rect"
Cohesion: 0.18
Nodes (32): Rect, a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), AREA, best_scale(), check_invariants(), css_conversion_is_relative_to_the_monitor_and_scaled(), empty_and_degenerate_input() (+24 more)

### Community 65 - "launcher.rs"
Cohesion: 0.11
Nodes (27): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+19 more)

### Community 66 - "Launcher — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+4 more)

### Community 67 - "main.tsx"
Cohesion: 0.22
Nodes (7): initGlass(), view, applyThemePref(), initTheme(), isThemePref(), readThemePref(), ThemePref

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.08
Nodes (13): detect_restart(), hide(), restore(), explorer_restart_detection(), FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding() (+5 more)

### Community 72 - "indicators.ts"
Cohesion: 0.21
Nodes (22): I3 — Pannello: FATTA (7 nov 2026), Glyph(), Panel(), batteryDetails(), batteryFill(), DetailRow, IndicatorItem, indicatorItems() (+14 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 75 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

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
Cohesion: 0.15
Nodes (6): cached_description(), file_description(), CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh()

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - "set_volume"
Cohesion: 0.18
Nodes (10): fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip(), sanitize_volume(), set_volume() (+2 more)

### Community 82 - "test-indicators-settings.ps1"
Cohesion: 0.20
Nodes (8): BarWin(), Choose(), Keys(), Label(), Menu-Element(), Menu-Items(), Open-Menu(), PJs()

### Community 83 - "show"
Cohesion: 0.33
Nodes (10): apply_styles(), create(), fit(), hide(), is_visible(), show(), start(), target_monitor() (+2 more)

### Community 84 - "Indicatori di sistema — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti (+4 more)

### Community 85 - "McMonitor"
Cohesion: 0.27
Nodes (12): a_window_without_a_registered_thumbnail_falls_back_to_a_card(), McMonitor, McWindow, monitor_with(), the_identity_never_reaches_the_page(), the_view_is_in_css_pixels_relative_to_the_monitor(), View, view_of() (+4 more)

### Community 86 - "window_aumid"
Cohesion: 0.20
Nodes (8): aumid_from_store(), cached_shell_name(), ComScope, pinned_dir(), read_pinned(), resolve_lnk(), shell_display_name(), window_aumid()

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "app_windows_for"
Cohesion: 0.22
Nodes (11): ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer() (+3 more)

### Community 90 - "@tauri-apps/api"
Cohesion: 0.24
Nodes (10): react, @tauri-apps/api, App(), ActiveApp, ActiveAppName(), Clock(), Indicators(), useIndicators() (+2 more)

### Community 91 - "Mission Control — specifica"
Cohesion: 0.20
Nodes (9): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), Mission Control — specifica, Rischi aperti (+1 more)

### Community 92 - "Channel"
Cohesion: 0.22
Nodes (10): Channel, Dock, Launcher, Panel, CHANNELS, latest(), latest_returns_nothing_until_there_is_a_newer_frame(), dock_backdrop() (+2 more)

### Community 93 - "Hotkey"
Cohesion: 0.27
Nodes (11): change_hotkey(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey, hotkey_thread() (+3 more)

### Community 94 - "Mc.tsx"
Cohesion: 0.45
Nodes (8): Mc(), boxStyle(), cardLetter(), cellTitle(), LABEL_H, McView, McWindow, neighbour()

### Community 95 - "start"
Cohesion: 0.25
Nodes (6): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next()

### Community 96 - "TimedCache"
Cohesion: 0.67
Nodes (3): ICONS, SHELL_NAMES, TimedCache

### Community 97 - "aumid_path"
Cohesion: 0.40
Nodes (6): aumid_path(), identity(), parse_aumid(), store_app_identifiers_roundtrip(), store_ids_never_match_as_the_same_app_unless_identical(), window_identity_prefers_the_store_identifier()

### Community 98 - "capture_rect"
Cohesion: 0.28
Nodes (9): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur(), capture_info() (+1 more)

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "place"
Cohesion: 0.33
Nodes (6): clamp_height(), clamp_height_limits_and_survives_garbage(), place(), place_centers_horizontally_and_sits_a_fifth_down(), place_never_leaves_the_work_area(), place_scales_with_dpi_and_follows_the_monitor()

### Community 101 - "on_window_event"
Cohesion: 0.33
Nodes (4): focus_loss_counts(), focus_loss_is_ignored_only_right_after_opening(), on_window_event(), since_start()

### Community 102 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

### Community 103 - "reconcile"
Cohesion: 0.31
Nodes (8): create(), fit(), lock(), reconcile(), reserve_allowed(), reserve_enabled(), start(), toggle_reserve()

### Community 105 - "run"
Cohesion: 0.47
Nodes (6): set_mc_open(), cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), run()

### Community 106 - "thumb_rect"
Cohesion: 0.33
Nodes (6): thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), fit_in(), the_thumbnail_box_leaves_room_for_padding_and_the_label(), thumb_box(), thumbnails_fit_their_box_keeping_proportions_and_never_grow()

### Community 107 - "mcwin.rs"
Cohesion: 0.07
Nodes (31): mc_activate(), mc_backdrop(), mc_close(), mc_close_window(), mc_state(), activate(), all_labels(), APP (+23 more)

### Community 108 - "PENDING_CHANGE"
Cohesion: 0.50
Nodes (3): Change, PENDING_CHANGE, START

## Knowledge Gaps
- **51 isolated node(s):** `Dock`, `Launcher`, `Panel`, `Exec`, `Open` (+46 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 598 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **35 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `aumid_path`, `dock.rs`, `items_basic`, `dockact.rs`, `app_windows_for`, `package.json`, `merge`?**
  _High betweenness centrality (0.150) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `layout()` (e.g. with `build()` and `empty_and_degenerate_input()`) actually correct?**
  _`layout()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Dock`, `Launcher`, `Panel` to the rest of the system?**
  _51 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.149) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Dock()` (e.g. with `Scelte tecniche` and `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)`) actually correct?**
  _`Dock()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._