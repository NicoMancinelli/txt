#[path = "../src/app.rs"]
mod app;
#[path = "../src/editor_view.rs"]
mod editor_view;
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
