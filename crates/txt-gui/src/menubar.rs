use egui::{self, Color32, Key, Pos2, Rounding, Stroke, Vec2};
use std::path::PathBuf;
use crate::theme::EditorTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuCategory {
    App,
    File,
    Edit,
    Selection,
    Find,
    View,
    Goto,
    Tools,
    Project,
    Help,
}

impl MenuCategory {
    pub fn title(&self) -> &'static str {
        match self {
            MenuCategory::App => "txt",
            MenuCategory::File => "File",
            MenuCategory::Edit => "Edit",
            MenuCategory::Selection => "Selection",
            MenuCategory::Find => "Find",
            MenuCategory::View => "View",
            MenuCategory::Goto => "Goto",
            MenuCategory::Tools => "Tools",
            MenuCategory::Project => "Project",
            MenuCategory::Help => "Help",
        }
    }
}

pub struct MenuBarState {
    pub active_menu: Option<MenuCategory>,
    pub active_submenu: Option<String>,
}

impl Default for MenuBarState {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuBarState {
    pub fn new() -> Self {
        Self {
            active_menu: None,
            active_submenu: None,
        }
    }

    pub fn close(&mut self) {
        self.active_menu = None;
        self.active_submenu = None;
    }
}

pub struct MenuBarContext<'a> {
    pub theme: EditorTheme,
    pub is_sidebar_visible: bool,
    pub is_open_files_visible: bool,
    pub is_minimap_visible: bool,
    pub is_word_wrap: bool,
    pub recent_files: &'a [PathBuf],
    pub recent_workspaces: &'a [PathBuf],
    pub active_workspace_name: Option<&'a str>,
}

pub struct MenuBar;

impl MenuBar {
    /// Renders the top menu bar strip and handles dropdown menus.
    /// Returns Some(command_id) if a menu action was triggered.
    pub fn show(
        state: &mut MenuBarState,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        menu_ctx: &MenuBarContext,
    ) -> Option<String> {
        let mut executed_command = None;

        // Escape closes any open menu
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            state.close();
        }

        let bar_height = 28.0_f32;
        let theme = menu_ctx.theme;

