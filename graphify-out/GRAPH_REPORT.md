# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 74 files · ~63,922 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1309 nodes · 2573 edges · 79 communities (50 shown, 29 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 168 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b9ede037`
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
- launcherfiles.rs
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
- FakeWindow
- launcher.ts
- main.tsx
- test-launcher.ps1
- tick
- App.tsx
- theme.ts
- scripts
- TimedCache
- merge

## God Nodes (most connected - your core abstractions)
1. `Pinned` - 25 edges
2. `Dock()` - 18 edges
3. `Rect` - 18 edges
4. `same_app()` - 17 edges
5. `compilerOptions` - 17 edges
6. `items_and_windows()` - 16 edges
7. `Settings` - 15 edges
8. `tick()` - 15 edges
9. `Launcher()` - 14 edges
10. `UsageMap` - 14 edges

## Surprising Connections (you probably didn't know these)
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `Fasi` --references--> `DockSettings`  [INFERRED]
  docs/DOCK.md → src/lib/dock.ts
- `Fasi` --references--> `Ghost()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `same_app()` --references--> `name`  [INFERRED]
  src-tauri/src/dock.rs → package.json
- `dock_settings()` --references--> `Settings`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/dockstore.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (79 total, 29 thin omitted)

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
Cohesion: 0.09
Nodes (3): detect_restart(), COUNTER, explorer_restart_detection()

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.05
Nodes (14): CACHE_FOUND_TTL, CACHE_MISSING_TTL, image_path(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES (+6 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.08
Nodes (17): downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input(), downscale_keeps_channel_order_per_block(), encode(), FACTOR, Frame, FRAME (+9 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.12
Nodes (12): appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), KEEP_HIDDEN, owned_by_explorer(), restore_autohide() (+4 more)

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
Cohesion: 0.09
Nodes (22): activate(), activate_or_launch(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id_roundtrip_with_colons_in_the_id(), MenuAction, Open (+14 more)

### Community 24 - "package.json"
Cohesion: 0.12
Nodes (21): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+13 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (17): [cmd, a, b], evaluate(), list, pending, send(), target, ws, BLACK (+9 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.06
Nodes (39): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, Hit, hit() (+31 more)

### Community 28 - "app_windows"
Cohesion: 0.25
Nodes (9): is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon(), processes_named(), running_of() (+1 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.05
Nodes (12): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), ICON_LOCK, ICON_SIZE, is_version_dir(), LAUNCHERS (+4 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (68): app_windows_for(), Pinned, same_app(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files() (+60 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - ".new"
Cohesion: 0.11
Nodes (16): bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), cached_icon(), cached_shell_name(), ComScope, encode_png(), icon_data_url() (+8 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.17
Nodes (20): args(), BLUR_MARGIN_CSS, BOTTOM_MARGIN, capsule_rect(), Fit, fit(), FitArgs, HIDE_DELAY_SECONDS (+12 more)

### Community 42 - "items_basic"
Cohesion: 0.21
Nodes (14): apply_run_order(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows(), items_basic() (+6 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.13
Nodes (20): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+12 more)

### Community 44 - "lib.rs"
Cohesion: 0.09
Nodes (18): menu_id(), cleanup(), dock_backdrop(), dock_fit(), dock_items(), dock_menu(), dock_settings(), dock_settings_menu() (+10 more)

### Community 45 - "launcher.rs"
Cohesion: 0.05
Nodes (54): AppEntry, apps(), cached(), clean(), clean_drops_empty_and_duplicates(), display(), e(), enumerate() (+46 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.07
Nodes (14): AUTOHIDE, BLUR, CHROME_STYLES, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES, RECONCILE_EVERY, RESERVE (+6 more)

### Community 47 - "collect_window"
Cohesion: 0.17
Nodes (16): aumid_from_store(), aumid_path(), collect_window(), identity(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized() (+8 more)

### Community 48 - "Rect"
Cohesion: 0.22
Nodes (9): at_bottom_edge(), bottom_edge_is_the_last_pixel_row_of_the_monitor(), hysteresis_enters_on_the_capsule_and_stays_in_the_whole_window(), is_active(), Rect, build_region(), Geometry, State (+1 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.12
Nodes (32): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, cssPx(), Dock() (+24 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.14
Nodes (31): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+23 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (34): SEQ, LATEST, apply_styles(), clamp_height(), clamp_height_limits_and_survives_garbage(), create(), fit(), FOCUS_CHECK (+26 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.13
Nodes (30): decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use, Entry (+22 more)

### Community 55 - "reconcile"
Cohesion: 0.31
Nodes (8): autohide(), create(), fit(), lock(), reconcile(), reserve_allowed(), start(), toggle_autohide()

### Community 59 - ".new"
Cohesion: 0.22
Nodes (13): find(), hide(), hide_matching(), hide_real(), restore(), tray_handle(), hide_does_not_touch_other_classes(), hide_then_restore_roundtrip() (+5 more)

### Community 62 - "capture_info"
Cohesion: 0.29
Nodes (7): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur()

### Community 63 - "Launcher — specifica"
Cohesion: 0.17
Nodes (11): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+3 more)

### Community 66 - "launcher.ts"
Cohesion: 0.18
Nodes (24): Launcher(), clampSelection(), combine(), fallbackLetter(), Filter, filterForKey(), filterLabel(), glyphFor() (+16 more)

### Community 67 - "main.tsx"
Cohesion: 0.19
Nodes (11): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+3 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.31
Nodes (6): Focused(), IsOpen(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tick"
Cohesion: 0.20
Nodes (10): only_the_part_inside_the_screen_is_visible(), slide_covers_the_whole_travel_with_ease_out(), slide_y(), step_visibility(), visibility_moves_at_a_constant_rate_and_saturates(), visible_height(), cursor(), enforce_styles() (+2 more)

### Community 72 - "App.tsx"
Cohesion: 0.36
Nodes (6): App(), ActiveApp, ActiveAppName(), Clock(), formatClock(), msToNextMinute()

### Community 73 - "theme.ts"
Cohesion: 0.36
Nodes (5): applyThemePref(), initTheme(), isThemePref(), readThemePref(), ThemePref

### Community 75 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 76 - "TimedCache"
Cohesion: 0.67
Nodes (3): ICONS, SHELL_NAMES, TimedCache

### Community 77 - "merge"
Cohesion: 0.50
Nodes (8): background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), pin(), run(), Running, versioned_subfolder_matches_the_pinned_launcher()

## Knowledge Gaps
- **30 isolated node(s):** `eosos`, `Index`, `Fallback`, `Open`, `Pin` (+25 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 438 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `dock.rs`, `items_basic`, `merge`, `collect_window`, `dockact.rs`, `package.json`, `app_windows`?**
  _High betweenness centrality (0.219) - this node is a cross-community bridge._
- **Are the 9 inferred relationships involving `same_app()` (e.g. with `only_a_single_version_folder_counts_as_the_same_app()` and `name`) actually correct?**
  _`same_app()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `eosos`, `Index`, `Fallback` to the rest of the system?**
  _30 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.206) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `launcher.ts`, `App.tsx`, `theme.ts`, `Dock.tsx`, `tokens.test.ts`?**
  _High betweenness centrality (0.084) - this node is a cross-community bridge._