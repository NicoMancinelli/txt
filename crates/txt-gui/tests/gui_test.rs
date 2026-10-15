#[path = "../src/app.rs"]
mod app;
#[path = "../src/editor_view.rs"]
mod editor_view;
#[path = "../src/find_replace_view.rs"]
mod find_replace_view;
#[path = "../src/menubar.rs"]
mod menubar;
#[path = "../src/palette.rs"]
mod palette;
#[path = "../src/sidebar.rs"]
mod sidebar;
#[path = "../src/status_bar.rs"]
mod status_bar;
#[path = "../src/tabs.rs"]
mod tabs;
#[path = "../src/theme.rs"]
mod theme;

use app::TxtEditorApp;
use theme::EditorTheme;

#[test]
fn test_app_initialization() {
    let app = TxtEditorApp::new();
    assert_eq!(app.buffers.len(), 1);
    assert_eq!(app.active_tab, 0);
    assert_eq!(app.theme, EditorTheme::Mariana);
    assert!(app.sidebar.is_visible);
    assert_eq!(app.sync_engine.active_client_count(), 1);
}

#[test]
fn test_app_new_file_command() {
    let mut app = TxtEditorApp::new();
    assert_eq!(app.buffers.len(), 1);

    app.execute_command("file.new");
    assert_eq!(app.buffers.len(), 2);
    assert_eq!(app.active_tab, 1);
}

#[test]
fn test_app_theme_switching() {
    let mut app = TxtEditorApp::new();
    assert_eq!(app.theme, EditorTheme::Mariana);

    app.execute_command("view.theme_monokai");
    assert_eq!(app.theme, EditorTheme::Monokai);

    app.execute_command("view.theme_mariana");
    assert_eq!(app.theme, EditorTheme::Mariana);
}

#[test]
fn test_app_command_palette_search() {
    let mut app = TxtEditorApp::new();
    app.palette.open(&app.registry);
    assert!(app.palette.is_open);

    app.palette.query = "save".to_string();
    app.palette.update_matches(&app.registry);

    assert!(!app.palette.matched_commands.is_empty());
    assert_eq!(app.palette.matched_commands[0].descriptor.id, "file.save");
}

#[test]
fn test_app_new_commands_integration() {
    let mut app = TxtEditorApp::new();

    // Test find in files command
    app.execute_command("find.in_files");
    assert!(app.last_notification.is_some());
    assert!(app.last_notification.as_ref().unwrap().contains("Find in Files"));

    // Test plugin word count stats
    app.execute_command("plugin.word_count");
    assert!(app.last_notification.as_ref().unwrap().contains("Document Stats"));

    // Test LSP status
    app.execute_command("lsp.status");
    assert!(app.last_notification.as_ref().unwrap().contains("LSP: Ready"));

    // Test CRDT delta sync
    app.execute_command("sync.crdt_delta");
    assert!(app.last_notification.as_ref().unwrap().contains("CRDT Patch synced"));
}

#[test]
fn test_sidebar_open_files_and_filtering() {
    let mut app = TxtEditorApp::new();
    assert!(app.sidebar.show_open_files);

    app.execute_command("view.toggle_open_files");
    assert!(!app.sidebar.show_open_files);

    app.execute_command("view.toggle_open_files");
    assert!(app.sidebar.show_open_files);

    app.sidebar.search_filter = "Cargo".to_string();
    assert_eq!(app.sidebar.search_filter, "Cargo");

    app.execute_command("project.refresh_folders");
    assert!(app.sidebar.current_directory.exists());
}

#[test]
fn test_editor_word_wrap_toggle() {
    let mut app = TxtEditorApp::new();
    assert!(app.editor_view.word_wrap);

    app.execute_command("view.toggle_word_wrap");
    assert!(!app.editor_view.word_wrap);

    app.execute_command("view.toggle_word_wrap");
    assert!(app.editor_view.word_wrap);
}

