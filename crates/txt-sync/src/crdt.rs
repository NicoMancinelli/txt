use std::collections::{BTreeMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use txt_core::history::Change;

/// Vector Clock tracking logical causality across distributed peers.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VectorClock {
    pub clock: BTreeMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clock: BTreeMap::new(),
        }
    }

    /// Gets the current counter for a peer, or 0 if not present.
    pub fn get(&self, peer_id: &str) -> u64 {
        self.clock.get(peer_id).copied().unwrap_or(0)
    }

    /// Sets the counter for a peer.
    pub fn set(&mut self, peer_id: impl Into<String>, counter: u64) {
        self.clock.insert(peer_id.into(), counter);
    }

    /// Increments the clock for the given peer and returns the updated counter.
    pub fn increment(&mut self, peer_id: &str) -> u64 {
        let entry = self.clock.entry(peer_id.to_string()).or_insert(0);
        *entry += 1;
        *entry
    }

    /// Merges another vector clock by taking component-wise maximums.
    pub fn merge(&mut self, other: &VectorClock) {
        for (peer, &val) in &other.clock {
            let entry = self.clock.entry(peer.clone()).or_insert(0);
            *entry = (*entry).max(val);
        }
    }

    /// Returns true if this clock strictly dominates another (greater or equal everywhere, strictly greater in at least one).
    pub fn dominates(&self, other: &VectorClock) -> bool {
        let mut strictly_greater = false;

        for (peer, &other_val) in &other.clock {
            let self_val = self.get(peer);
            if self_val < other_val {
                return false;
            }
            if self_val > other_val {
                strictly_greater = true;
            }
        }

        for (peer, &self_val) in &self.clock {
            if self_val > 0 && other.get(peer) == 0 {
                strictly_greater = true;
            }
        }

        strictly_greater
    }

    /// Returns true if two vector clocks are concurrent (neither dominates the other and they are not equal).
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        if self == other {
            return false;
        }
        !self.dominates(other) && !other.dominates(self)
    }

    pub fn is_empty(&self) -> bool {
        self.clock.is_empty()
    }

    pub fn len(&self) -> usize {
        self.clock.len()
    }
}

/// Character-level CRDT Operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrdtOp {
    Insert {
        id: Uuid,
        peer_id: String,
        lamport: u64,
        position: usize,
        character: char,
        origin_left: Option<Uuid>,
    },
    Delete {
        id: Uuid,
        peer_id: String,
        lamport: u64,
        position: usize,
        target_id: Option<Uuid>,
    },
}

impl CrdtOp {
    pub fn new_insert(
        id: Uuid,
        peer_id: impl Into<String>,
        lamport: u64,
        position: usize,
        character: char,
        origin_left: Option<Uuid>,
    ) -> Self {
        Self::Insert {
            id,
            peer_id: peer_id.into(),
            lamport,
            position,
            character,
            origin_left,
        }
    }

    pub fn new_delete(
        id: Uuid,
        peer_id: impl Into<String>,
        lamport: u64,
        position: usize,
        target_id: Option<Uuid>,
    ) -> Self {
        Self::Delete {
            id,
            peer_id: peer_id.into(),
            lamport,
            position,
            target_id,
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            CrdtOp::Insert { id, .. } => *id,
            CrdtOp::Delete { id, .. } => *id,
        }
    }

    pub fn peer_id(&self) -> &str {
        match self {
            CrdtOp::Insert { peer_id, .. } => peer_id.as_str(),
            CrdtOp::Delete { peer_id, .. } => peer_id.as_str(),
        }
    }

    pub fn lamport(&self) -> u64 {
        match self {
            CrdtOp::Insert { lamport, .. } => *lamport,
            CrdtOp::Delete { lamport, .. } => *lamport,
        }
    }

    pub fn position(&self) -> usize {
        match self {
            CrdtOp::Insert { position, .. } => *position,
            CrdtOp::Delete { position, .. } => *position,
        }
    }

