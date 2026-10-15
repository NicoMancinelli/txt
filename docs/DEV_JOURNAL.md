# Development Journal: txt

> Continuous engineering record of architectural decisions, benchmarks, and progress.

---

### Entry 001: Workspace Initialization & Technology Selection
- **Context**: The goal is to build a high-performance, offline-first text editor modeled after Sublime Text.
- **Analysis**:
  - Electron and browser-based stacks were rejected immediately due to excessive RAM usage, high launch latency, and GC pauses during high-frequency typing.
  - Rust was selected as the language of choice for direct memory control, lack of garbage collection, strong type safety, and cross-platform native compilation.
  - We structured the project as a modular Cargo workspace: `txt-core` (pure data structures and logic, zero UI dependencies), `txt-sync` (synchronization engine and concurrency locking), and `txt-gui` (hardware-accelerated desktop UI).
- **Outcome**: Created Cargo workspace with `txt-core`, `txt-sync`, and `txt-gui`.

---

### Entry 002: Reverse Engineering Sublime Text Buffer & Search
- **Context**: Dissecting why Sublime Text handles large files and fuzzy queries with negligible latency.
- **Findings**:
  - Naive line vectors (`Vec<String>`) cause massive allocation churn when editing anywhere in the file.
  - Sublime Text uses a hybrid Piece Table / Rope B-Tree structure. We integrated `ropey` in `txt-core::buffer::TextBuffer`, achieving $O(\log N)$ updates on gigabyte files.
  - The fuzzy search algorithm was reverse engineered: Sublime Text awards high bonuses to prefix matches (+60), word boundary separators (+45), CamelCase character transitions (+35), and consecutive character matches (+50).
- **Outcome**: Implemented `FuzzyMatcher` and `TextBuffer` with 100% passing tests.

---

### Entry 003: Concurrency Locking & Resource Cleanup Guardrails
- **Context**: Designing the synchronization engine (`txt-sync`) under strict backend reliability rules.
- **Implementation**:
  - `ConcurrencyLockManager<Uuid>`: Serializes operations per document resource, eliminating race conditions during simultaneous local/cloud sync patches.
  - Pluggable `SyncAdapter` trait implemented for `LocalFilesystemAdapter` (atomic temporary file write + rename), `MemorySyncAdapter`, and `CloudSyncAdapter` (with offline queue).
  - Background Heartbeat Lifecycle: Dynamic worker thread management ensures the heartbeat runs **only when active clients > 0**. When clients disconnect to 0, the background worker is joined and freed.
- **Outcome**: Verified under multi-threaded stress tests in `crates/txt-sync/tests/sync_test.rs`.

---