        egui::Frame::none()
            .fill(theme.menu_bar_bg())
            .stroke(Stroke::new(1.0_f32, theme.border()))
            .inner_margin(egui::Margin::symmetric(10.0, 2.0))
            .show(ui, |ui| {
                ui.set_height(bar_height);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

                    let categories = [
                        MenuCategory::App,
                        MenuCategory::File,
                        MenuCategory::Edit,
                        MenuCategory::Selection,
                        MenuCategory::Find,
                        MenuCategory::View,
                        MenuCategory::Goto,
                        MenuCategory::Tools,
                        MenuCategory::Project,
                        MenuCategory::Help,
                    ];

                    for &cat in &categories {
                        let is_active = state.active_menu == Some(cat);
                        let title = cat.title();

                        let text_color = if is_active {
                            Color32::WHITE
                        } else if cat == MenuCategory::App {
                            Color32::from_rgb(0xf9, 0x91, 0x57) // Sublime orange branding
                        } else {
                            Color32::from_rgb(0xd8, 0xde, 0xe9)
                        };

                        let font = if cat == MenuCategory::App {
                            egui::RichText::new(title).size(13.0).strong().color(text_color)
                        } else {
                            egui::RichText::new(title).size(12.5).color(text_color)
                        };

                        let button_resp = ui.button(font);

                        // Click toggles menu
                        if button_resp.clicked() {
                            if is_active {
                                state.close();
                            } else {
                                state.active_menu = Some(cat);
                                state.active_submenu = None;
                            }
                        }

                        // Hover switches active menu if already open
                        if state.active_menu.is_some() && button_resp.hovered() && !is_active {
                            state.active_menu = Some(cat);
                            state.active_submenu = None;
                        }

                        // If this menu is open, show the dropdown menu overlay
                        if state.active_menu == Some(cat) {
                            let dropdown_pos = Pos2::new(button_resp.rect.min.x, button_resp.rect.max.y + 2.0);
                            if let Some(cmd) = Self::show_dropdown(
                                cat,
                                dropdown_pos,
                                ctx,
                                theme,
                                menu_ctx,
                                state,
                            ) {
                                executed_command = Some(cmd);
                                state.close();
                            }
                        }
                    }

                    // Workspace indicator on the right side of the menu bar
                    if let Some(ws_name) = menu_ctx.active_workspace_name {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("Workspace: {}", ws_name))
                                    .size(11.0)
                                    .color(Color32::from_rgb(0xa6, 0xac, 0xcd)),
                            );
                        });
                    }
                });
            });

        executed_command
    }

    fn show_dropdown(
        category: MenuCategory,
        pos: Pos2,
        ctx: &egui::Context,
        theme: EditorTheme,
        menu_ctx: &MenuBarContext,
        state: &mut MenuBarState,
    ) -> Option<String> {
        let mut executed = None;

        let area_id = egui::Id::new("menubar_dropdown_overlay").with(category as usize);
        egui::Area::new(area_id)
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(theme.modal_bg())
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0_f32, theme.border()))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);

                        match category {
                            MenuCategory::App => {
                                Self::render_item(ui, "About txt", None, "app.about", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Preferences: Settings", Some("Cmd+,"), "preferences.settings", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Quit txt", Some("Cmd+Q"), "app.quit", &mut executed);
                            }
                            MenuCategory::File => {
                                Self::render_item(ui, "New File", Some("Cmd+N"), "file.new", &mut executed);
                                Self::render_item(ui, "Open File...", Some("Cmd+O"), "file.open_dialog", &mut executed);
                                Self::render_item(ui, "Open Folder...", Some("Cmd+Shift+O"), "project.open_folder_dialog", &mut executed);

                                // Recent files submenu
                                if !menu_ctx.recent_files.is_empty() {
                                    let is_recent_open = state.active_submenu.as_deref() == Some("recent_files");
                                    let resp = ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Open Recent").size(12.5).color(Color32::from_rgb(0xd8, 0xde, 0xe9)));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(egui::RichText::new("▶").size(10.0).color(Color32::from_rgb(0x8a, 0x93, 0xa0)));
                                        });
                                    });
                                    if resp.response.hovered() {
                                        state.active_submenu = Some("recent_files".to_string());
                                    }
                                    if is_recent_open {
                                        let sub_pos = Pos2::new(resp.response.rect.max.x + 4.0, resp.response.rect.min.y);
                                        if let Some(cmd) = Self::show_recent_submenu(sub_pos, ctx, theme, menu_ctx.recent_files, "file.open_path:", "menubar_recent_files") {
                                            executed = Some(cmd);
                                        }
                                    }
                                }

                                Self::render_separator(ui);
                                Self::render_item(ui, "Save", Some("Cmd+S"), "file.save", &mut executed);
                                Self::render_item(ui, "Save As...", Some("Cmd+Shift+S"), "file.save_as", &mut executed);
                                Self::render_item(ui, "Save All", Some("Cmd+Option+S"), "file.save_all", &mut executed);
                                Self::render_item(ui, "Revert File", None, "file.revert", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Close Tab", Some("Cmd+W"), "file.close_tab", &mut executed);
                                Self::render_item(ui, "Close Window", Some("Cmd+Shift+W"), "file.close_window", &mut executed);
                            }
                            MenuCategory::Edit => {
                                Self::render_item(ui, "Undo", Some("Cmd+Z"), "edit.undo", &mut executed);
                                Self::render_item(ui, "Redo", Some("Cmd+Shift+Z"), "edit.redo", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Duplicate Line", Some("Cmd+Shift+D"), "edit.duplicate_line", &mut executed);
                                Self::render_item(ui, "Delete Line", Some("Cmd+Shift+K"), "edit.delete_line", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Convert Case: Upper Case", None, "convert.upper_case", &mut executed);
                                Self::render_item(ui, "Convert Case: Lower Case", None, "convert.lower_case", &mut executed);
                                Self::render_item(ui, "Convert Case: Title Case", None, "convert.title_case", &mut executed);
                                Self::render_item(ui, "Convert Case: snake_case", None, "convert.snake_case", &mut executed);
                                Self::render_item(ui, "Convert Case: lowerCamelCase", None, "convert.camel_case", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Indentation: Convert to Spaces", None, "indent.convert_to_spaces", &mut executed);
                                Self::render_item(ui, "Indentation: Convert to Tabs", None, "indent.convert_to_tabs", &mut executed);
                            }
                            MenuCategory::Selection => {
                                Self::render_item(ui, "Select All", Some("Cmd+A"), "edit.select_all", &mut executed);
                                Self::render_item(ui, "Split Selection into Lines", Some("Cmd+Shift+L"), "selection.split_into_lines", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Quick Add Next", Some("Cmd+D"), "selection.add_next_occurrence", &mut executed);
                                Self::render_item(ui, "Quick Find All", Some("Cmd+Ctrl+G"), "selection.select_all_occurrences", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Add Cursor Above", Some("Cmd+Option+Up"), "selection.add_cursor_above", &mut executed);
                                Self::render_item(ui, "Add Cursor Below", Some("Cmd+Option+Down"), "selection.add_cursor_below", &mut executed);
                            }
                            MenuCategory::Find => {
                                Self::render_item(ui, "Find...", Some("Cmd+F"), "find.find", &mut executed);
                                Self::render_item(ui, "Find Next", Some("F3"), "find.next", &mut executed);
                                Self::render_item(ui, "Find Previous", Some("Shift+F3"), "find.prev", &mut executed);
                                Self::render_item(ui, "Replace...", Some("Cmd+Option+F"), "find.replace", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Find in Files...", Some("Cmd+Shift+F"), "find.in_files", &mut executed);
                            }
                            MenuCategory::View => {
                                Self::render_toggle_item(ui, "Side Bar", Some("Cmd+B"), "view.toggle_sidebar", menu_ctx.is_sidebar_visible, &mut executed);
                                Self::render_toggle_item(ui, "Open Files Section", None, "view.toggle_open_files", menu_ctx.is_open_files_visible, &mut executed);
                                Self::render_toggle_item(ui, "Minimap", None, "view.toggle_minimap", menu_ctx.is_minimap_visible, &mut executed);
                                Self::render_toggle_item(ui, "Word Wrap", Some("Option+Z"), "view.toggle_word_wrap", menu_ctx.is_word_wrap, &mut executed);
                                Self::render_separator(ui);

                                // Themes submenu
                                let is_themes_open = state.active_submenu.as_deref() == Some("themes");
                                let resp = ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Theme").size(12.5).color(Color32::from_rgb(0xd8, 0xde, 0xe9)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new("▶").size(10.0).color(Color32::from_rgb(0x8a, 0x93, 0xa0)));
                                    });
                                });
                                if resp.response.hovered() {
                                    state.active_submenu = Some("themes".to_string());
                                }
                                if is_themes_open {
                                    let sub_pos = Pos2::new(resp.response.rect.max.x + 4.0, resp.response.rect.min.y);
                                    if let Some(cmd) = Self::show_themes_submenu(sub_pos, ctx, theme, menu_ctx.theme) {
                                        executed = Some(cmd);
                                    }
                                }

                                Self::render_separator(ui);
                                Self::render_item(ui, "Increase Font Size", Some("Cmd+="), "view.font_increase", &mut executed);
                                Self::render_item(ui, "Decrease Font Size", Some("Cmd+-"), "view.font_decrease", &mut executed);
                                Self::render_item(ui, "Reset Font Size", Some("Cmd+0"), "view.font_reset", &mut executed);
                            }
                            MenuCategory::Goto => {
                                Self::render_item(ui, "Goto Anything...", Some("Cmd+P"), "tools.command_palette", &mut executed);
                                Self::render_item(ui, "Goto Symbol...", Some("Cmd+R"), "goto.symbol", &mut executed);
                                Self::render_item(ui, "Goto Line...", Some("Ctrl+G"), "goto.line", &mut executed);
                            }
                            MenuCategory::Tools => {
                                Self::render_item(ui, "Command Palette...", Some("Cmd+Shift+P"), "tools.command_palette", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Sync Document Now", None, "sync.now", &mut executed);
                                Self::render_item(ui, "Toggle Cloud Offline/Online", None, "sync.toggle_cloud", &mut executed);
                                Self::render_item(ui, "Broadcast CRDT Delta", None, "sync.crdt_delta", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Word Count Statistics", Some("Cmd+Option+C"), "plugin.word_count", &mut executed);
                                Self::render_item(ui, "LSP Status & Diagnostics", None, "lsp.status", &mut executed);
                            }
                            MenuCategory::Project => {
                                Self::render_item(ui, "Add Folder to Project...", None, "project.add_folder", &mut executed);
                                Self::render_item(ui, "Save Workspace As...", None, "project.save_workspace_as", &mut executed);
                                Self::render_item(ui, "Open Workspace...", None, "project.open_workspace_dialog", &mut executed);
                                if !menu_ctx.recent_workspaces.is_empty() {
                                    let is_recent_ws_open = state.active_submenu.as_deref() == Some("recent_workspaces");
                                    let resp = egui::Frame::none()
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Open Recent Workspace ▶").size(12.5).color(Color32::from_rgb(0xd8, 0xde, 0xe9)));
                                            });
                                        });
                                    if resp.response.hovered() {
                                        state.active_submenu = Some("recent_workspaces".to_string());
                                    }
                                    if is_recent_ws_open {
                                        let sub_pos = Pos2::new(resp.response.rect.max.x + 4.0, resp.response.rect.min.y);
                                        if let Some(cmd) = Self::show_recent_submenu(sub_pos, ctx, theme, menu_ctx.recent_workspaces, "project.open_path:", "menubar_recent_workspaces") {
                                            executed = Some(cmd);
                                        }
                                    }
                                }
                                Self::render_item(ui, "Close Workspace", None, "project.close_workspace", &mut executed);
                                Self::render_separator(ui);
                                Self::render_item(ui, "Refresh Folders", None, "project.refresh_folders", &mut executed);
                                Self::render_item(ui, "Expand All", None, "project.expand_all", &mut executed);
                                Self::render_item(ui, "Collapse All", None, "project.collapse_all", &mut executed);
                                Self::render_item(ui, "Toggle Hidden Files", None, "project.toggle_hidden", &mut executed);
                            }
                            MenuCategory::Help => {
                                Self::render_item(ui, "Documentation", None, "help.docs", &mut executed);
                                Self::render_item(ui, "Keyboard Shortcuts", None, "help.shortcuts", &mut executed);
                                Self::render_item(ui, "GitHub Repository", None, "help.github", &mut executed);
                            }
                        }
                    });
            });

        executed
    }

    fn show_themes_submenu(
        pos: Pos2,
        ctx: &egui::Context,
        theme: EditorTheme,
        current_theme: EditorTheme,
    ) -> Option<String> {
        let mut executed = None;
        egui::Area::new(egui::Id::new("menubar_themes_submenu"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(theme.modal_bg())
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0_f32, theme.border()))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_min_width(180.0);
                        let themes = [
                            ("Mariana", "view.theme_mariana", current_theme == EditorTheme::Mariana),
                            ("Monokai", "view.theme_monokai", current_theme == EditorTheme::Monokai),
                            ("Solarized Dark", "view.theme_solarized_dark", current_theme.name() == "Solarized Dark"),
                            ("Solarized Light", "view.theme_solarized_light", current_theme.name() == "Solarized Light"),
                            ("One Dark", "view.theme_one_dark", current_theme.name() == "One Dark"),
                            ("Dracula", "view.theme_dracula", current_theme.name() == "Dracula"),
                            ("Nord", "view.theme_nord", current_theme.name() == "Nord"),
                        ];
                        for (name, cmd, checked) in themes {
                            Self::render_toggle_item(ui, name, None, cmd, checked, &mut executed);
                        }
                    });
            });
        executed
    }

    fn show_recent_submenu(
        pos: Pos2,
        ctx: &egui::Context,
        theme: EditorTheme,
        recent_items: &[PathBuf],
        prefix: &str,
        area_id: &str,
    ) -> Option<String> {
        let mut executed = None;
        egui::Area::new(egui::Id::new(area_id))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(theme.modal_bg())
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0_f32, theme.border()))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);
                        for path in recent_items.iter().take(10) {
                            let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().to_string());
                            let path_str = path.to_string_lossy().to_string();
                            let cmd = format!("{}{}", prefix, path_str);
                            Self::render_item(ui, &file_name, None, &cmd, &mut executed);
                        }
                    });
            });
        executed
    }

    fn render_item(
        ui: &mut egui::Ui,
        label: &str,
        shortcut: Option<&str>,
        command_id: &str,
        executed: &mut Option<String>,
    ) {
        let resp = egui::Frame::none()
            .rounding(Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label).size(12.5).color(Color32::from_rgb(0xd8, 0xde, 0xe9)));
                    if let Some(sc) = shortcut {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(sc).size(11.0).color(Color32::from_rgb(0x8a, 0x93, 0xa0)));
                        });
                    }
                });
            });

        if resp.response.interact(egui::Sense::click()).clicked() {
            *executed = Some(command_id.to_string());
        }
    }

    fn render_toggle_item(
        ui: &mut egui::Ui,
        label: &str,
        shortcut: Option<&str>,
        command_id: &str,
        is_checked: bool,
        executed: &mut Option<String>,
    ) {
        let resp = egui::Frame::none()
            .rounding(Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let check_mark = if is_checked { "✓ " } else { "   " };
                    ui.label(egui::RichText::new(check_mark).size(12.0).color(Color32::from_rgb(0xa6, 0xe2, 0x2e)).strong());
                    ui.label(egui::RichText::new(label).size(12.5).color(Color32::from_rgb(0xd8, 0xde, 0xe9)));

                    if let Some(sc) = shortcut {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(sc).size(11.0).color(Color32::from_rgb(0x8a, 0x93, 0xa0)));
                        });
                    }
                });
            });

        if resp.response.interact(egui::Sense::click()).clicked() {
            *executed = Some(command_id.to_string());
        }
    }

    fn render_separator(ui: &mut egui::Ui) {
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(2.0);
    }
}
