pub mod buffer;
pub mod command;
pub mod cursor;
pub mod find_replace;
pub mod fuzzy;
pub mod git_status;
pub mod history;
pub mod lsp;
pub mod memory;
pub mod plugin;
pub mod project;
pub mod search;
pub mod session;
pub mod symbols;
pub mod syntax;
pub mod transform;
pub mod watcher;

pub use buffer::{LineEnding, TextBuffer};
pub use command::{Command, CommandDescriptor, CommandRegistry, MatchedCommand};
pub use cursor::{
    add_cursor_above, add_cursor_below, select_all_occurrences, select_next_occurrence,
    select_word_at, split_selection_into_lines, CursorPosition, Selection, SelectionSet,
};
pub use find_replace::{FindMatch, FindOptions, FindReplaceEngine};
pub use fuzzy::{FuzzyMatch, FuzzyMatcher};
pub use git_status::{GitFileStatus, GitStatusScanner};
pub use history::{Change, HistoryManager, Transaction};
pub use lsp::{
    CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity, HoverInfo,
    LspClientState, LspError, LspFramingError, LspFramingParser, Position, Range,
    RpcError, RpcId, RpcMessage, RpcNotification, RpcRequest, RpcResponse,
};
pub use memory::MemoryMetrics;
pub use plugin::{
    AutoClosingBracketsPlugin, BracketPair, CompletionCandidate, EventListener, PluginManager,
    WordCountPlugin,
};
pub use project::{FileOps, ProjectFolder, ProjectManager, TreeNode};
pub use search::{
    ProjectSearchEngine, SearchError, SearchQuery, SearchResult, SUBLIME_BATCH_SIZE,
};
pub use session::{SessionManager, SessionState};
pub use symbols::{DocumentSymbol, SymbolExtractor, SymbolKind};
pub use syntax::{Language, SyntaxHighlighter, SyntaxToken, TokenType};
pub use transform::TextTransform;
pub use watcher::{FileSystemWatcher, WatchEvent, WatchEventKind, WatchSubscription};

