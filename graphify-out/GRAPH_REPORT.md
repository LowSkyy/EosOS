# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 73 files · ~60,294 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1241 nodes · 2430 edges · 78 communities (50 shown, 28 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 156 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `dcc42074`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- EosOS: Piano di lavoro
- CLAUDE.md
- compilerOptions
- tauri.conf.json
- tests.rs
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
- aumid_path
- app_windows
- EosOS
- Checklist di test manuali
- dock.rs
- dockstore.rs
- launchercalc.rs
- .new
- dockgeom.rs
- items_basic
- test-dock-scales.ps1
- lib.rs
- launcher.rs
- dockwin.rs
- collect_window
- Rect
- Dock.tsx
- launcherrank.rs
- launcherwin.rs
- launcherstore.rs
- reconcile
- .new
- capture_info
- Launcher — specifica
- is_active
- launcher.ts
- main.tsx
- test-launcher.ps1
- tick
- App.tsx
- theme.ts
- scripts
- util.rs

## God Nodes (most connected - your core abstractions)
1. `Pinned` - 25 edges
2. `Dock()` - 18 edges
3. `Rect` - 18 edges
4. `same_app()` - 17 edges
5. `compilerOptions` - 17 edges
6. `items_and_windows()` - 16 edges
7. `search()` - 15 edges
8. `Settings` - 15 edges
9. `tick()` - 15 edges
10. `rank()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `Fasi` --references--> `DockSettings`  [INFERRED]
  docs/DOCK.md → src/lib/dock.ts
- `Fasi` --references--> `Ghost()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `same_app()` --references--> `name`  [INFERRED]
  src-tauri/src/dock.rs → package.json
- `search()` --calls--> `aumid_path()`  [INFERRED]
  src-tauri/src/launcher.rs → src-tauri/src/dock.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (78 total, 28 thin omitted)

### Community 0 - "EosOS: Piano di lavoro"
Cohesion: 0.25
Nodes (7): Dettagli sul ripristino (P4), EosOS: Piano di lavoro, LEGENDA dei passaggi, Principi di sicurezza (valgono per tutti i passaggi), Roadmap (FUORI SCOPE per ora), Stato finale (7 nov 2026), Traguardo della sessione

