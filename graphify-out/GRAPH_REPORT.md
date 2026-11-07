# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 103 files · ~117,506 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 2052 nodes · 4336 edges · 124 communities (86 shown, 38 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 287 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8a08fb7d`
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
- cached_icon
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
- mcstore.rs
- launcher.rs
- Launcher — specifica
- main.tsx
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
- History
- scripts
- test-panel.ps1
- test-mc-overlay.ps1
- indicatorstore.rs
- util.rs
- test-indicators.ps1
- .new
- test-indicators-settings.ps1
- show
- Indicatori di sistema — specifica
- McMonitor
- mctrigger.rs
- read_volume
- app_windows_for
- @tauri-apps/api
- Mission Control — specifica
- AppHandle
- Hotkey
- Mc.tsx
- start
- test-mc-trigger.ps1
- HWND
- capture_rect
- test-mc-windows.ps1
- place
- on_window_event
- Run
- reconcile
- execute
- on_window_event
- mcwin.rs
- Phase
- Clock.tsx
- Mutex
- battery_icon
- AtomicIsize
- AtomicU64
- APP
- MC_CORNER

## God Nodes (most connected - your core abstractions)
1. `Rect` - 60 edges
2. `Pinned` - 25 edges
3. `layout()` - 23 edges
4. `Dock()` - 19 edges
5. `st()` - 17 edges
6. `same_app()` - 17 edges
7. `compilerOptions` - 17 edges
8. `Launcher()` - 16 edges
9. `items_and_windows()` - 16 edges
10. `item()` - 16 edges

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

