use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use txt_core::buffer::TextBuffer;
use txt_core::plugin::{
    AutoClosingBracketsPlugin, EventListener, PluginManager, WordCountPlugin,
};

struct MockSaveTrackerPlugin {
    saved_paths: Mutex<Vec<PathBuf>>,
    called: AtomicBool,
}

impl MockSaveTrackerPlugin {
    fn new() -> Self {
        Self {
            saved_paths: Mutex::new(Vec::new()),
            called: AtomicBool::new(false),
        }
    }
}

impl EventListener for MockSaveTrackerPlugin {
    fn name(&self) -> &str {
        "MockSaveTrackerPlugin"
    }

    fn on_save(&self, path: &Path) {
        self.saved_paths.lock().push(path.to_path_buf());
        self.called.store(true, Ordering::SeqCst);
    }
}

#[test]
fn test_plugin_registration_and_lifecycle() {
    let manager = PluginManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.plugin_count(), 0);

    let word_count_plugin = Arc::new(WordCountPlugin::new());
    manager.register(word_count_plugin);

    assert_eq!(manager.plugin_count(), 1);
    assert!(!manager.is_empty());
    assert_eq!(manager.plugin_names(), vec!["WordCountPlugin"]);

    let brackets_plugin = Arc::new(AutoClosingBracketsPlugin::new());
    manager.register(brackets_plugin);
    assert_eq!(manager.plugin_count(), 2);

    // Unregister by name
    let removed = manager.unregister("WordCountPlugin");
    assert!(removed);
    assert_eq!(manager.plugin_count(), 1);
    assert_eq!(manager.plugin_names(), vec!["AutoClosingBracketsPlugin"]);

    // Unregister non-existent
    assert!(!manager.unregister("NonExistentPlugin"));

    manager.clear();
    assert!(manager.is_empty());
}

#[test]
fn test_word_count_plugin_on_modified() {
    let manager = PluginManager::new();
    let word_counter = Arc::new(WordCountPlugin::new());
    manager.register(word_counter.clone());

    let mut buffer = TextBuffer::from_str("Hello world Sublime Text editor");
    manager.on_modified(&buffer);

    assert_eq!(word_counter.word_count(), 5);
    assert_eq!(word_counter.char_count(), 31);
    assert_eq!(word_counter.line_count(), 1);

    buffer.insert(31, "\nAnother line with three");
    manager.on_modified(&buffer);

    assert_eq!(word_counter.word_count(), 9);
    assert_eq!(word_counter.line_count(), 2);

    // Test completion candidate from WordCountPlugin
    let completions = manager.on_query_completions("words");
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].label, "9 words");
    assert_eq!(completions[0].insert_text, "9");
}

#[test]
fn test_mock_save_tracker_on_save() {
    let manager = PluginManager::new();
    let save_tracker = Arc::new(MockSaveTrackerPlugin::new());
    manager.register(save_tracker.clone());

    let test_path = Path::new("/workspace/project/main.rs");
    manager.on_save(test_path);

    assert!(save_tracker.called.load(Ordering::SeqCst));
    let paths = save_tracker.saved_paths.lock().clone();
    assert_eq!(paths, vec![PathBuf::from("/workspace/project/main.rs")]);
}

#[test]
fn test_auto_closing_brackets_plugin() {
    let manager = PluginManager::new();
    let brackets = Arc::new(AutoClosingBracketsPlugin::new());
    manager.register(brackets);

    // Query with open paren
    let completions = manager.on_query_completions("(");
    assert!(!completions.is_empty());
    let paren_candidate = completions.iter().find(|c| c.label == "()");
    assert!(paren_candidate.is_some());
    let candidate = paren_candidate.unwrap();
    assert_eq!(candidate.insert_text, "()");
    assert_eq!(candidate.detail.as_deref(), Some("Auto-close Parentheses"));
    assert_eq!(candidate.kind.as_deref(), Some("Snippet"));

    // Query with quote
    let quote_completions = manager.on_query_completions("\"");
    assert!(quote_completions.iter().any(|c| c.label == "\"\""));

    // Query with empty prefix returns all bracket pairs
    let all_candidates = manager.on_query_completions("");
    assert!(all_candidates.len() >= 7);
}

#[test]
fn test_multiple_plugins_aggregated_completions() {
    let manager = PluginManager::new();
    let word_counter = Arc::new(WordCountPlugin::new());
    let brackets = Arc::new(AutoClosingBracketsPlugin::new());

    manager.register(word_counter.clone());
    manager.register(brackets);

    let buf = TextBuffer::from_str("one two three");
    manager.on_modified(&buf);

    // Query that triggers word counter stats
    let stats = manager.on_query_completions("stats");
    assert!(stats.iter().any(|c| c.label == "3 words"));

    // Query that matches bracket snippet
    let bracket = manager.on_query_completions("{");
    assert!(bracket.iter().any(|c| c.label == "{}"));
}
