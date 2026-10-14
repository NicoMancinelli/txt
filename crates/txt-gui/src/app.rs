use std::path::PathBuf;
use std::sync::Arc;
use egui::{self, Key};
use uuid::Uuid;
use txt_core::buffer::TextBuffer;
use txt_core::command::{CommandDescriptor, CommandRegistry};
use txt_core::cursor::{Selection, SelectionSet};
use txt_core::syntax::Language;
use txt_sync::adapter::{SyncAdapter, SyncStatus};
use txt_sync::cloud_adapter::{CloudConfig, CloudSyncAdapter};
use txt_sync::engine::SyncEngine;
use txt_sync::local_adapter::LocalFilesystemAdapter;
use txt_sync::memory_adapter::MemorySyncAdapter;

use crate::editor_view::EditorView;
use crate::palette::CommandPalette;
use crate::sidebar::Sidebar;
use crate::status_bar::StatusBar;
use crate::tabs::TabBar;
use crate::theme::EditorTheme;

pub struct TxtEditorApp {
    pub theme: EditorTheme,
    pub buffers: Vec<TextBuffer>,
    pub selections: Vec<SelectionSet>,
    pub active_tab: usize,
    pub sidebar: Sidebar,
    pub editor_view: EditorView,
    pub palette: CommandPalette,
    pub registry: CommandRegistry,
    pub sync_engine: Arc<SyncEngine>,
    pub cloud_adapter: Arc<CloudSyncAdapter>,
    pub current_document_id: Uuid,
    pub document_revision: u64,
}

impl Default for TxtEditorApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TxtEditorApp {
    pub fn new() -> Self {
        let mut initial_buffer = TextBuffer::from_str(
            r#"// Welcome to txt — A high-performance text editor modeled after Sublime Text.
// Fast. Offline-first. Direct memory management.

fn main() {
    let message = "Sublime Text speed with Rust safety!";
    println!("{}", message);
}
"#,
        );
        initial_buffer.set_file_path("welcome.rs");

        let sync_engine = Arc::new(SyncEngine::new());

        // Register default memory and local adapters
        let mem_adapter = Arc::new(MemorySyncAdapter::new());
        sync_engine.add_adapter(mem_adapter);

        let local_dir = std::env::temp_dir().join("txt_sync_store");
        if let Ok(local_adapter) = LocalFilesystemAdapter::new(local_dir) {
            sync_engine.add_adapter(Arc::new(local_adapter));
        }

        let cloud_config = CloudConfig::default();
        let cloud_adapter = Arc::new(CloudSyncAdapter::new(cloud_config));
        sync_engine.add_adapter(cloud_adapter.clone());

        // Register client to enable background sync worker
        sync_engine.register_client();

        let mut app = Self {
            theme: EditorTheme::Mariana,
            buffers: vec![initial_buffer],
            selections: vec![SelectionSet::new()],
            active_tab: 0,
            sidebar: Sidebar::new(),
            editor_view: EditorView::new(),
            palette: CommandPalette::new(),
            registry: CommandRegistry::new(),
            sync_engine,
            cloud_adapter,
            current_document_id: Uuid::new_v4(),
            document_revision: 1,
        };

        app.register_built_in_commands();
        app
    }

