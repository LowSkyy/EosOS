# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 128 files · ~165,116 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 2744 nodes · 5944 edges · 149 communities (104 shown, 45 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 374 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `16653277`
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
- menubar.rs
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
- Rect
- indicatorstore.rs
- launcherclip.rs
- menuread.rs
- mc.rs
- NetKind
- vdint.rs
- test-mc-vdesk.ps1
- main.tsx
- super
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
- Dock.tsx
- vdesk.rs
- test-panel.ps1
- test-mc-overlay.ps1
- TopMenu
- app_windows_for
- test-indicators.ps1
- mckeys.rs
- test-indicators-settings.ps1
- test-mc-desktops.ps1
- show
- test-menudrop.ps1
- mctrigger.rs
- read_volume
- History
- test-vdesk-internal.ps1
- Mission Control — specifica
- test-menuinvoke.ps1
- Mc.tsx
- start
- test-mc-trigger.ps1
- tick
- duration
- test-mc-windows.ps1
- launcherstore.rs
- Readability
- set_volume
- Phase
- open
- test-mc-keys.ps1
- HWND
- mcwin.rs
- handle
- Launcher — specifica
- Mutex
- Cmd
- test-menubar.ps1
- test-menusettings.ps1
- .new
- run_search
- read_menus
- HWND
- useBackdrop.ts
- Cmd
- run
- rank_hits
- Outcome
- Run
- fold
- capture_rect
- compose
- RawItem
- best_bar
- Dock — specifica
- Response

## God Nodes (most connected - your core abstractions)
1. `Rect` - 61 edges
2. `Pinned` - 25 edges
3. `st()` - 25 edges
4. `layout()` - 23 edges
5. `Dock()` - 19 edges
6. `Profile` - 18 edges
7. `open()` - 18 edges
8. `Settings` - 17 edges
9. `same_app()` - 17 edges
10. `compilerOptions` - 17 edges

## Surprising Connections (you probably didn't know these)
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `G3 — La barra: FATTA (8 nov 2026)` --references--> `acceptRow()`  [INFERRED]
  docs/MENU-GLOBALE.md → src/lib/menubar.ts
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (149 total, 45 thin omitted)

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
Nodes (57): SEQ, SCALE_BITS, SHOWN_AT_MS, anchor(), anchor_to_physical(), busy_for(), BUSY_UNTIL_MS, clamp_height() (+49 more)

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
Nodes (30): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+22 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.08
Nodes (26): activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id() (+18 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (27): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+19 more)

