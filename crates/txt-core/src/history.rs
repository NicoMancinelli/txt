use serde::{Deserialize, Serialize};

/// Atomic change unit within the text editor buffer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Insert { position: usize, text: String },
    Delete { position: usize, text: String },
}

/// Transaction containing one or more atomic changes that are undone/redone together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub changes: Vec<Change>,
}

/// Manages undo and redo history with transaction batching.
#[derive(Debug, Clone, Default)]
pub struct HistoryManager {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
    current_group: Option<Vec<Change>>,
    group_depth: usize,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_group: None,
            group_depth: 0,
        }
    }

    /// Begins a logical transaction group (e.g. multi-cursor edit or batch replacement).
    pub fn begin_group(&mut self) {
        if self.group_depth == 0 {
            self.current_group = Some(Vec::new());
        }
        self.group_depth += 1;
    }

    /// Ends the current transaction group and commits it to the undo stack.
    pub fn end_group(&mut self) {
        if self.group_depth > 0 {
            self.group_depth -= 1;
            if self.group_depth == 0 {
                if let Some(changes) = self.current_group.take() {
                    if !changes.is_empty() {
                        self.undo_stack.push(Transaction { changes });
                        self.redo_stack.clear();
                    }
                }
            }
        }
    }

    /// Records an atomic change, either inside an active transaction group or as a standalone edit.
    pub fn record_change(&mut self, change: Change) {
        if let Some(group) = &mut self.current_group {
            group.push(change);
        } else {
            self.undo_stack.push(Transaction {
                changes: vec![change],
            });
            self.redo_stack.clear();
        }
    }

    /// Undoes the last transaction and pushes it onto the redo stack.
    pub fn undo(&mut self) -> Option<Vec<Change>> {
        if let Some(txn) = self.undo_stack.pop() {
            let changes = txn.changes.clone();
            self.redo_stack.push(txn);
            Some(changes)
        } else {
            None
        }
    }

    /// Redoes the previously undone transaction and returns its changes.
    pub fn redo(&mut self) -> Option<Vec<Change>> {
        if let Some(txn) = self.redo_stack.pop() {
            let changes = txn.changes.clone();
            self.undo_stack.push(txn);
            Some(changes)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_group = None;
        self.group_depth = 0;
    }
}
