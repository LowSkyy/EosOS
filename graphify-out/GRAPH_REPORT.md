# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 77 files · ~73,669 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1469 nodes · 2936 edges · 99 communities (66 shown, 33 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 196 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f3e5a34a`
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
- collect_window
- History
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
- hist
- AtomicBool
- .new
- place
- Outcome
- theme.ts
- Run
- Channel
- app_windows_for
- cached_icon
- latest
- aumid_path
- list
- TimedCache
- AtomicU32

## God Nodes (most connected - your core abstractions)
1. `Pinned` - 25 edges
2. `Dock()` - 19 edges
3. `Rect` - 19 edges
4. `Launcher()` - 17 edges
5. `same_app()` - 17 edges
6. `compilerOptions` - 17 edges
7. `items_and_windows()` - 16 edges
8. `tick()` - 15 edges
9. `rank()` - 14 edges
10. `Settings` - 14 edges

## Surprising Connections (you probably didn't know these)
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Launcher()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Launcher.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `pointerMoved()`  [INFERRED]
  docs/LAUNCHER.md → src/lib/launcher.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (99 total, 33 thin omitted)

### Community 0 - "EosOS: Piano di lavoro"
Cohesion: 0.25
Nodes (7): Dettagli sul ripristino (P4), EosOS: Piano di lavoro, LEGENDA dei passaggi, Principi di sicurezza (valgono per tutti i passaggi), Roadmap (FUORI SCOPE per ora), Stato finale (7 nov 2026), Traguardo della sessione

