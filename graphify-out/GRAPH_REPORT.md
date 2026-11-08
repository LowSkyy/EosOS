# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 145 files · ~189,154 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 3076 nodes · 6580 edges · 196 communities (138 shown, 58 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 408 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2fde98ca`
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
- mclayout.rs
- test-dock-scales.ps1
- lib.rs
- launcheract.rs
- dockwin.rs
- collect_window
- indicators.rs
- launcher.ts
- launcherrank.rs
- launcherwin.rs
- menubar.ts
- indicatorstore.rs
- launcherclip.rs
- menuread.rs
- mc.rs
- NetKind
- vdint.rs
- test-mc-vdesk.ps1
- launcher.rs
- autostart.rs
- compose
- test-launcher.ps1
- tests.rs
- indicators.ts
- Indicators
- settingswin.rs
- vdesk.rs
- test-panel.ps1
- test-mc-overlay.ps1
- test-settings-s5.ps1
- folderpick.rs
- test-indicators.ps1
- mckeys.rs
- test-indicators-settings.ps1
- test-mc-desktops.ps1
- test-settings-s3.ps1
- test-menudrop.ps1
- mctrigger.rs
- read_volume
- Mc.tsx
- settingsreg.rs
- test-vdesk-internal.ps1
- Settings.tsx
- test-menuinvoke.ps1
- Config
- start
- test-mc-trigger.ps1
- Value
- duration
- test-mc-windows.ps1
- launcherstore.rs
- Readability
- .new
- InvokeErr
- AppHandle
- test-mc-keys.ps1
- tick
- mcwin.rs
- Rect
- Menu globale per app — specifica
- Mutex
- Dock.tsx
- test-menubar.ps1
- test-menusettings.ps1
- run_search
- RawItem
- KindOut
- test-settingswin.ps1
- TopMenu
- HWND
- Indicatori di sistema — specifica
- rank_hits
- AtomicBool
- LauncherItem
- fold
- super
- battery_icon
- History
- HWND
- tick
- MenuAction
- Response
- merge
- test-settings.ps1
- capture_rect
- enable
- Cmd
- Launcher — specifica
- test-settings-s4.ps1
- run
- Kind
- LPARAM
- AppMenus.tsx
- set_value
- Mission Control — specifica
- test-settings-s6.ps1
- handle
- merge
- run_child
- Cmd
- Dir
- AtomicU64
- best_bar
- vitest
- Inflight
- LPARAM
- Held
- Toggle
- Centro notifiche e calendario — specifica
- win_ctrl
- corner_thread
- Phase
- Dock — specifica
- Clock.tsx
- build_row
- mc_state
- dock_items
- dock_settings

