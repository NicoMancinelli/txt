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

## Milestone 9: Full Suite File Browser & Multi-Root Projects [COMPLETED]
- [x] Hierarchical project tree model (`txt-core::project`): `TreeNode`, `ProjectFolder`, `ProjectManager`, `FileOps`.
- [x] Multi-root project workspaces with dynamic folder registration and removal.
- [x] Recursive tree building with directory-first sorting and ignore rules (`.git`, `target/`, `node_modules/`, `.DS_Store`).
- [x] Toggle hidden files command and toolbar button (`project.toggle_hidden`).
- [x] Atomic file operations: file creation, folder creation, rename, duplicate, recursive delete.
- [x] Git working tree status scanner (`txt-core::git_status`): Porcelain v1 parser, Mariana color styling, `M`/`U`/`A`/`D` badge decorations.
- [x] Dynamic background filesystem watcher (`txt-core::watcher`): Zero-leak worker lifecycle tracking with RAII subscription handles.
- [x] Native right-click context menus for files, directories, and open tabs (New File, New Folder, Rename, Duplicate, Copy Path, Reveal in Finder/Explorer, Delete).
- [x] Interactive modal overlays for file creation, folder creation, renaming, and delete confirmation with Enter/Escape controls.
- [x] Reveal Active File in Side Bar command (`file.reveal_in_sidebar` / `Cmd+Option+R`).
- [x] Toolbar controls: Expand All (`⤢`), Collapse All (`⤡`), Toggle Hidden (`👁`), Refresh (`↻`), New Item (`+`), Search Filter (`🔍`).
- [x] 100% test pass rate across 74 unit, integration, benchmark, and stress tests.

## Milestone 10: v1.1 Advanced Editor Suite [COMPLETED]
- [x] Sublime Multi-Cursor Power Suite (`txt-core::cursor`):
  - Incremental next occurrence selection (`Cmd+D` / `Ctrl+D`) with word boundary expansion and EOF wrap-around.
  - Select all occurrences across buffer (`Cmd+Ctrl+G` / `Alt+F3`).
  - Split multi-line selections into lines (`Cmd+Shift+L` / `Ctrl+Shift+L`).
  - Add cursor above / below (`Cmd+Option+Up/Down` / `Ctrl+Alt+Up/Down`) with ragged-line length clamping.
- [x] High-Performance Buffer Find & Replace Engine (`txt-core::find_replace`):
  - Sub-millisecond literal, whole-word (`\b`), and regex pattern searching (`FindReplaceEngine`).
  - Wrap-around `find_next` and `find_prev`.
  - Single-occurrence `replace_next` with regex capture group substitutions.
  - Full-buffer `replace_all` applying edits in reverse index order to preserve earlier character offsets.
  - Slide-up bottom Find & Replace GUI panel with `Aa`, `\b`, `.*` toggles, status badge, and next/prev/replace buttons.
  - Viewport match highlights in editor canvas with active match distinct gold border.
- [x] Document Symbol Extraction & Goto Symbol (`txt-core::symbols`):
  - Fast lexical scanner extracting functions, structs, enums, traits, classes, headings, and constants across Rust, Python, Markdown, and JS/TS.
  - Fuzzy symbol search in Command Palette triggered with `@` (`Cmd+R`).
  - Direct Goto Line mode in Command Palette triggered with `:` (`Ctrl+G`).
  - Matched character highlights in symbol results and viewport scroll centering.
- [x] Persistent Session Management (`txt-core::session`):
  - `SessionState` tracking open files, active tab, sidebar visibility, font size, theme, and word wrap.
  - `SessionManager` auto-saving to `.txt/session.json` and restoring workspace state on startup.
- [x] 100% test pass rate across 123 unit, integration, benchmark, and stress tests.
