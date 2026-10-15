#[path = "../src/theme.rs"]
mod theme;

use theme::EditorTheme;
use txt_core::syntax::TokenType;

#[test]
fn test_all_themes_count_and_names() {
    let all = EditorTheme::all();
    assert_eq!(all.len(), 7);

    for theme in all {
        let name = theme.name();
        assert!(!name.is_empty());
        let round_trip = EditorTheme::from_name(name);
        assert_eq!(round_trip, Some(*theme));
    }
}

#[test]
fn test_theme_from_name_variations() {
    assert_eq!(EditorTheme::from_name("Mariana"), Some(EditorTheme::Mariana));
    assert_eq!(EditorTheme::from_name("mariana"), Some(EditorTheme::Mariana));
    assert_eq!(EditorTheme::from_name("monokai"), Some(EditorTheme::Monokai));
    assert_eq!(EditorTheme::from_name("Solarized Dark"), Some(EditorTheme::SolarizedDark));
    assert_eq!(EditorTheme::from_name("solarized-dark"), Some(EditorTheme::SolarizedDark));
    assert_eq!(EditorTheme::from_name("solarized_dark"), Some(EditorTheme::SolarizedDark));
    assert_eq!(EditorTheme::from_name("Solarized Light"), Some(EditorTheme::SolarizedLight));
    assert_eq!(EditorTheme::from_name("one-dark"), Some(EditorTheme::OneDark));
    assert_eq!(EditorTheme::from_name("dracula"), Some(EditorTheme::Dracula));
    assert_eq!(EditorTheme::from_name("nord"), Some(EditorTheme::Nord));
    assert_eq!(EditorTheme::from_name("unknown-theme"), None);
}

#[test]
fn test_theme_dark_light_classification() {
    assert!(!EditorTheme::SolarizedLight.is_dark());
    assert!(EditorTheme::SolarizedDark.is_dark());
    assert!(EditorTheme::Mariana.is_dark());
    assert!(EditorTheme::Monokai.is_dark());
    assert!(EditorTheme::OneDark.is_dark());
    assert!(EditorTheme::Dracula.is_dark());
    assert!(EditorTheme::Nord.is_dark());
}

#[test]
fn test_theme_syntax_colors_distinction() {
    for theme in EditorTheme::all() {
        let kw = theme.syntax_color(TokenType::Keyword);
        let comment = theme.syntax_color(TokenType::Comment);
        let str_col = theme.syntax_color(TokenType::String);
        let fn_col = theme.syntax_color(TokenType::Function);

        // Keywords and comments must never be the same color in any theme
        assert_ne!(kw, comment, "Keyword and Comment shouldn't match in {:?}", theme);
        assert_ne!(str_col, comment, "String and Comment shouldn't match in {:?}", theme);
        assert_ne!(fn_col, comment, "Function and Comment shouldn't match in {:?}", theme);
    }
}

#[test]
fn test_ui_element_colors_presence() {
    for theme in EditorTheme::all() {
        let _ = theme.bg();
        let _ = theme.sidebar_bg();
        let _ = theme.tab_bar_bg();
        let _ = theme.tab_active_bg();
        let _ = theme.tab_inactive_text();
        let _ = theme.tab_active_text();
        let _ = theme.gutter_bg();
        let _ = theme.gutter_text();
        let _ = theme.gutter_active_text();
        let _ = theme.current_line_bg();
        let _ = theme.selection_bg();
        let _ = theme.cursor_color();
        let _ = theme.status_bar_bg();
        let _ = theme.status_bar_text();
        let _ = theme.modal_bg();
        let _ = theme.modal_item_hover();
        let _ = theme.border();
        let _ = theme.menu_bar_bg();
        let _ = theme.menu_item_hover();
    }
}
