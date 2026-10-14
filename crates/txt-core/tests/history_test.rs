use txt_core::history::{Change, HistoryManager};

#[test]
fn test_history_grouping() {
    let mut history = HistoryManager::new();

    history.begin_group();
    history.record_change(Change::Insert {
        position: 0,
        text: "a".to_string(),
    });
    history.record_change(Change::Insert {
        position: 1,
        text: "b".to_string(),
    });
    history.end_group();

    assert!(history.can_undo());
    let undone = history.undo().unwrap();
    assert_eq!(undone.len(), 2);
    assert!(!history.can_undo());
    assert!(history.can_redo());

    let redone = history.redo().unwrap();
    assert_eq!(redone.len(), 2);
}