#[test]
fn test_editor_duplicate_and_delete_line() {
    let mut app = TxtEditorApp::new();
    if let Some(buf) = app.buffers.get_mut(app.active_tab) {
        buf.replace(0..buf.len_chars(), "first line\nsecond line\n");
    }

    // Duplicate line
    app.execute_command("edit.duplicate_line");
    let text = app.buffers[app.active_tab].text();
    assert_eq!(text, "first line\nfirst line\nsecond line\n");

    // Delete line
    app.execute_command("edit.delete_line");
    let text_after_delete = app.buffers[app.active_tab].text();
    assert_eq!(text_after_delete, "first line\nsecond line\n");
}

#[test]
fn test_sidebar_full_suite_project_and_dialogs() {
    let mut app = TxtEditorApp::new();
    assert!(!app.sidebar.project_manager.folders.is_empty());

    // Expand all / collapse all
    app.execute_command("project.expand_all");
    assert!(app.sidebar.project_manager.folders[0].root_node.is_expanded);

    app.execute_command("project.collapse_all");
    assert!(!app.sidebar.project_manager.folders[0].root_node.is_expanded);

    // Toggle hidden files
    assert!(!app.sidebar.project_manager.show_hidden);
    app.execute_command("project.toggle_hidden");
    assert!(app.sidebar.project_manager.show_hidden);
    app.execute_command("project.toggle_hidden");
    assert!(!app.sidebar.project_manager.show_hidden);

    // Reveal active buffer in sidebar
    let cargo_path = std::env::current_dir().unwrap().join("Cargo.toml");
    app.buffers[0].set_file_path(cargo_path.to_str().unwrap());
    app.execute_command("file.reveal_in_sidebar");
    assert_eq!(app.sidebar.selected_path.as_deref(), Some(cargo_path.as_path()));

    // Multi-root project addition
    let crates_dir = std::env::current_dir().unwrap().join("crates");
    let initial_count = app.sidebar.project_manager.folders.len();
    app.sidebar.add_folder_to_project(crates_dir.clone());
    assert_eq!(app.sidebar.project_manager.folders.len(), initial_count + 1);

    // Removal
    app.sidebar.remove_folder_from_project(&crates_dir);
    assert_eq!(app.sidebar.project_manager.folders.len(), initial_count);
}

#[test]
fn test_find_replace_panel_integration() {
    let mut app = TxtEditorApp::new();
    let sample = "apple orange banana apple kiwi apple";
    let len = app.buffers[0].len_chars();
    app.buffers[0].replace(0..len, sample);

    // Open find
    app.execute_command("find.find");
    assert!(app.find_replace.is_open);
    assert!(!app.find_replace.show_replace);

    // Set query
    app.find_replace.find_query = "apple".to_string();
    app.find_replace.update_matches(&app.buffers[0]);
    assert_eq!(app.find_replace.matches.len(), 3);
    assert!(app.find_replace.status_text.contains("1 of 3 matches"));

    // Find next
    app.execute_command("find.next");
    assert_eq!(app.find_replace.active_match_index, Some(1));

    // Open replace
    app.execute_command("find.replace");
    assert!(app.find_replace.show_replace);

    // Replace all
    app.find_replace.replace_query = "mango".to_string();
    app.find_replace.replace_all_matches(&mut app.buffers[0], &mut app.selections[0]);
    assert_eq!(app.buffers[0].text(), "mango orange banana mango kiwi mango");
    assert_eq!(app.find_replace.matches.len(), 0);
}

