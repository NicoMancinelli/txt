use std::path::PathBuf;
use tempfile::tempdir;
use txt_core::session::{SessionManager, SessionState, MAX_RECENT_FILES, MAX_RECENT_WORKSPACES};
use txt_core::workspace::{WorkspaceConfig, WorkspaceSettings};

#[test]
fn test_workspace_creation_and_mutations() {
    let mut config = WorkspaceConfig::new("MyProject");
    assert_eq!(config.name, "MyProject");
    assert!(config.folders.is_empty());
    assert_eq!(config.settings, WorkspaceSettings::default());

    let src_path = PathBuf::from("/workspace/src");
    let docs_path = PathBuf::from("/workspace/docs");

    // Add folders
    config.add_folder(src_path.clone(), Some("Source Code".to_string()));
    config.add_folder(docs_path.clone(), None);
    assert_eq!(config.folders.len(), 2);
    assert_eq!(config.folders[0].path, src_path);
    assert_eq!(config.folders[0].name, Some("Source Code".to_string()));
    assert_eq!(config.folders[1].path, docs_path);
    assert_eq!(config.folders[1].name, None);

    // Duplicate addition updates folder name
    config.add_folder(src_path.clone(), Some("Updated Sources".to_string()));
    assert_eq!(config.folders.len(), 2);
    assert_eq!(config.folders[0].name, Some("Updated Sources".to_string()));

    // Settings configuration
    config.settings = WorkspaceSettings::new()
        .with_tab_size(4)
        .with_word_wrap(true)
        .with_font_size(15.5);
    assert_eq!(config.settings.tab_size, Some(4));
    assert_eq!(config.settings.word_wrap, Some(true));
    assert_eq!(config.settings.font_size, Some(15.5));

    // Remove folder
    config.remove_folder(&src_path);
    assert_eq!(config.folders.len(), 1);
    assert_eq!(config.folders[0].path, docs_path);
}

#[test]
fn test_workspace_save_and_load() {
    let dir = tempdir().expect("failed to create temp dir");
    let file_path = dir.path().join("projects").join("sample.txt-workspace");

    let mut config = WorkspaceConfig::new("Sample Project");
    config.add_folder(PathBuf::from("/repo/core"), Some("Core".to_string()));
    config.add_folder(PathBuf::from("/repo/gui"), Some("GUI".to_string()));
    config.settings = WorkspaceSettings::new()
        .with_tab_size(2)
        .with_word_wrap(false)
        .with_font_size(14.0);

    // Save workspace (auto-creates parent dirs)
    config.save_to_file(&file_path).expect("failed to save workspace");
    assert!(file_path.exists());

    // Load workspace
    let loaded = WorkspaceConfig::load_from_file(&file_path).expect("failed to load workspace");
    assert_eq!(loaded, config);
    assert_eq!(loaded.name, "Sample Project");
    assert_eq!(loaded.folders.len(), 2);
    assert_eq!(loaded.settings.tab_size, Some(2));
}

#[test]
fn test_sublime_project_json_compatibility() {
    // Sublime Text .sublime-project format with extra keys and folders
    let sublime_json = r#"{
        "folders": [
            {
                "path": "src",
                "name": "Sources",
                "folder_exclude_patterns": ["backup", ".git"],
                "file_exclude_patterns": ["*.log"]
            },
            {
                "path": "/shared/assets"
            }
        ],
        "settings": {
            "tab_size": 2,
            "word_wrap": false,
            "font_size": 13.0,
            "translate_tabs_to_spaces": true
        },
        "build_systems": [
            {
                "name": "Cargo Test",
                "cmd": ["cargo", "test"]
            }
        ]
    }"#;

    // Parse from string directly via Serde
    let parsed: WorkspaceConfig = serde_json::from_str(sublime_json).expect("failed to parse sublime project");
    assert_eq!(parsed.folders.len(), 2);
    assert_eq!(parsed.folders[0].path, PathBuf::from("src"));
    assert_eq!(parsed.folders[0].name, Some("Sources".to_string()));
    assert_eq!(parsed.folders[1].path, PathBuf::from("/shared/assets"));
    assert_eq!(parsed.folders[1].name, None);
    assert_eq!(parsed.settings.tab_size, Some(2));
    assert_eq!(parsed.settings.word_wrap, Some(false));
    assert_eq!(parsed.settings.font_size, Some(13.0));

    // Test load_from_file with .sublime-project filename
    let dir = tempdir().expect("failed to create temp dir");
    let sublime_file = dir.path().join("editor.sublime-project");
    std::fs::write(&sublime_file, sublime_json).expect("failed to write sublime file");

    let loaded = WorkspaceConfig::load_from_file(&sublime_file).expect("failed to load sublime file");
    // Name should default to the file stem "editor"
    assert_eq!(loaded.name, "editor");
    assert_eq!(loaded.folders.len(), 2);
    assert_eq!(loaded.folders[0].name, Some("Sources".to_string()));

    // Test helper method from_sublime_project_str
    let config_helper = WorkspaceConfig::from_sublime_project_str(sublime_json, Some("FallbackName"))
        .expect("failed from_sublime_project_str");
    assert_eq!(config_helper.name, "FallbackName");
}

