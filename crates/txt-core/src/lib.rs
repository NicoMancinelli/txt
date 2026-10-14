pub mod buffer;
pub mod command;
pub mod cursor;
pub mod fuzzy;
pub mod history;
pub mod memory;
pub mod syntax;
pub mod transform;

pub use buffer::{LineEnding, TextBuffer};
pub use command::{Command, CommandDescriptor, CommandRegistry, MatchedCommand};
pub use cursor::{CursorPosition, Selection, SelectionSet};
pub use fuzzy::{FuzzyMatch, FuzzyMatcher};
pub use history::{Change, HistoryManager, Transaction};
pub use memory::MemoryMetrics;
pub use syntax::{Language, SyntaxHighlighter, SyntaxToken, TokenType};
pub use transform::TextTransform;
