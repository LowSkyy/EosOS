# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 139 files · ~180,132 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 2965 nodes · 6375 edges · 169 communities (113 shown, 56 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 390 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d102f64f`
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
- Pinned
- test-dock-scales.ps1
- lib.rs
- launcheract.rs
- dockwin.rs
- collect_window
- indicators.rs
- Dock.tsx
- launcherrank.rs
- launcherwin.rs
- AppHandle
- indicatorstore.rs
- launcherclip.rs
- menuread.rs
- mc.rs
- NetKind
- vdint.rs
- test-mc-vdesk.ps1
- mcstore.rs
- AppMenus.tsx
- super
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
- settingswin.rs
- vdesk.rs
- test-panel.ps1
- test-mc-overlay.ps1
- Item
- folderpick.rs
- test-indicators.ps1
- mckeys.rs
- test-indicators-settings.ps1
- test-mc-desktops.ps1
- test-settings-s3.ps1
- test-menudrop.ps1
- mctrigger.rs
- read_volume
- pin
- settingsreg.rs
- test-vdesk-internal.ps1
- Settings.tsx
- test-menuinvoke.ps1
- Mc.tsx
- start
- test-mc-trigger.ps1
- HWND
- duration
- test-mc-windows.ps1
- launcherstore.rs
- Readability
- .new
- launcher.rs
- AppHandle
- test-mc-keys.ps1
- HWND
- mcwin.rs
- handle
- menubar.ts
- Mutex
- decide
- test-menubar.ps1
- test-menusettings.ps1
- Window
- run_search
- read_menus
- Phase
- test-settingswin.ps1
- Cmd
- open
- MenuAction
- rank_hits
- Probe
- mc_state
- fold
- scripts
- battery_icon
- compose
- clip_id
- best_bar
- dock_items
- Response
- merge
- test-settings.ps1
- settings_all
- Menu globale per app — specifica
- Indicatori di sistema — specifica
- dock_settings
- run
- corner_thread
- Outcome
- Run
- LauncherItem

## God Nodes (most connected - your core abstractions)
1. `Rect` - 65 edges
2. `st()` - 25 edges
3. `Pinned` - 25 edges
4. `layout()` - 23 edges
5. `Value` - 22 edges
6. `Dock()` - 19 edges
7. `Profile` - 18 edges
8. `Settings` - 18 edges
9. `same_app()` - 17 edges
10. `compilerOptions` - 17 edges

## Surprising Connections (you probably didn't know these)
- `G3 — La barra: FATTA (8 nov 2026)` --references--> `acceptRow()`  [INFERRED]
  docs/MENU-GLOBALE.md → src/lib/menubar.ts