### Community 25 - "menubar.rs"
Cohesion: 0.10
Nodes (25): menubar_current(), APP, Cache, CACHE, cache_fresh(), current(), CURRENT, EVENT_NAME (+17 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.07
Nodes (15): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, KNOWN, KNOWN_CAP, known_folder(), LATEST, LIMIT (+7 more)

### Community 28 - "menumodel.rs"
Cohesion: 0.14
Nodes (21): app_menu(), app_menu_names_the_app_and_never_is_blank(), browsers_and_chats_are_never_queried(), exclusions_ignore_case_and_toggle(), is_excluded(), labels_lose_accelerators_and_split_the_shortcut(), limit_title(), long_titles_are_cut_without_breaking_characters() (+13 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.04
Nodes (23): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent() (+15 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (63): Pinned, same_app(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips() (+55 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.07
Nodes (31): HOTKEY_THREAD, C_CFG, C_STATE, C_SWALLOWED, C_TX, cfg(), CFG, CHILD_IN (+23 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (29): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+21 more)

### Community 42 - "items_basic"
Cohesion: 0.21
Nodes (18): apply_run_order(), background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), DockItem, dump_dock(), find(), find_with_windows(), items(), items_and_windows() (+10 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.06
Nodes (52): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+44 more)

### Community 44 - "lib.rs"
Cohesion: 0.08
Nodes (25): cancel_pending(), dock_menu(), dock_settings_menu(), indicators_menu(), launcher_files(), launcher_fit(), launcher_hide(), launcher_search() (+17 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.12
Nodes (28): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+20 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.05
Nodes (20): AUTOHIDE, BLUR, build_region(), CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES (+12 more)

### Community 47 - "collect_window"
Cohesion: 0.09
Nodes (28): aumid_from_store(), aumid_path(), cached_shell_name(), collect_window(), ComScope, identity(), is_app_window(), is_packaged_path() (+20 more)

### Community 48 - "indicators.rs"
Cohesion: 0.10
Nodes (15): battery_icon(), battery_icon_levels_and_states(), duration_label(), duration_labels(), every_icon_rust_can_emit_is_one_the_page_knows(), Fake, FakeBattery, FakeNetwork (+7 more)

### Community 49 - "launcher.ts"
Cohesion: 0.16
Nodes (27): L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Launcher(), clampSelection(), combine(), fallbackLetter(), Filter, filterForKey(), filterLabel() (+19 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.16
Nodes (28): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), frecency(), frecency_breaks_ties_but_cannot_beat_a_much_better_match() (+20 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.06
Nodes (41): arrow_keys_work_in_hotkeys(), Change, change_hotkey(), clamp_height(), clamp_height_limits_and_survives_garbage(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY (+33 more)

### Community 54 - "Rect"
Cohesion: 0.14
Nodes (40): Rect, thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), AREA, best_scale(), check_invariants() (+32 more)

### Community 55 - "indicatorstore.rs"
Cohesion: 0.06
Nodes (69): Read, Failed, Missing, Text, a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh (+61 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.06
Nodes (16): age_label(), age_labels(), CF_UNICODETEXT, format_id(), HISTORY, MAX_CHARS, MAX_ENTRIES, password_manager_copies_are_not_recorded() (+8 more)

### Community 60 - "menuread.rs"
Cohesion: 0.06
Nodes (7): INFLIGHT, LAZY_TIMEOUT, MAX_INFLIGHT, MIN_UIA_MENU_ITEMS, READ_DEPTH, READ_ITEMS, UIA_TIMEOUT

### Community 61 - "mc.rs"
Cohesion: 0.08
Nodes (43): app_display_name(), a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP (+35 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.06
Nodes (73): a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), build_info(), capabilities_follow_the_selection_all_or_nothing() (+65 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 67 - "main.tsx"
Cohesion: 0.13
Nodes (15): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+7 more)

### Community 68 - "super"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.09
Nodes (6): wndproc(), proc(), detect_restart(), explorer_restart_detection(), FakeWindow, wndproc()

### Community 72 - "indicators.ts"
Cohesion: 0.05
Nodes (70): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I3 — Pannello: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica (+62 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "Dock.tsx"
Cohesion: 0.21
Nodes (23): Fasi, cssPx(), Dock(), Ghost(), Icon(), applyMove(), clampGlass(), DockItem (+15 more)

### Community 75 - "vdesk.rs"
Cohesion: 0.06
Nodes (30): an_unknown_or_missing_current_desktop_falls_back_to_the_first(), BYTES, create(), current_index(), Desktop, desktops_from(), Dir, Left (+22 more)

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "test-mc-overlay.ps1"
Cohesion: 0.20
Nodes (13): Cell-Info(), CellJs(), Esc-If-Open(), Inv(), IsOpen(), Js(), Mc(), McIdx() (+5 more)

### Community 78 - "TopMenu"
Cohesion: 0.19
Nodes (14): menubar_menu(), is_lazy(), menu(), Item, item_at(), items_are_found_by_their_model_path(), lazy_items_get_paths_from_their_source_menu(), paths_keep_pointing_at_the_original_position_after_cleaning() (+6 more)

### Community 79 - "app_windows_for"
Cohesion: 0.32
Nodes (8): app_windows_for(), is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon(), processes_named()

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - "mckeys.rs"
Cohesion: 0.15
Nodes (28): a_lost_alt_release_is_noticed_by_the_safety_net(), alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle(), cycling_starts_from_the_second_window_and_wraps(), disabled_shortcuts_are_never_touched(), escape_cancels_and_a_later_alt_release_does_not_commit(), Held (+20 more)

### Community 82 - "test-indicators-settings.ps1"
Cohesion: 0.20
Nodes (8): BarWin(), Choose(), Keys(), Label(), Menu-Element(), Menu-Items(), Open-Menu(), PJs()

### Community 83 - "test-mc-desktops.ps1"
Cohesion: 0.43
Nodes (5): Click-Chip(), Desktops(), Inv(), Js(), McIdx()

### Community 84 - "show"
Cohesion: 0.19
Nodes (14): apply_styles(), capture_info(), create(), fit(), hide(), is_visible(), on_window_event(), show() (+6 more)

### Community 85 - "test-menudrop.ps1"
Cohesion: 0.26
Nodes (13): Bar-Click(), Bar-Hover(), Close-Panel(), Focus(), Inv(), Js(), Open-Menu(), Panel-Click() (+5 more)

### Community 86 - "mctrigger.rs"
Cohesion: 0.13
Nodes (24): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+16 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "History"
Cohesion: 0.15
Nodes (16): capped_at_max_entries_dropping_the_oldest(), clear(), copy_back(), disabling_clears_the_global_history(), enabled(), entries_expire_after_the_ttl(), Entry, hist() (+8 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Mission Control — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+5 more)

### Community 92 - "test-menuinvoke.ps1"
Cohesion: 0.24
Nodes (14): Bar-Click(), Close-Panel(), Focus(), Inv(), Js(), Log-Count(), Log-Has(), Log-Lines() (+6 more)

### Community 94 - "Mc.tsx"
Cohesion: 0.27
Nodes (17): Mc(), advancedHint(), boxStyle(), canRemove(), cardLetter(), cellTitle(), cleanDesktopName(), desktopAction() (+9 more)

### Community 95 - "start"
Cohesion: 0.25
Nodes (6): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "tick"
Cohesion: 0.19
Nodes (14): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), create(), cursor(), enforce_styles(), fit(), hwnd_of() (+6 more)

### Community 98 - "duration"
Cohesion: 0.15
Nodes (7): cached_icon(), cached_description(), file_description(), CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh()

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.05
Nodes (72): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+64 more)

### Community 101 - "Readability"
Cohesion: 0.20
Nodes (14): menu_probe_all(), MenuProbeRow, classification_covers_every_case(), classify(), p(), Probe, probe(), read_hmenu() (+6 more)

### Community 102 - "set_volume"
Cohesion: 0.18
Nodes (10): fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip(), sanitize_volume(), set_volume() (+2 more)

### Community 103 - "Phase"
Cohesion: 0.25
Nodes (8): diag(), Diag, diag(), Phase, Closed, Closing, Open, Opening

### Community 104 - "open"
Cohesion: 0.39
Nodes (7): create_one(), fit_exact(), hwnd_of(), label(), open(), take_focus(), win_of()

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "HWND"
Cohesion: 0.22
Nodes (12): a_shown_title_must_still_be_the_one_in_the_menu(), invoke_hmenu(), invoke_item(), invoke_uia(), InvokeErr, Changed, Failed, Gone (+4 more)

### Community 107 - "mcwin.rs"
Cohesion: 0.07
Nodes (45): activate(), all_labels(), alt_commit(), alt_step(), Anim, APP, backdrop(), button_down() (+37 more)

### Community 108 - "handle"
Cohesion: 0.33
Nodes (7): c_cfg(), c_send(), handle(), Msg, Release, Run, Swallowed

### Community 109 - "Launcher — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+4 more)

### Community 111 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 114 - "test-menubar.ps1"
Cohesion: 0.31
Nodes (5): BarTitles(), Focus(), Js(), Row(), Wait-Until()

### Community 116 - "test-menusettings.ps1"
Cohesion: 0.29
Nodes (6): Act(), Count-Menus(), Focus(), Js(), Row(), Wait-Until()

### Community 117 - ".new"
Cohesion: 0.29
Nodes (9): hide(), restore(), hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding(), restore_is_idempotent_and_safe_when_nothing_hidden(), watchdog_restores_immediately_if_parent_already_gone(), watchdog_restores_when_watched_process_is_killed() (+1 more)

### Community 119 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 123 - "read_menus"
Cohesion: 0.40
Nodes (3): abandoned_reads_are_capped(), Inflight, may_start_read()

### Community 124 - "HWND"
Cohesion: 0.31
Nodes (7): menubar_choose(), choose(), exe_stem_of(), forget_cache(), on_window(), quit_app(), run_cmd()

### Community 125 - "useBackdrop.ts"
Cohesion: 0.44
Nodes (5): useBackdrop(), Frame, FRAME_HEADER, parseFrame(), sourceRect()

### Community 126 - "Cmd"
Cohesion: 0.19
Nodes (13): Action, App, action_paths_and_flags_survive_normalization(), Cmd, Close, Maximize, Minimize, Quit (+5 more)

### Community 127 - "run"
Cohesion: 0.28
Nodes (9): set_mc_corner(), set_mc_keys(), set_mc_open(), set_menu_actions(), cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets() (+1 more)

### Community 137 - "rank_hits"
Cohesion: 0.26
Nodes (12): hit(), is_known(), key(), known_results_are_capped(), multi_word_queries_match_names_with_separators(), name_score(), only_remembered_paths_can_be_opened(), open() (+4 more)

### Community 138 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Confirm, Done, Failed, Unknown

### Community 139 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

### Community 140 - "fold"
Cohesion: 0.25
Nodes (9): files(), short_dir(), short_dir_uses_home_and_elides_long_paths(), fold(), fold_lowercases_and_strips_common_accents(), initials(), initials_cover_words_and_inner_capitals(), match_tiers() (+1 more)

### Community 141 - "capture_rect"
Cohesion: 0.29
Nodes (8): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur(), capture_info()

### Community 143 - "compose"
Cohesion: 0.21
Nodes (16): an_app_gets_the_hybrid_row(), build_row(), state(), the_desktop_shows_only_its_name(), Eos, an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), compose(), is_window_menu() (+8 more)

### Community 144 - "RawItem"
Cohesion: 0.38
Nodes (6): RawItem, RawMenu, lazy_uia(), read_hmenu_tree(), read_lazy_items(), read_menus()

### Community 145 - "best_bar"
Cohesion: 0.33
Nodes (4): best_bar(), is_system_bar(), read_uia(), the_title_bar_system_menu_is_not_the_app_menu()

### Community 146 - "Dock — specifica"
Cohesion: 0.33
Nodes (5): Comportamento (da replicare), Dock — specifica, Fuori scopo, Rischi aperti, Scelte tecniche

### Community 147 - "Response"
Cohesion: 0.13
Nodes (7): dock_backdrop(), dock_items(), dock_settings(), launcher_backdrop(), mc_backdrop(), mc_state(), panel_backdrop()

## Knowledge Gaps
- **86 isolated node(s):** `BackupAndFresh`, `Fresh`, `Use`, `Both`, `HMenu` (+81 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 783 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **45 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockstore.rs` to `dock.rs`, `items_basic`, `collect_window`, `app_windows_for`, `dockact.rs`, `package.json`?**
  _High betweenness centrality (0.150) - this node is a cross-community bridge._
- **What connects `BackupAndFresh`, `Fresh`, `Use` to the rest of the system?**
  _86 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockstore.rs`?**
  _High betweenness centrality (0.149) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `main.tsx`, `indicators.ts`, `Dock.tsx`, `launcher.ts`, `tokens.test.ts`, `useBackdrop.ts`, `Mc.tsx`?**
  _High betweenness centrality (0.058) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06349206349206349 - nodes in this community are weakly interconnected._