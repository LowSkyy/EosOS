# Graph Report - EosOS  (2026-11-07)

## Corpus Check
- 29 files · ~6,694 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 324 nodes · 460 edges · 22 communities (14 shown, 8 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 7 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4baf45b0`
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
- App.tsx

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `display_name()` - 8 edges
3. `run()` - 7 edges
4. `run()` - 7 edges
5. `EosOS: Piano di lavoro` - 6 edges
6. `scripts` - 6 edges
7. `app_from_hwnd()` - 6 edges
8. `hide()` - 6 edges
9. `compilerOptions` - 6 edges
10. `react` - 5 edges

## Surprising Connections (you probably didn't know these)
- `get_active_app()` --calls--> `current()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/foreground.rs
- `run()` --calls--> `start()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/foreground.rs
- `enable_taskbar_hiding()` --calls--> `spawn_ready()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/watchdog.rs
- `run()` --calls--> `run()`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/watchdog.rs
- `run()` --calls--> `image_path()`  [INFERRED]
  src-tauri/src/watchdog.rs → src-tauri/src/util.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/foreground.rs -> src-tauri/src/foreground.rs`

## Communities (22 total, 8 thin omitted)

### Community 0 - "EosOS: Piano di lavoro"
Cohesion: 0.29
Nodes (6): Dettagli sul ripristino (P4), EosOS: Piano di lavoro, LEGENDA dei passaggi, Principi di sicurezza (valgono per tutti i passaggi), Roadmap (FUORI SCOPE per ora), Traguardo della sessione

### Community 3 - "package.json"
Cohesion: 0.09
Nodes (25): devDependencies, @tauri-apps/cli@^2, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2, vitest@^5.1.0 (+17 more)

### Community 4 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+10 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 6 - "tests.rs"
Cohesion: 0.07
Nodes (11): hide(), hide_real(), COUNTER, FakeWindow, hide_does_not_touch_other_classes(), hide_then_restore_roundtrip(), restore_is_idempotent_and_safe_when_nothing_hidden(), watchdog_restores_immediately_if_parent_already_gone() (+3 more)

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
Cohesion: 0.33
Nodes (7): cleanup(), enable_taskbar_hiding(), get_active_app(), hide_requested(), install_local_safety_nets(), on_ctrl(), run()

### Community 15 - "taskbar.rs"
Cohesion: 0.16
Nodes (8): collect(), Ctx, find(), HIDDEN, restore(), restore_if_hidden(), restore_real(), TASKBAR_CLASSES

### Community 16 - "test-taskbar.ps1"
Cohesion: 0.52
Nodes (6): Cleanup(), EosProcs(), ForceRestore(), MainProc(), Scenario(), TaskbarVisible()

### Community 18 - "appbar.rs"
Cohesion: 0.09
Nodes (7): BAR_HEIGHT_LOGICAL, main_window(), new_data(), register(), REGISTERED, remove_if_registered(), setup()

### Community 20 - "foreground.rs"
Cohesion: 0.07
Nodes (22): ActiveApp, APP, app_from_hwnd(), cached_description(), current(), DESCRIPTIONS, DESKTOP_CLASSES, desktop_classes_show_desktop() (+14 more)

### Community 21 - "App.tsx"
Cohesion: 0.16
Nodes (15): dependencies, @fontsource-variable/inter@^5.3.0, react@^19.1.0, react-dom@^19.1.0, @tauri-apps/api@^2, @fontsource-variable/inter, react, react-dom (+7 more)

## Knowledge Gaps
- **1 isolated node(s):** `eosos`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 160 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **8 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `run()` connect `lib.rs` to `watchdog.rs`, `foreground.rs`?**
  _High betweenness centrality (0.018) - this node is a cross-community bridge._
- **Are the 6 inferred relationships involving `display_name()` (e.g. with `desktop_classes_show_desktop()` and `empty_everything_is_unknown()`) actually correct?**
  _`display_name()` has 6 INFERRED edges - model-reasoned connections that need verification._
- **What connects `eosos` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.09259259259259259 - nodes in this community are weakly interconnected._
- **Why does `image_path()` connect `watchdog.rs` to `foreground.rs`?**
  _High betweenness centrality (0.015) - this node is a cross-community bridge._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Why does `run()` connect `watchdog.rs` to `lib.rs`?**
  _High betweenness centrality (0.014) - this node is a cross-community bridge._