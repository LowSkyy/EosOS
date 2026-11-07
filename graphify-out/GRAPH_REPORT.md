# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 76 files · ~71,215 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1453 nodes · 2900 edges · 92 communities (61 shown, 31 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 185 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4a73d8cd`
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
- same_app
- EosOS
- Checklist di test manuali
- dock.rs
- dockstore.rs
- launchercalc.rs
- app_windows_for
- dockgeom.rs
- items_basic
- test-dock-scales.ps1
- lib.rs
- launcheract.rs
- dockwin.rs
- collect_window
- MenuAction
- Dock.tsx
- launcherrank.rs
- launcherwin.rs
- launcherstore.rs
- reconcile
- launcherclip.rs
- show
- Rect
- Launcher — specifica
- launcher.rs
- launcher.ts
- main.tsx
- test-launcher.ps1
- search
- App.tsx
- tick
- scripts
- on_window_event
- Hotkey
- AppHandle
- .new
- AtomicBool
- activate
- place
- Outcome
- clip_id
- Run
- PENDING_CHANGE

## God Nodes (most connected - your core abstractions)
1. `Pinned` - 25 edges
2. `Rect` - 19 edges
3. `Dock()` - 18 edges
4. `same_app()` - 17 edges
5. `compilerOptions` - 17 edges
6. `Launcher()` - 16 edges
7. `items_and_windows()` - 16 edges
8. `tick()` - 15 edges
9. `search()` - 14 edges
10. `State` - 14 edges

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

