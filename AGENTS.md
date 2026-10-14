# AGENTS.md — Autonomous AI Agent Operating Manual

> **Scope**: This document establishes the operational protocols, architectural rules, and development guidelines for any AI coding agent or multi-agent harness collaborating on the `txt` codebase.

---

## 1. System Mission & Identity

`txt` is a high-performance, offline-first text editor modeled after **Sublime Text**, written in modern Rust. It is engineered for instant startup (<50ms), sub-millisecond input latency, minimal memory footprint, and reliable synchronization across devices.

### Target Performance Invariants
1. **Zero Runtime Bloat**: No WebViews, no Chromium, no Electron, no heavy JavaScript runtimes.
2. **$O(\log N)$ Buffer Edits**: The text buffer is backed by a B-Tree Rope (`ropey`), ensuring instantaneous inserts, deletions, and line lookups even on multi-gigabyte files.
3. **Sub-Millisecond Viewport Rendering**: The rendering pipeline only processes lines inside the visible viewport slice.
4. **Predictable Memory Footprint**: Cache-friendly data structures with direct memory metrics tracking.

---

## 2. Workspace & Crate Structure

```
txt/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── txt-core/               # Core data structures & editor logic
│   │   ├── src/buffer.rs       # Rope-backed TextBuffer with slice caching
│   │   ├── src/cursor.rs       # Multi-cursor & selection range engine
│   │   ├── src/command.rs      # Command registry & dispatch
│   │   ├── src/fuzzy.rs        # Sublime Text fuzzy matching algorithm
│   │   ├── src/history.rs      # Undo/Redo transaction engine
│   │   ├── src/syntax.rs       # Lexical scanner & syntax tokenizer
│   │   └── src/memory.rs       # Direct memory metrics tracker
│   ├── txt-sync/               # Modular synchronization subsystem
│   │   ├── src/lock_manager.rs # Per-resource ConcurrencyLockManager
│   │   ├── src/adapter.rs      # SyncAdapter trait & operational patch types
│   │   ├── src/local_adapter.rs# Atomic filesystem sync adapter
│   │   ├── src/memory_adapter.rs# Ephemeral in-memory sync adapter
│   │   ├── src/cloud_adapter.rs# Cloud adapter interface & offline queue
│   │   └── src/engine.rs       # SyncEngine with dynamic heartbeat lifecycle
│   └── txt-gui/                # Native GPU-accelerated desktop interface
│       ├── src/main.rs         # Entry point & native window configuration
│       ├── src/app.rs          # Main TxtEditorApp state machine
│       ├── src/palette.rs      # Sublime Text-style fuzzy Command Palette modal
│       ├── src/editor_view.rs  # Text editing canvas, gutter, & cursor renderer
│       ├── src/sidebar.rs      # Collapsible file tree & project explorer
│       ├── src/tabs.rs         # Multi-buffer tab strip
│       ├── src/status_bar.rs   # Status bar (line, col, syntax, sync, memory)
│       └── src/theme.rs        # Mariana & Monokai color palettes
└── docs/                       # Architecture, REA analysis, Roadmap, Journal
```

---

## 3. Mandatory Safety Guardrails & Coding Rules

Every agent modifying this codebase **must** adhere to these strict rules:

### Rule 1: Concurrency Locking for Sequential/Stateful Updates
* Whenever performing non-atomic state updates or synchronization (e.g. applying patches, incrementing document revisions, modifying shared counters), **always** serialize operations per resource using `ConcurrencyLockManager<K>`.
* Never acquire locks out of order across different resources to avoid deadlocks.

### Rule 2: Resource Cleanup for Background Tasks & Persistent Connections
* Any background heartbeat, polling worker, or timer interval **must** be tied to active listener/client counts.
* In `SyncEngine`: when `active_clients` increases from 0 to 1, the background worker starts. When `active_clients` drops to 0, the background worker **must immediately be stopped and cleaned up** to prevent CPU and memory leaks.

### Rule 3: Memory & Allocation Discipline
* Do not convert whole `Rope` buffers to owned `String`s inside frame-by-frame rendering loops.
* Only slice visible line chunks within the viewport range.
* Use `SmallVec` or reuse allocated vectors where hot loop allocations can be avoided.

---

## 4. Verification & Testing Protocol

Before marking any task complete or committing changes, run the following verification commands:

```bash
# 1. Compile check across all workspace crates
cargo check --workspace

# 2. Run all unit and integration tests
cargo test --workspace

# 3. Build release/dev binary
cargo build -p txt-gui --bin txt
```

All tests must pass without warnings or errors.

---

## 5. Git & DevOps Workflow

- **Branching Strategy**:
  - `main`: Protected production trunk.
  - `feature/<name>`: New feature implementations.
  - `fix/<name>`: Bug fixes and performance patches.
- **Commit Messages**: Conventional commits (`feat:`, `fix:`, `refactor:`, `perf:`, `docs:`, `test:`, `ci:`).
