# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 21 files · ~3,770 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 212 nodes · 266 edges · 18 communities (11 shown, 7 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `9cb4e132`
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

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `EosOS: Piano di lavoro` - 6 edges
3. `hide()` - 6 edges
4. `compilerOptions` - 6 edges
5. `wait_and_restore()` - 6 edges
6. `FakeWindow` - 5 edges
7. `find()` - 5 edges
8. `restore()` - 5 edges
9. `scripts` - 5 edges
10. `build` - 5 edges

## Surprising Connections (you probably didn't know these)
- `wait_and_restore()` --calls--> `restore()`  [EXTRACTED]
  src-tauri/src/watchdog.rs → src-tauri/src/taskbar.rs
- `run()` --calls--> `restore_real()`  [EXTRACTED]
  src-tauri/src/watchdog.rs → src-tauri/src/taskbar.rs
- `watchdog_restores_immediately_if_parent_already_gone()` --calls--> `hide()`  [EXTRACTED]
  src-tauri/src/tests.rs → src-tauri/src/taskbar.rs
- `watchdog_restores_immediately_if_parent_already_gone()` --calls--> `wait_and_restore()`  [EXTRACTED]
  src-tauri/src/tests.rs → src-tauri/src/watchdog.rs
- `watchdog_restores_when_watched_process_is_killed()` --calls--> `wait_and_restore()`  [EXTRACTED]
  src-tauri/src/tests.rs → src-tauri/src/watchdog.rs

## Import Cycles
- None detected.

## Communities (18 total, 7 thin omitted)

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
Cohesion: 0.08
Nodes (6): COUNTER, FakeWindow, hide_then_restore_roundtrip(), restore_is_idempotent_and_safe_when_nothing_hidden(), watchdog_restores_immediately_if_parent_already_gone(), wndproc()

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
Cohesion: 0.39
Nodes (5): enable_taskbar_hiding(), hide_requested(), install_local_safety_nets(), on_ctrl(), run()

### Community 15 - "taskbar.rs"
Cohesion: 0.13
Nodes (12): collect(), Ctx, find(), HIDDEN, hide(), hide_real(), restore(), restore_if_hidden() (+4 more)

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

## Knowledge Gaps
- **1 isolated node(s):** `eosos`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 114 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What connects `eosos` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.07254623044096728 - nodes in this community are weakly interconnected._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.1111111111111111 - nodes in this community are weakly interconnected._
- **Should `tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07899159663865546 - nodes in this community are weakly interconnected._
- **Should `watchdog.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07526881720430108 - nodes in this community are weakly interconnected._
- **Should `taskbar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13043478260869565 - nodes in this community are weakly interconnected._