#[test]
fn test_palette_goto_symbol_and_line() {
    let mut app = TxtEditorApp::new();
    let rust_code = r#"
pub struct EditorState {
    pub line: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self { line: 0 }
    }

    pub fn reset(&mut self) {
        self.line = 0;
    }
}
"#;
    let len = app.buffers[0].len_chars();
    app.buffers[0].replace(0..len, rust_code);

    // Goto Symbol command
    app.execute_command("goto.symbol");
    assert!(app.palette.is_open);
    assert_eq!(app.palette.query, "@");
    assert!(!app.palette.matched_symbols.is_empty());

    // Search for a specific method
    app.palette.query = "@reset".to_string();
    app.palette.update_matches_with_buffer(&app.registry, Some(&app.buffers[0]), "rust");
    assert_eq!(app.palette.matched_symbols.len(), 1);
    assert_eq!(app.palette.matched_symbols[0].symbol.name, "reset");

    // Goto Line
    app.execute_command("goto.line");
    assert!(app.palette.is_open);
    assert_eq!(app.palette.query, ":");

    app.palette.query = ":7".to_string();
    app.palette.update_matches_with_buffer(&app.registry, Some(&app.buffers[0]), "rust");
    assert_eq!(app.palette.goto_line_target, Some(6));
}

#[test]
fn test_multicursor_sublime_commands_gui() {
    let mut app = TxtEditorApp::new();
    let text = "alpha beta alpha gamma alpha";
    let len = app.buffers[0].len_chars();
    app.buffers[0].replace(0..len, text);

    // Set point at first 'alpha'
    app.selections[0].set_primary(txt_core::Selection::point(0));

    // 1. Quick Add Next (Cmd+D)
    app.execute_command("selection.add_next_occurrence");
    // Expands to word "alpha"
    assert_eq!(app.selections[0].len(), 1);
    assert_eq!(app.selections[0].primary(), txt_core::Selection::range(0, 5));

    // 2. Next occurrence
    app.execute_command("selection.add_next_occurrence");
    assert_eq!(app.selections[0].len(), 2);

    // 3. Quick Find All (Cmd+Ctrl+G)
    app.execute_command("selection.select_all_occurrences");
    assert_eq!(app.selections[0].len(), 3);
}

