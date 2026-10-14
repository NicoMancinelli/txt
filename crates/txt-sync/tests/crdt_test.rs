use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use txt_sync::adapter::{SyncAdapter, SyncStatus};
use txt_sync::crdt::{CrdtBufferState, CrdtPatch, VectorClock};
use txt_sync::engine::SyncEngine;
use txt_sync::memory_adapter::MemorySyncAdapter;

#[test]
fn test_vector_clock_basic() {
    let mut v1 = VectorClock::new();
    assert_eq!(v1.get("peer-a"), 0);
    assert!(v1.is_empty());

    assert_eq!(v1.increment("peer-a"), 1);
    assert_eq!(v1.increment("peer-a"), 2);
    assert_eq!(v1.get("peer-a"), 2);
    assert_eq!(v1.len(), 1);

    v1.set("peer-b", 5);
    assert_eq!(v1.get("peer-b"), 5);
    assert_eq!(v1.len(), 2);
}

#[test]
fn test_vector_clock_merge_and_causality() {
    let mut v1 = VectorClock::new();
    v1.set("peer-a", 2);
    v1.set("peer-b", 1);

    let mut v2 = VectorClock::new();
    v2.set("peer-a", 1);
    v2.set("peer-b", 3);

    // v1 and v2 are concurrent
    assert!(v1.is_concurrent(&v2));
    assert!(v2.is_concurrent(&v1));
    assert!(!v1.dominates(&v2));
    assert!(!v2.dominates(&v1));

    // After v1 merges v2
    let mut v3 = v1.clone();
    v3.merge(&v2);
    assert_eq!(v3.get("peer-a"), 2);
    assert_eq!(v3.get("peer-b"), 3);

    // v3 dominates both v1 and v2
    assert!(v3.dominates(&v1));
    assert!(v3.dominates(&v2));
    assert!(!v1.dominates(&v3));
    assert!(!v3.is_concurrent(&v1));
}

#[test]
fn test_crdt_local_inserts_and_deletes() {
    let doc_id = Uuid::new_v4();
    let mut buffer = CrdtBufferState::new(doc_id, "peer-1");

    // Insert "Hello"
    buffer.local_insert_str(0, "Hello");
    assert_eq!(buffer.to_string(), "Hello");
    assert_eq!(buffer.len_chars(), 5);

    // Insert " World" at position 5
    buffer.local_insert_str(5, " World");
    assert_eq!(buffer.to_string(), "Hello World");

    // Delete " World" (positions 5..11)
    for _ in 5..11 {
        buffer.local_delete(5);
    }
    assert_eq!(buffer.to_string(), "Hello");
    assert_eq!(buffer.len_chars(), 5);

    // Insert at beginning
    buffer.local_insert_str(0, ">> ");
    assert_eq!(buffer.to_string(), ">> Hello");
}

#[test]
fn test_crdt_two_peers_concurrent_divergence_convergence() {
    let doc_id = Uuid::new_v4();
    let mut peer_a = CrdtBufferState::new(doc_id, "peer-a");
    let mut peer_b = CrdtBufferState::new(doc_id, "peer-b");

    // Both start with "Hello"
    let _ = peer_a.local_insert_str(0, "Hello");
    let patch_init = peer_a.generate_patch();
    peer_b.apply_patch(&patch_init);
    assert_eq!(peer_b.to_string(), "Hello");

    // Peer A concurrently appends " World"
    peer_a.local_insert_str(5, " World");
    assert_eq!(peer_a.to_string(), "Hello World");

    // Peer B concurrently appends " Rust"
    peer_b.local_insert_str(5, " Rust");
    assert_eq!(peer_b.to_string(), "Hello Rust");

    // Exchange patches
    let patch_a = peer_a.generate_patch();
    let patch_b = peer_b.generate_patch();

    peer_a.apply_patch(&patch_b);
    peer_b.apply_patch(&patch_a);

    // Both must converge to the exact same text
    assert_eq!(peer_a.to_string(), peer_b.to_string());
    assert!(
        peer_a.to_string() == "Hello Rust World" || peer_a.to_string() == "Hello World Rust"
    );
}

#[test]
fn test_crdt_concurrent_insertion_at_same_index() {
    let doc_id = Uuid::new_v4();
    let mut peer_a = CrdtBufferState::new(doc_id, "peer-a");
    let mut peer_b = CrdtBufferState::new(doc_id, "peer-b");

    // Both insert at position 0 concurrently on empty document
    let op_a = peer_a.local_insert(0, 'X');
    let op_b = peer_b.local_insert(0, 'Y');

    // Cross apply
    peer_a.apply_remote_op(op_b);
    peer_b.apply_remote_op(op_a);

    // Conflict resolution must produce deterministic identical sequence
    assert_eq!(peer_a.to_string(), peer_b.to_string());
    assert_eq!(peer_a.len_chars(), 2);
}

