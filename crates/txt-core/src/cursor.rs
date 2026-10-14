use serde::{Deserialize, Serialize};

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
}