#[test]
fn test_persistent_session_state_save_and_restore() {
    let temp_dir = std::env::temp_dir().join(format!("txt_session_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let session_file = temp_dir.join("session.json");

    let state = txt_core::SessionState {
        open_files: vec![temp_dir.join("file1.rs"), temp_dir.join("file2.rs")],
        active_file: Some(temp_dir.join("file2.rs")),
        sidebar_visible: false,
        font_size: 16.0,
        theme: "Monokai".to_string(),
        word_wrap: false,
        ..Default::default()
    };

    txt_core::SessionManager::save_session(&session_file, &state).unwrap();
    let loaded = txt_core::SessionManager::load_session(&session_file).unwrap();

    assert_eq!(loaded.open_files.len(), 2);
    assert_eq!(loaded.active_file, Some(temp_dir.join("file2.rs")));
    assert!(!loaded.sidebar_visible);
    assert_eq!(loaded.font_size, 16.0);
    assert_eq!(loaded.theme, "Monokai");
    assert!(!loaded.word_wrap);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_all_seven_themes_via_commands() {
    let mut app = TxtEditorApp::new();

    let theme_commands = [
        ("view.theme_monokai", EditorTheme::Monokai),
        ("view.theme_solarized_dark", EditorTheme::SolarizedDark),
        ("view.theme_solarized_light", EditorTheme::SolarizedLight),
        ("view.theme_one_dark", EditorTheme::OneDark),
        ("view.theme_dracula", EditorTheme::Dracula),
        ("view.theme_nord", EditorTheme::Nord),
        ("view.theme_mariana", EditorTheme::Mariana),
    ];

    for (cmd, expected_theme) in theme_commands {
        app.execute_command(cmd);
        assert_eq!(app.theme, expected_theme, "Failed switching with command {}", cmd);
    }
}

#[test]
fn test_font_scaling_commands() {
    let mut app = TxtEditorApp::new();
    let initial_font = app.editor_view.font_size;

    app.execute_command("view.font_increase");
    assert_eq!(app.editor_view.font_size, initial_font + 1.0);

    app.execute_command("view.font_decrease");
    assert_eq!(app.editor_view.font_size, initial_font);

    app.editor_view.font_size = 24.0;
    app.execute_command("view.font_reset");
    assert_eq!(app.editor_view.font_size, 14.0);
}

#[test]
fn test_workspace_management_and_recent_tracking() {
    let mut app = TxtEditorApp::new();
    let temp_dir = std::env::temp_dir().join(format!("txt_ws_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let file_a = temp_dir.join("file_a.rs");
    let file_b = temp_dir.join("file_b.rs");
    std::fs::write(&file_a, "fn a() {}").unwrap();
    std::fs::write(&file_b, "fn b() {}").unwrap();

    // Opening files adds to recent files and selects tabs without duplicate buffers
    app.open_file(file_a.clone());
    assert_eq!(app.buffers.len(), 2);
    assert_eq!(app.recent_files[0], file_a);

    app.open_file(file_b.clone());
    assert_eq!(app.buffers.len(), 3);
    assert_eq!(app.recent_files[0], file_b);

    // Opening already open file activates tab and puts to front of recent files
    app.open_file(file_a.clone());
    assert_eq!(app.buffers.len(), 3);
    assert_eq!(app.active_tab, 1);
    assert_eq!(app.recent_files[0], file_a);

    // Save workspace
    let ws_path = temp_dir.join("test.txt-workspace");
    app.save_workspace_as(ws_path.clone());
    assert_eq!(app.active_workspace, Some(ws_path.clone()));
    assert_eq!(app.active_workspace_name, Some("test".to_string()));
    assert_eq!(app.recent_workspaces[0], ws_path);

    // Close workspace
    app.execute_command("project.close_workspace");
    assert_eq!(app.active_workspace, None);
    assert_eq!(app.active_workspace_name, None);

    // Reopen workspace
    app.open_workspace(ws_path);
    assert!(app.active_workspace.is_some());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_menubar_state_and_categories() {
    use menubar::{MenuCategory, MenuBarState};
    let mut state = MenuBarState::new();
    assert_eq!(state.active_menu, None);
    assert_eq!(state.active_submenu, None);

    state.active_menu = Some(MenuCategory::File);
    state.active_submenu = Some("recent_files".to_string());
    assert_eq!(state.active_menu, Some(MenuCategory::File));

    state.close();
    assert_eq!(state.active_menu, None);
    assert_eq!(state.active_submenu, None);

    assert_eq!(MenuCategory::App.title(), "txt");
    assert_eq!(MenuCategory::File.title(), "File");
    assert_eq!(MenuCategory::Edit.title(), "Edit");
    assert_eq!(MenuCategory::Selection.title(), "Selection");
    assert_eq!(MenuCategory::Find.title(), "Find");
    assert_eq!(MenuCategory::View.title(), "View");
    assert_eq!(MenuCategory::Goto.title(), "Goto");
    assert_eq!(MenuCategory::Tools.title(), "Tools");
    assert_eq!(MenuCategory::Project.title(), "Project");
    assert_eq!(MenuCategory::Help.title(), "Help");
}

#[test]
fn test_save_all_and_revert_commands() {
    let mut app = TxtEditorApp::new();
    let temp_dir = std::env::temp_dir().join(format!("txt_revert_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let test_file = temp_dir.join("test_revert.rs");
    std::fs::write(&test_file, "original content").unwrap();

    app.open_file(test_file.clone());
    let active_idx = app.active_tab;

    // Mutate buffer
    app.buffers[active_idx].insert(0, "modified ");
    assert!(app.buffers[active_idx].is_dirty());

    // Revert file reloads original content
    app.execute_command("file.revert");
    assert_eq!(app.buffers[active_idx].text(), "original content");
    assert!(!app.buffers[active_idx].is_dirty());

    // Mutate and save all
    app.buffers[active_idx].insert(0, "persisted ");
    assert!(app.buffers[active_idx].is_dirty());
    app.execute_command("file.save_all");
    assert!(!app.buffers[active_idx].is_dirty());
    let saved_disk = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(saved_disk, "persisted original content");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