#[test]
fn test_recent_files_deduplication_and_lru_truncation() {
    let mut state = SessionState::new();
    assert!(state.recent_files.is_empty());

    // Push files
    for i in 1..=25 {
        state.push_recent_file(PathBuf::from(format!("file_{}.rs", i)));
    }

    // Must be bounded by MAX_RECENT_FILES (20)
    assert_eq!(state.recent_files.len(), MAX_RECENT_FILES);
    assert_eq!(state.recent_files[0], PathBuf::from("file_25.rs"));
    assert_eq!(state.recent_files[19], PathBuf::from("file_6.rs"));

    // Push an existing file (e.g. file_10.rs): it should move to index 0 without duplicates
    state.push_recent_file(PathBuf::from("file_10.rs"));
    assert_eq!(state.recent_files.len(), MAX_RECENT_FILES);
    assert_eq!(state.recent_files[0], PathBuf::from("file_10.rs"));
    assert_eq!(state.recent_files[1], PathBuf::from("file_25.rs"));

    // Ensure file_10.rs is not duplicated anywhere else
    let count = state.recent_files.iter().filter(|p| **p == PathBuf::from("file_10.rs")).count();
    assert_eq!(count, 1);
}

#[test]
fn test_recent_workspaces_deduplication_and_lru_truncation() {
    let mut state = SessionState::new();
    assert!(state.recent_workspaces.is_empty());

    // Push workspaces
    for i in 1..=15 {
        state.push_recent_workspace(PathBuf::from(format!("/workspaces/project_{}.txt-workspace", i)));
    }

    // Must be bounded by MAX_RECENT_WORKSPACES (10)
    assert_eq!(state.recent_workspaces.len(), MAX_RECENT_WORKSPACES);
    assert_eq!(state.recent_workspaces[0], PathBuf::from("/workspaces/project_15.txt-workspace"));
    assert_eq!(state.recent_workspaces[9], PathBuf::from("/workspaces/project_6.txt-workspace"));

    // Re-push project_8
    state.push_recent_workspace(PathBuf::from("/workspaces/project_8.txt-workspace"));
    assert_eq!(state.recent_workspaces.len(), MAX_RECENT_WORKSPACES);
    assert_eq!(state.recent_workspaces[0], PathBuf::from("/workspaces/project_8.txt-workspace"));
    assert_eq!(state.recent_workspaces[1], PathBuf::from("/workspaces/project_15.txt-workspace"));

    let count = state.recent_workspaces.iter()
        .filter(|p| **p == PathBuf::from("/workspaces/project_8.txt-workspace"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn test_session_state_active_workspace_persistence() {
    let dir = tempdir().expect("failed to create temp dir");
    let session_file = dir.path().join("session.json");
    let workspace_file = dir.path().join("my_app.txt-workspace");

    let state = SessionState::new()
        .with_active_workspace(Some(workspace_file.clone()))
        .with_recent_files(vec![PathBuf::from("main.rs")])
        .with_recent_workspaces(vec![workspace_file.clone()]);

    assert_eq!(state.active_workspace, Some(workspace_file.clone()));
    assert_eq!(state.recent_files.len(), 1);
    assert_eq!(state.recent_workspaces.len(), 1);

    // Save and reload session
    SessionManager::save_session(&session_file, &state).expect("failed to save session");
    let loaded = SessionManager::load_session(&session_file).expect("failed to load session");

    assert_eq!(loaded.active_workspace, Some(workspace_file));
    assert_eq!(loaded.recent_files, vec![PathBuf::from("main.rs")]);
    assert_eq!(loaded.recent_workspaces.len(), 1);
}