    /// Converts this CRDT operation to an editor Change.
    pub fn to_change(&self) -> Change {
        match self {
            CrdtOp::Insert {
                position,
                character,
                ..
            } => Change::Insert {
                position: *position,
                text: character.to_string(),
            },
            CrdtOp::Delete { position, .. } => Change::Delete {
                position: *position,
                text: String::new(),
            },
        }
    }
}

/// A character node in the CRDT sequence (RGA structure).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrdtNode {
    pub id: Uuid,
    pub peer_id: String,
    pub lamport: u64,
    pub character: char,
    pub origin_left: Option<Uuid>,
    pub is_deleted: bool,
}

/// Serialized collection of CRDT operations forming an atomic sync patch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrdtPatch {
    pub patch_id: Uuid,
    pub document_id: Uuid,
    pub peer_id: String,
    pub clock: VectorClock,
    pub ops: Vec<CrdtOp>,
    pub timestamp: DateTime<Utc>,
}

impl CrdtPatch {
    pub fn new(
        document_id: Uuid,
        peer_id: impl Into<String>,
        clock: VectorClock,
        ops: Vec<CrdtOp>,
    ) -> Self {
        Self {
            patch_id: Uuid::new_v4(),
            document_id,
            peer_id: peer_id.into(),
            clock,
            ops,
            timestamp: Utc::now(),
        }
    }

    /// Converts this patch into an adapter `SyncPatch`.
    pub fn to_sync_patch(&self, revision: u64) -> crate::adapter::SyncPatch {
        let changes = self.ops.iter().map(|op| op.to_change()).collect();
        crate::adapter::SyncPatch {
            patch_id: self.patch_id,
            document_id: self.document_id,
            revision,
            author_id: self.peer_id.clone(),
            timestamp: self.timestamp,
            changes,
            base_checksum: 0,
        }
    }
}

/// Replicated sequence buffer state using RGA (Replicated Growable Array).
/// Ensures deterministic conflict resolution across concurrent edits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtBufferState {
    pub document_id: Uuid,
    pub peer_id: String,
    pub clock: VectorClock,
    pub current_lamport: u64,
    nodes: Vec<CrdtNode>,
    applied_op_ids: HashSet<Uuid>,
    pending_ops: Vec<CrdtOp>,
    tombstones: HashSet<Uuid>,
    unflushed_ops: Vec<CrdtOp>,
}

impl CrdtBufferState {
    pub fn new(document_id: Uuid, peer_id: impl Into<String>) -> Self {
        let peer = peer_id.into();
        let mut clock = VectorClock::new();
        clock.set(&peer, 0);

        Self {
            document_id,
            peer_id: peer,
            clock,
            current_lamport: 0,
            nodes: Vec::new(),
            applied_op_ids: HashSet::new(),
            pending_ops: Vec::new(),
            tombstones: HashSet::new(),
            unflushed_ops: Vec::new(),
        }
    }

    /// Initializes state with existing text content.
    pub fn from_str(document_id: Uuid, peer_id: impl Into<String>, initial_text: &str) -> Self {
        let mut state = Self::new(document_id, peer_id);
        state.local_insert_str(0, initial_text);
        state
    }

    /// Number of visible (non-deleted) characters.
    pub fn len_chars(&self) -> usize {
        self.nodes.iter().filter(|n| !n.is_deleted).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len_chars() == 0
    }

    /// Reconstructs the full visible text sequence.
    pub fn to_string(&self) -> String {
        self.nodes
            .iter()
            .filter(|n| !n.is_deleted)
            .map(|n| n.character)
            .collect()
    }

    /// Gets visible character at 0-indexed position.
    pub fn char_at(&self, position: usize) -> Option<char> {
        let mut visible_idx = 0;
        for node in &self.nodes {
            if !node.is_deleted {
                if visible_idx == position {
                    return Some(node.character);
                }
                visible_idx += 1;
            }
        }
        None
    }