### Community 4 - "compilerOptions"
Cohesion: 0.10
Nodes (19): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+11 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 6 - "tests.rs"
Cohesion: 0.08
Nodes (5): detect_restart(), COUNTER, explorer_restart_detection(), FakeWindow, wndproc()

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
Nodes (17): downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input(), downscale_keeps_channel_order_per_block(), encode(), FACTOR, Frame, FRAME (+9 more)

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
Cohesion: 0.07
Nodes (25): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+17 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.10
Nodes (24): only_a_single_version_folder_counts_as_the_same_app(), same_app(), activate(), activate_or_launch(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id_roundtrip_with_colons_in_the_id() (+16 more)

### Community 24 - "package.json"
Cohesion: 0.12
Nodes (21): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+13 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (17): [cmd, a, b], evaluate(), list, pending, send(), target, ws, BLACK (+9 more)

### Community 27 - "aumid_path"
Cohesion: 0.40
Nodes (6): aumid_path(), identity(), parse_aumid(), store_app_identifiers_roundtrip(), store_ids_never_match_as_the_same_app_unless_identical(), window_identity_prefers_the_store_identifier()

### Community 28 - "app_windows"
Cohesion: 0.28
Nodes (8): app_windows_for(), is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon(), processes_named()

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.04
Nodes (13): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), ICON_LOCK, ICON_SIZE, is_version_dir(), LAUNCHERS (+5 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (65): Pinned, a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips(), clamp_glass() (+57 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - ".new"
Cohesion: 0.13
Nodes (12): bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), cached_shell_name(), ComScope, encode_png(), icon_data_url(), icon_of_notepad_is_a_valid_png() (+4 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.16
Nodes (20): BLUR_MARGIN_CSS, BOTTOM_MARGIN, capsule_rect(), Fit, fit(), HIDE_DELAY_SECONDS, maximized_windows_stop_above_the_capsule(), never_wider_than_the_work_area() (+12 more)

### Community 42 - "items_basic"
Cohesion: 0.21
Nodes (18): apply_run_order(), background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows() (+10 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.13
Nodes (20): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+12 more)

### Community 44 - "lib.rs"
Cohesion: 0.10
Nodes (16): menu_id(), cancel_pending(), cleanup(), dock_backdrop(), dock_items(), dock_menu(), dock_settings_menu(), enable_taskbar_hiding() (+8 more)

### Community 45 - "launcher.rs"
Cohesion: 0.05
Nodes (54): AppEntry, apps(), cached(), clean(), clean_drops_empty_and_duplicates(), display(), e(), enumerate() (+46 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.07
Nodes (12): AUTOHIDE, BLUR, CHROME_STYLES, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES, RECONCILE_EVERY, RESERVE (+4 more)

### Community 47 - "collect_window"
Cohesion: 0.18
Nodes (14): aumid_from_store(), collect_window(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized(), process_aumid(), running_of() (+6 more)

### Community 48 - "Rect"
Cohesion: 0.29
Nodes (7): at_bottom_edge(), bottom_edge_is_the_last_pixel_row_of_the_monitor(), Rect, build_region(), Geometry, State, MON

### Community 49 - "Dock.tsx"
Cohesion: 0.12
Nodes (32): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, cssPx(), Dock() (+24 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.14
Nodes (31): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+23 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (33): SEQ, apply_styles(), clamp_height(), clamp_height_limits_and_survives_garbage(), create(), fit(), FOCUS_CHECK, FOCUS_GRACE (+25 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.12
Nodes (29): decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use, Entry (+21 more)

### Community 55 - "reconcile"
Cohesion: 0.27
Nodes (9): set_reserve(), create(), fit(), lock(), reconcile(), reserve_allowed(), reserve_enabled(), start() (+1 more)

### Community 59 - ".new"
Cohesion: 0.29
Nodes (9): hide(), restore(), hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding(), restore_is_idempotent_and_safe_when_nothing_hidden(), watchdog_restores_immediately_if_parent_already_gone(), watchdog_restores_when_watched_process_is_killed() (+1 more)

### Community 62 - "capture_info"
Cohesion: 0.25
Nodes (8): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), set_blur(), blur_enabled(), capture_info(), toggle_blur()

### Community 63 - "Launcher — specifica"
Cohesion: 0.18
Nodes (10): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), Launcher — specifica (+2 more)

### Community 65 - "is_active"
Cohesion: 0.29
Nodes (7): args(), FitArgs, hysteresis_enters_on_the_capsule_and_stays_in_the_whole_window(), is_active(), sanitize(), sanitize_accepts_valid_and_rejects_nonsense(), dock_fit()

### Community 66 - "launcher.ts"
Cohesion: 0.23
Nodes (19): Launcher(), clampSelection(), fallbackLetter(), Filter, filterForKey(), glyphFor(), IconCache, INPUT_HEIGHT (+11 more)

### Community 67 - "main.tsx"
Cohesion: 0.17
Nodes (12): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+4 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.31
Nodes (6): Focused(), IsOpen(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tick"
Cohesion: 0.22
Nodes (9): slide_covers_the_whole_travel_with_ease_out(), slide_y(), set_autohide(), autohide(), cursor(), enforce_styles(), hwnd_of(), tick() (+1 more)

### Community 72 - "App.tsx"
Cohesion: 0.47
Nodes (5): App(), ActiveAppName(), Clock(), formatClock(), msToNextMinute()

### Community 73 - "theme.ts"
Cohesion: 0.36
Nodes (5): applyThemePref(), initTheme(), isThemePref(), readThemePref(), ThemePref

### Community 75 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 76 - "util.rs"
Cohesion: 0.12
Nodes (10): cached_icon(), ICONS, SHELL_NAMES, cached_description(), file_description(), CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly() (+2 more)

## Knowledge Gaps
- **28 isolated node(s):** `Exec`, `Open`, `Done`, `Confirm`, `Unknown` (+23 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 412 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **28 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockact.rs` to `dock.rs`, `dockstore.rs`, `items_basic`, `package.json`, `aumid_path`, `app_windows`?**
  _High betweenness centrality (0.177) - this node is a cross-community bridge._
- **Are the 9 inferred relationships involving `same_app()` (e.g. with `only_a_single_version_folder_counts_as_the_same_app()` and `name`) actually correct?**
  _`same_app()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Exec`, `Open`, `Done` to the rest of the system?**
  _28 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockact.rs`?**
  _High betweenness centrality (0.172) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `launcher.ts`, `App.tsx`, `theme.ts`, `Dock.tsx`, `tokens.test.ts`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._