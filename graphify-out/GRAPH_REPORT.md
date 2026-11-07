# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 43 files · ~14,884 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 485 nodes · 726 edges · 37 communities (18 shown, 19 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 13 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `554a5b02`
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
- lib.rs
- eosos
- taskbar.rs
- test-taskbar.ps1
- test-workarea.ps1
- appbar.rs
- foreground.rs
- test-multimonitor.ps1
- main.tsx
- glass.rs
- tokens.test.ts
- EosOS
- Checklist di test manuali

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 17 edges
2. `reconcile()` - 12 edges
3. `MonitorInfo` - 10 edges
4. `display_name()` - 10 edges
5. `Glass` - 9 edges
6. `find()` - 9 edges
7. `restore_autohide()` - 8 edges
8. `apply()` - 8 edges
9. `desired()` - 8 edges
10. `mon()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `cleanup()` --calls--> `restore_if_hidden()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/taskbar.rs
- `enable_taskbar_hiding()` --calls--> `hide_real()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/taskbar.rs
- `enable_taskbar_hiding()` --calls--> `restore_real()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/taskbar.rs
- `run()` --calls--> `restore_autohide()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/taskbar.rs
- `only_explorer_windows_are_accepted_for_hiding()` --calls--> `find()`  [EXTRACTED]
  src-tauri/src/tests.rs → src-tauri/src/taskbar.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (37 total, 19 thin omitted)

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
Cohesion: 0.07
Nodes (6): COUNTER, FakeWindow, hide_then_restore_roundtrip(), only_explorer_windows_are_accepted_for_hiding(), restore_is_idempotent_and_safe_when_nothing_hidden(), wndproc()

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.06
Nodes (13): image_path(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, log(), LOG_MAX_BYTES, log_path(), ready_event_name() (+5 more)

### Community 11 - "lib.rs"
Cohesion: 0.26
Nodes (7): cleanup(), enable_taskbar_hiding(), get_active_app(), hide_requested(), install_local_safety_nets(), on_ctrl(), run()

### Community 15 - "taskbar.rs"
Cohesion: 0.09
Nodes (23): appbar_state(), autohide_marker(), collect(), Ctx, enable_autohide(), find(), HIDDEN, hide() (+15 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (36): APPLIED, AppliedMap, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), create_window() (+28 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (24): ActiveApp, APP, app_from_hwnd(), cached_description(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop() (+16 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

### Community 24 - "main.tsx"
Cohesion: 0.05
Nodes (48): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/node@^26.6.6 (+40 more)

### Community 25 - "glass.rs"
Cohesion: 0.11
Nodes (12): acrylic_wins_when_available(), apply(), choose(), falls_back_to_blur(), falls_back_to_none(), Glass, Acrylic, Blur (+4 more)

### Community 26 - "tokens.test.ts"
Cohesion: 0.12
Nodes (10): BLACK, contrast(), css, darkForced, darkMedia, light, luminance(), RGB (+2 more)

### Community 30 - "EosOS"
Cohesion: 0.17
Nodes (11): Avvio in sviluppo, Build, Fuori scopo (roadmap), Icona, Limiti noti, Provarlo in sicurezza, Requisiti, Test automatici (+3 more)

### Community 31 - "Checklist di test manuali"
Cohesion: 0.29
Nodes (6): A. Senza nascondere la taskbar (`npm run tauri dev`), B. Con taskbar nascosta (`$env:EOSOS_HIDE_TASKBAR="1"`), C. Script automatici (sulla macchina di prova), Checklist di test manuali, D. Traguardo finale, E. Scale DPI diverse da 100% (solo su macchina o utente di prova)

## Knowledge Gaps
- **4 isolated node(s):** `eosos`, `Acrylic`, `Blur`, `None`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 234 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `vitest` connect `main.tsx` to `tokens.test.ts`?**
  _High betweenness centrality (0.012) - this node is a cross-community bridge._
- **Are the 7 inferred relationships involving `display_name()` (e.g. with `desktop_classes_show_desktop()` and `empty_everything_is_unknown()`) actually correct?**
  _`display_name()` has 7 INFERRED edges - model-reasoned connections that need verification._
- **What connects `eosos`, `Acrylic`, `Blur` to the rest of the system?**
  _4 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Why does `image_path()` connect `watchdog.rs` to `foreground.rs`?**
  _High betweenness centrality (0.007) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Should `tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07112375533428165 - nodes in this community are weakly interconnected._