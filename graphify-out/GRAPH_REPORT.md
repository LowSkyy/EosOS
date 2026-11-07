# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 88 files · ~89,509 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1690 nodes · 3434 edges · 108 communities (74 shown, 34 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 223 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `64fe9f27`
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
- tests.rs
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
- show
- NetKind
- Rect
- launcher.rs
- launcher.ts
- main.tsx
- test-launcher.ps1
- react
- indicators.ts
- Indicators
- scripts
- test-panel.ps1
- HWND
- .new
- test-indicators.ps1
- set_volume
- Dock — specifica
- open_settings
- Indicatori di sistema — specifica
- search
- aumid_path
- util.rs
- app_windows_for
- react
- Hotkey
- AppHandle
- Channel
- latest
- cached_icon
- TimedCache
- is_active
- capture_rect
- Toggle
- Outcome
- clip_id
- Run
- AtomicIsize

## God Nodes (most connected - your core abstractions)
1. `Rect` - 28 edges
2. `Pinned` - 25 edges
3. `Dock()` - 19 edges
4. `same_app()` - 17 edges
5. `compilerOptions` - 17 edges
6. `Launcher()` - 16 edges
7. `items_and_windows()` - 16 edges
8. `rank()` - 15 edges
9. `tick()` - 15 edges
10. `Panel()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts
- `I3 — Pannello: FATTA (7 nov 2026)` --references--> `Panel()`  [INFERRED]
  docs/INDICATORI.md → src/components/Panel.tsx
- `I1 — Dati: FATTA (7 nov 2026)` --references--> `Battery`  [INFERRED]
  docs/INDICATORI.md → src/lib/indicators.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (108 total, 34 thin omitted)

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
Nodes (27): anchor(), clamp_height(), EDGE, EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, GAP, heights_are_clamped() (+19 more)

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
Cohesion: 0.09
Nodes (10): downscale(), downscale_averages_blocks_and_swaps_to_rgba(), downscale_drops_the_odd_edge_and_rejects_bad_input(), downscale_keeps_channel_order_per_block(), FACTOR, Frame, FRAMES, HEADER_LEN (+2 more)

### Community 15 - "tests.rs"
Cohesion: 0.06
Nodes (32): HOTKEY_THREAD, on_ctrl(), appbar_state(), autohide_marker(), collect(), Ctx, detect_restart(), enable_autohide() (+24 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (41): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+33 more)

### Community 20 - "foreground.rs"
Cohesion: 0.08
Nodes (14): APP, DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name(), empty_everything_is_unknown(), EVENT_NAME, explorer_file_window_is_not_desktop() (+6 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.09
Nodes (22): activate(), activate_or_launch(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id_roundtrip_with_colons_in_the_id(), MenuAction, Open (+14 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (27): dependencies, @fontsource-variable/inter@^5.3.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8 (+19 more)

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
Nodes (66): Pinned, pinned_dir(), read_pinned(), same_app(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths() (+58 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "icon_data_url"
Cohesion: 0.29
Nodes (6): bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), icon_data_url(), icon_of_notepad_is_a_valid_png()

### Community 41 - "dockgeom.rs"
Cohesion: 0.14
Nodes (22): at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit(), HIDE_DELAY_SECONDS (+14 more)

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
Nodes (24): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+16 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.06
Nodes (17): AUTOHIDE, BLUR, build_region(), CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES (+9 more)

### Community 47 - "HWND"
Cohesion: 0.16
Nodes (16): cached_shell_name(), collect_window(), ComScope, is_app_window(), is_packaged_path(), packaged_aumid(), packaged_executables_are_recognized(), process_aumid() (+8 more)

### Community 48 - "indicators.rs"
Cohesion: 0.09
Nodes (14): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork (+6 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.21
Nodes (23): Fasi, cssPx(), Dock(), Ghost(), Icon(), applyMove(), clampGlass(), DockItem (+15 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.13
Nodes (34): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), fold(), fold_lowercases_and_strips_common_accents() (+26 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.06
Nodes (30): Change, clamp_height(), clamp_height_limits_and_survives_garbage(), EXCLUDED, FOCUS_CHECK, FOCUS_GRACE, focus_loss_counts(), focus_loss_is_ignored_only_right_after_opening() (+22 more)

### Community 54 - "launcherstore.rs"
Cohesion: 0.09
Nodes (43): Read, Failed, Missing, clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh (+35 more)

### Community 55 - "reconcile"
Cohesion: 0.19
Nodes (14): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), create(), cursor(), enforce_styles(), fit(), hwnd_of() (+6 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (34): age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history(), enabled() (+26 more)

### Community 60 - "show"
Cohesion: 0.25
Nodes (11): create(), fit(), hide(), is_visible(), on_window_event(), show(), since_start(), start() (+3 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "Rect"
Cohesion: 0.15
Nodes (20): Rect, apply_styles(), MON, panel_toggle(), anchor_to_physical(), create(), fit(), hide() (+12 more)

### Community 65 - "launcher.rs"
Cohesion: 0.12
Nodes (16): AppEntry, clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), display(), e(), enumerate(), INDEX (+8 more)

### Community 66 - "launcher.ts"
Cohesion: 0.09
Nodes (39): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+31 more)

### Community 67 - "main.tsx"
Cohesion: 0.22
Nodes (7): initGlass(), view, applyThemePref(), initTheme(), isThemePref(), readThemePref(), ThemePref

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "react"
Cohesion: 0.44
Nodes (5): useBackdrop(), Frame, FRAME_HEADER, parseFrame(), sourceRect()

### Community 72 - "indicators.ts"
Cohesion: 0.18
Nodes (26): I3 — Pannello: FATTA (7 nov 2026), @tauri-apps/api, App(), Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails() (+18 more)

### Community 73 - "Indicators"
Cohesion: 0.21
Nodes (14): current(), Indicators, read_fake(), read_once(), read_real(), read_volume(), real_read_smoke(), serialized_state_is_flat_for_the_page() (+6 more)

### Community 75 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "HWND"
Cohesion: 0.22
Nodes (10): ActiveApp, app_from_hwnd(), current(), initial_window(), last_window(), note_activated(), on_event(), start() (+2 more)

### Community 78 - ".new"
Cohesion: 0.32
Nodes (7): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), duration_label(), duration_labels(), read_battery()

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - "set_volume"
Cohesion: 0.18
Nodes (9): MtaScope, real_set_volume(), real_volume_roundtrip(), sanitize_volume(), set_volume(), start(), volume_requests_are_sanitized(), volume_mute() (+1 more)

### Community 82 - "Dock — specifica"
Cohesion: 0.33
Nodes (5): Comportamento (da replicare), Dock — specifica, Fuori scopo, Rischi aperti, Scelte tecniche

### Community 83 - "open_settings"
Cohesion: 0.67
Nodes (4): only_known_indicators_open_a_settings_page(), open_settings(), settings_uri(), indicator_open()

### Community 84 - "Indicatori di sistema — specifica"
Cohesion: 0.17
Nodes (11): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti, Scelte tecniche (proposte) (+3 more)

### Community 85 - "search"
Cohesion: 0.23
Nodes (12): apps(), cached(), clips(), launch(), LauncherItem, now_secs(), real_apps_folder_lists_apps(), real_launch_calculator() (+4 more)

### Community 86 - "aumid_path"
Cohesion: 0.25
Nodes (8): aumid_from_store(), aumid_path(), identity(), parse_aumid(), resolve_lnk(), store_app_identifiers_roundtrip(), store_ids_never_match_as_the_same_app_unless_identical(), window_identity_prefers_the_store_identifier()

### Community 88 - "app_windows_for"
Cohesion: 0.25
Nodes (8): ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent(), is_launcher(), launcher_names(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon()

### Community 90 - "react"
Cohesion: 0.29
Nodes (7): react@^19.1.0, react, ActiveApp, ActiveAppName(), Clock(), formatClock(), msToNextMinute()

### Community 91 - "Hotkey"
Cohesion: 0.27
Nodes (11): hotkey_presets_are_valid_and_distinct(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey, hotkey_thread() (+3 more)

### Community 92 - "AppHandle"
Cohesion: 0.28
Nodes (7): menu_id(), dock_menu(), dock_settings_menu(), launcher_fit(), launcher_hide(), panel_fit(), panel_hide()

### Community 93 - "Channel"
Cohesion: 0.20
Nodes (10): Channel, Dock, Launcher, Panel, CHANNELS, grab(), grab_reads_real_pixels_from_the_screen(), start() (+2 more)

### Community 94 - "latest"
Cohesion: 0.32
Nodes (7): encode(), frame_header_layout_is_stable(), latest(), latest_returns_nothing_until_there_is_a_newer_frame(), dock_backdrop(), launcher_backdrop(), panel_backdrop()

### Community 95 - "cached_icon"
Cohesion: 0.29
Nodes (8): cached_icon(), key(), live_process_scan_excludes_self_and_finds_explorer(), processes_named(), cached_description(), file_description(), failures_expire_quickly_and_successes_slowly(), is_fresh()

### Community 96 - "TimedCache"
Cohesion: 0.67
Nodes (3): ICONS, SHELL_NAMES, TimedCache

### Community 97 - "is_active"
Cohesion: 0.29
Nodes (7): args(), FitArgs, hysteresis_enters_on_the_capsule_and_stays_in_the_whole_window(), is_active(), sanitize(), sanitize_accepts_valid_and_rejects_nonsense(), dock_fit()

### Community 98 - "capture_rect"
Cohesion: 0.29
Nodes (8): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur(), capture_info()

### Community 99 - "Toggle"
Cohesion: 0.33
Nodes (6): click_decisions(), decide(), Toggle, Hide, Ignore, Show

### Community 100 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Confirm, Done, Failed, Unknown

### Community 101 - "clip_id"
Cohesion: 0.83
Nodes (4): clip_delete(), clip_id(), clip_ids_parse_strictly(), clip_use()

### Community 102 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

## Knowledge Gaps
- **44 isolated node(s):** `Dock`, `Launcher`, `Panel`, `Wifi`, `Ethernet` (+39 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 536 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **34 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `dock.rs`, `items_basic`, `aumid_path`, `dockact.rs`, `app_windows_for`, `package.json`, `merge`, `cached_icon`?**
  _High betweenness centrality (0.166) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Dock()` (e.g. with `Scelte tecniche` and `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)`) actually correct?**
  _`Dock()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Dock`, `Launcher`, `Panel` to the rest of the system?**
  _44 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.164) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `launcher.ts`, `main.tsx`, `tokens.test.ts`, `react`, `indicators.ts`, `Dock.tsx`, `react`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._