    fn register_built_in_commands(&mut self) {
        // File commands
        self.registry.register(
            CommandDescriptor::new("file.new", "New File", "File")
                .with_shortcut("Cmd+N")
                .with_description("Creates a new empty text buffer"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("file.save", "Save", "File")
                .with_shortcut("Cmd+S")
                .with_description("Saves the active file to disk"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("file.close_tab", "Close File", "File")
                .with_shortcut("Cmd+W")
                .with_description("Closes the current tab"),
            || {},
        );

        // Edit commands
        self.registry.register(
            CommandDescriptor::new("edit.undo", "Undo", "Edit")
                .with_shortcut("Cmd+Z")
                .with_description("Undoes the previous change"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("edit.redo", "Redo", "Edit")
                .with_shortcut("Cmd+Shift+Z")
                .with_description("Redoes the undone change"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("edit.select_all", "Select All", "Edit")
                .with_shortcut("Cmd+A")
                .with_description("Selects all text in the current buffer"),
            || {},
        );

        // View commands
        self.registry.register(
            CommandDescriptor::new("view.toggle_sidebar", "Toggle Sidebar", "View")
                .with_shortcut("Cmd+K, Cmd+B")
                .with_description("Hides or shows the file navigation sidebar"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("view.theme_mariana", "Theme: Mariana", "Preferences")
                .with_description("Switch color scheme to Sublime Mariana"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("view.theme_monokai", "Theme: Monokai", "Preferences")
                .with_description("Switch color scheme to Sublime Monokai"),
            || {},
        );

        // Sync commands
        self.registry.register(
            CommandDescriptor::new("sync.now", "Sync Document Now", "Sync")
                .with_description("Pushes delta patches to sync adapters"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("sync.toggle_cloud", "Toggle Cloud Offline/Online", "Sync")
                .with_description("Switches cloud adapter state"),
            || {},
        );

        // Tools
        self.registry.register(
            CommandDescriptor::new("tools.command_palette", "Command Palette", "Tools")
                .with_shortcut("Cmd+Shift+P")
                .with_description("Opens the Sublime Text command palette"),
            || {},
        );
    }

    pub fn execute_command(&mut self, command_id: &str) {
        match command_id {
            "file.new" => {
                let mut new_buf = TextBuffer::new();
                new_buf.set_file_path("untitled");
                self.buffers.push(new_buf);
                self.selections.push(SelectionSet::new());
                self.active_tab = self.buffers.len().saturating_sub(1);
            }
            "file.save" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let _ = buf.save();
                }
            }
            "file.close_tab" => {
                self.close_tab(self.active_tab);
            }
            "edit.undo" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    buf.undo();
                }
            }
            "edit.redo" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    buf.redo();
                }
            }
            "edit.select_all" => {
                if let Some(buf) = self.buffers.get(self.active_tab) {
                    let total = buf.len_chars();
                    if let Some(sel) = self.selections.get_mut(self.active_tab) {
                        sel.set_primary(Selection::range(0, total));
                    }
                }
            }
            "view.toggle_sidebar" => {
                self.sidebar.toggle();
            }
            "view.theme_mariana" => {
                self.theme = EditorTheme::Mariana;
            }
            "view.theme_monokai" => {
                self.theme = EditorTheme::Monokai;
            }
            "sync.now" => {
                self.trigger_sync();
            }
            "sync.toggle_cloud" => {
                let current = self.cloud_adapter.status();
                self.cloud_adapter.set_online(current == SyncStatus::Offline);
            }
            "tools.command_palette" => {
                self.palette.open(&self.registry);
            }
            _ => {}
        }
    }

    pub fn trigger_sync(&mut self) {
        if let Some(buf) = self.buffers.get(self.active_tab) {
            self.document_revision += 1;
            let changes = vec![txt_core::history::Change::Insert {
                position: 0,
                text: buf.text(),
            }];
            let _ = self.sync_engine.sync_document_changes(
                self.current_document_id,
                self.document_revision,
                "local_author",
                changes,
            );
        }
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if let Ok(buf) = TextBuffer::from_file(&path) {
            self.buffers.push(buf);
            self.selections.push(SelectionSet::new());
            self.active_tab = self.buffers.len().saturating_sub(1);
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if self.buffers.len() > 1 {
            self.buffers.remove(idx);
            self.selections.remove(idx);
            if self.active_tab >= self.buffers.len() {
                self.active_tab = self.buffers.len().saturating_sub(1);
            }
        }
    }

    pub fn active_language(&self) -> Language {
        self.buffers
            .get(self.active_tab)
            .and_then(|b| b.file_path())
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .map(Language::from_extension)
            .unwrap_or(Language::Rust)
    }
}

impl eframe::App for TxtEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global keyboard shortcut dispatcher
        let cmd = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);

        if cmd && shift && ctx.input(|i| i.key_pressed(Key::P)) {
            self.palette.open(&self.registry);
        } else if cmd && ctx.input(|i| i.key_pressed(Key::S)) {
            self.execute_command("file.save");
        } else if cmd && ctx.input(|i| i.key_pressed(Key::N)) {
            self.execute_command("file.new");
        } else if cmd && ctx.input(|i| i.key_pressed(Key::W)) {
            self.execute_command("file.close_tab");
        } else if cmd && !shift && ctx.input(|i| i.key_pressed(Key::Z)) {
            self.execute_command("edit.undo");
        } else if (cmd && shift && ctx.input(|i| i.key_pressed(Key::Z)))
            || (cmd && ctx.input(|i| i.key_pressed(Key::Y)))
        {
            self.execute_command("edit.redo");
        } else if cmd && ctx.input(|i| i.key_pressed(Key::A)) {
            self.execute_command("edit.select_all");
        } else if cmd && ctx.input(|i| i.key_pressed(Key::B)) {
            self.execute_command("view.toggle_sidebar");
        }

        // Handle Command Palette execution
        if let Some(cmd_id) = self.palette.show(ctx, self.theme, &self.registry) {
            self.execute_command(&cmd_id);
        }

        // Main Layout: Top Tab Bar, Bottom Status Bar, Left Sidebar, Center Editor
        egui::TopBottomPanel::top("top_tab_bar")
            .frame(egui::Frame::none().fill(self.theme.tab_bar_bg()))
            .show(ctx, |ui| {
                let mut close_tab = None;
                TabBar::show(
                    ui,
                    self.theme,
                    &self.buffers,
                    &mut self.active_tab,
                    &mut close_tab,
                );
                if let Some(idx) = close_tab {
                    self.close_tab(idx);
                }
            });

        egui::TopBottomPanel::bottom("bottom_status_bar")
            .frame(egui::Frame::none().fill(self.theme.status_bar_bg()))
            .show(ctx, |ui| {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get(self.active_tab),
                ) {
                    let lang = self.active_language();
                    let sync_status = self.cloud_adapter.status();
                    StatusBar::show(ui, self.theme, buf, sels, lang, sync_status);
                }
            });

        let mut opened_file = None;
        if self.sidebar.is_visible {
            egui::SidePanel::left("left_sidebar")
                .frame(egui::Frame::none().fill(self.theme.sidebar_bg()))
                .resizable(true)
                .default_width(200.0)
                .width_range(140.0..=360.0)
                .show(ctx, |ui| {
                    self.sidebar.show(ui, self.theme, &mut opened_file);
                });
        }

        if let Some(path) = opened_file {
            self.open_file(path);
        }

        let lang = self.active_language();
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.bg()))
            .show(ctx, |ui| {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get_mut(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    self.editor_view.show(ui, self.theme, buf, sels, lang);
                }
            });
    }
}
