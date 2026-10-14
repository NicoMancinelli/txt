use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use crate::buffer::TextBuffer;

/// Candidate item returned when querying completion providers from plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionCandidate {
    pub label: String,
    pub insert_text: String,
    pub detail: Option<String>,
    pub kind: Option<String>,
}

impl CompletionCandidate {
    pub fn new(label: impl Into<String>, insert_text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            insert_text: insert_text.into(),
            detail: None,
            kind: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }
}

/// Core event listener trait for editor plugins.
/// Plugins can react to buffer modifications, file saves, and provide autocompletion candidates.
pub trait EventListener: Send + Sync + 'static {
    /// Unique identifier for this plugin instance.
    fn name(&self) -> &str;

    /// Invoked immediately after a text buffer has been modified.
    fn on_modified(&self, _buffer: &TextBuffer) {}

    /// Invoked when a file buffer is saved to disk.
    fn on_save(&self, _path: &Path) {}

    /// Invoked to gather completion candidates matching the provided prefix.
    fn on_query_completions(&self, _prefix: &str) -> Vec<CompletionCandidate> {
        Vec::new()
    }
}

/// Central registry and event dispatcher for editor plugins.
#[derive(Default)]
pub struct PluginManager {
    plugins: RwLock<Vec<Arc<dyn EventListener>>>,
}

impl PluginManager {
    /// Creates a new empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(Vec::new()),
        }
    }

    /// Registers an event listener plugin.
    pub fn register(&self, plugin: Arc<dyn EventListener>) {
        let mut plugins = self.plugins.write();
        plugins.push(plugin);
    }

    /// Unregisters all plugins matching the specified name. Returns true if at least one was removed.
    pub fn unregister(&self, name: &str) -> bool {
        let mut plugins = self.plugins.write();
        let initial_len = plugins.len();
        plugins.retain(|p| p.name() != name);
        plugins.len() < initial_len
    }

    /// Returns the number of currently active plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.read().len()
    }

    /// Returns true if no plugins are currently registered.
    pub fn is_empty(&self) -> bool {
        self.plugins.read().is_empty()
    }

    /// Returns a list of active plugin names.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.read().iter().map(|p| p.name().to_string()).collect()
    }

    /// Clears all registered plugins.
    pub fn clear(&self) {
        self.plugins.write().clear();
    }

    /// Dispatches `on_modified` to all registered plugins.
    pub fn on_modified(&self, buffer: &TextBuffer) {
        // Snapshot the plugin list to prevent deadlocks during event dispatching.
        let plugins = self.plugins.read().clone();
        for plugin in plugins {
            plugin.on_modified(buffer);
        }
    }

    /// Dispatches `on_save` to all registered plugins.
    pub fn on_save(&self, path: &Path) {
        let plugins = self.plugins.read().clone();
        for plugin in plugins {
            plugin.on_save(path);
        }
    }

    /// Queries completions across all active plugins and returns aggregated candidates.
    pub fn on_query_completions(&self, prefix: &str) -> Vec<CompletionCandidate> {
        let plugins = self.plugins.read().clone();
        let mut candidates = Vec::new();
        for plugin in plugins {
            candidates.extend(plugin.on_query_completions(prefix));
        }
        candidates
    }
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("plugins", &self.plugin_names())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Built-in Sample Plugins
// ---------------------------------------------------------------------------

/// Built-in plugin tracking word, character, and line count statistics on buffer updates.
pub struct WordCountPlugin {
    word_count: AtomicUsize,
    char_count: AtomicUsize,
    line_count: AtomicUsize,
}

impl Default for WordCountPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WordCountPlugin {
    pub fn new() -> Self {
        Self {
            word_count: AtomicUsize::new(0),
            char_count: AtomicUsize::new(0),
            line_count: AtomicUsize::new(0),
        }
    }

    pub fn word_count(&self) -> usize {
        self.word_count.load(Ordering::Relaxed)
    }

    pub fn char_count(&self) -> usize {
        self.char_count.load(Ordering::Relaxed)
    }

    pub fn line_count(&self) -> usize {
        self.line_count.load(Ordering::Relaxed)
    }
}

impl EventListener for WordCountPlugin {
    fn name(&self) -> &str {
        "WordCountPlugin"
    }

    fn on_modified(&self, buffer: &TextBuffer) {
        let text = buffer.text();
        let words = text.split_whitespace().count();
        self.word_count.store(words, Ordering::Relaxed);
        self.char_count.store(buffer.len_chars(), Ordering::Relaxed);
        self.line_count.store(buffer.len_lines(), Ordering::Relaxed);
    }

    fn on_query_completions(&self, prefix: &str) -> Vec<CompletionCandidate> {
        if prefix.eq_ignore_ascii_case("stats") || prefix.eq_ignore_ascii_case("words") {
            let count = self.word_count();
            vec![CompletionCandidate::new(
                format!("{count} words"),
                count.to_string(),
            )
            .with_detail("WordCount statistics")
            .with_kind("Metric")]
        } else {
            Vec::new()
        }
    }
}

/// Built-in pair definition for delimiters and brackets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BracketPair {
    pub open: &'static str,
    pub close: &'static str,
    pub name: &'static str,
}

/// Built-in plugin providing autocompletion pairs for brackets, braces, and quotes.
pub struct AutoClosingBracketsPlugin {
    pairs: Vec<BracketPair>,
}

impl Default for AutoClosingBracketsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoClosingBracketsPlugin {
    pub fn new() -> Self {
        Self {
            pairs: vec![
                BracketPair { open: "(", close: ")", name: "Parentheses" },
                BracketPair { open: "[", close: "]", name: "Square Brackets" },
                BracketPair { open: "{", close: "}", name: "Curly Braces" },
                BracketPair { open: "\"", close: "\"", name: "Double Quotes" },
                BracketPair { open: "'", close: "'", name: "Single Quotes" },
                BracketPair { open: "`", close: "`", name: "Backticks" },
                BracketPair { open: "<", close: ">", name: "Angle Brackets" },
            ],
        }
    }

    pub fn with_pairs(pairs: Vec<BracketPair>) -> Self {
        Self { pairs }
    }

    pub fn closing_for(&self, open: &str) -> Option<&'static str> {
        self.pairs.iter().find(|p| p.open == open).map(|p| p.close)
    }
}

impl EventListener for AutoClosingBracketsPlugin {
    fn name(&self) -> &str {
        "AutoClosingBracketsPlugin"
    }

    fn on_query_completions(&self, prefix: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();
        for pair in &self.pairs {
            if prefix.is_empty() || prefix.ends_with(pair.open) || pair.open.starts_with(prefix) {
                let full = format!("{}{}", pair.open, pair.close);
                candidates.push(
                    CompletionCandidate::new(full.clone(), full)
                        .with_detail(format!("Auto-close {}", pair.name))
                        .with_kind("Snippet"),
                );
            }
        }
        candidates
    }
}
