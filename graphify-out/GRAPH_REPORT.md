# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 30 files · ~8,843 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 363 nodes · 556 edges · 24 communities (14 shown, 10 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 8 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `9ae5ffac`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- EosOS: Piano di lavoro
- CLAUDE.md
- package.json
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
- appbar.rs
- foreground.rs
- test-multimonitor.ps1

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `MonitorInfo` - 11 edges
3. `reconcile()` - 11 edges
4. `desired()` - 8 edges
5. `apply()` - 8 edges
6. `mon()` - 8 edges
7. `display_name()` - 8 edges
8. `bar_rect()` - 7 edges
9. `EosOS: Piano di lavoro` - 6 edges
10. `diff()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `cleanup()` --calls--> `remove_if_registered()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/appbar.rs
- `run()` --calls--> `start()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/appbar.rs
- `run()` --calls--> `image_path()`  [INFERRED]
  src-tauri/src/watchdog.rs → src-tauri/src/util.rs
- `App()` --calls--> `ActiveAppName()`  [EXTRACTED]
  src/App.tsx → src/components/ActiveApp.tsx
- `App()` --calls--> `Clock()`  [EXTRACTED]
  src/App.tsx → src/components/Clock.tsx

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (24 total, 10 thin omitted)

### Community 0 - "EosOS: Piano di lavoro"
Cohesion: 0.29
Nodes (6): Dettagli sul ripristino (P4), EosOS: Piano di lavoro, LEGENDA dei passaggi, Principi di sicurezza (valgono per tutti i passaggi), Roadmap (FUORI SCOPE per ora), Traguardo della sessione

### Community 3 - "package.json"
Cohesion: 0.06
Nodes (40): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/react@^19.1.8 (+32 more)

### Community 4 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+10 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 6 - "tests.rs"
Cohesion: 0.08
Nodes (3): COUNTER, FakeWindow, wndproc()

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
Cohesion: 0.29
Nodes (7): cleanup(), enable_taskbar_hiding(), get_active_app(), hide_requested(), install_local_safety_nets(), on_ctrl(), run()

### Community 15 - "taskbar.rs"
Cohesion: 0.11
Nodes (16): collect(), Ctx, find(), HIDDEN, hide(), hide_real(), restore(), restore_if_hidden() (+8 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.07
Nodes (31): APPLIED, apply(), BAR_HEIGHT_LOGICAL, bar_height_px(), bar_rect(), bar_rect_follows_monitor_with_negative_origin(), create_window(), debug_enabled() (+23 more)

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (22): ActiveApp, APP, app_from_hwnd(), cached_description(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop() (+14 more)

### Community 21 - "test-multimonitor.ps1"
Cohesion: 0.46
Nodes (6): Bars(), Check(), CheckBars(), CheckRestored(), MainPid(), Mons()

## Knowledge Gaps
- **1 isolated node(s):** `eosos`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 168 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **10 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `image_path()` connect `watchdog.rs` to `foreground.rs`?**
  _High betweenness centrality (0.012) - this node is a cross-community bridge._
- **What connects `eosos` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.0636734693877551 - nodes in this community are weakly interconnected._
- **Why does `remove_if_registered()` connect `appbar.rs` to `lib.rs`, `foreground.rs`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `start()` connect `appbar.rs` to `lib.rs`?**
  _High betweenness centrality (0.008) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._