    /// Inserts a character locally at the specified visible position.
    pub fn local_insert(&mut self, position: usize, character: char) -> CrdtOp {
        self.current_lamport += 1;
        let lamport = self.current_lamport;
        self.clock.increment(&self.peer_id);
        let op_id = Uuid::new_v4();

        // Determine origin_left: visible character immediately before position
        let origin_left = if position == 0 {
            None
        } else {
            let mut visible_count = 0;
            let mut last_id = None;
            for node in &self.nodes {
                if !node.is_deleted {
                    if visible_count == position {
                        break;
                    }
                    last_id = Some(node.id);
                    visible_count += 1;
                }
            }
            last_id
        };

        let node = CrdtNode {
            id: op_id,
            peer_id: self.peer_id.clone(),
            lamport,
            character,
            origin_left,
            is_deleted: false,
        };

        self.insert_node(node);
        self.applied_op_ids.insert(op_id);

        let op = CrdtOp::Insert {
            id: op_id,
            peer_id: self.peer_id.clone(),
            lamport,
            position,
            character,
            origin_left,
        };

        self.unflushed_ops.push(op.clone());
        op
    }

    /// Inserts a string locally at the specified position.
    pub fn local_insert_str(&mut self, mut position: usize, text: &str) -> Vec<CrdtOp> {
        let mut ops = Vec::new();
        for ch in text.chars() {
            let op = self.local_insert(position, ch);
            ops.push(op);
            position += 1;
        }
        ops
    }

    /// Deletes a character locally at the specified visible position.
    pub fn local_delete(&mut self, position: usize) -> Option<CrdtOp> {
        let mut visible_idx = 0;
        let mut target_id = None;

        for node in &mut self.nodes {
            if !node.is_deleted {
                if visible_idx == position {
                    node.is_deleted = true;
                    target_id = Some(node.id);
                    break;
                }
                visible_idx += 1;
            }
        }

        let target_id = target_id?;
        self.current_lamport += 1;
        let lamport = self.current_lamport;
        self.clock.increment(&self.peer_id);
        let op_id = Uuid::new_v4();
        self.applied_op_ids.insert(op_id);

        let op = CrdtOp::Delete {
            id: op_id,
            peer_id: self.peer_id.clone(),
            lamport,
            position,
            target_id: Some(target_id),
        };

        self.unflushed_ops.push(op.clone());
        Some(op)
    }

    /// Inserts an RGA node using deterministic tie-breaking (Lamport timestamp, peer ID, op ID).
    fn insert_node(&mut self, node: CrdtNode) -> bool {
        let mut idx = 0;
        let mut skipped_ids = HashSet::new();

        if let Some(orig_id) = node.origin_left {
            if let Some(pos) = self.nodes.iter().position(|n| n.id == orig_id) {
                idx = pos + 1;
            } else {
                // Dependency missing: cannot insert yet
                return false;
            }
        }

        while idx < self.nodes.len() {
            let curr = &self.nodes[idx];
            if curr.id == node.id {
                return true; // Already applied
            }

            if curr.origin_left == node.origin_left {
                // Siblings with same origin: compare (lamport, peer_id, id)
                let curr_key = (curr.lamport, curr.peer_id.as_str(), curr.id);
                let new_key = (node.lamport, node.peer_id.as_str(), node.id);
                if curr_key > new_key {
                    skipped_ids.insert(curr.id);
                    idx += 1;
                } else {
                    break;
                }
            } else if curr
                .origin_left
                .as_ref()
                .map_or(false, |orig| skipped_ids.contains(orig))
            {
                // Descendant of a skipped sibling
                skipped_ids.insert(curr.id);
                idx += 1;
            } else {
                // Sibling subtree boundary reached
                break;
            }
        }

        self.nodes.insert(idx, node);
        true
    }

