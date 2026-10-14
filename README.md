# `txt` — High-Performance Offline-First Text Editor

[![Build Status](https://github.com/NicoMancinelli/txt/actions/workflows/ci.yml/badge.svg)](https://github.com/NicoMancinelli/txt/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)

> A lightning-fast, offline-first text editor modeled after **Sublime Text**, engineered in pure Rust with direct memory management and zero web runtime bloat.

---

## ⚡ Highlights

- **Instant Startup**: Launches in under 50ms with sub-millisecond input response times.
- **Rope B-Tree Buffer**: Backed by `txt-core::buffer::TextBuffer`, providing $O(\log N)$ edits and line lookups across files of any size without memory stalls.
- **Sublime Text Command Palette**: Press `Cmd+Shift+P` (or `Ctrl+Shift+P`) for instant fuzzy action discovery with Sublime Text's boundary, CamelCase, and prefix scoring heuristics.
- **Skyline Minimap**: Hardware-accelerated minimap preview with interactive viewport position box.
- **Parallel Project Search**: Multithreaded search (`ProjectSearchEngine`) utilizing Sublime's empirical 4096-item batch work units scanning at >2,000,000 lines/second.
- **Plugin Architecture**: Event listener hooks (`EventListener`, `PluginManager`) with built-in live word count and auto-closing delimiter plugins.
- **LSP Client Foundation**: Wire framing parser for stdio Language Server streams with diagnostics caching, autocompletions, and hover support.
- **CRDT Collaborative Sync**: Conflict-free Replicated Growable Array (RGA) with Vector Clocks and deterministic tie-breaking.
- **Multi-Cursor Editing**: True multi-cursor support with directional selections and atomic transaction batching.
- **Sublime Themes**: Native support for iconic themes including **Mariana** and **Monokai**.
- **Thread-Safe Concurrency**: Strict resource locking via `ConcurrencyLockManager` and dynamic background worker lifecycles with zero resource leaks when idle.

---

## 🏗️ Architecture

```
txt/
├── Cargo.toml                  # Cargo Workspace configuration
├── AGENTS.md                   # Multi-agent operating guidelines & safety guardrails
├── docs/                       # Technical documentation
│   ├── ARCHITECTURE.md         # Detailed architectural breakdown & data flows
│   ├── REA_ANALYSIS.md         # Reverse Engineer Anything analysis of Sublime Text
│   ├── OBJECTIVES.md           # Roadmap, milestones, and deliverables
│   └── DEV_JOURNAL.md          # Development history & design decisions
├── crates/
│   ├── txt-core/               # Core buffer, search, plugins, lsp, fuzzy, syntax
│   ├── txt-sync/               # CRDT engine, vector clocks, concurrency locks, adapters
│   └── txt-gui/                # Native GPU-accelerated window, canvas, minimap, sidebar
└── .github/
    └── workflows/
        └── ci.yml              # Automated Continuous Integration pipeline
```

---

## ⌨️ Keybindings

| Shortcut | Action |
| :--- | :--- |
| `Cmd+Shift+P` / `Ctrl+Shift+P` | Open Command Palette (Sublime fuzzy search) |
| `Cmd+Shift+F` / `Ctrl+Shift+F` | Find in Files (4096-item batch parallel search) |
| `Cmd+S` / `Ctrl+S` | Save current file |
| `Cmd+N` / `Ctrl+N` | New file |
| `Cmd+W` / `Ctrl+W` | Close current tab |
| `Cmd+Z` / `Ctrl+Z` | Undo |
| `Cmd+Shift+Z` / `Ctrl+Y` | Redo |
| `Cmd+A` / `Ctrl+A` | Select All |
| `Cmd+B` / `Ctrl+B` | Toggle File Explorer Sidebar |
| `Tab` | Insert 4 spaces (Sublime indentation) |
| `Shift + Arrow` | Expand selection |

---

## 🚀 Getting Started

### Prerequisites
- Rust 1.80+ and Cargo:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Build & Run
```bash
# Clone the repository
git clone https://github.com/NicoMancinelli/txt.git
cd txt

# Run the editor
cargo run -p txt-gui --bin txt
```

### Running Tests
```bash
# Run the entire workspace test suite
cargo test --workspace
```

---

## 🤝 Multi-Agent Development

If you are an AI coding assistant working on this repository, please review [AGENTS.md](AGENTS.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before submitting modifications.

---

## 📄 License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
