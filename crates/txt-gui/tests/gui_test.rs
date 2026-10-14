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
