# Architecture Specification: txt

> Modeled after Sublime Text: High Performance, Offline-First Native Editor.

---

## 1. High-Level Architectural Diagram

```mermaid
graph TD
    subgraph Frontend["txt-gui (Native GPU UI)"]
        App["TxtEditorApp (State Machine)"]
        Palette["CommandPalette (Fuzzy Modal)"]
        Canvas["EditorView (Viewport Line Renderer)"]
        Sidebar["Sidebar (Project Tree Explorer)"]
        Tabs["TabBar (Multi-buffer Strip)"]
        StatusBar["StatusBar (Metrics & Sync State)"]
    end

    subgraph Core["txt-core (Domain & Algorithms)"]
        Buffer["TextBuffer (B-Tree Rope Engine)"]
        Cursor["SelectionSet (Multi-Cursor & Selections)"]
        Fuzzy["FuzzyMatcher (Sublime Scoring Heuristics)"]
        Command["CommandRegistry (Actions & Shortcuts)"]
        Syntax["SyntaxHighlighter (Lexical Scanner)"]
        History["HistoryManager (Transaction Grouping)"]
        Memory["MemoryMetrics (Allocation Profiling)"]
    end

    subgraph Sync["txt-sync (Sync Subsystem)"]
        LockMgr["ConcurrencyLockManager<Uuid>"]
        Engine["SyncEngine (Dynamic Worker Lifecycle)"]
        LocalAdapter["LocalFilesystemAdapter (Atomic Shadow Journal)"]
        MemAdapter["MemorySyncAdapter (In-Memory Relay)"]
        CloudAdapter["CloudSyncAdapter (Offline Buffer Queue)"]
    end

    App --> Canvas
    App --> Palette
    App --> Sidebar
    App --> Tabs
    App --> StatusBar

    Canvas --> Buffer
    Canvas --> Cursor
    Canvas --> Syntax

    Palette --> Fuzzy
    Palette --> Command

    App --> Engine
    Engine --> LockMgr
    Engine --> LocalAdapter
    Engine --> MemAdapter
    Engine --> CloudAdapter
```

---

## 2. Text Buffer & Memory Management

Sublime Text achieved legendary reputation through its custom buffer design:
1. **B-Tree Rope (`txt-core::buffer::TextBuffer`)**:
   - Backed by `ropey`, text is chunked into node leaves rather than stored in flat contiguous memory.
   - Insertions and deletions take $O(\log N)$ time regardless of file size (handling 100MB+ files seamlessly).
   - Fast line lookups (`char_to_line`, `line_to_char`) without linear scans.
2. **Direct Memory Metrics**:
   - `MemoryMetrics` tracks estimated heap allocations, node tree overhead, line counts, and history stack depths.
   - Visible to the user in the status bar in real time.

---

## 3. Sublime Text Fuzzy Matcher Heuristics

The fuzzy search engine (`txt-core::fuzzy::FuzzyMatcher`) implements Sublime Text's matching algorithm:
- **Character Subsequence Check**: Case-insensitive sequence validation.
- **Prefix Match Bonus**: +60 points for matches at the beginning of the string.
- **Word Boundary Bonus**: +45 points for matches following separators (`/`, `\`, `_`, `-`, `:`, `.`, space).
- **CamelCase Bonus**: +35 points for uppercase characters following lowercase characters.
- **Consecutive Match Bonus**: +50 points for contiguous characters.
- **Acronym Matching**: Enables abbreviations like `cp` to instantly match `Tools: Command Palette`.
- **Length Penalty**: Penalizes long candidates to favor concise exact matches.

---

## 4. Multi-Cursor & Selection Range Model

- Each selection (`Selection`) consists of an `anchor` (origin) and `head` (caret).
- `SelectionSet` maintains multiple cursors, automatically sorting and merging overlapping selections.
- Multi-cursor edits execute in descending order to prevent index drift, with transaction grouping for single-step undo.

---

## 5. Modular Synchronization & Concurrency Safety

The synchronization architecture (`txt-sync`) follows strict concurrency guardrails:
1. **`ConcurrencyLockManager<K>`**:
   - Shared document resources require serialized operations to prevent race conditions.
   - Acquires fine-grained locks per document ID (`Uuid`) during push and pull patch cycles.
2. **Pluggable Sync Adapters**:
   - `LocalFilesystemAdapter`: Writes atomic shadow journals to disk using temp files and atomic rename.
   - `MemorySyncAdapter`: Ultra-fast in-memory patch store for testing and local relay.
   - `CloudSyncAdapter`: Supports offline patch buffering and replaying when connectivity is restored.
3. **Dynamic Background Worker Lifecycle**:
   - The sync heartbeat worker starts dynamically when active clients increase from 0 to 1.
   - **Immediately shuts down and cleans up background threads when active clients reach 0**, preventing idle resource drain.