## Communities (92 total, 31 thin omitted)

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
Nodes (4): HOTKEY_THREAD, COUNTER, FakeWindow, wndproc()

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
Nodes (24): Channel, Dock, Launcher, CHANNELS, downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input(), downscale_keeps_channel_order_per_block() (+16 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.11
Nodes (18): appbar_state(), autohide_marker(), collect(), Ctx, detect_restart(), enable_autohide(), find(), hide_matching() (+10 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (40): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+32 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (27): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+19 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.12
Nodes (6): foreign_or_malformed_menu_ids_are_ignored(), menu_id_roundtrip_with_colons_in_the_id(), minimizes_only_when_the_app_is_already_in_front(), parse_menu_id(), PREFIX, window_to_minimize()

### Community 24 - "package.json"
Cohesion: 0.12
Nodes (20): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+12 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (17): [cmd, a, b], evaluate(), list, pending, send(), target, ws, BLACK (+9 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.05
Nodes (43): files(), bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, Hit (+35 more)

### Community 28 - "same_app"
Cohesion: 0.20
Nodes (17): name, background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), key(), live_process_scan_excludes_self_and_finds_explorer(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), only_a_single_version_folder_counts_as_the_same_app() (+9 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.04
Nodes (16): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), ICON_LOCK, ICON_SIZE, ICONS, is_version_dir() (+8 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (66): Pinned, a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips(), clamp_glass() (+58 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "app_windows_for"
Cohesion: 0.12
Nodes (14): app_windows_for(), bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), ComScope, encode_png(), icon_data_url(), icon_of_notepad_is_a_valid_png() (+6 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.16
Nodes (22): args(), BLUR_MARGIN_CSS, BOTTOM_MARGIN, capsule_rect(), Fit, fit(), FitArgs, HIDE_DELAY_SECONDS (+14 more)

### Community 42 - "items_basic"
Cohesion: 0.15
Nodes (19): apply_run_order(), cached_icon(), cached_shell_name(), DockItem, dump_dock(), find(), find_with_windows(), items() (+11 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.13
Nodes (20): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+12 more)

### Community 44 - "lib.rs"
Cohesion: 0.10
Nodes (11): cleanup(), dock_items(), dock_settings(), enable_taskbar_hiding(), get_active_app(), hide_requested(), install_local_safety_nets(), launcher_files() (+3 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.17
Nodes (23): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+15 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (8): CHROME_STYLES, LABEL, NO_ACTIVATE_STYLES, RECONCILE_EVERY, reserve_enabled(), STATE, TICK, toggle_reserve()

### Community 47 - "collect_window"
Cohesion: 0.20
Nodes (15): aumid_from_store(), aumid_path(), collect_window(), identity(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized() (+7 more)

### Community 48 - "MenuAction"
Cohesion: 0.22
Nodes (9): MenuAction, Open, Pin, Quit, SetGlass, ToggleAutoHide, ToggleBlur, ToggleReserve (+1 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.12
Nodes (33): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, cssPx(), Dock() (+25 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.14
Nodes (31): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+23 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.06
Nodes (16): FOCUS_CHECK, FOCUS_GRACE, HOTKEY_ID, LABEL, MAX_HEIGHT, MIN_HEIGHT, MOD_ALT_BIT, MOD_CTRL_BIT (+8 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.10
Nodes (40): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+32 more)

### Community 55 - "reconcile"
Cohesion: 0.31
Nodes (8): slide_covers_the_whole_travel_with_ease_out(), slide_y(), create(), fit(), lock(), reconcile(), reserve_allowed(), start()

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (34): age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history(), enabled() (+26 more)

### Community 60 - "show"
Cohesion: 0.29
Nodes (11): apply_styles(), create(), fit(), hide(), is_visible(), show(), start(), target_monitor() (+3 more)

### Community 62 - "Rect"
Cohesion: 0.15
Nodes (15): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), hysteresis_enters_on_the_capsule_and_stays_in_the_whole_window(), is_active(), Rect, blur_enabled() (+7 more)

### Community 63 - "Launcher — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+4 more)

### Community 65 - "launcher.rs"
Cohesion: 0.10
Nodes (16): AppEntry, clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), display(), e(), enumerate(), INDEX (+8 more)

### Community 66 - "launcher.ts"
Cohesion: 0.17
Nodes (26): Launcher(), clampSelection(), combine(), fallbackLetter(), Filter, filterForKey(), filterLabel(), glyphFor() (+18 more)

### Community 67 - "main.tsx"
Cohesion: 0.14
Nodes (14): dependencies, @fontsource-variable/inter@^5.3.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react-dom, @tauri-apps/api, initGlass() (+6 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "search"
Cohesion: 0.23
Nodes (12): apps(), cached(), clips(), launch(), LauncherItem, now_secs(), real_apps_folder_lists_apps(), real_launch_calculator() (+4 more)

### Community 72 - "App.tsx"
Cohesion: 0.29
Nodes (8): react@^19.1.0, react, App(), ActiveApp, ActiveAppName(), Clock(), formatClock(), msToNextMinute()

### Community 73 - "tick"
Cohesion: 0.20
Nodes (10): at_bottom_edge(), bottom_edge_is_the_last_pixel_row_of_the_monitor(), step_visibility(), visibility_moves_at_a_constant_rate_and_saturates(), autohide(), cursor(), enforce_styles(), hwnd_of() (+2 more)

### Community 75 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 76 - "on_window_event"
Cohesion: 0.33
Nodes (4): focus_loss_counts(), focus_loss_is_ignored_only_right_after_opening(), on_window_event(), since_start()

### Community 77 - "Hotkey"
Cohesion: 0.27
Nodes (11): hotkey_presets_are_valid_and_distinct(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey, hotkey_thread() (+3 more)

### Community 78 - "AppHandle"
Cohesion: 0.32
Nodes (6): menu_id(), dock_fit(), dock_menu(), dock_settings_menu(), launcher_fit(), launcher_hide()

### Community 80 - ".new"
Cohesion: 0.29
Nodes (9): hide(), restore(), hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding(), restore_is_idempotent_and_safe_when_nothing_hidden(), watchdog_restores_immediately_if_parent_already_gone(), watchdog_restores_when_watched_process_is_killed() (+1 more)

### Community 81 - "AtomicBool"
Cohesion: 0.22
Nodes (8): AUTOHIDE, BLUR, MENU_OPEN, RESERVE, ENABLED, EXCLUDED, VISIBLE, HIDDEN

### Community 82 - "activate"
Cohesion: 0.53
Nodes (7): activate(), activate_or_launch(), focus(), launch(), quit(), run_menu_action(), windows_of()

### Community 83 - "place"
Cohesion: 0.33
Nodes (6): clamp_height(), clamp_height_limits_and_survives_garbage(), place(), place_centers_horizontally_and_sits_a_fifth_down(), place_never_leaves_the_work_area(), place_scales_with_dpi_and_follows_the_monitor()

### Community 84 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Confirm, Done, Failed, Unknown

### Community 85 - "clip_id"
Cohesion: 0.83
Nodes (4): clip_delete(), clip_id(), clip_ids_parse_strictly(), clip_use()

### Community 86 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

### Community 87 - "PENDING_CHANGE"
Cohesion: 0.50
Nodes (3): Change, PENDING_CHANGE, START

## Knowledge Gaps
- **33 isolated node(s):** `Exec`, `Open`, `Internal`, `Done`, `Confirm` (+28 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 482 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **31 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `same_app` to `dock.rs`, `dockstore.rs`, `app_windows_for`, `items_basic`, `activate`, `dockact.rs`?**
  _High betweenness centrality (0.143) - this node is a cross-community bridge._
- **Are the 9 inferred relationships involving `same_app()` (e.g. with `only_a_single_version_folder_counts_as_the_same_app()` and `name`) actually correct?**
  _`same_app()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Exec`, `Open`, `Internal` to the rest of the system?**
  _33 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `same_app` to `package.json`?**
  _High betweenness centrality (0.139) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `launcher.ts`, `main.tsx`, `App.tsx`, `Dock.tsx`, `tokens.test.ts`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._