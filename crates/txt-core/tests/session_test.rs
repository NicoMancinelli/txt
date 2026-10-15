use std::path::{Path, PathBuf};
use tempfile::tempdir;
use txt_core::session::{SessionManager, SessionState};

#[test]
fn test_session_state_default() {
    let state = SessionState::default();
    assert!(state.open_files.is_empty());
    assert_eq!(state.active_file, None);
    assert!(state.sidebar_visible);
    assert_eq!(state.font_size, 14.0);
    assert_eq!(state.theme, "Mariana");
    assert!(state.word_wrap);
}

#[test]
fn test_session_state_mutations() {
    let mut state = SessionState::new();

    let f1 = PathBuf::from("src/main.rs");
    let f2 = PathBuf::from("src/buffer.rs");
    let f3 = PathBuf::from("README.md");

    state.add_open_file(f1.clone());
    state.add_open_file(f2.clone());
    // Adding duplicate does not duplicate
    state.add_open_file(f1.clone());
    assert_eq!(state.open_files.len(), 2);
    assert!(state.is_open(&f1));
    assert!(state.is_open(&f2));
    assert!(!state.is_open(&f3));

    state.set_active_file(Some(f1.clone()));
    assert_eq!(state.active_file, Some(f1.clone()));

    // Setting active file for an unopened file adds it
    state.set_active_file(Some(f3.clone()));
    assert_eq!(state.open_files.len(), 3);
    assert!(state.is_open(&f3));
    assert_eq!(state.active_file, Some(f3.clone()));

    // Removing active file updates active_file to another open file
    state.remove_open_file(&f3);
    assert_eq!(state.open_files.len(), 2);
    assert!(!state.is_open(&f3));
    assert_eq!(state.active_file, Some(f1.clone()));

    // Removing all files
    state.remove_open_file(&f1);
    state.remove_open_file(&f2);
    assert!(state.open_files.is_empty());
    assert_eq!(state.active_file, None);
}

#[test]
fn test_session_state_builders() {
    let state = SessionState::new()
        .with_open_files(vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")])
        .with_active_file(Some(PathBuf::from("a.txt")))
        .with_sidebar_visible(false)
        .with_font_size(18.5)
        .with_theme("Monokai")
        .with_word_wrap(false);

    assert_eq!(state.open_files.len(), 2);
    assert_eq!(state.active_file, Some(PathBuf::from("a.txt")));
    assert!(!state.sidebar_visible);
    assert_eq!(state.font_size, 18.5);
    assert_eq!(state.theme, "Monokai");
    assert!(!state.word_wrap);
}

#[test]
fn test_session_manager_save_and_load() {
    let dir = tempdir().expect("failed to create temp dir");
    let session_file = dir.path().join("subfolder").join("session.json");

    let mut state = SessionState::new();
    state.add_open_file(PathBuf::from("crates/txt-core/src/lib.rs"));
    state.add_open_file(PathBuf::from("crates/txt-gui/src/main.rs"));
    state.set_active_file(Some(PathBuf::from("crates/txt-core/src/lib.rs")));
    state.font_size = 16.0;
    state.theme = "Monokai".to_string();
    state.sidebar_visible = true;
    state.word_wrap = false;

    assert!(!SessionManager::exists(&session_file));

    // Save session (should create parent directories)
    SessionManager::save_session(&session_file, &state).expect("save_session failed");
    assert!(SessionManager::exists(&session_file));

    // Load session
    let loaded = SessionManager::load_session(&session_file).expect("load_session failed");
    assert_eq!(state, loaded);
}

#[test]
fn test_session_manager_error_handling() {
    let dir = tempdir().expect("failed to create temp dir");
    let non_existent = dir.path().join("does_not_exist.json");
    assert!(SessionManager::load_session(&non_existent).is_err());

    let invalid_json = dir.path().join("invalid.json");
    std::fs::write(&invalid_json, "not valid json { content").unwrap();
    assert!(SessionManager::load_session(&invalid_json).is_err());
}

#[test]
fn test_default_session_path() {
    let default_path = SessionManager::default_session_path();
    assert!(default_path.ends_with(Path::new(".txt").join("session.json")));
}
