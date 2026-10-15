use std::ops::Range;
use serde::{Deserialize, Serialize};

use crate::buffer::TextBuffer;

/// 0-indexed line and column coordinate in the editor buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

impl CursorPosition {
    pub const ZERO: Self = Self { line: 0, column: 0 };

    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// A selection or cursor point in the buffer.
/// An anchor and head allow directional selections (e.g. dragging backwards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Selection {
    pub anchor: usize, // Character offset where selection started
    pub head: usize,   // Character offset where cursor currently sits
}

impl Selection {
    /// Creates a single-point cursor with no range selected.
    pub fn point(offset: usize) -> Self {
        Self {
            anchor: offset,
            head: offset,
        }
    }

    /// Creates a range selection.
    pub fn range(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn min_char(&self) -> usize {
        self.anchor.min(self.head)
    }

    pub fn max_char(&self) -> usize {
        self.anchor.max(self.head)
    }

    pub fn len(&self) -> usize {
        self.max_char() - self.min_char()
    }

    /// True if the selection ranges overlap or touch.
    pub fn overlaps(&self, other: &Selection) -> bool {
        !(self.max_char() < other.min_char() || self.min_char() > other.max_char())
    }

    /// Merges two overlapping selections into one.
    pub fn merge(&self, other: &Selection) -> Selection {
        let min = self.min_char().min(other.min_char());
        let max = self.max_char().max(other.max_char());
        // Preserve direction of the current selection's head
        if self.head >= self.anchor {
            Selection::range(min, max)
        } else {
            Selection::range(max, min)
        }
    }
}

/// Manages multiple cursors and selections in a text buffer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionSet {
    selections: Vec<Selection>,
    primary_idx: usize,
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionSet {
    pub fn new() -> Self {
        Self {
            selections: vec![Selection::point(0)],
            primary_idx: 0,
        }
    }

    pub fn from_selection(selection: Selection) -> Self {
        Self {
            selections: vec![selection],
            primary_idx: 0,
        }
    }

    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn primary(&self) -> Selection {
        self.selections.get(self.primary_idx).copied().unwrap_or(Selection::point(0))
    }

    pub fn set_primary(&mut self, selection: Selection) {
        self.selections = vec![selection];
        self.primary_idx = 0;
    }

    /// Adds a new selection, sorting and merging any overlaps.
    pub fn add(&mut self, selection: Selection) {
        self.selections.push(selection);
        self.normalize();
    }

    /// Replaces all selections with a new list, normalizing overlaps.
    pub fn set_all(&mut self, mut list: Vec<Selection>) {
        if list.is_empty() {
            list.push(Selection::point(0));
        }
        self.selections = list;
        self.normalize();
    }

    /// Clears secondary cursors, keeping only the primary selection.
    pub fn collapse_to_primary(&mut self) {
        let prim = self.primary();
        self.selections = vec![prim];
        self.primary_idx = 0;
    }

    /// Sorts and merges overlapping selections.
    pub fn normalize(&mut self) {
        if self.selections.is_empty() {
            self.selections = vec![Selection::point(0)];
            self.primary_idx = 0;
            return;
        }

        self.selections.sort_by_key(|s| s.min_char());

        let mut merged: Vec<Selection> = Vec::with_capacity(self.selections.len());
        for sel in self.selections.drain(..) {
            if let Some(last) = merged.last_mut() {
                if last.overlaps(&sel) {
                    *last = last.merge(&sel);
                    continue;
                }
            }
            merged.push(sel);
        }

        self.selections = merged;
        if self.primary_idx >= self.selections.len() {
            self.primary_idx = self.selections.len().saturating_sub(1);
        }
    }

    pub fn len(&self) -> usize {
        self.selections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    pub fn primary_index(&self) -> usize {
        self.primary_idx
    }

    pub fn set_primary_index(&mut self, idx: usize) {
        if idx < self.selections.len() {
            self.primary_idx = idx;
        }
    }
}

/// Returns true if character is alphanumeric or an underscore.
#[inline]
pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Finds all non-overlapping character index ranges of `needle` in `buffer`.
pub fn find_all_occurrences(buffer: &TextBuffer, needle: &str) -> Vec<Range<usize>> {
    if needle.is_empty() || buffer.is_empty() {
        return Vec::new();
    }

    let text = buffer.text();
    let needle_chars = needle.chars().count();
    let mut occurrences = Vec::new();
    let mut char_offset = 0;
    let mut last_byte_offset = 0;

    for (byte_idx, _) in text.match_indices(needle) {
        char_offset += text[last_byte_offset..byte_idx].chars().count();
        last_byte_offset = byte_idx;
        occurrences.push(char_offset..char_offset + needle_chars);
    }

    occurrences
}

/// Scans backward and forward from `char_idx` to find word boundaries (alphanumeric characters + underscores).
pub fn select_word_at(buffer: &TextBuffer, char_idx: usize) -> Selection {
    let total = buffer.len_chars();
    if total == 0 {
        return Selection::point(0);
    }

    let mut idx = char_idx.min(total);
    // If idx is at EOF or on a non-word char, but previous char is a word char,
    // cursor is at the boundary following a word (e.g. `foo| ` or `foo|EOF`).
    if (idx == total || !buffer.char_at(idx).map_or(false, is_word_char))
        && idx > 0
        && buffer.char_at(idx - 1).map_or(false, is_word_char)
    {
        idx -= 1;
    }

    if !buffer.char_at(idx).map_or(false, is_word_char) {
        return Selection::point(char_idx.min(total));
    }

    // Scan backwards
    let mut start = idx;
    while start > 0 {
        if let Some(c) = buffer.char_at(start - 1) {
            if is_word_char(c) {
                start -= 1;
                continue;
            }
        }
        break;
    }

    // Scan forwards
    let mut end = idx;
    while end < total {
        if let Some(c) = buffer.char_at(end) {
            if is_word_char(c) {
                end += 1;
                continue;
            }
        }
        break;
    }

    Selection::range(start, end)
}

/// Sublime Text Cmd+D / Ctrl+D behavior:
/// - If the primary selection is empty (a point cursor), expand it to the word under cursor using `select_word_at`.
/// - If the primary selection is already a range, search forward in `buffer` for the next occurrence of that text.
///   If found, add a new `Selection::range` to `selections`. If reaching EOF, wrap around to beginning.
pub fn select_next_occurrence(buffer: &TextBuffer, selections: &mut SelectionSet) {
    let prim = selections.primary();
    if prim.is_empty() {
        let word_sel = select_word_at(buffer, prim.head);
        if !word_sel.is_empty() {
            let p_idx = selections.primary_idx;
            if p_idx < selections.selections.len() {
                selections.selections[p_idx] = word_sel;
                selections.normalize();
            } else {
                selections.set_primary(word_sel);
            }
        }
        return;
    }

    let search_text = buffer.slice_text(prim.min_char()..prim.max_char());
    if search_text.is_empty() {
        return;
    }

    let all_matches = find_all_occurrences(buffer, &search_text);
    if all_matches.is_empty() {
        return;
    }

    let is_already_selected = |m: &Range<usize>| {
        selections
            .selections()
            .iter()
            .any(|s| s.min_char() == m.start && s.max_char() == m.end)
    };

    let search_from = prim.max_char();

    // Search forward from search_from, wrap around if needed
    let next_match = all_matches
        .iter()
        .find(|m| m.start >= search_from && !is_already_selected(m))
        .or_else(|| {
            all_matches
                .iter()
                .find(|m| !is_already_selected(m))
        });

    if let Some(m) = next_match {
        let new_sel = Selection::range(m.start, m.end);
        selections.add(new_sel);
        // Focus the newly added selection as the primary selection
        if let Some(pos) = selections
            .selections()
            .iter()
            .position(|s| s.min_char() == m.start && s.max_char() == m.end)
        {
            selections.set_primary_index(pos);
        }
    }
}

/// Sublime Text Cmd+Ctrl+G / Alt+F3 behavior:
/// - Finds all occurrences of the word/selected text across the entire buffer and adds a selection cursor for each.
pub fn select_all_occurrences(buffer: &TextBuffer, selections: &mut SelectionSet) {
    let prim = selections.primary();
    let target_text = if prim.is_empty() {
        let word_sel = select_word_at(buffer, prim.head);
        if word_sel.is_empty() {
            return;
        }
        buffer.slice_text(word_sel.min_char()..word_sel.max_char())
    } else {
        buffer.slice_text(prim.min_char()..prim.max_char())
    };

    if target_text.is_empty() {
        return;
    }

    let occurrences = find_all_occurrences(buffer, &target_text);
    if occurrences.is_empty() {
        return;
    }

    let orig_head = prim.head;
    let new_selections: Vec<Selection> = occurrences
        .into_iter()
        .map(|r| Selection::range(r.start, r.end))
        .collect();

    selections.set_all(new_selections);

    // Keep primary selection close to the original cursor position if possible
    if let Some(idx) = selections
        .selections()
        .iter()
        .position(|s| s.min_char() <= orig_head && orig_head <= s.max_char())
    {
        selections.set_primary_index(idx);
    }
}

/// Sublime Text Cmd+Shift+L behavior:
/// - For any selection spanning multiple lines, splits it into one selection per line
///   spanning from the line start (or selection start) to line end (or selection end).
pub fn split_selection_into_lines(buffer: &TextBuffer, selections: &mut SelectionSet) {
    if buffer.is_empty() {
        return;
    }

    let mut result_selections = Vec::new();

    for sel in selections.selections() {
        let start_pos = buffer.char_to_pos(sel.min_char());
        let end_pos = buffer.char_to_pos(sel.max_char());

        if start_pos.line == end_pos.line {
            result_selections.push(*sel);
            continue;
        }

        let total_lines = buffer.len_lines();
        for line_idx in start_pos.line..=end_pos.line.min(total_lines.saturating_sub(1)) {
            let line_start_char = buffer.pos_to_char(CursorPosition::new(line_idx, 0));

            let line_slice = buffer.line(line_idx);
            let (line_len, newline_len) = if let Some(slice) = line_slice {
                let len = slice.len_chars();
                let nl = if len >= 2 && slice.char(len - 2) == '\r' && slice.char(len - 1) == '\n' {
                    2
                } else if len >= 1 && (slice.char(len - 1) == '\n' || slice.char(len - 1) == '\r') {
                    1
                } else {
                    0
                };
                (len, nl)
            } else {
                (0, 0)
            };
            let line_content_end = line_start_char + line_len.saturating_sub(newline_len);

            // If the selection ends at column 0 of the final line, skip that line
            if line_idx == end_pos.line && sel.max_char() == line_start_char {
                continue;
            }

            let line_sel_start = if line_idx == start_pos.line {
                sel.min_char()
            } else {
                line_start_char
            };

            let line_sel_end = if line_idx == end_pos.line {
                sel.max_char().min(line_content_end)
            } else {
                line_content_end
            };

            let actual_start = line_sel_start.min(line_content_end);
            let actual_end = line_sel_end.max(actual_start);

            if sel.head >= sel.anchor {
                result_selections.push(Selection::range(actual_start, actual_end));
            } else {
                result_selections.push(Selection::range(actual_end, actual_start));
            }
        }
    }

    if !result_selections.is_empty() {
        selections.set_all(result_selections);
    }
}

/// Adds a cursor one line up at the same column position.
pub fn add_cursor_above(buffer: &TextBuffer, selections: &mut SelectionSet) {
    if buffer.is_empty() {
        return;
    }
    let mut new_cursors = Vec::new();
    for sel in selections.selections() {
        let pos = buffer.char_to_pos(sel.head);
        if pos.line > 0 {
            let target_line = pos.line - 1;
            let new_char = buffer.pos_to_char(CursorPosition::new(target_line, pos.column));
            new_cursors.push(Selection::point(new_char));
        }
    }
    for cursor in new_cursors {
        selections.add(cursor);
    }
}

/// Adds a cursor one line down at the same column position.
pub fn add_cursor_below(buffer: &TextBuffer, selections: &mut SelectionSet) {
    if buffer.is_empty() {
        return;
    }
    let total_lines = buffer.len_lines();
    let mut new_cursors = Vec::new();
    for sel in selections.selections() {
        let pos = buffer.char_to_pos(sel.head);
        if pos.line + 1 < total_lines {
            let target_line = pos.line + 1;
            let new_char = buffer.pos_to_char(CursorPosition::new(target_line, pos.column));
            new_cursors.push(Selection::point(new_char));
        }
    }
    for cursor in new_cursors {
        selections.add(cursor);
    }
}