#[test]
fn test_crdt_out_of_order_and_duplicate_delivery() {
    let doc_id = Uuid::new_v4();
    let mut peer_a = CrdtBufferState::new(doc_id, "peer-a");
    let mut peer_b = CrdtBufferState::new(doc_id, "peer-b");

    // Peer A inserts "A", "B", "C"
    let op1 = peer_a.local_insert(0, 'A');
    let op2 = peer_a.local_insert(1, 'B');
    let op3 = peer_a.local_insert(2, 'C');

    // Deliver to Peer B in reverse order: C, B, A
    let app3 = peer_b.apply_remote_op(op3.clone());
    assert!(!app3); // Buffered as pending due to missing origin dependency
    assert_eq!(peer_b.pending_ops_count(), 1);

    let app2 = peer_b.apply_remote_op(op2.clone());
    assert!(!app2); // Also buffered
    assert_eq!(peer_b.pending_ops_count(), 2);

    let app1 = peer_b.apply_remote_op(op1.clone());
    assert!(app1); // A applies, which triggers B and C resolution
    assert_eq!(peer_b.pending_ops_count(), 0);
    assert_eq!(peer_b.to_string(), "ABC");

    // Test duplicate delivery idempotency
    assert!(!peer_b.apply_remote_op(op1));
    assert!(!peer_b.apply_remote_op(op2));
    assert!(!peer_b.apply_remote_op(op3));
    assert_eq!(peer_b.to_string(), "ABC");
}

#[test]
fn test_crdt_tombstone_delete_before_insert() {
    let doc_id = Uuid::new_v4();
    let mut peer_a = CrdtBufferState::new(doc_id, "peer-a");
    let mut peer_b = CrdtBufferState::new(doc_id, "peer-b");

    let op_ins = peer_a.local_insert(0, 'Z');
    let op_del = peer_a.local_delete(0).unwrap();

    // Deliver delete first to Peer B
    peer_b.apply_remote_op(op_del);
    assert_eq!(peer_b.to_string(), "");

    // Deliver insert afterwards
    peer_b.apply_remote_op(op_ins);

    // It should be tombstoned and not visible
    assert_eq!(peer_b.to_string(), "");
    assert_eq!(peer_b.len_chars(), 0);
}

#[test]
fn test_sync_engine_crdt_concurrency_locking() {
    let engine = Arc::new(SyncEngine::new());
    let doc_id = Uuid::new_v4();

    // Initialize document CRDT
    engine.init_crdt_buffer(doc_id, "main-peer", "Initial");

    let num_threads = 8;
    let mut handles = Vec::new();

    for t in 0..num_threads {
        let eng = Arc::clone(&engine);
        let peer_name = format!("thread-peer-{}", t);
        handles.push(thread::spawn(move || {
            for i in 0..10 {
                let text = format!("{}-{}", t, i);
                let _ = eng.local_crdt_insert(doc_id, &peer_name, 0, &text);
                thread::sleep(Duration::from_millis(1));
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // Verify all edits completed under ConcurrencyLockManager without panics
    let final_text = engine.get_crdt_text(&doc_id).unwrap();
    assert!(final_text.contains("Initial"));
    assert!(final_text.len() > 10);
}

#[test]
fn test_sync_engine_crdt_patch_adapters_propagation() {
    let engine = SyncEngine::new();
    let memory_adapter = Arc::new(MemorySyncAdapter::new());
    engine.add_adapter(memory_adapter.clone());

    let doc_id = Uuid::new_v4();
    let mut peer = CrdtBufferState::new(doc_id, "author-1");
    peer.local_insert_str(0, "Distributed CRDT Sync");
    let patch = peer.generate_patch();

    let statuses = engine.sync_crdt_patch(&patch, 1).unwrap();
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0], SyncStatus::Synced);

    // Pull from adapter and verify
    let pulled = memory_adapter.pull_patches(&doc_id, 0).unwrap();
    assert_eq!(pulled.len(), 1);
    assert_eq!(pulled[0].author_id, "author-1");
    assert_eq!(pulled[0].changes.len(), "Distributed CRDT Sync".len());
}

#[test]
fn test_multi_peer_concurrent_stress() {
    let doc_id = Uuid::new_v4();
    let num_peers = 4;
    let peers: Vec<Arc<parking_lot::Mutex<CrdtBufferState>>> = (0..num_peers)
        .map(|i| {
            Arc::new(parking_lot::Mutex::new(CrdtBufferState::new(
                doc_id,
                format!("peer-{}", i),
            )))
        })
        .collect();

    // Base document
    {
        let mut p0 = peers[0].lock();
        p0.local_insert_str(0, "BASE_ROOT");
        let base_patch = p0.generate_patch();
        for p in &peers[1..] {
            p.lock().apply_patch(&base_patch);
        }
    }

    // Concurrent edits across threads
    let mut handles = Vec::new();
    for i in 0..num_peers {
        let peer_arc = Arc::clone(&peers[i]);
        handles.push(thread::spawn(move || {
            let mut local_patches = Vec::new();
            for step in 0..15 {
                let mut p = peer_arc.lock();
                let len = p.len_chars();
                let pos = if len == 0 { 0 } else { step % len };
                p.local_insert_str(pos, &format!("[P{}:{}]", i, step));
                if step % 3 == 0 && p.len_chars() > 5 {
                    p.local_delete(0);
                }
                local_patches.push(p.generate_patch());
            }
            local_patches
        }));
    }

    let all_patch_batches: Vec<Vec<CrdtPatch>> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // Cross-distribute all patches to all peers
    for patch_batch in all_patch_batches {
        for patch in patch_batch {
            for peer in &peers {
                peer.lock().apply_patch(&patch);
            }
        }
    }

    // Verify EVENTUAL CONSISTENCY: Every peer has the exact identical string!
    let reference_text = peers[0].lock().to_string();
    assert!(!reference_text.is_empty());

    for (idx, peer) in peers.iter().enumerate() {
        let peer_text = peer.lock().to_string();
        assert_eq!(
            reference_text, peer_text,
            "Peer {} did not converge to reference text!",
            idx
        );
    }
}
