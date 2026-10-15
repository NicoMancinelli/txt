# Project Objectives & Roadmap: txt

> Clear tracking of development milestones and upcoming technical deliverables.

---

## Milestone 1: Core Architecture & Native Workspace [COMPLETED]
- [x] Verify toolchain (`rustc`, `cargo`, `gh`, `git`).
- [x] Establish Cargo workspace architecture (`txt-core`, `txt-sync`, `txt-gui`).
- [x] Configure workspace-level dependency resolution and licensing.
- [x] Implement multi-agent operating manual (`AGENTS.md`) and REA analysis (`docs/REA_ANALYSIS.md`).

## Milestone 2: Core Text Editing Engine (`txt-core`) [COMPLETED]
- [x] B-Tree Rope backed `TextBuffer` with $O(\log N)$ inserts and deletes.
- [x] Multi-cursor & selection range engine (`SelectionSet` & `Selection`).
- [x] Transaction-grouped undo/redo history stack (`HistoryManager`).
- [x] Sublime Text fuzzy matching scoring algorithm (`FuzzyMatcher`).
- [x] Fast lexical tokenizer for Rust, Python, JSON, and Markdown (`SyntaxHighlighter`).
- [x] Memory metrics tracker for buffer allocations (`MemoryMetrics`).
- [x] 100% unit and integration test coverage for core module.

## Milestone 3: Modular Synchronization Subsystem (`txt-sync`) [COMPLETED]
- [x] Resource-level concurrency lock manager (`ConcurrencyLockManager<K>`).
- [x] Pluggable `SyncAdapter` trait and atomic `SyncPatch` data structures.
- [x] Atomic local filesystem adapter with shadow journaling (`LocalFilesystemAdapter`).
- [x] Fast in-memory adapter for testing and local relay (`MemorySyncAdapter`).
- [x] Cloud adapter interface with offline patch buffering (`CloudSyncAdapter`).
- [x] Dynamic background heartbeat lifecycle: auto-starts on client registration, cleans up on zero clients.
- [x] Concurrency and stress test suite (`sync_test.rs`).

## Milestone 4: Native Desktop UI (`txt-gui`) [COMPLETED]
- [x] Sublime Text themes: Mariana and Monokai color palettes.
- [x] Sublime-style Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`) with fuzzy filtering and keyboard navigation.
- [x] High-performance text canvas: line numbers gutter, active line highlight, syntax token coloring, cursor rendering.
- [x] Multi-buffer tab strip with dirty status indicators and close buttons.
- [x] Collapsible project file navigation sidebar (`Cmd+B` / `Cmd+K, Cmd+B`).
- [x] Sublime-style status bar with line/column coordinates, multi-cursor count, encoding, spaces, and sync status.
- [x] GUI integration test suite (`gui_test.rs`).

## Milestone 5: GitHub Pipeline & Continuous Integration [COMPLETED]
- [x] Git repository initialization and `.gitignore` setup.
- [x] GitHub Actions CI workflow for automated checking and testing (`.github/workflows/ci.yml`).
- [x] Create remote GitHub repository via `gh` CLI: [NicoMancinelli/txt](https://github.com/NicoMancinelli/txt).
- [x] Push codebase to `main` branch.
- [x] Branching & PR workflow: Created and merged Pull Request #1 (`feature/benchmarks`) and Pull Request #2 (`feature/rea-sublime-enhancements`).

## Milestone 6: Empirical REA Dissection & Sublime Features [COMPLETED]
- [x] Reverse engineer `Sublime Text.app` universal Mach-O binary (`sublime_text`).
- [x] Extract linked frameworks, class hierarchies, and symbol tables.
- [x] Implement Skyline Minimap on editor view with viewport indicator box and toggle control.
- [x] Implement Sublime case transformation engine (`TextTransform`: Upper, Lower, Title, snake_case, kebab-case, CamelCase).
- [x] Implement indentation transformations (tabs to spaces, spaces to tabs).

## Milestone 7: Advanced Systems & Collaboration [COMPLETED]
- [x] Language Server Protocol (LSP) client protocol foundation (`txt-core::lsp`): JSON-RPC streaming framer, diagnostics, completions, and hover state.
- [x] Extensible Event Listener Plugin architecture (`txt-core::plugin`): EventListener trait, PluginManager, WordCountPlugin, AutoClosingBracketsPlugin.
- [x] High-performance Parallel Project Search (`txt-core::search`): 4096-item batching parallel worker pool reproducing Sublime Text's `parallel_for_task` (>2,000,000 lines/sec).
- [x] CRDT & Vector Clock Collaborative Synchronization (`txt-sync::crdt`): Replicated Growable Array (RGA), Lamport timestamps, tombstone deletes, deterministic peer conflict resolution, and SyncEngine integration under ConcurrencyLockManager.
- [x] Multi-agent autonomous harness collaboration: Dispatched concurrent specialist subagents for parallel development.
- [x] 100% test pass rate across 66 unit, integration, throughput, and multi-threaded stress tests.

## Milestone 8: v1.0.0 Release & v1.0.1 File Browser & Editor Enhancements [COMPLETED]
- [x] Release and tag production `v1.0.0` and `v1` on GitHub.
- [x] Build out Sublime Text "OPEN FILES" section in sidebar with dirty indicators and close buttons.
- [x] Add real-time live search filter (`🔍`) for directory tree filtering.
- [x] Add inline New File dialog (`+`) directly in the file browser.
- [x] Add file extension iconography (Rust, Markdown, TOML/JSON, Python, JS/TS, C/C++, Go, Shell, etc.).
- [x] Add file size and path hover tooltips in file browser.
- [x] Add soft word wrap toggle (`view.toggle_word_wrap` / `Option+Z`).
- [x] Add Sublime line duplication (`edit.duplicate_line` / `Cmd+Shift+D`) and line deletion (`edit.delete_line` / `Cmd+Shift+K`).
- [x] Add GUI integration tests in `crates/txt-gui/tests/gui_test.rs`.

