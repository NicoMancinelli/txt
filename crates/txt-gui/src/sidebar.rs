use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use egui::{self, Color32, Key, Rounding, Stroke, Vec2};
use txt_core::buffer::TextBuffer;
use txt_core::git_status::{GitFileStatus, GitStatusScanner};
use txt_core::project::{FileOps, ProjectManager, TreeNode};
use txt_core::watcher::{FileSystemWatcher, WatchSubscription};
use crate::theme::EditorTheme;

/// Active modal dialog state for filesystem actions in the sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarDialog {
    NewFile { parent: PathBuf, input: String },
    NewFolder { parent: PathBuf, input: String },
    Rename { target: PathBuf, input: String },
    DeleteConfirm { target: PathBuf, is_dir: bool },
    AddFolder { input: String },
}

/// Rich, Sublime Text-inspired project file browser and workspace explorer.
pub struct Sidebar {
    pub is_visible: bool,
    pub current_directory: PathBuf,
    pub project_manager: ProjectManager,
    pub git_statuses: HashMap<PathBuf, GitFileStatus>,
    #[allow(dead_code)]
    pub watcher: Arc<FileSystemWatcher>,
    pub watch_subscription: Option<WatchSubscription>,
    pub show_open_files: bool,
    pub search_filter: String,
    pub show_search_filter: bool,
    pub selected_path: Option<PathBuf>,
    pub dialog: Option<SidebarDialog>,
    #[allow(dead_code)]
    pub last_action_notification: Option<String>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    /// Creates a new Sidebar instance initialized with the current directory as primary root.
    pub fn new() -> Self {
        let current_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut project_manager = ProjectManager::new();
        project_manager.add_folder(current_directory.clone());

        let watcher = Arc::new(FileSystemWatcher::new());
        watcher.watch_path(&current_directory);
        let watch_subscription = Some(watcher.subscribe());

        let mut sidebar = Self {
            is_visible: true,
            current_directory,
            project_manager,
            git_statuses: HashMap::new(),
            watcher,
            watch_subscription,
            show_open_files: true,
            search_filter: String::new(),
            show_search_filter: false,
            selected_path: None,
            dialog: None,
            last_action_notification: None,
        };

        sidebar.refresh();
        sidebar
    }

    /// Toggles sidebar visibility.
    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    /// Toggles the "OPEN FILES" section in the sidebar.
    pub fn toggle_open_files(&mut self) {
        self.show_open_files = !self.show_open_files;
    }

    /// Toggles hidden files visibility (.git, target/, .DS_Store, etc.).
    pub fn toggle_show_hidden(&mut self) {
        self.project_manager.toggle_show_hidden();
        self.refresh();
    }

    /// Expands all folders in the project tree.
    pub fn expand_all(&mut self) {
        for folder in &mut self.project_manager.folders {
            folder.expand_all();
        }
    }

    /// Collapses all folders in the project tree.
    pub fn collapse_all(&mut self) {
        for folder in &mut self.project_manager.folders {
            folder.collapse_all();
        }
    }

    /// Reveals a given path in the project explorer, expanding parent folders.
    pub fn reveal_path(&mut self, path: &Path) -> bool {
        let revealed = self.project_manager.reveal_path(path);
        if revealed {
            self.selected_path = Some(path.to_path_buf());
        }
        revealed
    }

    /// Adds a folder to the project workspace and starts watching it.
    pub fn add_folder_to_project(&mut self, path: PathBuf) {
        self.watcher.watch_path(&path);
        self.project_manager.add_folder(path);
        self.refresh();
    }

    /// Removes a root folder from the project workspace.
    pub fn remove_folder_from_project(&mut self, path: &Path) {
        self.project_manager.remove_folder(path);
        self.refresh();
    }

    /// Re-scans all project folders and git working tree status.
    pub fn refresh(&mut self) {
        let filter_opt = if self.search_filter.trim().is_empty() {
            None
        } else {
            Some(self.search_filter.trim())
        };

        self.project_manager.refresh_all(filter_opt);

        // Refresh Git status for all project roots
        self.git_statuses.clear();
        for folder in &self.project_manager.folders {
            let statuses = GitStatusScanner::scan_repository(&folder.root_path);
            for (path, status) in statuses {
                self.git_statuses.insert(path, status);
            }
        }
    }