- `S3 — Le altre sezioni: FATTA (8 nov 2026)` --references--> `comboFromEvent()`  [INFERRED]
  docs/IMPOSTAZIONI.md → src/lib/settings.ts
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `I1 — Dati: FATTA (7 nov 2026)` --references--> `Battery`  [INFERRED]
  docs/INDICATORI.md → src/lib/indicators.ts

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (169 total, 56 thin omitted)

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
Nodes (51): anchor(), anchor_to_physical(), busy_for(), clamp_height(), click_decisions(), create(), decide(), EDGE (+43 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.07
Nodes (12): CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name(), READY_TIMEOUT (+4 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.07
Nodes (23): capture_bytes(), Channel, Dock, Launcher, Panel, CHANNELS, downscale(), downscale_averages_blocks_and_swaps_to_rgba() (+15 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.10
Nodes (19): on_ctrl(), appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), find(), hide_matching() (+11 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.08
Nodes (40): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+32 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (30): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+22 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.11
Nodes (14): same_app(), activate(), activate_or_launch(), alt_or_win_down(), focus(), launch(), minimizes_only_when_the_app_is_already_in_front(), PREFIX (+6 more)

### Community 24 - "package.json"
Cohesion: 0.09
Nodes (28): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+20 more)

### Community 25 - "menubar.rs"
Cohesion: 0.10
Nodes (27): APP, Cache, CACHE, cache_fresh(), current(), CURRENT, EVENT_NAME, EXCLUDED (+19 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.07
Nodes (21): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, is_known(), key(), KNOWN, KNOWN_CAP, known_folder() (+13 more)

### Community 28 - "menumodel.rs"
Cohesion: 0.13
Nodes (25): browsers_and_chats_are_never_queried(), exclusions_ignore_case_and_toggle(), is_excluded(), labels_lose_accelerators_and_split_the_shortcut(), lazy_items_get_paths_from_their_source_menu(), limit_title(), long_titles_are_cut_without_breaking_characters(), MAX_DEPTH (+17 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.04
Nodes (28): ancestor_search_stops_at_four_levels_and_survives_cycles(), apply_run_order(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor() (+20 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.11
Nodes (25): a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips(), clamp_glass(), Entry, File, glass_defaults_and_levels() (+17 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.06
Nodes (42): C_CFG, c_cfg(), C_STATE, C_TX, cfg(), CFG, CHILD_IN, child_lines_round_trip_and_garbage_is_ignored() (+34 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.10
Nodes (36): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), capture_adds_context_and_stays_on_the_monitor(), capture_margin() (+28 more)

### Community 42 - "Pinned"
Cohesion: 0.19
Nodes (21): Pinned, default_settings(), last_list(), last_settings(), load(), load_full(), Loaded, modify() (+13 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.06
Nodes (52): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+44 more)

### Community 44 - "lib.rs"
Cohesion: 0.10
Nodes (7): mc_thumb_visible(), mc_vd_report(), menu_probe_all(), menubar_current(), MenuProbeRow, SetReply, settings_set()

### Community 45 - "launcheract.rs"
Cohesion: 0.11
Nodes (28): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+20 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.05
Nodes (38): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), AUTOHIDE, BLUR, blur_enabled(), build_region(), capture_info() (+30 more)

### Community 47 - "collect_window"
Cohesion: 0.10
Nodes (33): app_display_name(), app_windows_for(), aumid_from_store(), aumid_path(), cached_shell_name(), collect_window(), ComScope, find_with_windows() (+25 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (16): duration_label(), duration_labels(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume (+8 more)

### Community 49 - "Dock.tsx"
Cohesion: 0.05
Nodes (73): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026) (+65 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.16
Nodes (28): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), frecency(), frecency_breaks_ties_but_cannot_beat_a_much_better_match() (+20 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.05
Nodes (59): SEQ, apply_styles(), arrow_keys_work_in_hotkeys(), capture_info(), Change, change_hotkey(), clamp_height(), clamp_height_limits_and_survives_garbage() (+51 more)

### Community 54 - "AppHandle"
Cohesion: 0.10
Nodes (20): dock_fit(), launcher_fit(), launcher_hide(), mc_activate(), mc_close(), mc_close_window(), mc_move_window(), mc_new_desktop() (+12 more)

### Community 55 - "indicatorstore.rs"
Cohesion: 0.11
Nodes (33): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+25 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (33): age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history(), enabled() (+25 more)

### Community 60 - "menuread.rs"
Cohesion: 0.05
Nodes (7): INFLIGHT, LAZY_TIMEOUT, MAX_INFLIGHT, MIN_UIA_MENU_ITEMS, READ_DEPTH, READ_ITEMS, UIA_TIMEOUT

### Community 61 - "mc.rs"
Cohesion: 0.05
Nodes (77): raw_app_windows(), a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP (+69 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.06
Nodes (73): a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), build_info(), capabilities_follow_the_selection_all_or_nothing() (+65 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 66 - "mcstore.rs"
Cohesion: 0.13
Nodes (30): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+22 more)

### Community 67 - "AppMenus.tsx"
Cohesion: 0.31
Nodes (8): App(), AppMenus(), rectArgs(), Clock(), formatClock(), msToNextMinute(), acceptRow(), isOpenable()

### Community 68 - "super"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.07
Nodes (16): HOTKEY_THREAD, C_SWALLOWED, SWALLOWED, HOTKEY_THREAD, detect_restart(), hide(), COUNTER, explorer_restart_detection() (+8 more)

### Community 72 - "indicators.ts"
Cohesion: 0.19
Nodes (25): I3 — Pannello: FATTA (7 nov 2026), @tauri-apps/api, Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails(), batteryFill() (+17 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "settingswin.rs"
Cohesion: 0.10
Nodes (17): a_small_monitor_shrinks_the_window_but_never_exceeds_it(), APP, create(), cursor_monitor(), HEIGHT, hide(), LABEL, MARGIN (+9 more)

### Community 75 - "vdesk.rs"
Cohesion: 0.06
Nodes (31): mc_desktops(), an_unknown_or_missing_current_desktop_falls_back_to_the_first(), BYTES, create(), current_index(), Desktop, desktops_from(), Dir (+23 more)

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "test-mc-overlay.ps1"
Cohesion: 0.20
Nodes (13): Cell-Info(), CellJs(), Esc-If-Open(), Inv(), IsOpen(), Js(), Mc(), McIdx() (+5 more)

### Community 78 - "Item"
Cohesion: 0.23
Nodes (13): Action, App, action_paths_and_flags_survive_normalization(), it(), Item, item_at(), items_are_found_by_their_model_path(), levels_and_counts_are_limited() (+5 more)

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

### Community 84 - "test-settings-s3.ps1"
Cohesion: 0.25
Nodes (10): Click-Nav(), Close-Mc(), Focus-Settings(), Inv(), Js(), Mc-Phase(), Press(), Val() (+2 more)

### Community 85 - "test-menudrop.ps1"
Cohesion: 0.26
Nodes (13): Bar-Click(), Bar-Hover(), Close-Panel(), Focus(), Inv(), Js(), Open-Menu(), Panel-Click() (+5 more)

### Community 86 - "mctrigger.rs"
Cohesion: 0.15
Nodes (21): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), CLOSE_MS, CORNER_PX, corner_step(), CornerState, DWELL_MS (+13 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "pin"
Cohesion: 0.31
Nodes (9): add(), add_ignores_duplicates_including_versioned_paths(), dedup(), dedup_keeps_the_first_of_each_program(), pin(), place(), place_pins_an_unpinned_app_at_the_given_position(), place_reorders_within_the_pinned_list() (+1 more)

### Community 89 - "settingsreg.rs"
Cohesion: 0.05
Nodes (69): settings_search(), b(), launcher_hotkey_text(), Live, want_bool(), want_list(), want_text(), a_failing_backend_reports_the_error_and_changes_nothing() (+61 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Settings.tsx"
Cohesion: 0.07
Nodes (45): Come si apre, Contenuto proposto (sezioni), Cosa c'è oggi (inventario), Cosa fa macOS (da replicare) e da dove viene, Decisioni (prese dall'utente, 8 nov 2026), Fasi, Fuori scopo, Impostazioni di EosOS — specifica (+37 more)

### Community 92 - "test-menuinvoke.ps1"
Cohesion: 0.24
Nodes (14): Bar-Click(), Close-Panel(), Focus(), Inv(), Js(), Log-Count(), Log-Has(), Log-Lines() (+6 more)

### Community 94 - "Mc.tsx"
Cohesion: 0.12
Nodes (30): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+22 more)

### Community 95 - "start"
Cohesion: 0.25
Nodes (6): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "HWND"
Cohesion: 0.31
Nodes (7): choose(), exe_stem_of(), forget_cache(), menu(), on_window(), quit_app(), run_cmd()

### Community 98 - "duration"
Cohesion: 0.15
Nodes (7): cached_icon(), cached_description(), file_description(), CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh()

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.09
Nodes (45): Read, Failed, Missing, Text, clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision (+37 more)

### Community 101 - "Readability"
Cohesion: 0.33
Nodes (6): Readability, Both, HMenu, None, Uia, Unknown

### Community 102 - ".new"
Cohesion: 0.33
Nodes (4): MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip()

### Community 103 - "launcher.rs"
Cohesion: 0.10
Nodes (28): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clips(), display() (+20 more)

### Community 104 - "AppHandle"
Cohesion: 0.16
Nodes (27): activate(), all_labels(), alt_commit(), alt_step(), APP, backdrop(), close(), close_animated() (+19 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "HWND"
Cohesion: 0.22
Nodes (12): a_shown_title_must_still_be_the_one_in_the_menu(), invoke_hmenu(), invoke_item(), invoke_uia(), InvokeErr, Changed, Failed, Gone (+4 more)

### Community 107 - "mcwin.rs"
Cohesion: 0.06
Nodes (22): Anim, apply_keys(), Change, configured_hotkey(), DEFAULT_HOTKEY, FRAME_STYLES, HOTKEY_ID, hotkey_thread() (+14 more)

### Community 108 - "handle"
Cohesion: 0.27
Nodes (8): wndproc(), c_send(), handle(), Msg, Release, Run, Swallowed, proc()

### Community 109 - "menubar.ts"
Cohesion: 0.25
Nodes (12): G4 — La tendina: FATTA (8 nov 2026), MenuPanel, firstSelectable(), flatRows(), MbAction, MbItem, MbMenu, MbRow (+4 more)

### Community 111 - "decide"
Cohesion: 0.32
Nodes (7): decide(), Decision, BackupAndSeed, KeepLast, Seed, Use, only_a_failed_read_is_unreliable()

### Community 114 - "test-menubar.ps1"
Cohesion: 0.31
Nodes (5): BarTitles(), Focus(), Js(), Row(), Wait-Until()

### Community 116 - "test-menusettings.ps1"
Cohesion: 0.29
Nodes (6): Act(), Count-Menus(), Focus(), Js(), Row(), Wait-Until()

### Community 117 - "Window"
Cohesion: 0.33
Nodes (6): dock_menu(), dock_settings_menu(), indicators_menu(), menubar_hover(), menubar_open(), panel_toggle()

### Community 119 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 123 - "read_menus"
Cohesion: 0.25
Nodes (5): abandoned_reads_are_capped(), Inflight, may_start_read(), read_hmenu_tree(), read_menus()

### Community 124 - "Phase"
Cohesion: 0.29
Nodes (7): Diag, diag(), Phase, Closed, Closing, Open, Opening

### Community 125 - "test-settingswin.ps1"
Cohesion: 0.22
Nodes (11): Click-Nav(), Click-Switch(), Heading(), Js(), Labels(), Navs(), Row-Js(), Switch-On() (+3 more)

### Community 126 - "Cmd"
Cohesion: 0.33
Nodes (6): Cmd, Close, Maximize, Minimize, Quit, Restore

### Community 127 - "open"
Cohesion: 0.24
Nodes (13): McMonitor, create_one(), cursor_monitor(), fit_exact(), hwnd_of(), icons_for(), label(), open() (+5 more)

### Community 132 - "MenuAction"
Cohesion: 0.17
Nodes (13): foreign_or_malformed_menu_ids_are_ignored(), menu_id(), menu_id_roundtrip_with_colons_in_the_id(), MenuAction, Open, Pin, Quit, SetGlass (+5 more)

### Community 137 - "rank_hits"
Cohesion: 0.53
Nodes (6): hit(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders(), rank_hits_respects_limit_and_empty()

### Community 138 - "Probe"
Cohesion: 0.47
Nodes (6): classification_covers_every_case(), classify(), p(), Probe, probe(), read_hmenu()

### Community 140 - "fold"
Cohesion: 0.25
Nodes (9): files(), short_dir(), short_dir_uses_home_and_elides_long_paths(), fold(), fold_lowercases_and_strips_common_accents(), initials(), initials_cover_words_and_inner_capitals(), match_tiers() (+1 more)

### Community 141 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 142 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 143 - "compose"
Cohesion: 0.18
Nodes (19): an_app_gets_the_hybrid_row(), build_row(), state(), the_desktop_shows_only_its_name(), Eos, an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), app_menu(), app_menu_names_the_app_and_never_is_blank() (+11 more)

### Community 144 - "clip_id"
Cohesion: 0.83
Nodes (4): clip_delete(), clip_id(), clip_ids_parse_strictly(), clip_use()

### Community 145 - "best_bar"
Cohesion: 0.25
Nodes (6): best_bar(), is_system_bar(), lazy_uia(), read_lazy_items(), read_uia(), the_title_bar_system_menu_is_not_the_app_menu()

### Community 147 - "Response"
Cohesion: 0.40
Nodes (4): dock_backdrop(), launcher_backdrop(), mc_backdrop(), panel_backdrop()

### Community 148 - "merge"
Cohesion: 0.23
Nodes (15): background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), DockItem, dump_dock(), find(), items(), items_basic(), live_items_do_not_crash_and_have_names(), mark_background() (+7 more)

### Community 149 - "test-settings.ps1"
Cohesion: 0.36
Nodes (4): All-S(), Js(), Set-S(), Val()

### Community 151 - "Menu globale per app — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, G1 — Misura in sola lettura: FATTA (8 nov 2026), G2 — Modello e disposizione (puri): FATTA (8 nov 2026), G3 — La barra: FATTA (8 nov 2026) (+5 more)

### Community 152 - "Indicatori di sistema — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti (+4 more)

### Community 159 - "run"
Cohesion: 0.60
Nodes (5): cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), run()

### Community 164 - "corner_thread"
Cohesion: 0.33
Nodes (7): monitors(), blocks_corner(), fullscreen_and_presentation_states_block_the_corner(), button_down(), corner_thread(), fullscreen_busy(), start()

### Community 167 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Confirm, Done, Failed, Unknown

### Community 169 - "Run"
Cohesion: 0.50
Nodes (4): Run, Exec, Internal, Open

## Knowledge Gaps
- **104 isolated node(s):** `Closed`, `Opening`, `Open`, `Closing`, `Appearance` (+99 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 866 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **56 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `dockact.rs` to `dock.rs`, `dockstore.rs`, `Pinned`, `collect_window`, `merge`, `package.json`, `pin`?**
  _High betweenness centrality (0.208) - this node is a cross-community bridge._
- **What connects `Closed`, `Opening`, `Open` to the rest of the system?**
  _104 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `dockact.rs`?**
  _High betweenness centrality (0.207) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `Dock.tsx` to `AppMenus.tsx`, `indicators.ts`, `menubar.ts`, `package.json`, `tokens.test.ts`, `Settings.tsx`, `Mc.tsx`?**
  _High betweenness centrality (0.087) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07078039927404718 - nodes in this community are weakly interconnected._