# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 95 files · ~103,120 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1851 nodes · 3807 edges · 114 communities (79 shown, 35 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 257 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `a4e91ce5`
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
- is_active
- launcher.rs
- launcher.ts
- main.tsx
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
- scripts
- test-panel.ps1
- History
- indicatorstore.rs
- util.rs
- test-indicators.ps1
- set_volume
- test-indicators-settings.ps1
- show
- Indicatori di sistema — specifica
- .new
- step
- read_volume
- app_windows_for
- @tauri-apps/api
- Mission Control — specifica
- Hotkey
- HWND
- start
- TimedCache
- aumid_path
- capture_rect
- test-mc-windows.ps1
- place
- on_window_event
- Action
- ActiveApp
- APP
- PENDING_CHANGE
- AtomicIsize
- Indicators
- AtomicU32

## God Nodes (most connected - your core abstractions)
1. `Rect` - 45 edges
2. `Pinned` - 25 edges
3. `layout()` - 23 edges
4. `Dock()` - 19 edges
5. `same_app()` - 17 edges
6. `compilerOptions` - 17 edges
7. `items_and_windows()` - 16 edges
8. `item()` - 16 edges
9. `Launcher()` - 16 edges
10. `build()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts
- `same_app()` --references--> `name`  [INFERRED]
  src-tauri/src/dock.rs → package.json
- `Fasi` --references--> `DockSettings`  [INFERRED]
  docs/DOCK.md → src/lib/dock.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (114 total, 35 thin omitted)

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
Cohesion: 0.07
Nodes (24): Channel, Dock, Launcher, Panel, CHANNELS, downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input() (+16 more)

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
Cohesion: 0.10
Nodes (12): DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name(), empty_everything_is_unknown(), EVENT_NAME, explorer_file_window_is_not_desktop(), falls_back_to_capitalized_exe_name(), file_description() (+4 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.09
Nodes (23): activate(), activate_or_launch(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id(), MenuAction (+15 more)

### Community 24 - "package.json"
Cohesion: 0.12
Nodes (21): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+13 more)

### Community 25 - "glass.rs"
Cohesion: 0.12
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.05
Nodes (43): files(), bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, Hit (+35 more)

### Community 28 - "merge"
Cohesion: 0.28
Nodes (13): background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), key(), live_process_scan_excludes_self_and_finds_explorer(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), only_a_single_version_folder_counts_as_the_same_app(), pin() (+5 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.05
Nodes (8): APPS_FOLDER, ICON_LOCK, ICON_SIZE, is_version_dir(), LAUNCHERS, PKEY_APP_USER_MODEL_ID, RUN_ORDER, version_folder_names()

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (66): Pinned, a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips(), clamp_glass() (+58 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "icon_data_url"
Cohesion: 0.15
Nodes (10): bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), ComScope, encode_png(), icon_data_url(), icon_of_notepad_is_a_valid_png(), pinned_dir() (+2 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.15
Nodes (22): at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit(), HIDE_DELAY_SECONDS (+14 more)

### Community 42 - "items_basic"
Cohesion: 0.26
Nodes (12): apply_run_order(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows(), items_basic() (+4 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.08
Nodes (35): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+27 more)

### Community 44 - "lib.rs"
Cohesion: 0.07
Nodes (24): cleanup(), dock_activate(), dock_backdrop(), dock_items(), dock_menu(), dock_reorder(), dock_settings(), dock_settings_menu() (+16 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.15
Nodes (8): CONFIRM_WINDOW, names(), Pending, PENDING, scattered_letters_do_not_surface_actions(), search(), search_by_name_and_keywords_in_both_languages(), search_ignores_empty_and_unrelated_queries()

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (17): AUTOHIDE, BLUR, build_region(), CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES (+9 more)

### Community 47 - "HWND"
Cohesion: 0.18
Nodes (15): aumid_from_store(), collect_window(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized(), process_aumid(), raw_app_windows() (+7 more)

### Community 48 - "indicators.rs"
Cohesion: 0.09
Nodes (16): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), Fake, FakeBattery, FakeNetwork, FakeVolume, Link (+8 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.12
Nodes (33): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, cssPx(), Dock() (+25 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.13
Nodes (34): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+26 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (20): EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, HOTKEY_ID, HWND_RAW, LABEL, MAX_HEIGHT, MIN_HEIGHT (+12 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.10
Nodes (40): set_clipboard(), start(), clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh (+32 more)

### Community 55 - "reconcile"
Cohesion: 0.19
Nodes (14): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), create(), cursor(), enforce_styles(), fit(), hwnd_of() (+6 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.06
Nodes (17): age_label(), age_labels(), CF_UNICODETEXT, copy_back(), format_id(), HISTORY, MAX_CHARS, MAX_ENTRIES (+9 more)

### Community 60 - "search"
Cohesion: 0.24
Nodes (12): apps(), clips(), launch(), LauncherItem, now_secs(), real_apps_folder_lists_apps(), real_launch_calculator(), real_search_examples() (+4 more)

### Community 61 - "mc.rs"
Cohesion: 0.07
Nodes (62): Rect, mc_windows(), area_for(), BAND, build(), CARD, GAP, geometry() (+54 more)

### Community 62 - "NetKind"
Cohesion: 0.18
Nodes (16): NetKind, Cellular, Ethernet, None, Wifi, Network, network_icon(), network_icons() (+8 more)

### Community 63 - "is_active"
Cohesion: 0.29
Nodes (7): args(), FitArgs, hysteresis_enters_on_the_capsule_and_stays_in_the_whole_window(), is_active(), sanitize(), sanitize_accepts_valid_and_rejects_nonsense(), dock_fit()

### Community 65 - "launcher.rs"
Cohesion: 0.13
Nodes (20): AppEntry, cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id(), clip_ids_parse_strictly() (+12 more)

### Community 66 - "launcher.ts"
Cohesion: 0.09
Nodes (39): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+31 more)

### Community 67 - "main.tsx"
Cohesion: 0.14
Nodes (13): dependencies, @fontsource-variable/inter@^5.3.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react-dom, initGlass(), view (+5 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.09
Nodes (4): detect_restart(), explorer_restart_detection(), FakeWindow, wndproc()

### Community 72 - "indicators.ts"
Cohesion: 0.20
Nodes (24): I3 — Pannello: FATTA (7 nov 2026), Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails(), batteryFill(), DetailRow (+16 more)

### Community 73 - "Indicators"
Cohesion: 0.28
Nodes (8): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), duration_label(), duration_labels(), read_battery(), serialized_state_is_flat_for_the_page()

### Community 75 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "History"
Cohesion: 0.14
Nodes (16): capped_at_max_entries_dropping_the_oldest(), clear(), disabling_clears_the_global_history(), enabled(), entries_expire_after_the_ttl(), Entry, hist(), History (+8 more)

### Community 78 - "indicatorstore.rs"
Cohesion: 0.10
Nodes (33): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+25 more)

### Community 79 - "util.rs"
Cohesion: 0.14
Nodes (9): app_display_name(), cached_icon(), cached_shell_name(), shell_display_name(), cached_description(), CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly() (+1 more)

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
Cohesion: 0.27
Nodes (11): apply_styles(), create(), fit(), hide(), is_visible(), show(), start(), target_monitor() (+3 more)

### Community 84 - "Indicatori di sistema — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti (+4 more)

### Community 85 - ".new"
Cohesion: 0.29
Nodes (9): hide(), restore(), hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding(), restore_is_idempotent_and_safe_when_nothing_hidden(), watchdog_restores_immediately_if_parent_already_gone(), watchdog_restores_when_watched_process_is_killed() (+1 more)

### Community 86 - "step"
Cohesion: 0.21
Nodes (13): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), commands_are_fixed_and_exact(), confirmation_expires_and_does_not_transfer_between_actions(), dangerous_actions_need_a_second_press_on_the_same_action(), Outcome, Confirm (+5 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "app_windows_for"
Cohesion: 0.29
Nodes (8): ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), is_launcher(), launcher_names(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon()

### Community 90 - "@tauri-apps/api"
Cohesion: 0.25
Nodes (9): react@^19.1.0, react, @tauri-apps/api, App(), ActiveApp, ActiveAppName(), Clock(), formatClock() (+1 more)

### Community 91 - "Mission Control — specifica"
Cohesion: 0.22
Nodes (8): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), Mission Control — specifica, Rischi aperti, Scelte tecniche (proposte)

### Community 93 - "Hotkey"
Cohesion: 0.27
Nodes (11): hotkey_presets_are_valid_and_distinct(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey, hotkey_thread() (+3 more)

### Community 94 - "HWND"
Cohesion: 0.31
Nodes (8): app_from_hwnd(), initial_window(), last_window(), note_activated(), on_event(), start(), update(), window_string()

### Community 95 - "start"
Cohesion: 0.22
Nodes (7): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next(), load()

### Community 96 - "TimedCache"
Cohesion: 0.67
Nodes (3): ICONS, SHELL_NAMES, TimedCache

### Community 97 - "aumid_path"
Cohesion: 0.40
Nodes (6): aumid_path(), identity(), parse_aumid(), store_app_identifiers_roundtrip(), store_ids_never_match_as_the_same_app_unless_identical(), window_identity_prefers_the_store_identifier()

### Community 98 - "capture_rect"
Cohesion: 0.29
Nodes (8): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur(), capture_info()

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "place"
Cohesion: 0.33
Nodes (6): clamp_height(), clamp_height_limits_and_survives_garbage(), place(), place_centers_horizontally_and_sits_a_fifth_down(), place_never_leaves_the_work_area(), place_scales_with_dpi_and_follows_the_monitor()

### Community 101 - "on_window_event"
Cohesion: 0.33
Nodes (4): focus_loss_counts(), focus_loss_is_ignored_only_right_after_opening(), on_window_event(), since_start()

### Community 102 - "Action"
Cohesion: 0.20
Nodes (12): Action, ACTIONS, execute(), find(), now_ms(), Run, run(), Exec (+4 more)

### Community 103 - "ActiveApp"
Cohesion: 0.50
Nodes (3): ActiveApp, current(), LAST

### Community 108 - "PENDING_CHANGE"
Cohesion: 0.50
Nodes (3): Change, PENDING_CHANGE, START

### Community 110 - "Indicators"
Cohesion: 0.24
Nodes (8): current(), Indicators, read_fake(), read_once(), read_real(), STATE, WAKE, indicators_get()

## Knowledge Gaps
- **51 isolated node(s):** `Exec`, `Internal`, `Open`, `Dock`, `Launcher` (+46 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 576 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **35 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `merge` to `aumid_path`, `dock.rs`, `dockstore.rs`, `items_basic`, `dockact.rs`, `app_windows_for`, `package.json`?**
  _High betweenness centrality (0.178) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `layout()` (e.g. with `build()` and `empty_and_degenerate_input()`) actually correct?**
  _`layout()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Exec`, `Internal`, `Open` to the rest of the system?**
  _51 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `merge`?**
  _High betweenness centrality (0.175) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Dock()` (e.g. with `Scelte tecniche` and `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)`) actually correct?**
  _`Dock()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._