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

## Milestone 7: Future Extensions [PLANNED]
- [ ] Language Server Protocol (LSP) client over async stdio.
- [ ] WASM plugin runtime (Wasmtime sandbox for custom commands and extensions).
- [ ] CRDT-based operational transform for real-time collaborative editing.