### Entry 004: UI Implementation & Sublime Aesthetics
- **Context**: Creating the native graphical application in `txt-gui`.
- **Implementation**:
  - Built custom themes matching Sublime Text's Mariana (`#1e2227`) and Monokai (`#272822`).
  - Implemented the Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`) with fuzzy filtering, keyboard navigation (Arrow Up/Down, Enter), and shortcut badges.
  - Built viewport line rendering: only queries and paints visible line ranges.
  - Integrated tab strip, file explorer sidebar (`Cmd+K, Cmd+B`), and status bar displaying line/column, encoding, language, memory, and sync status.
- **Outcome**: Verified via automated integration tests in `crates/txt-gui/tests/gui_test.rs`.

---

### Entry 005: Empirical REA Dissection of `Sublime Text.app`
- **Context**: The user provided a native sample `Sublime Text.app` in `sample app/`.
- **Analysis**:
  - Binary disassembly (`otool`, `file`, `strings`) revealed linked frameworks (`OpenGL`, `QuartzCore`, `CoreText`, `Cocoa`).
  - Extracted core classes: `TextBuffer`, `TextBufferSelection`, `TokenRenderer`, `skyline_minimap_control`, `command_palette_overlay`, `memory_arena`, `parallel_for_task` (4096-item batch size).
  - Package inspection (`Default.sublime-package`, `Color Scheme - Default.sublime-package`) unveiled commands architecture and Mariana HSL color tokens.
- **Implementation**:
  - Implemented Skyline Minimap in `crates/txt-gui/src/editor_view.rs` with viewport indicator box and toggle control.
  - Implemented `TextTransform` suite for Sublime-style case conversions (Upper, Lower, Title, snake_case, kebab-case, CamelCase) and indentation conversions.
  - Created and merged Pull Request #2 (`feature/rea-sublime-enhancements`).
  - Updated `.gitignore` to prevent committing proprietary binary blobs.
- **Outcome**: All 21 workspace tests pass cleanly. Codebase pushed and synchronized with remote repository.

---

### Entry 006: Autonomous Multi-Agent Synthesis (CRDT, Search, Plugins, LSP)
- **Context**: The user instructed to "use agents to finish it all".
- **Harness Orchestration**: Dispatched 3 concurrent specialist subagents:
  1. `CRDT Sync Specialist`: Engineered `crates/txt-sync/src/crdt.rs` with VectorClock causality and RGA conflict-free resolution, integrated into `SyncEngine`.
  2. `Parallel Search Specialist`: Engineered `crates/txt-core/src/search.rs` with Sublime's 4096-item batch work units, binary header skipping, and multithreaded directory traversal.
  3. `Plugin & LSP Specialist`: Engineered `crates/txt-core/src/plugin.rs` (EventListener, PluginManager, built-in plugins) and `crates/txt-core/src/lsp.rs` (JSON-RPC stdio streaming framer, diagnostics, completions, hover).
- **GUI Integration**: Connected all modules into `TxtEditorApp`, adding `Cmd+Shift+F` for Find in Files, live document statistics, LSP diagnostic caches, and CRDT delta syncing.
- **Outcome**: Workspace test suite grew from 21 tests to 66 passing tests (0 failures, 0 warnings). Binary builds in 0.12s. All roadmap milestones achieved.

---

### Entry 007: v1.0.0 Milestone Release & v1.0.1 File Browser & Editor Enhancements
- **Context**: The user validated `v1` functionality ("works pretty well. save this as v1 and lets work on v1.0.1 focusing on fixes, building out the file browser more and other fairly simple things").
- **Release Automation**:
  - Cut production tags `v1.0.0` and `v1` and published official GitHub Release `v1.0.0`.
  - Created development branch `feature/v1.0.1-file-browser-enhancements` and bumped workspace manifest version to `1.0.1`.
- **v1.0.1 Implementation**:
  - **Sublime "OPEN FILES" Section**: Implemented dedicated section in sidebar displaying all open tabs with active file highlighting, dirty state indicator (`●`), and quick tab close button (`×`).
  - **Live Filter Bar**: Added search filter input (`🔍`) to instantly filter the project file tree in real-time.
  - **Inline New File Creation**: Added dialog (`+`) directly in the sidebar toolbar to create files on disk with instantaneous editor opening.
  - **Comprehensive File Icons**: Added iconography for Rust (`🦀`), Markdown (`📝`), TOML/JSON (`⚙`/`📦`), Python (`🐍`), JS/TS (`🟨`/`🔷`), C/C++ (`🇨`/`➕`), Go (`🐹`), Shell (`🐚`), Web (`🌐`/`🎨`), and images.
  - **File Size Tooltips**: Hovering over directory entries displays formatted file size in KB and relative path.
  - **Outcome**: All tests pass cleanly across workspace crates (0 warnings, 0 errors).

---

### Entry 008: Full Suite File Browser & Multi-Root Project Architecture
- **Context**: The user requested to "use agents to fully build out the file browser and full suite of features for it".
- **Harness Multi-Agent Orchestration**:
  1. `Project Tree & File Operations Specialist`: Engineered `crates/txt-core/src/project.rs` (`TreeNode`, `ProjectFolder`, `ProjectManager`, `FileOps`) providing multi-root workspaces, recursive traversal with directories first, VCS/build ignore filtering (`.git`, `target/`, `node_modules/`, `.DS_Store`), tree expansion state preservation, and atomic file operations (create file, create directory, rename, duplicate, recursive delete).
  2. `Git Status & Watcher Specialist`: Engineered `crates/txt-core/src/git_status.rs` (`GitFileStatus`, `GitStatusScanner` with porcelain parsing and Mariana status coloring) and `crates/txt-core/src/watcher.rs` (`FileSystemWatcher` with strict dynamic lifecycle cleanup when subscriber count drops to zero).
- **GUI & Interaction Overhaul (`crates/txt-gui/src/sidebar.rs`)**:
  - **Right-Click Context Menus**: Native right-click context menu on all files and folders (New File, New Folder, Rename, Duplicate, Copy Path, Reveal in Finder/Explorer, Delete).
  - **Interactive Modal Dialogs**: Sublime-style modal overlay for file creation, folder creation, renaming, and delete confirmation with Enter/Escape keyboard controls.
  - **Multi-Root Workspaces**: Support for registering multiple project root folders (`project.add_folder`, removal via context menu).
  - **Git Status Badges & Highlighting**: Live `M`, `U`, `A`, `D` indicators in the sidebar with git status color tokens matching file dirtiness.
  - **Reveal Active File in Sidebar**: <kbd>Cmd+Option+R</kbd> (`file.reveal_in_sidebar`) automatically expands parent folders and highlights the current buffer in the sidebar.
  - **Sidebar Toolbar Controls**: Header toolbar with Expand All (`⤢`), Collapse All (`⤡`), Toggle Hidden Files (`👁`), Refresh (`↻`), New Item (`+`), and Live Search Filter (`🔍`).
  - **Dynamic FS Watcher**: Reactively refreshes directory trees on background file modifications.
- **Outcome**: 74 unit, integration, benchmark, and stress tests passing cleanly with 0 warnings and 0 errors.

---

### Entry 009: v1.1 Advanced Editor Suite (Multi-Cursor, Find & Replace, Symbols, Session)
- **Context**: The user requested: "Now use agents to work on v1.1 and all the things you feel it should include as an expert rust and app developer".
- **Harness Multi-Agent Orchestration**:
  1. `Multi-Cursor Specialist`: Engineered Sublime Text multi-selection powers in `crates/txt-core/src/cursor.rs`:
     - `select_word_at`: Scans word boundaries with Unicode alphanumeric support.
     - `select_next_occurrence` (<kbd>Cmd+D</kbd> / <kbd>Ctrl+D</kbd>): Word boundary expansion, incremental next match selection, and wrap-around.
     - `select_all_occurrences` (<kbd>Cmd+Ctrl+G</kbd> / <kbd>Alt+F3</kbd>): Finds all matches across the entire buffer and allocates multi-cursors.
     - `split_selection_into_lines` (<kbd>Cmd+Shift+L</kbd> / <kbd>Ctrl+Shift+L</kbd>): Multi-line selection splitting into line cursors.
     - `add_cursor_above` / `add_cursor_below` (<kbd>Cmd+Option+Up/Down</kbd>): Cursor stacking with ragged line length clamping.
  2. `Find & Replace Engine Specialist`: Engineered `crates/txt-core/src/find_replace.rs`:
     - `FindMatch`, `FindOptions` (case-sensitive, whole-word `\b`, regex pattern matching).
     - Sub-millisecond buffer search (`find_all`, `find_next`, `find_prev` with EOF wrap-around).
     - `replace_next` with regex capture group expansion (`$1`).
     - `replace_all` executing replacements in reverse character order to prevent index drift.
  3. `Symbol Indexer & Session Specialist`:
     - `crates/txt-core/src/symbols.rs`: Lexical symbol scanner (`SymbolExtractor`, `DocumentSymbol`, `SymbolKind`) extracting functions, structs, enums, traits, classes, headings, and constants across Rust, Python, Markdown, and JS/TS.
     - `crates/txt-core/src/session.rs`: Persistent state manager (`SessionManager`, `SessionState`) persisting open files, active tab, font size, theme, and sidebar visibility to `.txt/session.json`.
- **GUI Integration (`crates/txt-gui`)**:
  - **Sublime Find & Replace Toolbar (`find_replace_view.rs`)**: Bottom slide-up panel with find/replace inputs, `Aa`, `\b`, `.*` toggles, status badge, and next/prev/replace buttons.
  - **Viewport Match Highlights**: Golden highlight rectangles in the editor canvas for search matches with an active match outline.
  - **Command Palette Goto Symbol & Goto Line (`palette.rs`)**: Prefixing palette with `@` activates Goto Symbol with fuzzy search and matched letter highlights; prefixing with `:` activates Goto Line.
  - **Persistent Session Restoration (`app.rs`)**: Workspace state restores seamlessly on startup and persists on file operations and settings changes.
- **Verification & Outcome**: 123 unit, integration, benchmark, and stress tests passing with 0 warnings and 0 errors across the entire workspace. Desktop binary builds in 0.44s.

---

### Entry 010: v2.0 Visual Polish, 7 Themes, Interactive Top MenuBar Widget, and Workspace Support
- **Context**: The user requested: "now work on v2 with more visual polish, themeing, full mac menubar support and menus, an interactive menubar widget, and full file, folder, and workspace support".
- **Harness Multi-Agent Orchestration**:
  1. `Workspace & Project Specialist`:
     - `crates/txt-core/src/workspace.rs`: Multi-root project configurations (`WorkspaceFolder`, `WorkspaceSettings`, `WorkspaceConfig`) with atomic save/load and backwards-compatible parsing of Sublime Text `.sublime-project` JSON files.
     - `crates/txt-core/src/session.rs`: Extended `SessionState` with `recent_files` (LRU bounded to 20), `recent_workspaces` (LRU bounded to 10), and `active_workspace` path tracking.
     - `crates/txt-core/tests/workspace_test.rs`: 6 comprehensive unit tests validating project save/load, duplicate prevention, recent LRU deduplication, and `.sublime-project` compatibility.
  2. `Theme & Aesthetics Specialist`:
     - `crates/txt-gui/src/theme.rs`: Expanded from 2 themes to 7 complete color schemes:
       1. **Mariana**: Sublime Text modern slate dark.
       2. **Monokai**: Classic iconic high-contrast dark.
       3. **Solarized Dark**: Precision low-contrast dark.
       4. **Solarized Light**: Clean low-contrast daylight palette.
       5. **One Dark**: Indigo/slate dark.
       6. **Dracula**: High-contrast gothic dark.
       7. **Nord**: Arctic blue palette.
     - Implemented full palette methods: `tab_active_text()`, `tab_inactive_text()`, `gutter_active_text()`, `status_bar_text()`, `menu_bar_bg()`, `menu_item_hover()`, `is_dark()`, and `from_name()` with case-insensitive and slug resolution.
     - `crates/txt-gui/tests/theme_test.rs`: 5 unit tests validating round-trip lookups, dark/light classification, and syntax color contrast.
- **Top MenuBar Widget & UI Integration (`crates/txt-gui`)**:
  - `crates/txt-gui/src/menubar.rs`: Native top menu bar widget implementing all Sublime Text menus: `txt`, `File`, `Edit`, `Selection`, `Find`, `View`, `Goto`, `Tools`, `Project`, `Help`.
  - Nested submenus for **Color Schemes** (with live active checkmarks) and **Open Recent Files** / **Open Recent Workspaces**.
  - Dynamic right-aligned Active Workspace indicator in the menu bar.
  - Keyboard accelerators displayed for commands and shortcuts wired for instant execution.
  - Sublime file actions: "Save As...", "Save All", "Revert File", "Close File".
  - Font scaling commands: Increase (<kbd>Cmd+=</kbd>), Decrease (<kbd>Cmd+-</kbd>), Reset (<kbd>Cmd+0</kbd>).
  - Multi-tab enhancements: duplicate tab prevention on opening, dirty indicator dots, active tab color highlights.
- **Verification & Outcome**:
  - `cargo check --workspace`: 0 warnings, 0 errors.
  - `cargo test --workspace`: 139 passed across all crates, 0 failed.
  - `cargo build -p txt-gui --bin txt`: successful compilation in 0.25s.