### Community 4 - "compilerOptions"
Cohesion: 0.10
Nodes (19): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+11 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.05
Nodes (13): CACHE_FOUND_TTL, CACHE_MISSING_TTL, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path() (+5 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.09
Nodes (10): downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input(), downscale_keeps_channel_order_per_block(), FACTOR, Frame, FRAMES, HEADER_LEN (+2 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.08
Nodes (28): on_ctrl(), appbar_state(), autohide_marker(), collect(), Ctx, detect_restart(), enable_autohide(), find() (+20 more)

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
Cohesion: 0.05
Nodes (43): files(), bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, Hit (+35 more)

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
Cohesion: 0.16
Nodes (22): args(), BLUR_MARGIN_CSS, BOTTOM_MARGIN, capsule_rect(), Fit, fit(), FitArgs, HIDE_DELAY_SECONDS (+14 more)

### Community 42 - "items_basic"
Cohesion: 0.26
Nodes (12): apply_run_order(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows(), items_basic() (+4 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.10
Nodes (28): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+20 more)

### Community 44 - "lib.rs"
Cohesion: 0.10
Nodes (10): cleanup(), dock_items(), dock_settings(), enable_taskbar_hiding(), get_active_app(), hide_requested(), install_local_safety_nets(), launcher_files() (+2 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.13
Nodes (23): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+15 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (8): CHROME_STYLES, LABEL, NO_ACTIVATE_STYLES, RECONCILE_EVERY, reserve_enabled(), STATE, TICK, toggle_reserve()

### Community 47 - "collect_window"
Cohesion: 0.24
Nodes (13): collect_window(), is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized(), process_aumid(), running_of(), running_of_deduplicates_by_executable_keeping_z_order() (+5 more)

### Community 48 - "History"
Cohesion: 0.21
Nodes (8): capped_at_max_entries_dropping_the_oldest(), entries_expire_after_the_ttl(), Entry, History, HISTORY, newest_first_and_duplicates_move_up_without_growing(), remove(), search_requires_every_word_ignoring_case_and_accents()

### Community 49 - "Dock.tsx"
Cohesion: 0.12
Nodes (33): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, cssPx(), Dock() (+25 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.13
Nodes (34): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+26 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.07
Nodes (19): Change, FOCUS_CHECK, FOCUS_GRACE, HOTKEY_ID, LABEL, MAX_HEIGHT, MIN_HEIGHT, MOD_ALT_BIT (+11 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.10
Nodes (40): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+32 more)

### Community 55 - "reconcile"
Cohesion: 0.31
Nodes (8): slide_covers_the_whole_travel_with_ease_out(), slide_y(), create(), fit(), lock(), reconcile(), reserve_allowed(), start()

### Community 59 - "launcherclip.rs"
Cohesion: 0.07
Nodes (12): CF_UNICODETEXT, format_id(), MAX_CHARS, MAX_ENTRIES, password_manager_copies_are_not_recorded(), read_current(), read_global_u32(), should_record() (+4 more)

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
Cohesion: 0.11
Nodes (21): AppEntry, cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id(), clip_ids_parse_strictly() (+13 more)

### Community 66 - "launcher.ts"
Cohesion: 0.16
Nodes (27): L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Launcher(), clampSelection(), combine(), fallbackLetter(), Filter, filterForKey(), filterLabel() (+19 more)

### Community 67 - "main.tsx"
Cohesion: 0.19
Nodes (10): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react-dom, @tauri-apps/api (+2 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "search"
Cohesion: 0.25
Nodes (11): apps(), clips(), launch(), LauncherItem, now_secs(), real_apps_folder_lists_apps(), real_launch_calculator(), real_search_examples() (+3 more)

### Community 72 - "App.tsx"
Cohesion: 0.33
Nodes (7): react, App(), ActiveApp, ActiveAppName(), Clock(), formatClock(), msToNextMinute()

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
Cohesion: 0.24
Nodes (12): hotkey_presets_are_valid_and_distinct(), change_hotkey(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey (+4 more)

### Community 78 - "AppHandle"
Cohesion: 0.32
Nodes (6): menu_id(), dock_fit(), dock_menu(), dock_settings_menu(), launcher_fit(), launcher_hide()

### Community 80 - "hist"
Cohesion: 0.22
Nodes (9): clear(), copy_back(), disabling_clears_the_global_history(), enabled(), hist(), now_ms(), remove_and_clear(), set_enabled() (+1 more)

### Community 81 - "AtomicBool"
Cohesion: 0.22
Nodes (8): AUTOHIDE, BLUR, MENU_OPEN, RESERVE, ENABLED, EXCLUDED, VISIBLE, HIDDEN

### Community 82 - ".new"
Cohesion: 0.20
Nodes (7): aumid_from_store(), cached_shell_name(), ComScope, pinned_dir(), read_pinned(), resolve_lnk(), shell_display_name()

### Community 83 - "place"
Cohesion: 0.33
Nodes (6): clamp_height(), clamp_height_limits_and_survives_garbage(), place(), place_centers_horizontally_and_sits_a_fifth_down(), place_never_leaves_the_work_area(), place_scales_with_dpi_and_follows_the_monitor()

### Community 84 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Confirm, Done, Failed, Unknown

### Community 85 - "theme.ts"
Cohesion: 0.36
Nodes (5): applyThemePref(), initTheme(), isThemePref(), readThemePref(), ThemePref

### Community 86 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

### Community 87 - "Channel"
Cohesion: 0.25
Nodes (8): Channel, Dock, Launcher, CHANNELS, grab(), grab_reads_real_pixels_from_the_screen(), start(), step()

### Community 88 - "app_windows_for"
Cohesion: 0.29
Nodes (8): ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), is_launcher(), launcher_names(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon()

### Community 92 - "cached_icon"
Cohesion: 0.29
Nodes (8): cached_icon(), key(), live_process_scan_excludes_self_and_finds_explorer(), processes_named(), cached_description(), file_description(), failures_expire_quickly_and_successes_slowly(), is_fresh()

### Community 93 - "latest"
Cohesion: 0.33
Nodes (6): encode(), frame_header_layout_is_stable(), latest(), latest_returns_nothing_until_there_is_a_newer_frame(), dock_backdrop(), launcher_backdrop()

### Community 94 - "aumid_path"
Cohesion: 0.40
Nodes (6): aumid_path(), identity(), parse_aumid(), store_app_identifiers_roundtrip(), store_ids_never_match_as_the_same_app_unless_identical(), window_identity_prefers_the_store_identifier()

### Community 95 - "list"
Cohesion: 0.33
Nodes (5): age_label(), age_labels(), list(), preview(), preview_takes_the_first_meaningful_line_and_truncates()

### Community 96 - "TimedCache"
Cohesion: 0.67
Nodes (3): ICONS, SHELL_NAMES, TimedCache

## Knowledge Gaps
- **33 isolated node(s):** `Exec`, `Open`, `Internal`, `Done`, `Confirm` (+28 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 485 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **33 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `dock.rs`, `items_basic`, `cached_icon`, `dockact.rs`, `app_windows_for`, `package.json`, `merge`, `aumid_path`?**
  _High betweenness centrality (0.200) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Dock()` (e.g. with `Scelte tecniche` and `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)`) actually correct?**
  _`Dock()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Exec`, `Open`, `Internal` to the rest of the system?**
  _33 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.198) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `launcher.ts`, `App.tsx`, `Dock.tsx`, `theme.ts`, `tokens.test.ts`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._