    /// Renders the sidebar GUI with Open Files, multi-root file tree, context menus, and dialogs.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: EditorTheme,
        buffers: &[TextBuffer],
        active_tab: &mut usize,
        on_close_tab: &mut Option<usize>,
        on_open_file: &mut Option<PathBuf>,
    ) {
        if !self.is_visible {
            return;
        }

        // 1. Drain background file watcher events
        if let Some(sub) = &self.watch_subscription {
            let mut changed = false;
            while let Ok(_event) = sub.try_recv() {
                changed = true;
            }
            if changed {
                self.refresh();
            }
        }

        let active_file_path = buffers.get(*active_tab).and_then(|b| b.file_path().map(|p| p.to_path_buf()));

        egui::Frame::none()
            .fill(theme.sidebar_bg())
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.set_width(220.0);

                // --- Top Header Toolbar & Actions ---
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("EXPLORER")
                            .color(Color32::from_rgb(0x7a, 0x82, 0x8e))
                            .size(11.0)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Refresh button
                        let refresh_btn = ui.small_button("↻");
                        if refresh_btn.on_hover_text("Refresh Folders & Git Status").clicked() {
                            self.refresh();
                        }

                        // Toggle Hidden files button
                        let hidden_btn = ui.small_button(if self.project_manager.show_hidden { "👁" } else { "🕶" });
                        if hidden_btn.on_hover_text("Toggle Hidden / Ignored Files").clicked() {
                            self.toggle_show_hidden();
                        }

                        // Collapse All button
                        let collapse_btn = ui.small_button("⤡");
                        if collapse_btn.on_hover_text("Collapse All Folders").clicked() {
                            self.collapse_all();
                        }

                        // Expand All button
                        let expand_btn = ui.small_button("⤢");
                        if expand_btn.on_hover_text("Expand All Folders").clicked() {
                            self.expand_all();
                        }

                        // New item button
                        let new_btn = ui.small_button("+");
                        if new_btn.on_hover_text("New File...").clicked() {
                            self.dialog = Some(SidebarDialog::NewFile {
                                parent: self.current_directory.clone(),
                                input: String::new(),
                            });
                        }

                        // Search filter button
                        let filter_btn = ui.small_button("🔍");
                        if filter_btn.on_hover_text("Filter Files").clicked() {
                            self.show_search_filter = !self.show_search_filter;
                            if !self.show_search_filter {
                                self.search_filter.clear();
                                self.refresh();
                            }
                        }
                    });
                });

                // --- Live Search Filter Bar ---
                if self.show_search_filter {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔎").size(10.0));
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.search_filter)
                                .hint_text("Filter files...")
                                .desired_width(ui.available_width() - 20.0),
                        );
                        if resp.changed() {
                            self.refresh();
                        }
                        if ui.small_button("✕").clicked() {
                            self.search_filter.clear();
                            self.refresh();
                        }
                    });
                }

                ui.add_space(6.0);

                // --- Section 1: OPEN FILES ---
                if self.show_open_files && !buffers.is_empty() {
                    egui::CollapsingHeader::new(
                        egui::RichText::new("OPEN FILES")
                            .color(Color32::from_rgb(0x7a, 0x82, 0x8e))
                            .size(11.0)
                            .strong(),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for (i, buf) in buffers.iter().enumerate() {
                            let is_active = i == *active_tab;
                            let bg = if is_active {
                                theme.selection_bg()
                            } else {
                                Color32::TRANSPARENT
                            };

                            let frame = egui::Frame::none()
                                .fill(bg)
                                .rounding(Rounding::same(3.0))
                                .inner_margin(egui::Margin::symmetric(6.0, 3.0));

                            let resp = frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if buf.is_dirty() {
                                        ui.label(egui::RichText::new("●").color(Color32::from_rgb(0xf9, 0x91, 0x57)).size(8.0));
                                    }
                                    let icon = Self::file_icon(Path::new(&buf.file_name()));
                                    ui.label(
                                        egui::RichText::new(format!("{} {}", icon, buf.file_name()))
                                            .color(if is_active { Color32::WHITE } else { Color32::from_rgb(0xd8, 0xde, 0xe9) })
                                            .size(12.0),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("×").clicked() {
                                            *on_close_tab = Some(i);
                                        }
                                    });
                                });
                            });

                            if resp.response.clicked() {
                                *active_tab = i;
                            }

                            // Right-click context menu on open tab
                            resp.response.context_menu(|ui| {
                                if ui.button("Close Tab").clicked() {
                                    *on_close_tab = Some(i);
                                    ui.close_menu();
                                }
                                if let Some(path) = buf.file_path() {
                                    if ui.button("Copy File Path").clicked() {
                                        ui.output_mut(|o| o.copied_text = path.to_string_lossy().to_string());
                                        ui.close_menu();
                                    }
                                    if ui.button("Reveal in Side Bar").clicked() {
                                        self.reveal_path(path);
                                        ui.close_menu();
                                    }
                                    if ui.button("Reveal in Finder / Explorer").clicked() {
                                        reveal_in_file_manager(path);
                                        ui.close_menu();
                                    }
                                }
                            });
                        }
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                }

                // --- Section 2: FOLDERS Project Tree ---
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut folder_to_remove = None;
                        let mut dialog_to_open = None;

                        for (f_idx, folder) in self.project_manager.folders.iter_mut().enumerate() {
                            let root_label = format!("📁 {}", folder.name);
                            let is_root_expanded = folder.root_node.is_expanded;

                            let header_resp = ui.horizontal(|ui| {
                                let arrow = if is_root_expanded { "▾" } else { "▸" };
                                let toggle_resp = ui.selectable_label(false, egui::RichText::new(arrow).size(10.0));
                                if toggle_resp.clicked() {
                                    folder.root_node.is_expanded = !folder.root_node.is_expanded;
                                }

                                let label_resp = ui.label(
                                    egui::RichText::new(&root_label)
                                        .color(Color32::from_rgb(0xd8, 0xde, 0xe9))
                                        .strong()
                                        .size(12.0),
                                );
                                label_resp
                            });

                            // Right-click context menu on root folder
                            header_resp.inner.context_menu(|ui| {
                                if ui.button("📄 New File...").clicked() {
                                    dialog_to_open = Some(SidebarDialog::NewFile {
                                        parent: folder.root_path.clone(),
                                        input: String::new(),
                                    });
                                    ui.close_menu();
                                }
                                if ui.button("📁 New Folder...").clicked() {
                                    dialog_to_open = Some(SidebarDialog::NewFolder {
                                        parent: folder.root_path.clone(),
                                        input: String::new(),
                                    });
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("📋 Copy Path").clicked() {
                                    ui.output_mut(|o| o.copied_text = folder.root_path.to_string_lossy().to_string());
                                    ui.close_menu();
                                }
                                if ui.button("🔍 Reveal in Finder / Explorer").clicked() {
                                    reveal_in_file_manager(&folder.root_path);
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("❌ Remove Folder from Project").clicked() {
                                    folder_to_remove = Some(f_idx);
                                    ui.close_menu();
                                }
                            });

                            if is_root_expanded {
                                for child in &mut folder.root_node.children {
                                    Self::render_tree_node(
                                        ui,
                                        theme,
                                        child,
                                        1,
                                        active_file_path.as_deref(),
                                        self.selected_path.as_deref(),
                                        &self.git_statuses,
                                        on_open_file,
                                        &mut dialog_to_open,
                                    );
                                }
                            }
                            ui.add_space(4.0);
                        }

                        if let Some(idx) = folder_to_remove {
                            if let Some(folder) = self.project_manager.folders.get(idx) {
                                let root = folder.root_path.clone();
                                self.remove_folder_from_project(&root);
                            }
                        }

                        if let Some(d) = dialog_to_open {
                            self.dialog = Some(d);
                        }
                    });
            });

        // 3. Render Modal Dialogs (New File, New Folder, Rename, Delete Confirmation)
        self.render_dialog_overlay(ui.ctx(), theme, on_open_file);
    }

    /// Recursively renders a TreeNode (file or directory) with git status, icons, and context menus.
    fn render_tree_node(
        ui: &mut egui::Ui,
        theme: EditorTheme,
        node: &mut TreeNode,
        depth: usize,
        active_file: Option<&Path>,
        selected_path: Option<&Path>,
        git_statuses: &HashMap<PathBuf, GitFileStatus>,
        on_open_file: &mut Option<PathBuf>,
        dialog_to_open: &mut Option<SidebarDialog>,
    ) {
        let is_current = active_file.map_or(false, |af| af == node.path);
        let is_selected = selected_path.map_or(false, |sp| sp == node.path);

        let bg = if is_current || is_selected {
            theme.selection_bg()
        } else {
            Color32::TRANSPARENT
        };

        let indent = (depth as f32) * 12.0;

        let frame = egui::Frame::none()
            .fill(bg)
            .rounding(Rounding::same(3.0))
            .inner_margin(egui::Margin::symmetric(4.0, 2.0));

        let resp = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(indent);

                if node.is_dir {
                    let arrow = if node.is_expanded { "▾" } else { "▸" };
                    ui.label(egui::RichText::new(arrow).size(10.0).color(Color32::from_rgb(0x8a, 0x92, 0x9e)));
                    ui.label(
                        egui::RichText::new(format!("📁 {}", node.name))
                            .color(if is_current { Color32::WHITE } else { Color32::from_rgb(0xd8, 0xde, 0xe9) })
                            .size(12.0),
                    );
                } else {
                    let icon = Self::file_icon(&node.path);
                    let git_status = git_statuses.get(&node.path).copied().unwrap_or(GitFileStatus::Clean);

                    let (r, g, b) = git_status.color_rgb();
                    let name_color = if is_current {
                        Color32::WHITE
                    } else if git_status.is_dirty() {
                        Color32::from_rgb(r, g, b)
                    } else {
                        Color32::from_rgb(0xd8, 0xde, 0xe9)
                    };

                    ui.label(
                        egui::RichText::new(format!("{} {}", icon, node.name))
                            .color(name_color)
                            .size(12.0),
                    );

                    // Git status badge indicator (M, U, A, D) on right side
                    if git_status.is_dirty() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(git_status.badge())
                                    .color(Color32::from_rgb(r, g, b))
                                    .size(10.0)
                                    .strong(),
                            );
                        });
                    }
                }
            });
        });

        // Hover tooltip with file size
        let kb = (node.size_bytes as f64) / 1024.0;
        let tooltip_text = if node.is_dir {
            format!("Folder: {}\nItems: {}", node.path.display(), node.children.len())
        } else {
            format!("File: {}\nSize: {:.1} KB", node.path.display(), kb)
        };
        resp.response.clone().on_hover_text(tooltip_text);

        // Click handler
        if resp.response.clicked() {
            if node.is_dir {
                node.is_expanded = !node.is_expanded;
            } else {
                *on_open_file = Some(node.path.clone());
            }
        }

        // Context Menu
        let node_path = node.path.clone();
        let is_dir = node.is_dir;
        let file_name = node.name.clone();

        resp.response.context_menu(|ui| {
            if is_dir {
                if ui.button("📄 New File...").clicked() {
                    *dialog_to_open = Some(SidebarDialog::NewFile {
                        parent: node_path.clone(),
                        input: String::new(),
                    });
                    ui.close_menu();
                }
                if ui.button("📁 New Folder...").clicked() {
                    *dialog_to_open = Some(SidebarDialog::NewFolder {
                        parent: node_path.clone(),
                        input: String::new(),
                    });
                    ui.close_menu();
                }
                ui.separator();
            }

            if ui.button("✏ Rename...").clicked() {
                *dialog_to_open = Some(SidebarDialog::Rename {
                    target: node_path.clone(),
                    input: file_name.clone(),
                });
                ui.close_menu();
            }

            if ui.button("📑 Duplicate").clicked() {
                let _ = FileOps::duplicate(&node_path);
                ui.close_menu();
            }

            ui.separator();

            if ui.button("📋 Copy Path").clicked() {
                ui.output_mut(|o| o.copied_text = node_path.to_string_lossy().to_string());
                ui.close_menu();
            }

            if ui.button("🔍 Reveal in Finder / Explorer").clicked() {
                reveal_in_file_manager(&node_path);
                ui.close_menu();
            }

            ui.separator();

            if ui.button("🗑 Delete...").clicked() {
                *dialog_to_open = Some(SidebarDialog::DeleteConfirm {
                    target: node_path.clone(),
                    is_dir,
                });
                ui.close_menu();
            }
        });

        // Recursively render children if directory is expanded
        if node.is_dir && node.is_expanded {
            for child in &mut node.children {
                Self::render_tree_node(
                    ui,
                    theme,
                    child,
                    depth + 1,
                    active_file,
                    selected_path,
                    git_statuses,
                    on_open_file,
                    dialog_to_open,
                );
            }
        }
    }

    /// Renders modal popups for filesystem actions (New File, New Folder, Rename, Delete).
    fn render_dialog_overlay(
        &mut self,
        ctx: &egui::Context,
        theme: EditorTheme,
        on_open_file: &mut Option<PathBuf>,
    ) {
        let mut close_dialog = false;
        let mut executed_action = false;
        let mut folder_to_add = None;

        if let Some(dialog) = &mut self.dialog {
            let (title, confirm_label) = match dialog {
                SidebarDialog::NewFile { .. } => ("New File", "Create"),
                SidebarDialog::NewFolder { .. } => ("New Folder", "Create"),
                SidebarDialog::Rename { .. } => ("Rename Item", "Rename"),
                SidebarDialog::DeleteConfirm { .. } => ("Confirm Deletion", "Delete"),
                SidebarDialog::AddFolder { .. } => ("Add Folder to Project", "Add"),
            };

            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(egui::Frame::window(&ctx.style()).fill(theme.modal_bg()).stroke(Stroke::new(1.0_f32, theme.border())))
                .show(ctx, |ui| {
                    ui.set_min_width(320.0);

                    match dialog {
                        SidebarDialog::NewFile { parent, input } => {
                            ui.label(egui::RichText::new(format!("Location: {}", parent.display())).size(11.0).color(Color32::from_rgb(0xa0, 0xa8, 0xb6)));
                            ui.add_space(4.0);
                            let text_input = ui.add(
                                egui::TextEdit::singleline(input)
                                    .hint_text("e.g. style.css, utils.rs")
                                    .desired_width(ui.available_width()),
                            );
                            text_input.request_focus();

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                let enter = ui.input(|i| i.key_pressed(Key::Enter));
                                if (ui.button(confirm_label).clicked() || enter) && !input.trim().is_empty() {
                                    let new_file_path = parent.join(input.trim());
                                    if FileOps::create_file(&new_file_path).is_ok() {
                                        *on_open_file = Some(new_file_path);
                                        executed_action = true;
                                    }
                                    close_dialog = true;
                                }
                                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                                    close_dialog = true;
                                }
                            });
                        }
                        SidebarDialog::NewFolder { parent, input } => {
                            ui.label(egui::RichText::new(format!("Parent Folder: {}", parent.display())).size(11.0).color(Color32::from_rgb(0xa0, 0xa8, 0xb6)));
                            ui.add_space(4.0);
                            let text_input = ui.add(
                                egui::TextEdit::singleline(input)
                                    .hint_text("Folder name")
                                    .desired_width(ui.available_width()),
                            );
                            text_input.request_focus();

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                let enter = ui.input(|i| i.key_pressed(Key::Enter));
                                if (ui.button(confirm_label).clicked() || enter) && !input.trim().is_empty() {
                                    let new_dir_path = parent.join(input.trim());
                                    let _ = FileOps::create_dir(&new_dir_path);
                                    executed_action = true;
                                    close_dialog = true;
                                }
                                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                                    close_dialog = true;
                                }
                            });
                        }
                        SidebarDialog::Rename { target, input } => {
                            ui.label(egui::RichText::new(format!("Target: {}", target.display())).size(11.0).color(Color32::from_rgb(0xa0, 0xa8, 0xb6)));
                            ui.add_space(4.0);
                            let text_input = ui.add(
                                egui::TextEdit::singleline(input)
                                    .hint_text("New name")
                                    .desired_width(ui.available_width()),
                            );
                            text_input.request_focus();

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                let enter = ui.input(|i| i.key_pressed(Key::Enter));
                                if (ui.button(confirm_label).clicked() || enter) && !input.trim().is_empty() {
                                    if let Some(parent) = target.parent() {
                                        let new_path = parent.join(input.trim());
                                        let _ = FileOps::rename(target, &new_path);
                                        executed_action = true;
                                    }
                                    close_dialog = true;
                                }
                                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                                    close_dialog = true;
                                }
                            });
                        }
                        SidebarDialog::DeleteConfirm { target, is_dir } => {
                            let item_type = if *is_dir { "folder (and all its contents)" } else { "file" };
                            ui.label(
                                egui::RichText::new(format!("Are you sure you want to permanently delete this {}?", item_type))
                                    .color(Color32::from_rgb(0xf9, 0x91, 0x57))
                                    .size(12.0),
                            );
                            ui.label(egui::RichText::new(target.display().to_string()).size(11.0).color(Color32::WHITE));

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                let enter = ui.input(|i| i.key_pressed(Key::Enter));
                                if ui.button(egui::RichText::new("Delete Permanently").color(Color32::from_rgb(0xec, 0x5f, 0x67))).clicked() || enter {
                                    let _ = FileOps::delete(target);
                                    executed_action = true;
                                    close_dialog = true;
                                }
                                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                                    close_dialog = true;
                                }
                            });
                        }
                        SidebarDialog::AddFolder { input } => {
                            ui.label(egui::RichText::new("Enter absolute or relative path to project folder:").size(11.0).color(Color32::from_rgb(0xa0, 0xa8, 0xb6)));
                            ui.add_space(4.0);
                            let text_input = ui.add(
                                egui::TextEdit::singleline(input)
                                    .hint_text("/path/to/folder")
                                    .desired_width(ui.available_width()),
                            );
                            text_input.request_focus();

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                let enter = ui.input(|i| i.key_pressed(Key::Enter));
                                if (ui.button(confirm_label).clicked() || enter) && !input.trim().is_empty() {
                                    let path = PathBuf::from(input.trim());
                                    if path.exists() && path.is_dir() {
                                        folder_to_add = Some(path);
                                        executed_action = true;
                                    }
                                    close_dialog = true;
                                }
                                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                                    close_dialog = true;
                                }
                            });
                        }
                    }
                });
        }

        if let Some(folder_path) = folder_to_add {
            self.add_folder_to_project(folder_path);
        }

        if executed_action {
            self.refresh();
        }

        if close_dialog {
            self.dialog = None;
        }
    }

    /// File extension icon mapping matching modern editor aesthetics.
    pub fn file_icon(path: &Path) -> &'static str {
        match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
            "rs" => "🦀",
            "toml" => "⚙",
            "md" | "markdown" => "📝",
            "json" => "📦",
            "py" => "🐍",
            "js" | "mjs" | "cjs" => "🟨",
            "ts" | "tsx" => "🔷",
            "html" | "htm" => "🌐",
            "css" | "scss" | "sass" => "🎨",
            "c" | "h" => "🇨",
            "cpp" | "hpp" | "cc" => "➕",
            "go" => "🐹",
            "sh" | "bash" | "zsh" => "🐚",
            "sql" => "🗄",
            "yaml" | "yml" => "📋",
            "txt" => "📄",
            "png" | "jpg" | "jpeg" | "gif" | "svg" => "🖼",
            _ => "📄",
        }
    }
}

/// Helper to reveal a path in the platform's native file explorer (Finder, Explorer, or xdg-open).
pub fn reveal_in_file_manager(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(format!("/select,\"{}\"", path.display())).spawn();
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let dir = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
    }
}