## Communities (124 total, 38 thin omitted)

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
Cohesion: 0.06
Nodes (12): image_path(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name() (+4 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.07
Nodes (23): capture_bytes(), Channel, Dock, Launcher, Panel, CHANNELS, downscale(), downscale_averages_blocks_and_swaps_to_rgba() (+15 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.14
Nodes (16): appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), find(), hide_matching(), hide_real() (+8 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (41): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+33 more)

### Community 20 - "foreground.rs"
Cohesion: 0.08
Nodes (14): ActiveApp, current(), DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name(), empty_everything_is_unknown(), EVENT_NAME, explorer_file_window_is_not_desktop() (+6 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.08
Nodes (23): activate(), activate_or_launch(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id(), MenuAction (+15 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (28): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+20 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.06
Nodes (43): files(), bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, Hit (+35 more)

### Community 28 - "merge"
Cohesion: 0.27
Nodes (13): background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), key(), live_process_scan_excludes_self_and_finds_explorer(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), pin(), processes_named() (+5 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.05
Nodes (17): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), ICON_LOCK, ICON_SIZE, ICONS, is_version_dir() (+9 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (67): Pinned, same_app(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips() (+59 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "cached_icon"
Cohesion: 0.11
Nodes (16): bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), cached_icon(), cached_shell_name(), ComScope, encode_png(), icon_data_url() (+8 more)

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
Cohesion: 0.06
Nodes (32): cancel_pending(), set_mc_corner(), set_mc_open(), cleanup(), dock_backdrop(), dock_fit(), dock_items(), dock_menu() (+24 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.19
Nodes (20): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+12 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (15): AUTOHIDE, BLUR, build_region(), CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES (+7 more)

### Community 47 - "HWND"
Cohesion: 0.19
Nodes (16): aumid_from_store(), aumid_path(), collect_window(), identity(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized() (+8 more)

### Community 48 - "indicators.rs"
Cohesion: 0.09
Nodes (16): duration_label(), duration_labels(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume (+8 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.07
Nodes (60): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026), cssPx() (+52 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.13
Nodes (34): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+26 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (22): Change, EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, HOTKEY_ID, HOTKEY_THREAD, LABEL, MAX_HEIGHT (+14 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.10
Nodes (40): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+32 more)

### Community 55 - "reconcile"
Cohesion: 0.25
Nodes (8): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), cursor(), enforce_styles(), hwnd_of(), tick(), toggle_autohide()

### Community 59 - "launcherclip.rs"
Cohesion: 0.06
Nodes (16): age_label(), age_labels(), CF_UNICODETEXT, format_id(), HISTORY, MAX_CHARS, MAX_ENTRIES, password_manager_copies_are_not_recorded() (+8 more)

### Community 60 - "search"
Cohesion: 0.21
Nodes (14): AppEntry, apps(), cached(), clips(), launch(), LauncherItem, now_secs(), real_apps_folder_lists_apps() (+6 more)

### Community 61 - "mc.rs"
Cohesion: 0.07
Nodes (77): app_display_name(), Rect, a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD (+69 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "mcstore.rs"
Cohesion: 0.15
Nodes (23): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+15 more)

### Community 65 - "launcher.rs"
Cohesion: 0.10
Nodes (17): clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id(), clip_ids_parse_strictly(), clip_use(), display() (+9 more)

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
Cohesion: 0.09
Nodes (13): detect_restart(), hide(), restore(), explorer_restart_detection(), FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding() (+5 more)

### Community 72 - "indicators.ts"
Cohesion: 0.17
Nodes (26): I1 — Dati: FATTA (7 nov 2026), I3 — Pannello: FATTA (7 nov 2026), Glyph(), Panel(), Battery, batteryDetails(), batteryFill(), DetailRow (+18 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "History"
Cohesion: 0.14
Nodes (18): capped_at_max_entries_dropping_the_oldest(), clear(), copy_back(), disabling_clears_the_global_history(), enabled(), entries_expire_after_the_ttl(), Entry, hist() (+10 more)

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
Nodes (32): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+24 more)

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - ".new"
Cohesion: 0.33
Nodes (4): MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip()

### Community 82 - "test-indicators-settings.ps1"
Cohesion: 0.20
Nodes (8): BarWin(), Choose(), Keys(), Label(), Menu-Element(), Menu-Items(), Open-Menu(), PJs()

### Community 83 - "show"
Cohesion: 0.26
Nodes (11): capture_info(), create(), fit(), hide(), is_visible(), show(), start(), target_monitor() (+3 more)

### Community 84 - "Indicatori di sistema — specifica"
Cohesion: 0.22
Nodes (8): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti, Scelte tecniche (proposte)

### Community 85 - "McMonitor"
Cohesion: 0.47
Nodes (6): mc_windows(), McMonitor, McWindow, icons_for(), register_thumbs(), views()

### Community 86 - "mctrigger.rs"
Cohesion: 0.13
Nodes (24): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+16 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "app_windows_for"
Cohesion: 0.29
Nodes (7): app_windows_for(), is_launcher(), launcher_names(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon(), raw_app_windows(), real_windows_are_consistent()

### Community 90 - "@tauri-apps/api"
Cohesion: 0.33
Nodes (7): react, @tauri-apps/api, App(), ActiveApp, ActiveAppName(), Indicators(), useIndicators()

### Community 91 - "Mission Control — specifica"
Cohesion: 0.18
Nodes (10): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), Mission Control — specifica (+2 more)

### Community 92 - "AppHandle"
Cohesion: 0.22
Nodes (18): apply_styles(), all_labels(), button_down(), close(), close_animated(), corner_thread(), create_one(), label() (+10 more)

### Community 93 - "Hotkey"
Cohesion: 0.23
Nodes (15): hotkey_presets_are_valid_and_distinct(), arrow_keys_work_in_hotkeys(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey (+7 more)

### Community 94 - "Mc.tsx"
Cohesion: 0.45
Nodes (8): Mc(), boxStyle(), cardLetter(), cellTitle(), LABEL_H, McView, McWindow, neighbour()

### Community 95 - "start"
Cohesion: 0.22
Nodes (7): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next(), load()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "HWND"
Cohesion: 0.31
Nodes (8): app_from_hwnd(), initial_window(), last_window(), note_activated(), on_event(), start(), update(), window_string()

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

### Community 102 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

### Community 103 - "reconcile"
Cohesion: 0.31
Nodes (8): create(), fit(), lock(), reconcile(), reserve_allowed(), reserve_enabled(), start(), toggle_reserve()

### Community 105 - "execute"
Cohesion: 0.22
Nodes (10): execute(), now_ms(), Outcome, Confirm, Done, Failed, Unknown, run() (+2 more)

### Community 106 - "on_window_event"
Cohesion: 0.22
Nodes (6): activate(), close_window(), hwnd_from(), hwnd_of(), is_known(), on_window_event()

### Community 107 - "mcwin.rs"
Cohesion: 0.08
Nodes (11): Anim, APP, CORNER, DEFAULT_HOTKEY, HOTKEY_ID, PREFIX, REFRESH_AFTER, RETRY_EVERY (+3 more)

### Community 108 - "Phase"
Cohesion: 0.29
Nodes (8): mc_state(), Diag, diag(), Phase, Closed, Closing, Open, Opening

### Community 109 - "Clock.tsx"
Cohesion: 0.73
Nodes (3): Clock(), formatClock(), msToNextMinute()

### Community 111 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 117 - "AtomicIsize"
Cohesion: 0.50
Nodes (3): LAST_HWND, OWNER, HWND_RAW

### Community 118 - "AtomicU64"
Cohesion: 0.50
Nodes (3): SEQ, SCALE_BITS, SHOWN_AT_MS

## Knowledge Gaps
- **58 isolated node(s):** `Exec`, `Open`, `Internal`, `Done`, `Confirm` (+53 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 612 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **38 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `dock.rs`, `items_basic`, `HWND`, `dockact.rs`, `app_windows_for`, `package.json`, `merge`?**
  _High betweenness centrality (0.197) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `layout()` (e.g. with `build()` and `empty_and_degenerate_input()`) actually correct?**
  _`layout()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Exec`, `Open`, `Internal` to the rest of the system?**
  _58 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.194) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Dock()` (e.g. with `Scelte tecniche` and `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)`) actually correct?**
  _`Dock()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._