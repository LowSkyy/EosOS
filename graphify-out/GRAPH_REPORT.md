# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 23 files · ~5,297 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 265 nodes · 346 edges · 20 communities (12 shown, 8 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `281f6df5`
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

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `run()` - 8 edges
3. `run()` - 8 edges
4. `EosOS: Piano di lavoro` - 6 edges
5. `cleanup()` - 6 edges
6. `hide()` - 6 edges
7. `compilerOptions` - 6 edges
8. `new_data()` - 5 edges
9. `register()` - 5 edges
10. `remove_if_registered()` - 5 edges

## Surprising Connections (you probably didn't know these)
- `cleanup()` --calls--> `remove_if_registered()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/appbar.rs
- `restore_everything()` --calls--> `remove_if_registered()`  [EXTRACTED]
  src-tauri/src/watchdog.rs → src-tauri/src/appbar.rs
- `run()` --calls--> `setup()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/appbar.rs
- `run()` --calls--> `main_window()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/appbar.rs
- `enable_taskbar_hiding()` --calls--> `spawn_ready()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/watchdog.rs

## Import Cycles
- None detected.

## Communities (20 total, 8 thin omitted)

### Community 0 - "EosOS: Piano di lavoro"
Cohesion: 0.29
Nodes (6): Dettagli sul ripristino (P4), EosOS: Piano di lavoro, LEGENDA dei passaggi, Principi di sicurezza (valgono per tutti i passaggi), Roadmap (FUORI SCOPE per ora), Traguardo della sessione

### Community 3 - "package.json"
Cohesion: 0.07
Nodes (32): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, devDependencies, @tauri-apps/cli@^2, @types/react@^19.1.8 (+24 more)

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
Nodes (13): CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, image_path(), log(), LOG_MAX_BYTES, log_path(), ready_event_name() (+5 more)

### Community 11 - "lib.rs"
Cohesion: 0.44
Nodes (6): cleanup(), enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), on_ctrl(), run()

### Community 15 - "taskbar.rs"
Cohesion: 0.11
Nodes (16): collect(), Ctx, find(), HIDDEN, hide(), hide_real(), restore(), restore_if_hidden() (+8 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.10
Nodes (7): BAR_HEIGHT_LOGICAL, main_window(), new_data(), register(), REGISTERED, remove_if_registered(), setup()

## Knowledge Gaps
- **1 isolated node(s):** `eosos`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 147 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **8 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `run()` connect `lib.rs` to `watchdog.rs`, `appbar.rs`?**
  _High betweenness centrality (0.017) - this node is a cross-community bridge._
- **What connects `eosos` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.07254623044096728 - nodes in this community are weakly interconnected._
- **Why does `run()` connect `watchdog.rs` to `lib.rs`?**
  _High betweenness centrality (0.016) - this node is a cross-community bridge._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `spawn_ready()` connect `watchdog.rs` to `lib.rs`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._