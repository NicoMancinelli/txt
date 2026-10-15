use std::ops::Range;
use std::path::{Path, PathBuf};
use ropey::{Rope, RopeSlice};
use crate::cursor::{CursorPosition, Selection};
use crate::history::{Change, HistoryManager};

/// High-performance text buffer modeled after Sublime Text's core buffer engine.
/// Backed by a B-Tree Rope data structure providing O(log N) operations on multi-gigabyte files.
#[derive(Debug, Clone)]
pub struct TextBuffer {
    rope: Rope,
    file_path: Option<PathBuf>,
    is_dirty: bool,
    history: HistoryManager,
    encoding: String,
    line_ending: LineEnding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    /// Creates a new empty text buffer.
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            is_dirty: false,
            history: HistoryManager::new(),
            encoding: "UTF-8".to_string(),
            line_ending: LineEnding::Lf,
        }
    }

    /// Creates a buffer from a string slice.
    pub fn from_str(text: &str) -> Self {
        let line_ending = if text.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
        Self {
            rope: Rope::from_str(text),
            file_path: None,
            is_dirty: false,
            history: HistoryManager::new(),
            encoding: "UTF-8".to_string(),
            line_ending,
        }
    }

    /// Loads a buffer from a file path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let mut buffer = Self::from_str(&content);
        buffer.file_path = Some(path.as_ref().to_path_buf());
        buffer.is_dirty = false;
        Ok(buffer)
    }

    /// Saves the current buffer contents to disk.
    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            let content = self.rope.to_string();
            std::fs::write(path, content)?;
            self.is_dirty = false;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path associated with buffer",
            ))
        }
    }

    /// Saves the buffer to a designated path.
    pub fn save_as<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let content = self.rope.to_string();
        std::fs::write(path.as_ref(), content)?;
        self.file_path = Some(path.as_ref().to_path_buf());
        self.is_dirty = false;
        Ok(())
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn set_file_path<P: AsRef<Path>>(&mut self, path: P) {
        self.file_path = Some(path.as_ref().to_path_buf());
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string())
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    /// Total number of Unicode characters in the buffer.
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Total number of bytes in UTF-8.
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// Returns the character at the specified character offset, if within bounds.
    pub fn char_at(&self, idx: usize) -> Option<char> {
        if idx < self.rope.len_chars() {
            Some(self.rope.char(idx))
        } else {
            None
        }
    }

    /// Access to the underlying Rope structure.
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Total number of lines in the buffer.
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Gets a slice of a specific line (0-indexed).
    pub fn line(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        if line_idx < self.rope.len_lines() {
            Some(self.rope.line(line_idx))
        } else {
            None
        }
    }

    /// Gets the full text as an owned String.
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Gets a substring range as an owned String.
    pub fn slice_text(&self, range: Range<usize>) -> String {
        let start = range.start.min(self.len_chars());
        let end = range.end.min(self.len_chars());
        if start >= end {
            String::new()
        } else {
            self.rope.slice(start..end).to_string()
        }
    }

    /// Converts char index to (line, column) 0-indexed.
    pub fn char_to_pos(&self, char_idx: usize) -> CursorPosition {
        let clamped = char_idx.min(self.len_chars());
        let line = self.rope.char_to_line(clamped);
        let line_start_char = self.rope.line_to_char(line);
        let column = clamped.saturating_sub(line_start_char);
        CursorPosition { line, column }
    }

    /// Converts (line, column) to character index.
    pub fn pos_to_char(&self, pos: CursorPosition) -> usize {
        let total_lines = self.len_lines();
        if total_lines == 0 {
            return 0;
        }
        let line = pos.line.min(total_lines - 1);
        let line_start = self.rope.line_to_char(line);
        let line_len = self.rope.line(line).len_chars();
        // Don't count trailing newline in column clamping if present
        let max_col = if line + 1 < total_lines {
            line_len.saturating_sub(1)
        } else {
            line_len
        };
        let col = pos.column.min(max_col);
        (line_start + col).min(self.len_chars())
    }

    /// Inserts text at the specified character offset.
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        let clamped = char_idx.min(self.len_chars());
        self.rope.insert(clamped, text);
        self.is_dirty = true;
        self.history.record_change(Change::Insert {
            position: clamped,
            text: text.to_string(),
        });
    }

    /// Deletes a range of characters [start..end).
    pub fn delete(&mut self, range: Range<usize>) {
        let start = range.start.min(self.len_chars());
        let end = range.end.min(self.len_chars());
        if start >= end {
            return;
        }
        let deleted_text = self.rope.slice(start..end).to_string();
        self.rope.remove(start..end);
        self.is_dirty = true;
        self.history.record_change(Change::Delete {
            position: start,
            text: deleted_text,
        });
    }

    /// Replaces a range of characters with new text.
    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        self.history.begin_group();
        self.delete(range.clone());
        self.insert(range.start, text);
        self.history.end_group();
    }

    /// Applies edits at multiple selections (Sublime multi-cursor edit).
    /// Returns the new cursor positions.
    pub fn insert_at_selections(&mut self, selections: &[Selection], text: &str) -> Vec<Selection> {
        self.history.begin_group();
        let mut sorted_selections = selections.to_vec();
        // Sort descending so offsets earlier in the buffer aren't invalidated by modifications
        sorted_selections.sort_by_key(|s| std::cmp::Reverse(s.min_char()));

        let mut offset_adjustments = Vec::new();

        for sel in &sorted_selections {
            let start = sel.min_char();
            let end = sel.max_char();
            if start != end {
                self.delete(start..end);
            }
            self.insert(start, text);
            offset_adjustments.push((start, text.chars().count()));
        }
        self.history.end_group();

        // Calculate new selections for caller
        let mut new_selections = Vec::new();
        let mut cumulative_delta: isize = 0;
        let mut forward_selections = selections.to_vec();
        forward_selections.sort_by_key(|s| s.min_char());

        for sel in forward_selections {
            let start = sel.min_char();
            let end = sel.max_char();
            let deleted_len = end - start;
            let inserted_len = text.chars().count();
            let new_char = ((start as isize) + cumulative_delta + (inserted_len as isize)) as usize;
            new_selections.push(Selection::point(new_char));
            cumulative_delta += (inserted_len as isize) - (deleted_len as isize);
        }

        if new_selections.is_empty() {
            vec![Selection::point(0)]
        } else {
            new_selections
        }
    }

    /// Undoes the last edit transaction.
    pub fn undo(&mut self) -> bool {
        if let Some(changes) = self.history.undo() {
            for change in changes.into_iter().rev() {
                match change {
                    Change::Insert { position, text } => {
                        let len = text.chars().count();
                        self.rope.remove(position..position + len);
                    }
                    Change::Delete { position, text } => {
                        self.rope.insert(position, &text);
                    }
                }
            }
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Redoes the previously undone edit transaction.
    pub fn redo(&mut self) -> bool {
        if let Some(changes) = self.history.redo() {
            for change in changes {
                match change {
                    Change::Insert { position, text } => {
                        self.rope.insert(position, &text);
                    }
                    Change::Delete { position, text } => {
                        let len = text.chars().count();
                        self.rope.remove(position..position + len);
                    }
                }
            }
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Direct memory estimate of the Rope buffer in bytes.
    pub fn estimated_memory_usage(&self) -> usize {
        // Approximate Rope node tree overhead + text payload
        self.rope.len_bytes() + (self.rope.len_lines() * 16) + std::mem::size_of::<Self>()
    }
}