## God Nodes (most connected - your core abstractions)
1. `Rect` - 65 edges
2. `Pinned` - 25 edges
3. `st()` - 25 edges
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
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `M2 — Panoramica: FATTA (7 nov 2026)` --references--> `neighbour()`  [INFERRED]
  docs/MISSION-CONTROL.md → src/lib/mc.ts
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Launcher()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Launcher.tsx

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (196 total, 58 thin omitted)

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
Cohesion: 0.08
Nodes (47): anchor(), anchor_to_physical(), busy_for(), clamp_height(), click_decisions(), create(), decide(), EDGE (+39 more)

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
Cohesion: 0.11
Nodes (17): on_ctrl(), appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), find(), hide_matching() (+9 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.08
Nodes (40): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), BOTTOM_LOCK (+32 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (31): ActiveApp, APP, app_from_hwnd(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop(), display_name() (+23 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 23 - "dockact.rs"
Cohesion: 0.10
Nodes (17): activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id() (+9 more)

### Community 24 - "package.json"
Cohesion: 0.08
Nodes (32): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+24 more)

### Community 25 - "menubar.rs"
Cohesion: 0.09
Nodes (30): ICON_LOCK, RUN_ORDER, menubar_current(), APP, Cache, CACHE, cache_fresh(), current() (+22 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.07
Nodes (21): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, is_known(), key(), KNOWN, KNOWN_CAP, known_folder() (+13 more)

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
Cohesion: 0.05
Nodes (17): ancestor_search_stops_at_four_levels_and_survives_cycles(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor(), helper_windows_are_attributed_to_the_pinned_parent() (+9 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (62): Pinned, pinned_dir(), read_pinned(), a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files() (+54 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.08
Nodes (16): HOTKEY_THREAD, C_CFG, C_STATE, C_SWALLOWED, C_TX, CFG, CHILD_IN, CREATE_NO_WINDOW (+8 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (28): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+20 more)

### Community 42 - "mclayout.rs"
Cohesion: 0.23
Nodes (26): a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), best_scale(), check_invariants(), empty_and_degenerate_input(), gaps_are_respected_when_there_is_room(), hit_test(), hit_testing_finds_the_cell_under_the_pointer() (+18 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.05
Nodes (56): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+48 more)

### Community 44 - "lib.rs"
Cohesion: 0.07
Nodes (31): dock_fit(), dock_menu(), dock_settings_menu(), indicators_menu(), launcher_fit(), launcher_hide(), mc_activate(), mc_close() (+23 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.08
Nodes (37): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+29 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.07
Nodes (9): build_region(), CHROME_STYLES, Geometry, LABEL, NO_ACTIVATE_STYLES, RECONCILE_EVERY, State, STATE (+1 more)

### Community 47 - "collect_window"
Cohesion: 0.15
Nodes (17): aumid_from_store(), aumid_path(), collect_window(), ComScope, identity(), is_app_window(), is_packaged_path(), packaged_aumid() (+9 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (16): duration_label(), duration_labels(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume (+8 more)

### Community 49 - "launcher.ts"
Cohesion: 0.17
Nodes (26): Launcher(), clampSelection(), combine(), fallbackLetter(), Filter, filterForKey(), filterLabel(), glyphFor() (+18 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.16
Nodes (28): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), frecency(), frecency_breaks_ties_but_cannot_beat_a_much_better_match() (+20 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.05
Nodes (55): apply_styles(), arrow_keys_work_in_hotkeys(), capture_info(), Change, change_hotkey(), clamp_height(), clamp_height_limits_and_survives_garbage(), configured_hotkey() (+47 more)

### Community 54 - "menubar.ts"
Cohesion: 0.25
Nodes (12): G4 — La tendina: FATTA (8 nov 2026), MenuPanel, firstSelectable(), flatRows(), MbAction, MbItem, MbMenu, MbRow (+4 more)

### Community 55 - "indicatorstore.rs"
Cohesion: 0.06
Nodes (66): Read, Failed, Missing, a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh (+58 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.06
Nodes (17): age_label(), age_labels(), CF_UNICODETEXT, copy_back(), format_id(), HISTORY, MAX_CHARS, MAX_ENTRIES (+9 more)

### Community 60 - "menuread.rs"
Cohesion: 0.06
Nodes (7): INFLIGHT, LAZY_TIMEOUT, MAX_INFLIGHT, MIN_UIA_MENU_ITEMS, READ_DEPTH, READ_ITEMS, UIA_TIMEOUT

### Community 61 - "mc.rs"
Cohesion: 0.08
Nodes (37): raw_app_windows(), a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP (+29 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.06
Nodes (73): a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), build_info(), capabilities_follow_the_selection_all_or_nothing() (+65 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 66 - "launcher.rs"
Cohesion: 0.09
Nodes (32): AppEntry, apps(), cached(), clean(), clean_also_drops_noise(), clean_drops_empty_and_duplicates(), clip_delete(), clip_id() (+24 more)

### Community 67 - "autostart.rs"
Cohesion: 0.09
Nodes (12): APPROVED, blocked_by_user(), current_target(), exe_of(), NAME, open(), read(), RUN (+4 more)

### Community 68 - "compose"
Cohesion: 0.29
Nodes (12): an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), compose(), is_window_menu(), it(), lazy_menus_stay_in_the_bar_even_when_empty(), levels_and_counts_are_limited(), normalize(), separators_are_cleaned_and_empty_items_dropped() (+4 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.08
Nodes (12): detect_restart(), hide(), restore(), explorer_restart_detection(), FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding() (+4 more)

### Community 72 - "indicators.ts"
Cohesion: 0.20
Nodes (24): I3 — Pannello: FATTA (7 nov 2026), Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails(), batteryFill(), DetailRow (+16 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "settingswin.rs"
Cohesion: 0.10
Nodes (22): settings_close(), a_small_monitor_shrinks_the_window_but_never_exceeds_it(), APP, create(), cursor_monitor(), HEIGHT, hide(), is_menu_id() (+14 more)

### Community 75 - "vdesk.rs"
Cohesion: 0.09
Nodes (20): mc_desktops(), an_unknown_or_missing_current_desktop_falls_back_to_the_first(), BYTES, current_index(), Desktop, desktops_from(), guid_from_bytes(), guids_are_read_in_windows_byte_order() (+12 more)

### Community 76 - "test-panel.ps1"
Cohesion: 0.19
Nodes (9): BarWindowFor(), Click-Indicator(), Panel(), PanelFocused(), PanelOpen(), PanelWin(), PJs(), Press() (+1 more)

### Community 77 - "test-mc-overlay.ps1"
Cohesion: 0.20
Nodes (13): Cell-Info(), CellJs(), Esc-If-Open(), Inv(), IsOpen(), Js(), Mc(), McIdx() (+5 more)

### Community 78 - "test-settings-s5.ps1"
Cohesion: 0.24
Nodes (14): Choose-Settings-In-Panel(), Close-Panel(), Close-Settings(), Eos-Pids(), Focus(), Inv(), Js(), Native-Last() (+6 more)

### Community 80 - "test-indicators.ps1"
Cohesion: 0.24
Nodes (3): Event-Count(), Get-State(), Js()

### Community 81 - "mckeys.rs"
Cohesion: 0.20
Nodes (22): alt_tab(), alt_tab_opens_then_cycles_and_commits_on_alt_release(), cycle(), cycling_starts_from_the_second_window_and_wraps(), disabled_shortcuts_are_never_touched(), escape_cancels_and_a_later_alt_release_does_not_commit(), is_alt(), Key (+14 more)

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

### Community 88 - "Mc.tsx"
Cohesion: 0.27
Nodes (17): Mc(), advancedHint(), boxStyle(), canRemove(), cardLetter(), cellTitle(), cleanDesktopName(), desktopAction() (+9 more)

### Community 89 - "settingsreg.rs"
Cohesion: 0.14
Nodes (16): fold(), folding_removes_accents_and_case(), GLASS, LANGUAGES, LAUNCHER_HOTKEYS, MAX_FOLDERS, MAX_NAME_LEN, MAX_NAMES (+8 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Settings.tsx"
Cohesion: 0.06
Nodes (51): Come si apre, Contenuto proposto (sezioni), Cosa c'è oggi (inventario), Cosa fa macOS (da replicare) e da dove viene, Decisioni (prese dall'utente, 8 nov 2026), Fasi, Fuori scopo, Impostazioni di EosOS — specifica (+43 more)

### Community 92 - "test-menuinvoke.ps1"
Cohesion: 0.24
Nodes (14): Bar-Click(), Close-Panel(), Focus(), Inv(), Js(), Log-Count(), Log-Has(), Log-Lines() (+6 more)

### Community 94 - "Config"
Cohesion: 0.24
Nodes (12): cfg(), config_line(), config_lines_round_trip_and_garbage_is_ignored(), Diag, diag(), may_restart(), parse_config_line(), send_cfg() (+4 more)

### Community 95 - "start"
Cohesion: 0.25
Nodes (6): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "Value"
Cohesion: 0.16
Nodes (15): real_keys(), b(), launcher_hotkey_text(), Live, want_bool(), want_list(), want_text(), Backend (+7 more)

### Community 98 - "duration"
Cohesion: 0.14
Nodes (8): cached_icon(), ICONS, SHELL_NAMES, CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh(), TimedCache

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.10
Nodes (42): set_clipboard(), start(), clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh (+34 more)

### Community 101 - "Readability"
Cohesion: 0.22
Nodes (10): classification_covers_every_case(), classify(), p(), Probe, Readability, Both, HMenu, None (+2 more)

### Community 102 - ".new"
Cohesion: 0.33
Nodes (4): MtaScope, real_read_smoke(), real_set_volume(), real_volume_roundtrip()

### Community 103 - "InvokeErr"
Cohesion: 0.21
Nodes (12): a_shown_title_must_still_be_the_one_in_the_menu(), invoke_hmenu(), invoke_item(), invoke_uia(), InvokeErr, Changed, Failed, Gone (+4 more)

### Community 104 - "AppHandle"
Cohesion: 0.18
Nodes (26): activate(), all_labels(), alt_commit(), alt_step(), APP, backdrop(), close(), close_animated() (+18 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "tick"
Cohesion: 0.27
Nodes (9): set_reserve(), create(), fit(), lock(), reconcile(), reserve_allowed(), reserve_enabled(), start() (+1 more)

### Community 107 - "mcwin.rs"
Cohesion: 0.06
Nodes (22): Anim, apply_keys(), Change, configured_hotkey(), DEFAULT_HOTKEY, FRAME_STYLES, HOTKEY_ID, hotkey_thread() (+14 more)

### Community 108 - "Rect"
Cohesion: 0.14
Nodes (16): Rect, inside(), thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), AREA, css_conversion_is_relative_to_the_monitor_and_scaled(), fit_in(), monitor_of() (+8 more)

### Community 109 - "Menu globale per app — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, G1 — Misura in sola lettura: FATTA (8 nov 2026), G2 — Modello e disposizione (puri): FATTA (8 nov 2026), G3 — La barra: FATTA (8 nov 2026) (+5 more)

### Community 111 - "Dock.tsx"
Cohesion: 0.21
Nodes (23): Fasi, cssPx(), Dock(), Ghost(), Icon(), applyMove(), clampGlass(), DockItem (+15 more)

### Community 114 - "test-menubar.ps1"
Cohesion: 0.31
Nodes (5): BarTitles(), Focus(), Js(), Row(), Wait-Until()

### Community 116 - "test-menusettings.ps1"
Cohesion: 0.29
Nodes (6): Act(), Count-Menus(), Focus(), Js(), Row(), Wait-Until()

### Community 119 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 123 - "RawItem"
Cohesion: 0.29
Nodes (9): RawItem, RawMenu, lazy_uia(), probe(), read_hmenu(), read_hmenu_tree(), read_lazy_items(), read_menus() (+1 more)

### Community 124 - "KindOut"
Cohesion: 0.11
Nodes (19): Entry, kind_out(), KindOut, Action, Bool, Choice, Folders, Hotkey (+11 more)

### Community 125 - "test-settingswin.ps1"
Cohesion: 0.22
Nodes (11): Click-Nav(), Click-Switch(), Heading(), Js(), Labels(), Navs(), Row-Js(), Switch-On() (+3 more)

### Community 126 - "TopMenu"
Cohesion: 0.22
Nodes (12): menubar_menu(), is_lazy(), menu(), Item, lazy_items_get_paths_from_their_source_menu(), paths_keep_pointing_at_the_original_position_after_cleaning(), raw_item(), raw_items() (+4 more)

### Community 127 - "HWND"
Cohesion: 0.22
Nodes (8): choose(), exe_stem_of(), forget_cache(), quit_app(), run_cmd(), item_at(), items_are_found_by_their_model_path(), open_soon()

### Community 132 - "Indicatori di sistema — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti (+4 more)

### Community 137 - "rank_hits"
Cohesion: 0.53
Nodes (6): hit(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders(), rank_hits_respects_limit_and_empty()

### Community 138 - "AtomicBool"
Cohesion: 0.20
Nodes (9): AUTOHIDE, BLUR, MENU_OPEN, RESERVE, ENABLED, CORNER, EXCLUDED, VISIBLE (+1 more)

### Community 140 - "fold"
Cohesion: 0.25
Nodes (9): files(), short_dir(), short_dir_uses_home_and_elides_long_paths(), fold(), fold_lowercases_and_strips_common_accents(), initials(), initials_cover_words_and_inner_capitals(), match_tiers() (+1 more)

### Community 141 - "super"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 142 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 143 - "History"
Cohesion: 0.15
Nodes (15): capped_at_max_entries_dropping_the_oldest(), clear(), disabling_clears_the_global_history(), enabled(), entries_expire_after_the_ttl(), Entry, hist(), History (+7 more)

### Community 144 - "HWND"
Cohesion: 0.21
Nodes (14): McMonitor, create_one(), cursor_monitor(), fit_exact(), hwnd_of(), icons_for(), label(), open() (+6 more)

### Community 145 - "tick"
Cohesion: 0.22
Nodes (9): slide_covers_the_whole_travel_with_ease_out(), slide_y(), set_autohide(), autohide(), cursor(), enforce_styles(), hwnd_of(), tick() (+1 more)

### Community 146 - "MenuAction"
Cohesion: 0.22
Nodes (9): MenuAction, Open, Pin, Quit, SetGlass, ToggleAutoHide, ToggleBlur, ToggleReserve (+1 more)

### Community 147 - "Response"
Cohesion: 0.40
Nodes (4): dock_backdrop(), launcher_backdrop(), mc_backdrop(), panel_backdrop()

### Community 148 - "merge"
Cohesion: 0.17
Nodes (18): app_display_name(), apply_run_order(), cached_shell_name(), DockItem, dump_dock(), find(), find_with_windows(), items() (+10 more)

### Community 149 - "test-settings.ps1"
Cohesion: 0.36
Nodes (4): All-S(), Js(), Set-S(), Val()

### Community 150 - "capture_rect"
Cohesion: 0.25
Nodes (9): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), set_blur(), blur_enabled(), capture_info(), toggle_blur() (+1 more)

### Community 151 - "enable"
Cohesion: 0.32
Nodes (13): a_missing_or_relative_exe_is_refused_and_nothing_is_written(), a_task_manager_block_counts_as_off_and_turning_on_clears_it(), clean(), delete_value(), disable(), enable(), enable_and_disable_touch_only_our_value(), is_enabled() (+5 more)

### Community 152 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 153 - "Launcher — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+5 more)

### Community 158 - "test-settings-s4.ps1"
Cohesion: 0.24
Nodes (4): Inv(), Js(), Val(), Win()

### Community 159 - "run"
Cohesion: 0.32
Nodes (6): cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), run(), on_window_event()

### Community 164 - "Kind"
Cohesion: 0.25
Nodes (8): Kind, Action, Bool, Choice, Folders, Hotkey, Names, Pinned

### Community 165 - "LPARAM"
Cohesion: 0.31
Nodes (9): app_windows_for(), is_launcher(), key(), launcher_names(), live_process_scan_excludes_self_and_finds_explorer(), may_inherit(), only_real_helpers_inherit_the_ancestors_icon(), processes_named() (+1 more)

### Community 166 - "AppMenus.tsx"
Cohesion: 0.46
Nodes (6): @tauri-apps/api, App(), AppMenus(), rectArgs(), acceptRow(), isOpenable()

### Community 167 - "set_value"
Cohesion: 0.18
Nodes (18): a_failing_backend_reports_the_error_and_changes_nothing(), Applied, choices_accept_only_their_options(), clean_list(), folders_must_be_absolute_and_safe(), hotkeys_are_validated_and_written_in_a_canonical_form(), l(), name_lists_are_cleaned() (+10 more)

### Community 168 - "Mission Control — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio), Fasi, Fuori scopo, M1 — Dati e disposizione: FATTA (7 nov 2026), M2 — Panoramica: FATTA (7 nov 2026), M3 — Apertura e animazione: FATTA (7-8 nov 2026), M4 — Più monitor, scale e fascia dei desktop (backend `Documented`): FATTA (8 nov 2026) (+5 more)

### Community 169 - "test-settings-s6.ps1"
Cohesion: 0.39
Nodes (5): Active(), Heading(), Js(), Key(), Win()

### Community 170 - "handle"
Cohesion: 0.33
Nodes (7): c_cfg(), c_send(), handle(), Msg, Release, Run, Swallowed

### Community 172 - "merge"
Cohesion: 0.29
Nodes (12): name, background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), only_a_single_version_folder_counts_as_the_same_app(), pin(), run() (+4 more)

### Community 173 - "run_child"
Cohesion: 0.29
Nodes (7): down(), is_extended(), release_after_neutral(), run_child(), say(), a_lost_alt_release_is_noticed_by_the_safety_net(), stuck()

### Community 174 - "Cmd"
Cohesion: 0.16
Nodes (13): Action, App, Eos, action_paths_and_flags_survive_normalization(), Cmd, Close, Maximize, Minimize (+5 more)

### Community 178 - "Dir"
Cohesion: 0.33
Nodes (6): Dir, Left, Right, plan_switch(), switch_to(), switching_plans_the_right_number_of_steps()

### Community 179 - "AtomicU64"
Cohesion: 0.33
Nodes (5): SEQ, SCALE_BITS, SHOWN_AT_MS, BUSY_UNTIL_MS, SCALE_BITS

### Community 180 - "best_bar"
Cohesion: 0.40
Nodes (3): best_bar(), is_system_bar(), the_title_bar_system_menu_is_not_the_app_menu()

### Community 181 - "vitest"
Cohesion: 0.33
Nodes (7): vitest@^5.1.0, vitest, useBackdrop(), Frame, FRAME_HEADER, parseFrame(), sourceRect()

### Community 182 - "Inflight"
Cohesion: 0.40
Nodes (3): abandoned_reads_are_capped(), Inflight, may_start_read()

### Community 183 - "LPARAM"
Cohesion: 0.31
Nodes (3): wndproc(), proc(), wndproc()

### Community 184 - "Held"
Cohesion: 0.50
Nodes (4): Held, Alt, None, Win

### Community 185 - "Toggle"
Cohesion: 0.50
Nodes (4): Toggle, Hide, Ignore, Show

### Community 186 - "Centro notifiche e calendario — specifica"
Cohesion: 0.22
Nodes (8): Centro notifiche e calendario — specifica, Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisioni che servono da te, Fasi, Fuori scopo, Principi (gli stessi di sempre), Rischi aperti

### Community 187 - "win_ctrl"
Cohesion: 0.38
Nodes (5): create(), key_event(), key_events_are_well_formed(), remove_current(), win_ctrl()

### Community 188 - "corner_thread"
Cohesion: 0.33
Nodes (7): monitors(), blocks_corner(), fullscreen_and_presentation_states_block_the_corner(), button_down(), corner_thread(), fullscreen_busy(), start()

### Community 189 - "Phase"
Cohesion: 0.29
Nodes (7): Diag, diag(), Phase, Closed, Closing, Open, Opening

### Community 190 - "Dock — specifica"
Cohesion: 0.33
Nodes (5): Comportamento (da replicare), Dock — specifica, Fuori scopo, Rischi aperti, Scelte tecniche

### Community 191 - "Clock.tsx"
Cohesion: 0.73
Nodes (3): Clock(), formatClock(), msToNextMinute()

### Community 192 - "build_row"
Cohesion: 0.60
Nodes (5): an_app_gets_the_hybrid_row(), build_row(), state(), the_desktop_shows_only_its_name(), WinState

## Knowledge Gaps
- **108 isolated node(s):** `BackupAndFresh`, `Fresh`, `Use`, `Both`, `HMenu` (+103 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 902 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **58 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `merge` to `dock.rs`, `dockstore.rs`, `LPARAM`, `merge`, `dockact.rs`?**
  _High betweenness centrality (0.141) - this node is a cross-community bridge._
- **What connects `BackupAndFresh`, `Fresh`, `Use` to the rest of the system?**
  _108 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `merge` to `package.json`?**
  _High betweenness centrality (0.141) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `vitest` to `indicators.ts`, `Settings.tsx`, `Dock.tsx`, `launcher.ts`, `menubar.ts`, `package.json`, `tokens.test.ts`, `Mc.tsx`, `Clock.tsx`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07826694619147449 - nodes in this community are weakly interconnected._