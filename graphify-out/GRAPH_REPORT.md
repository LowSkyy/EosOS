# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 20 files · ~3,349 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 204 nodes · 262 edges · 16 communities (10 shown, 6 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `483fa742`
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

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `run()` - 6 edges
3. `hide()` - 6 edges
4. `wait_and_restore()` - 6 edges
5. `run()` - 6 edges
6. `EosOS: Piano di lavoro` - 6 edges
7. `compilerOptions` - 6 edges
8. `enable_taskbar_hiding()` - 5 edges
9. `find()` - 5 edges
10. `restore()` - 5 edges

## Surprising Connections (you probably didn't know these)
- `enable_taskbar_hiding()` --calls--> `restore_real()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/taskbar.rs
- `enable_taskbar_hiding()` --calls--> `spawn()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/watchdog.rs
- `run()` --calls--> `run()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/watchdog.rs
- `hide_does_not_touch_other_classes()` --calls--> `restore()`  [EXTRACTED]
  src-tauri/src/tests.rs → src-tauri/src/taskbar.rs
- `wait_and_restore()` --calls--> `restore()`  [EXTRACTED]
  src-tauri/src/watchdog.rs → src-tauri/src/taskbar.rs

## Import Cycles
- None detected.

## Communities (16 total, 6 thin omitted)

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
Nodes (17): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+9 more)

### Community 6 - "tests.rs"
Cohesion: 0.07
Nodes (9): hide(), COUNTER, FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), restore_is_idempotent_and_safe_when_nothing_hidden(), watchdog_restores_immediately_if_parent_already_gone(), watchdog_restores_when_watched_process_is_killed() (+1 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "watchdog.rs"
Cohesion: 0.08
Nodes (8): CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, HOTKEY_ID, kill_process(), log(), run(), spawn(), wait_and_restore()

### Community 11 - "lib.rs"
Cohesion: 0.33
Nodes (7): enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), on_ctrl(), run(), hide_real(), restore_if_hidden()

### Community 15 - "taskbar.rs"
Cohesion: 0.16
Nodes (7): collect(), Ctx, find(), HIDDEN, restore(), restore_real(), TASKBAR_CLASSES

## Knowledge Gaps
- **1 isolated node(s):** `eosos`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 113 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `restore_if_hidden()` connect `lib.rs` to `taskbar.rs`?**
  _High betweenness centrality (0.016) - this node is a cross-community bridge._
- **What connects `eosos` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.07254623044096728 - nodes in this community are weakly interconnected._
- **Why does `run()` connect `watchdog.rs` to `lib.rs`, `taskbar.rs`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `run()` connect `lib.rs` to `watchdog.rs`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.1111111111111111 - nodes in this community are weakly interconnected._