    /// Deletes an RGA node by marking it as a tombstone.
    fn delete_node(&mut self, target_id: Option<Uuid>, position: usize) -> bool {
        if let Some(id) = target_id {
            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
                node.is_deleted = true;
                true
            } else {
                // Target node not yet received: buffer as tombstone
                self.tombstones.insert(id);
                true
            }
        } else {
            // Fallback by position
            let mut visible_idx = 0;
            for node in self.nodes.iter_mut() {
                if !node.is_deleted {
                    if visible_idx == position {
                        node.is_deleted = true;
                        return true;
                    }
                    visible_idx += 1;
                }
            }
            false
        }
    }

    /// Applies a remote operation and resolves dependencies.
    pub fn apply_remote_op(&mut self, op: CrdtOp) -> bool {
        let op_id = op.id();
        if self.applied_op_ids.contains(&op_id) {
            return false;
        }

        // Update Lamport logical clock
        self.current_lamport = self.current_lamport.max(op.lamport());
        let current_peer_val = self.clock.get(op.peer_id());
        self.clock
            .set(op.peer_id(), current_peer_val.max(op.lamport()));

        let applied = match op.clone() {
            CrdtOp::Insert {
                id,
                peer_id,
                lamport,
                position: _,
                character,
                origin_left,
            } => {
                let is_deleted = self.tombstones.contains(&id);
                let node = CrdtNode {
                    id,
                    peer_id,
                    lamport,
                    character,
                    origin_left,
                    is_deleted,
                };
                self.insert_node(node)
            }
            CrdtOp::Delete {
                id: _,
                peer_id: _,
                lamport: _,
                position,
                target_id,
            } => self.delete_node(target_id, position),
        };

        if applied {
            self.applied_op_ids.insert(op_id);
            self.process_pending_ops();
            true
        } else {
            self.pending_ops.push(op);
            false
        }
    }

    /// Processes any buffered operations whose causal dependencies have arrived.
    fn process_pending_ops(&mut self) {
        let mut progress = true;
        while progress {
            progress = false;
            let mut remaining = Vec::new();
            let pending = std::mem::take(&mut self.pending_ops);

            for op in pending {
                let op_id = op.id();
                if self.applied_op_ids.contains(&op_id) {
                    continue;
                }

                let applied = match op.clone() {
                    CrdtOp::Insert {
                        id,
                        peer_id,
                        lamport,
                        position: _,
                        character,
                        origin_left,
                    } => {
                        let is_deleted = self.tombstones.contains(&id);
                        let node = CrdtNode {
                            id,
                            peer_id,
                            lamport,
                            character,
                            origin_left,
                            is_deleted,
                        };
                        self.insert_node(node)
                    }
                    CrdtOp::Delete {
                        id: _,
                        peer_id: _,
                        lamport: _,
                        position,
                        target_id,
                    } => self.delete_node(target_id, position),
                };

                if applied {
                    self.applied_op_ids.insert(op_id);
                    progress = true;
                } else {
                    remaining.push(op);
                }
            }

            self.pending_ops = remaining;
        }
    }

    /// Merges remote vector clock.
    pub fn merge_clock(&mut self, other: &VectorClock) {
        self.clock.merge(other);
    }

    /// Generates a patch containing all pending local operations and drains them.
    pub fn generate_patch(&mut self) -> CrdtPatch {
        let ops = std::mem::take(&mut self.unflushed_ops);
        CrdtPatch::new(self.document_id, &self.peer_id, self.clock.clone(), ops)
    }

    /// Applies a remote patch to this buffer state.
    pub fn apply_patch(&mut self, patch: &CrdtPatch) -> usize {
        self.merge_clock(&patch.clock);
        let mut applied_count = 0;
        for op in &patch.ops {
            if self.apply_remote_op(op.clone()) {
                applied_count += 1;
            }
        }
        applied_count
    }

    pub fn pending_ops_count(&self) -> usize {
        self.pending_ops.len()
    }

    pub fn unflushed_ops_count(&self) -> usize {
        self.unflushed_ops.len()
    }

    pub fn applied_ops_count(&self) -> usize {
        self.applied_op_ids.len()
    }

    pub fn nodes(&self) -> &[CrdtNode] {
        &self.nodes
    }
}
