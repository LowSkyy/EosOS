# Graph Report - EosOS  (2026-11-08)

## Corpus Check
- 142 files · ~185,523 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 3050 nodes · 6538 edges · 179 communities (120 shown, 59 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 403 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `eab455cb`
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
- mcstore.rs
- autostart.rs
- super
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
- Dock.tsx
- settingsreg.rs
- test-vdesk-internal.ps1
- Settings.tsx
- test-menuinvoke.ps1
- Impostazioni di EosOS — specifica
- start
- test-mc-trigger.ps1
- Value
- duration
- test-mc-windows.ps1
- launcherstore.rs
- Readability
- .new
- launcher.rs
- AppHandle
- test-mc-keys.ps1
- tick
- mcwin.rs
- Rect
- Menu globale per app — specifica
- Mutex
- main.tsx
- test-menubar.ps1
- test-menusettings.ps1
- run_search
- Hotkey
- KindOut
- test-settingswin.ps1
- TopMenu
- open
- Indicatori di sistema — specifica
- rank_hits
- HWND
- dock_items
- fold
- SettingsError
- battery_icon
- list
- clip_id
- Phase
- MenuAction
- Response
- merge
- test-settings.ps1
- capture_rect
- enable
- Cmd
- dock_settings
- test-settings-s4.ps1
- run
- Kind
- scripts
- Clock.tsx
- l
- AtomicIsize
- ActiveApp
- LauncherItem
- same_app
- detect_restart
- KEEP_HIDDEN
- Dir

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
- `S3 — Le altre sezioni: FATTA (8 nov 2026)` --references--> `comboFromEvent()`  [INFERRED]
  docs/IMPOSTAZIONI.md → src/lib/settings.ts
- `G3 — La barra: FATTA (8 nov 2026)` --references--> `acceptRow()`  [INFERRED]
  docs/MENU-GLOBALE.md → src/lib/menubar.ts
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `useBackdrop()`  [INFERRED]
  docs/LAUNCHER.md → src/components/useBackdrop.ts
- `Scelte tecniche` --references--> `Dock()`  [INFERRED]
  docs/DOCK.md → src/components/Dock.tsx
- `L6 — Rifiniture: FATTA, tranne una prova (7 nov 2026)` --references--> `Dock()`  [INFERRED]
  docs/LAUNCHER.md → src/components/Dock.tsx

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (179 total, 59 thin omitted)

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
Nodes (56): SEQ, SCALE_BITS, SHOWN_AT_MS, anchor(), anchor_to_physical(), busy_for(), BUSY_UNTIL_MS, clamp_height() (+48 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.06
Nodes (12): spawn_child(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name() (+4 more)

### Community 11 - "backdrop.rs"
Cohesion: 0.07
Nodes (23): capture_bytes(), Channel, Dock, Launcher, Panel, CHANNELS, downscale(), downscale_averages_blocks_and_swaps_to_rgba() (+15 more)

### Community 15 - "taskbar.rs"
Cohesion: 0.15
Nodes (16): on_ctrl(), appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), find(), hide_matching() (+8 more)

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
Cohesion: 0.10
Nodes (17): activate(), activate_or_launch(), alt_or_win_down(), focus(), foreign_or_malformed_menu_ids_are_ignored(), launch(), menu_id(), menu_id_roundtrip_with_colons_in_the_id() (+9 more)

### Community 24 - "package.json"
Cohesion: 0.12
Nodes (21): devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2 (+13 more)

### Community 25 - "menubar.rs"
Cohesion: 0.07
Nodes (40): ICON_LOCK, RUN_ORDER, menubar_choose(), menubar_current(), an_app_gets_the_hybrid_row(), APP, build_row(), Cache (+32 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.09
Nodes (16): [cmd, a, b], evaluate(), list, pending, send(), ws, BLACK, contrast() (+8 more)

### Community 27 - "launcherfiles.rs"
Cohesion: 0.07
Nodes (21): FALLBACK_BUDGET, FALLBACK_DEPTH, FALLBACK_MAX_ENTRIES, is_known(), key(), KNOWN, KNOWN_CAP, known_folder() (+13 more)

### Community 28 - "menumodel.rs"
Cohesion: 0.10
Nodes (40): an_app_that_has_its_own_window_menu_does_not_get_a_second_one(), app_menu(), app_menu_names_the_app_and_never_is_blank(), browsers_and_chats_are_never_queried(), compose(), exclusions_ignore_case_and_toggle(), is_excluded(), is_window_menu() (+32 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

### Community 37 - "dock.rs"
Cohesion: 0.05
Nodes (22): ancestor_search_stops_at_four_levels_and_survives_cycles(), app_windows_for(), APPS_FOLDER, bgra_is_swapped_and_unpremultiplied(), bgra_to_rgba(), bitmap_to_rgba(), encode_png(), find_ancestor() (+14 more)

### Community 38 - "dockstore.rs"
Cohesion: 0.07
Nodes (66): Pinned, a_leading_bom_is_tolerated(), a_read_error_never_reseeds_or_overwrites_the_file(), add(), add_ignores_duplicates_including_versioned_paths(), autohide_defaults_to_false_for_old_files(), blur_is_on_by_default_and_roundtrips(), clamp_glass() (+58 more)

### Community 39 - "launchercalc.rs"
Cohesion: 0.13
Nodes (25): a_bare_number_or_percentage_is_not_a_calculation(), alternative_operator_symbols(), app_names_and_text_do_not_evaluate(), basic_arithmetic_and_precedence(), close(), decimals_accept_comma_and_dot(), division_by_zero_and_overflow_give_nothing(), ev() (+17 more)

### Community 40 - "mchook.rs"
Cohesion: 0.06
Nodes (39): HOTKEY_THREAD, C_CFG, c_cfg(), c_send(), C_STATE, C_SWALLOWED, C_TX, cfg() (+31 more)

### Community 41 - "dockgeom.rs"
Cohesion: 0.12
Nodes (29): args(), at_bottom_edge(), BLUR_MARGIN_CSS, bottom_edge_is_the_last_pixel_row_of_the_monitor(), BOTTOM_MARGIN, capsule_rect(), Fit, fit() (+21 more)

### Community 42 - "mclayout.rs"
Cohesion: 0.23
Nodes (26): a_huge_window_is_scaled_down_keeping_its_proportions(), a_single_small_window_keeps_its_natural_size_and_is_centered(), best_scale(), check_invariants(), empty_and_degenerate_input(), gaps_are_respected_when_there_is_room(), hit_test(), hit_testing_finds_the_cell_under_the_pointer() (+18 more)

### Community 43 - "test-dock-scales.ps1"
Cohesion: 0.06
Nodes (52): Get-Monitors(), Get-MonitorScale(), Get-MonitorScaleRange(), Get-PrimaryGdi(), Get-PrimaryScale(), Get-PrimaryScaleRange(), Set-MonitorScale(), Set-PrimaryScale() (+44 more)

### Community 44 - "lib.rs"
Cohesion: 0.07
Nodes (28): dock_menu(), dock_settings_menu(), indicators_menu(), launcher_fit(), launcher_hide(), mc_activate(), mc_close(), mc_close_window() (+20 more)

### Community 45 - "launcheract.rs"
Cohesion: 0.08
Nodes (37): a_clock_going_backwards_never_confirms(), a_plain_action_in_between_cancels_the_pending_confirmation(), act(), Action, ACTIONS, commands_are_fixed_and_exact(), CONFIRM_WINDOW, confirmation_expires_and_does_not_transfer_between_actions() (+29 more)

### Community 46 - "dockwin.rs"
Cohesion: 0.05
Nodes (20): AUTOHIDE, BLUR, build_region(), CHROME_STYLES, Geometry, LABEL, MENU_OPEN, NO_ACTIVATE_STYLES (+12 more)

### Community 47 - "collect_window"
Cohesion: 0.13
Nodes (20): aumid_from_store(), aumid_path(), collect_window(), ComScope, identity(), is_app_window(), is_packaged_path(), packaged_aumid() (+12 more)

### Community 48 - "indicators.rs"
Cohesion: 0.11
Nodes (16): duration_label(), duration_labels(), Fake, fake_file_volume_edit_keeps_everything_else(), fake_with_volume(), FakeBattery, FakeNetwork, FakeVolume (+8 more)

### Community 49 - "launcher.ts"
Cohesion: 0.09
Nodes (39): Cosa fa Spotlight (da replicare) e da dove viene, Decisioni dell'utente (7 nov 2026), Fasi, Fuori scopo, L1 — Dati e classifica: FATTA (7 nov 2026), L2 — Finestra e scorciatoia: FATTA (7 nov 2026), L3 — Azioni e calcolatrice: FATTA (7 nov 2026), L4 — File: FATTA (7 nov 2026) (+31 more)

### Community 52 - "launcherrank.rs"
Cohesion: 0.16
Nodes (28): accents_and_case_do_not_matter(), bonus(), bonus_is_capped(), DAY, empty_query_lists_only_used_apps_by_frecency(), entries(), frecency(), frecency_breaks_ties_but_cannot_beat_a_much_better_match() (+20 more)

### Community 53 - "launcherwin.rs"
Cohesion: 0.06
Nodes (41): apply_styles(), Change, clamp_height(), clamp_height_limits_and_survives_garbage(), create(), EXCLUDED, fit(), FOCUS_CHECK (+33 more)

### Community 54 - "menubar.ts"
Cohesion: 0.19
Nodes (17): G4 — La tendina: FATTA (8 nov 2026), App(), AppMenus(), rectArgs(), MenuPanel, acceptRow(), firstSelectable(), flatRows() (+9 more)

### Community 55 - "indicatorstore.rs"
Cohesion: 0.11
Nodes (33): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+25 more)

### Community 59 - "launcherclip.rs"
Cohesion: 0.05
Nodes (34): age_label(), age_labels(), capped_at_max_entries_dropping_the_oldest(), CF_UNICODETEXT, clear(), copy_back(), disabling_clears_the_global_history(), enabled() (+26 more)

### Community 60 - "menuread.rs"
Cohesion: 0.04
Nodes (38): RawItem, RawMenu, a_shown_title_must_still_be_the_one_in_the_menu(), abandoned_reads_are_capped(), best_bar(), classification_covers_every_case(), classify(), Inflight (+30 more)

### Community 61 - "mc.rs"
Cohesion: 0.08
Nodes (37): a_window_without_a_registered_thumbnail_falls_back_to_a_card(), area_for(), BAND, build(), CARD, CELL_PAD, GAP, geometry() (+29 more)

### Community 62 - "NetKind"
Cohesion: 0.15
Nodes (19): Link, link(), NetKind, Cellular, Ethernet, None, Wifi, Network (+11 more)

### Community 63 - "vdint.rs"
Cohesion: 0.05
Nodes (73): a_crash_flag_disables_the_internal_backend_until_it_is_removed(), ARRAY_GET_AT, ARRAY_GET_COUNT, array_items(), Attempt, attempt(), build_info(), capabilities_follow_the_selection_all_or_nothing() (+65 more)

### Community 65 - "test-mc-vdesk.ps1"
Cohesion: 0.23
Nodes (11): CellJs(), Close-Mc(), Inv(), IsOpen(), Js(), Mc(), McIdx(), Monitor-Of() (+3 more)

### Community 66 - "mcstore.rs"
Cohesion: 0.13
Nodes (30): a_failed_read_is_never_treated_as_missing(), decide(), Decision, BackupAndFresh, Fresh, KeepLast, Use, last() (+22 more)

### Community 67 - "autostart.rs"
Cohesion: 0.09
Nodes (11): APPROVED, current_target(), exe_of(), NAME, open(), read(), RUN, run_value() (+3 more)

### Community 68 - "super"
Cohesion: 0.12
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 70 - "test-launcher.ps1"
Cohesion: 0.19
Nodes (8): ClipRows(), Focused(), IsOpen(), Js(), Launcher(), Press(), PressCtrl(), TypeText()

### Community 71 - "tests.rs"
Cohesion: 0.08
Nodes (12): proc(), hide(), restore(), FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding(), restore_is_idempotent_and_safe_when_nothing_hidden() (+4 more)

### Community 72 - "indicators.ts"
Cohesion: 0.19
Nodes (25): I3 — Pannello: FATTA (7 nov 2026), @tauri-apps/api, Glyph(), Indicators(), Panel(), useIndicators(), batteryDetails(), batteryFill() (+17 more)

### Community 73 - "Indicators"
Cohesion: 0.23
Nodes (12): Battery, battery_from_status(), battery_from_windows_status(), battery_texts_always_show_the_percentage(), current(), Indicators, read_battery(), read_fake() (+4 more)

### Community 74 - "settingswin.rs"
Cohesion: 0.11
Nodes (22): settings_close(), a_small_monitor_shrinks_the_window_but_never_exceeds_it(), APP, create(), cursor_monitor(), HEIGHT, hide(), is_menu_id() (+14 more)

### Community 75 - "vdesk.rs"
Cohesion: 0.10
Nodes (14): BYTES, create(), guid_from_bytes(), guids_are_read_in_windows_byte_order(), id(), key_event(), key_events_are_well_formed(), parse_ids() (+6 more)

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
Cohesion: 0.12
Nodes (26): a_blocked_corner_never_fires_and_the_dwell_restarts_when_unblocked(), a_clock_going_backwards_never_fires(), a_thumbnail_travels_there_and_back(), blocks_corner(), CLOSE_MS, CORNER_PX, corner_step(), CornerState (+18 more)

### Community 87 - "read_volume"
Cohesion: 0.33
Nodes (6): device_name(), read_volume(), Volume, volume_from_scalar(), volume_from_scalar_is_robust(), volume_title_names_the_device()

### Community 88 - "Dock.tsx"
Cohesion: 0.06
Nodes (63): Comportamento (da replicare), Dock — specifica, Fasi, Fuori scopo, Rischi aperti, Scelte tecniche, Cosa fa macOS (da replicare) e da dove viene, Cosa permette Windows (e dove sta il rischio) (+55 more)

### Community 89 - "settingsreg.rs"
Cohesion: 0.11
Nodes (28): a_failing_backend_reports_the_error_and_changes_nothing(), Applied, choices_accept_only_their_options(), clean_list(), fold(), folding_removes_accents_and_case(), GLASS, hotkeys_are_validated_and_written_in_a_canonical_form() (+20 more)

### Community 90 - "test-vdesk-internal.ps1"
Cohesion: 0.30
Nodes (13): CellJs(), Click-Js(), Desktops(), Drag-Cell-To-Chip(), Inv(), Js(), McIdx(), Open-Mc() (+5 more)

### Community 91 - "Settings.tsx"
Cohesion: 0.17
Nodes (25): FolderList(), HotkeyField(), NameList(), PinnedList(), Row(), Settings(), addName(), asList() (+17 more)

### Community 92 - "test-menuinvoke.ps1"
Cohesion: 0.24
Nodes (14): Bar-Click(), Close-Panel(), Focus(), Inv(), Js(), Log-Count(), Log-Has(), Log-Lines() (+6 more)

### Community 94 - "Impostazioni di EosOS — specifica"
Cohesion: 0.12
Nodes (16): Come si apre, Contenuto proposto (sezioni), Cosa c'è oggi (inventario), Cosa fa macOS (da replicare) e da dove viene, Decisioni (prese dall'utente, 8 nov 2026), Fasi, Fuori scopo, Impostazioni di EosOS — specifica (+8 more)

### Community 95 - "start"
Cohesion: 0.25
Nodes (6): apply_settings(), settings_hide_indicators_and_the_battery_percentage(), start(), update_state(), update_state_reports_only_real_changes(), wait_next()

### Community 96 - "test-mc-trigger.ps1"
Cohesion: 0.27
Nodes (8): Close-Mc(), Esc-If-Open(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 97 - "Value"
Cohesion: 0.25
Nodes (9): real_keys(), b(), launcher_hotkey_text(), Live, want_bool(), want_list(), want_text(), Value (+1 more)

### Community 98 - "duration"
Cohesion: 0.14
Nodes (9): cached_icon(), ICONS, SHELL_NAMES, APP, CACHE_FOUND_TTL, CACHE_MISSING_TTL, failures_expire_quickly_and_successes_slowly(), is_fresh() (+1 more)

### Community 99 - "test-mc-windows.ps1"
Cohesion: 0.38
Nodes (3): All-Windows(), Mc(), Wait-Until-Gone()

### Community 100 - "launcherstore.rs"
Cohesion: 0.10
Nodes (41): clipboard_consent_defaults_to_unasked_and_roundtrips(), decide(), decide_never_treats_a_failed_read_as_missing(), Decision, BackupAndFresh, Fresh, KeepLast, Use (+33 more)

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
Cohesion: 0.22
Nodes (22): activate(), all_labels(), alt_commit(), alt_step(), close(), close_animated(), close_window(), hwnd_from() (+14 more)

### Community 105 - "test-mc-keys.ps1"
Cohesion: 0.18
Nodes (8): Close-Mc(), Focus(), Inv(), IsOpen(), Js(), Overlays(), St(), Wait-Until()

### Community 106 - "tick"
Cohesion: 0.19
Nodes (14): slide_covers_the_whole_travel_with_ease_out(), slide_y(), autohide(), create(), cursor(), enforce_styles(), fit(), hwnd_of() (+6 more)

### Community 107 - "mcwin.rs"
Cohesion: 0.06
Nodes (21): apply_keys(), backdrop(), Change, DEFAULT_HOTKEY, FRAME_STYLES, HOTKEY_ID, hotkey_thread(), keys_config() (+13 more)

### Community 108 - "Rect"
Cohesion: 0.14
Nodes (16): Rect, inside(), thumb_rect(), thumbnails_leave_room_for_padding_and_the_label_and_keep_the_source_proportions(), AREA, css_conversion_is_relative_to_the_monitor_and_scaled(), fit_in(), monitor_of() (+8 more)

### Community 109 - "Menu globale per app — specifica"
Cohesion: 0.14
Nodes (13): Cosa fa macOS (da replicare), Cosa permette Windows (e dove sta il rischio), Decisione che serve da te prima di costruire, Fasi (stesso metodo del resto: pure, testate, prove dal vivo con finestre di prova), Fuori scopo, G1 — Misura in sola lettura: FATTA (8 nov 2026), G2 — Modello e disposizione (puri): FATTA (8 nov 2026), G3 — La barra: FATTA (8 nov 2026) (+5 more)

### Community 111 - "main.tsx"
Cohesion: 0.13
Nodes (15): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+7 more)

### Community 114 - "test-menubar.ps1"
Cohesion: 0.31
Nodes (5): BarTitles(), Focus(), Js(), Row(), Wait-Until()

### Community 116 - "test-menusettings.ps1"
Cohesion: 0.29
Nodes (6): Act(), Count-Menus(), Focus(), Js(), Row(), Wait-Until()

### Community 119 - "run_search"
Cohesion: 0.17
Nodes (13): bounded_walk(), bounded_walk_finds_by_name_within_depth_and_skips_noise(), bounded_walk_stops_at_budget_and_deadline_and_survives_missing_roots(), Hit, index_query(), MtaScope, real_search_smoke(), run_search() (+5 more)

### Community 123 - "Hotkey"
Cohesion: 0.23
Nodes (14): arrow_keys_work_in_hotkeys(), change_hotkey(), configured_hotkey(), dangerous_or_malformed_hotkeys_are_rejected(), DEFAULT_HOTKEY, display_hotkey(), format_hotkey(), Hotkey (+6 more)

### Community 124 - "KindOut"
Cohesion: 0.11
Nodes (19): Entry, kind_out(), KindOut, Action, Bool, Choice, Folders, Hotkey (+11 more)

### Community 125 - "test-settingswin.ps1"
Cohesion: 0.22
Nodes (11): Click-Nav(), Click-Switch(), Heading(), Js(), Labels(), Navs(), Row-Js(), Switch-On() (+3 more)

### Community 126 - "TopMenu"
Cohesion: 0.12
Nodes (21): menubar_menu(), is_lazy(), menu(), Action, App, Eos, action_paths_and_flags_survive_normalization(), Cmd (+13 more)

### Community 127 - "open"
Cohesion: 0.16
Nodes (18): mc_windows(), McMonitor, McWindow, create_one(), fit_exact(), hwnd_of(), icons_for(), label() (+10 more)

### Community 132 - "Indicatori di sistema — specifica"
Cohesion: 0.15
Nodes (12): Cosa fa macOS (da replicare) e da dove viene, Fasi, Fuori scopo, I1 — Dati: FATTA (7 nov 2026), I2 — Barra: FATTA (7 nov 2026), I4 — Rifiniture: FATTA, tranne una prova (7 nov 2026), Indicatori di sistema — specifica, Rischi aperti (+4 more)

### Community 137 - "rank_hits"
Cohesion: 0.53
Nodes (6): hit(), multi_word_queries_match_names_with_separators(), name_score(), rank_hits(), rank_hits_keeps_name_matches_dedupes_and_orders(), rank_hits_respects_limit_and_empty()

### Community 138 - "HWND"
Cohesion: 0.24
Nodes (10): app_from_hwnd(), initial_window(), last_window(), note_activated(), on_event(), poll_thread(), start(), update() (+2 more)

### Community 140 - "fold"
Cohesion: 0.25
Nodes (9): files(), short_dir(), short_dir_uses_home_and_elides_long_paths(), fold(), fold_lowercases_and_strips_common_accents(), initials(), initials_cover_words_and_inner_capitals(), match_tiers() (+1 more)

### Community 141 - "SettingsError"
Cohesion: 0.27
Nodes (6): Backend, buttons_run_actions_but_have_no_value(), Fake, plan_pins(), run_action(), SettingsError

### Community 142 - "battery_icon"
Cohesion: 0.40
Nodes (5): battery_icon(), battery_icon_levels_and_states(), every_icon_rust_can_emit_is_one_the_page_knows(), volume_icon(), volume_icons_and_titles()

### Community 143 - "list"
Cohesion: 0.24
Nodes (11): mc_desktops(), an_unknown_or_missing_current_desktop_falls_back_to_the_first(), current_index(), Desktop, desktops_from(), list(), read_value(), the_list_names_desktops_and_marks_the_current_one() (+3 more)

### Community 144 - "clip_id"
Cohesion: 0.83
Nodes (4): clip_delete(), clip_id(), clip_ids_parse_strictly(), clip_use()

### Community 145 - "Phase"
Cohesion: 0.20
Nodes (10): Anim, Diag, diag(), Phase, Closed, Closing, Open, Opening (+2 more)

### Community 146 - "MenuAction"
Cohesion: 0.22
Nodes (9): MenuAction, Open, Pin, Quit, SetGlass, ToggleAutoHide, ToggleBlur, ToggleReserve (+1 more)

### Community 147 - "Response"
Cohesion: 0.40
Nodes (4): dock_backdrop(), launcher_backdrop(), mc_backdrop(), panel_backdrop()

### Community 148 - "merge"
Cohesion: 0.19
Nodes (16): app_display_name(), apply_run_order(), cached_shell_name(), DockItem, dump_dock(), find(), find_with_windows(), items() (+8 more)

### Community 149 - "test-settings.ps1"
Cohesion: 0.36
Nodes (4): All-S(), Js(), Set-S(), Val()

### Community 150 - "capture_rect"
Cohesion: 0.28
Nodes (9): capture_adds_context_and_stays_on_the_monitor(), capture_margin(), capture_margin_grows_with_the_scale(), capture_rect(), blur_enabled(), capture_info(), toggle_blur(), capture_info() (+1 more)

### Community 151 - "enable"
Cohesion: 0.29
Nodes (14): a_missing_or_relative_exe_is_refused_and_nothing_is_written(), a_task_manager_block_counts_as_off_and_turning_on_clears_it(), blocked_by_user(), clean(), delete_value(), disable(), enable(), enable_and_disable_touch_only_our_value() (+6 more)

### Community 152 - "Cmd"
Cohesion: 0.18
Nodes (13): child_lines_round_trip_and_garbage_is_ignored(), cmd_name(), FromChild, Cmd, Installed, Swallowed, parse_child_line(), Cmd (+5 more)

### Community 158 - "test-settings-s4.ps1"
Cohesion: 0.24
Nodes (4): Inv(), Js(), Val(), Win()

### Community 159 - "run"
Cohesion: 0.32
Nodes (6): cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), run(), on_window_event()

### Community 164 - "Kind"
Cohesion: 0.25
Nodes (8): Kind, Action, Bool, Choice, Folders, Hotkey, Names, Pinned

### Community 165 - "scripts"
Cohesion: 0.33
Nodes (6): scripts, build, dev, preview, tauri, test

### Community 166 - "Clock.tsx"
Cohesion: 0.73
Nodes (3): Clock(), formatClock(), msToNextMinute()

### Community 167 - "l"
Cohesion: 0.53
Nodes (5): folders_must_be_absolute_and_safe(), l(), name_lists_are_cleaned(), List, values_parse_from_plain_json()

### Community 168 - "AtomicIsize"
Cohesion: 0.40
Nodes (4): LAST_HWND, OWNER, HWND_RAW, HWND_RAW

### Community 169 - "ActiveApp"
Cohesion: 0.40
Nodes (4): ActiveApp, current(), LAST, get_active_app()

### Community 172 - "same_app"
Cohesion: 0.22
Nodes (16): background_apps_get_the_grey_dot_only_when_pinned_and_without_windows(), key(), live_process_scan_excludes_self_and_finds_explorer(), merge(), merge_has_no_duplicates(), merge_puts_pinned_first_then_unpinned_running(), only_a_single_version_folder_counts_as_the_same_app(), pin() (+8 more)

### Community 178 - "Dir"
Cohesion: 0.33
Nodes (6): Dir, Left, Right, plan_switch(), switch_to(), switching_plans_the_right_number_of_steps()

## Knowledge Gaps
- **108 isolated node(s):** `Quit`, `Minimize`, `Maximize`, `Restore`, `Close` (+103 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 891 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **59 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `same_app()` connect `same_app` to `dock.rs`, `dockstore.rs`, `merge`, `dockact.rs`, `package.json`?**
  _High betweenness centrality (0.143) - this node is a cross-community bridge._
- **What connects `Quit`, `Minimize`, `Maximize` to the rest of the system?**
  _108 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `name` connect `package.json` to `same_app`?**
  _High betweenness centrality (0.142) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `package.json` to `Clock.tsx`, `indicators.ts`, `main.tsx`, `launcher.ts`, `menubar.ts`, `Dock.tsx`, `tokens.test.ts`, `Settings.tsx`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Should `panelwin.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06504494976203067 - nodes in this community are weakly interconnected._