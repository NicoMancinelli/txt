use std::path::PathBuf;
use std::sync::Arc;
use egui::{self, Key};
use uuid::Uuid;
use txt_core::buffer::TextBuffer;
use txt_core::command::{CommandDescriptor, CommandRegistry};
use txt_core::cursor::{CursorPosition, Selection, SelectionSet};
use txt_core::lsp::LspClientState;
use txt_core::plugin::{AutoClosingBracketsPlugin, PluginManager, WordCountPlugin};
use txt_core::search::{ProjectSearchEngine, SearchQuery};
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
    pub find_replace: crate::find_replace_view::FindReplacePanel,
    pub registry: CommandRegistry,
    pub sync_engine: Arc<SyncEngine>,
    pub cloud_adapter: Arc<CloudSyncAdapter>,
    pub current_document_id: Uuid,
    pub document_revision: u64,
    pub plugin_manager: Arc<PluginManager>,
    pub lsp_state: Arc<LspClientState>,
    pub last_notification: Option<String>,
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

        let initial_doc_id = Uuid::new_v4();
        sync_engine.init_crdt_buffer(initial_doc_id, "local_peer", &initial_buffer.text());

        let plugin_manager = Arc::new(PluginManager::new());
        plugin_manager.register(Arc::new(WordCountPlugin::new()));
        plugin_manager.register(Arc::new(AutoClosingBracketsPlugin::new()));
        plugin_manager.on_modified(&initial_buffer);

        let lsp_state = Arc::new(LspClientState::new());

        let mut app = Self {
            theme: EditorTheme::Mariana,
            buffers: vec![initial_buffer],
            selections: vec![SelectionSet::new()],
            active_tab: 0,
            sidebar: Sidebar::new(),
            editor_view: EditorView::new(),
            palette: CommandPalette::new(),
            find_replace: crate::find_replace_view::FindReplacePanel::new(),
            registry: CommandRegistry::new(),
            sync_engine,
            cloud_adapter,
            current_document_id: initial_doc_id,
            document_revision: 1,
            plugin_manager,
            lsp_state,
            last_notification: None,
        };

        // Restore persistent session if present
        let session_path = txt_core::SessionManager::default_session_path();
        if let Ok(saved_session) = txt_core::SessionManager::load_session(&session_path) {
            let mut restored_buffers = Vec::new();
            let mut restored_selections = Vec::new();
            let mut active_idx = 0;

            for file_path in &saved_session.open_files {
                if let Ok(buf) = TextBuffer::from_file(file_path) {
                    if Some(file_path) == saved_session.active_file.as_ref() {
                        active_idx = restored_buffers.len();
                    }
                    restored_buffers.push(buf);
                    restored_selections.push(SelectionSet::new());
                }
            }

            if !restored_buffers.is_empty() {
                app.buffers = restored_buffers;
                app.selections = restored_selections;
                app.active_tab = active_idx.min(app.buffers.len().saturating_sub(1));
            }

            app.sidebar.is_visible = saved_session.sidebar_visible;
            app.editor_view.font_size = saved_session.font_size;
            app.editor_view.word_wrap = saved_session.word_wrap;
            if saved_session.theme == "Monokai" {
                app.theme = EditorTheme::Monokai;
            } else {
                app.theme = EditorTheme::Mariana;
            }
        }

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
        self.registry.register(
            CommandDescriptor::new("edit.duplicate_line", "Line: Duplicate", "Edit")
                .with_shortcut("Cmd+Shift+D")
                .with_description("Duplicates the current line down"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("edit.delete_line", "Line: Delete", "Edit")
                .with_shortcut("Cmd+Shift+K")
                .with_description("Deletes the current line"),
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
            CommandDescriptor::new("view.toggle_open_files", "View: Toggle Open Files in Side Bar", "View")
                .with_description("Shows or hides the Open Files section in the sidebar"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("view.toggle_word_wrap", "Word Wrap: Toggle", "View")
                .with_shortcut("Option+Z")
                .with_description("Toggles soft word wrapping in the editor view"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("project.refresh_folders", "Project: Refresh Folders", "Project")
                .with_description("Refreshes project directory and file tree in sidebar"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("project.collapse_all", "Project: Collapse All Folders", "Project")
                .with_description("Recursively collapses all project folders in sidebar"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("project.expand_all", "Project: Expand All Folders", "Project")
                .with_description("Recursively expands all project folders in sidebar"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("project.toggle_hidden", "Project: Toggle Hidden / Ignored Files", "Project")
                .with_description("Toggles visibility of hidden and ignored files (.git, target/, etc.)"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("project.add_folder", "Project: Add Folder to Project...", "Project")
                .with_description("Adds an external directory root to the project workspace"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("file.reveal_in_sidebar", "File: Reveal in Side Bar", "File")
                .with_shortcut("Cmd+Option+R")
                .with_description("Reveals and selects active buffer file in sidebar tree"),
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

        self.registry.register(
            CommandDescriptor::new("view.toggle_minimap", "Toggle Minimap", "View")
                .with_description("Shows or hides the skyline minimap"),
            || {},
        );

        // Convert Case commands (Sublime Text modeled)
        self.registry.register(
            CommandDescriptor::new("convert.upper_case", "Convert Case: Upper Case", "Edit")
                .with_description("Converts selection or document to uppercase"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("convert.lower_case", "Convert Case: Lower Case", "Edit")
                .with_description("Converts selection or document to lowercase"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("convert.title_case", "Convert Case: Title Case", "Edit")
                .with_description("Converts text to Title Case"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("convert.snake_case", "Convert Case: snake_case", "Edit")
                .with_description("Converts identifiers to snake_case"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("convert.kebab_case", "Convert Case: kebab-case", "Edit")
                .with_description("Converts identifiers to kebab-case"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("convert.camel_case", "Convert Case: lowerCamelCase", "Edit")
                .with_description("Converts identifiers to lowerCamelCase"),
            || {},
        );

        // Indentation commands
        self.registry.register(
            CommandDescriptor::new("indent.convert_to_spaces", "Indentation: Convert to Spaces", "Indentation")
                .with_description("Converts all tabs to spaces"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("indent.convert_to_tabs", "Indentation: Convert to Tabs", "Indentation")
                .with_description("Converts spaces to tabs"),
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
        self.registry.register(
            CommandDescriptor::new("sync.crdt_delta", "Sync: Broadcast CRDT Delta", "Sync")
                .with_description("Generates and synchronizes deterministic CRDT patch across adapters"),
            || {},
        );

        // Find & Search
        self.registry.register(
            CommandDescriptor::new("find.find", "Find...", "Find")
                .with_shortcut("Cmd+F")
                .with_description("Opens the in-buffer find panel"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("find.replace", "Replace...", "Find")
                .with_shortcut("Cmd+Option+F")
                .with_description("Opens the in-buffer find and replace panel"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("find.next", "Find Next", "Find")
                .with_shortcut("Enter / F3")
                .with_description("Jumps to the next search match"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("find.prev", "Find Previous", "Find")
                .with_shortcut("Shift+Enter / Shift+F3")
                .with_description("Jumps to the previous search match"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("find.in_files", "Find in Files (Sublime Parallel Search)", "Find")
                .with_shortcut("Cmd+Shift+F")
                .with_description("Scans project files using 4096-item batch parallel workers"),
            || {},
        );

        // Goto commands
        self.registry.register(
            CommandDescriptor::new("goto.symbol", "Goto Symbol in File...", "Goto")
                .with_shortcut("Cmd+R")
                .with_description("Extracts and fuzzy searches document symbols"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("goto.line", "Goto Line...", "Goto")
                .with_shortcut("Ctrl+G")
                .with_description("Jumps cursor to a specified line number"),
            || {},
        );

        // Multi-cursor commands
        self.registry.register(
            CommandDescriptor::new("selection.add_next_occurrence", "Quick Add Next Occurrence", "Selection")
                .with_shortcut("Cmd+D")
                .with_description("Selects the next occurrence of current word or text"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("selection.select_all_occurrences", "Quick Find All Occurrences", "Selection")
                .with_shortcut("Cmd+Ctrl+G")
                .with_description("Selects all occurrences of current text across buffer"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("selection.split_into_lines", "Split Selection into Lines", "Selection")
                .with_shortcut("Cmd+Shift+L")
                .with_description("Splits multi-line selections into one cursor per line"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("selection.add_cursor_above", "Add Cursor Above", "Selection")
                .with_shortcut("Cmd+Option+Up")
                .with_description("Adds a new cursor one line above"),
            || {},
        );
        self.registry.register(
            CommandDescriptor::new("selection.add_cursor_below", "Add Cursor Below", "Selection")
                .with_shortcut("Cmd+Option+Down")
                .with_description("Adds a new cursor one line below"),
            || {},
        );

        // Plugins & Extensibility
        self.registry.register(
            CommandDescriptor::new("plugin.word_count", "Plugins: Word Count Statistics", "Plugins")
                .with_shortcut("Cmd+Option+C")
                .with_description("Displays live word, character, and line count from active plugin"),
            || {},
        );

        // LSP & Diagnostics
        self.registry.register(
            CommandDescriptor::new("lsp.status", "LSP: Show Server Status & Diagnostics", "Tools")
                .with_description("Inspects Language Server Protocol diagnostic cache and status"),
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
                    self.save_session();
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
            "edit.duplicate_line" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get_mut(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    let pos = buf.char_to_pos(sels.primary().head);
                    if let Some(line_slice) = buf.line(pos.line) {
                        let line_text = line_slice.to_string();
                        let next_line_start = buf.pos_to_char(CursorPosition::new(pos.line + 1, 0));
                        buf.insert(next_line_start, &line_text);
                    }
                }
            }
            "edit.delete_line" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get_mut(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    let pos = buf.char_to_pos(sels.primary().head);
                    let line_start = buf.pos_to_char(CursorPosition::new(pos.line, 0));
                    let line_end = buf.pos_to_char(CursorPosition::new(pos.line + 1, 0));
                    if line_end > line_start {
                        buf.delete(line_start..line_end);
                        sels.set_primary(Selection::point(line_start.min(buf.len_chars())));
                    }
                }
            }
            "view.toggle_sidebar" => {
                self.sidebar.toggle();
                self.save_session();
            }
            "view.toggle_open_files" => {
                self.sidebar.toggle_open_files();
            }
            "view.toggle_word_wrap" => {
                self.editor_view.word_wrap = !self.editor_view.word_wrap;
                self.save_session();
            }
            "project.refresh_folders" => {
                self.sidebar.refresh();
            }
            "project.collapse_all" => {
                self.sidebar.collapse_all();
            }
            "project.expand_all" => {
                self.sidebar.expand_all();
            }
            "project.toggle_hidden" => {
                self.sidebar.toggle_show_hidden();
            }
            "project.add_folder" => {
                self.sidebar.dialog = Some(crate::sidebar::SidebarDialog::AddFolder { input: String::new() });
                self.sidebar.is_visible = true;
            }
            "file.reveal_in_sidebar" => {
                if let Some(buf) = self.buffers.get(self.active_tab) {
                    if let Some(path) = buf.file_path() {
                        self.sidebar.reveal_path(path);
                        self.sidebar.is_visible = true;
                    }
                }
            }
            "view.theme_mariana" => {
                self.theme = EditorTheme::Mariana;
                self.save_session();
            }
            "view.theme_monokai" => {
                self.theme = EditorTheme::Monokai;
                self.save_session();
            }
            "sync.now" => {
                self.trigger_sync();
            }
            "sync.toggle_cloud" => {
                let current = self.cloud_adapter.status();
                self.cloud_adapter.set_online(current == SyncStatus::Offline);
            }
            "view.toggle_minimap" => {
                self.editor_view.show_minimap = !self.editor_view.show_minimap;
            }
            "convert.upper_case" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::to_uppercase(&text);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "convert.lower_case" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::to_lowercase(&text);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "convert.title_case" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::to_title_case(&text);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "convert.snake_case" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::to_snake_case(&text);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "convert.kebab_case" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::to_kebab_case(&text);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "convert.camel_case" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::to_camel_case(&text);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "indent.convert_to_spaces" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::convert_tabs_to_spaces(&text, self.editor_view.tab_size);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "indent.convert_to_tabs" => {
                if let Some(buf) = self.buffers.get_mut(self.active_tab) {
                    let text = buf.text();
                    let transformed = txt_core::transform::TextTransform::convert_spaces_to_tabs(&text, self.editor_view.tab_size);
                    buf.replace(0..buf.len_chars(), &transformed);
                }
            }
            "find.in_files" => {
                let search_engine = ProjectSearchEngine::new();
                let query = SearchQuery::new("fn");
                if let Ok(results) = search_engine.search_project(&self.sidebar.current_directory, &query) {
                    self.last_notification = Some(format!("Find in Files: {} occurrences discovered", results.len()));
                }
            }
            "plugin.word_count" => {
                let candidates = self.plugin_manager.on_query_completions("stats");
                let stats = candidates.first().map(|c| c.insert_text.clone()).unwrap_or_else(|| "0 words".to_string());
                self.last_notification = Some(format!("Document Stats: {}", stats));
            }
            "lsp.status" => {
                let diag_count = self.lsp_state.get_diagnostics(std::path::Path::new("welcome.rs")).len();
                self.last_notification = Some(format!("LSP: Ready (Active Diagnostics: {})", diag_count));
            }
            "sync.crdt_delta" => {
                if let Ok(Some(patch)) = self.sync_engine.generate_crdt_patch(self.current_document_id) {
                    self.document_revision += 1;
                    let _ = self.sync_engine.sync_crdt_patch(&patch, self.document_revision);
                    self.last_notification = Some(format!("CRDT Patch synced ({} ops)", patch.ops.len()));
                } else {
                    self.last_notification = Some("CRDT: Buffer is up-to-date".to_string());
                }
            }
            "find.find" => {
                let initial_text = self.buffers.get(self.active_tab).and_then(|buf| {
                    self.selections.get(self.active_tab).and_then(|sels| {
                        let p = sels.primary();
                        if !p.is_empty() {
                            Some(buf.slice_text(p.min_char()..p.max_char()))
                        } else {
                            None
                        }
                    })
                });
                if let Some(buf) = self.buffers.get(self.active_tab) {
                    self.find_replace.open_find(initial_text, buf);
                }
            }
            "find.replace" => {
                let initial_text = self.buffers.get(self.active_tab).and_then(|buf| {
                    self.selections.get(self.active_tab).and_then(|sels| {
                        let p = sels.primary();
                        if !p.is_empty() {
                            Some(buf.slice_text(p.min_char()..p.max_char()))
                        } else {
                            None
                        }
                    })
                });
                if let Some(buf) = self.buffers.get(self.active_tab) {
                    self.find_replace.open_replace(initial_text, buf);
                }
            }
            "find.next" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    self.find_replace.next_match(buf, sels);
                    let head = sels.primary().head;
                    let pos = buf.char_to_pos(head);
                    self.editor_view.scroll_to_line(pos.line);
                }
            }
            "find.prev" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    self.find_replace.prev_match(buf, sels);
                    let head = sels.primary().head;
                    let pos = buf.char_to_pos(head);
                    self.editor_view.scroll_to_line(pos.line);
                }
            }
            "goto.symbol" => {
                if let Some(buf) = self.buffers.get(self.active_tab) {
                    let lang = self.active_language();
                    self.palette.open_symbols(&self.registry, buf, lang.display_name());
                }
            }
            "goto.line" => {
                if let Some(buf) = self.buffers.get(self.active_tab) {
                    self.palette.open_goto_line(&self.registry, buf);
                }
            }
            "selection.add_next_occurrence" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    txt_core::select_next_occurrence(buf, sels);
                }
            }
            "selection.select_all_occurrences" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    txt_core::select_all_occurrences(buf, sels);
                }
            }
            "selection.split_into_lines" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    txt_core::split_selection_into_lines(buf, sels);
                }
            }
            "selection.add_cursor_above" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    txt_core::add_cursor_above(buf, sels);
                }
            }
            "selection.add_cursor_below" => {
                if let (Some(buf), Some(sels)) = (
                    self.buffers.get(self.active_tab),
                    self.selections.get_mut(self.active_tab),
                ) {
                    txt_core::add_cursor_below(buf, sels);
                }
            }
            "tools.command_palette" => {
                self.palette.open(&self.registry);
            }
            _ => {}
        }
    }

    pub fn save_session(&self) {
        let open_files: Vec<PathBuf> = self
            .buffers
            .iter()
            .filter_map(|b| b.file_path().map(|p| p.to_path_buf()))
            .collect();

        let active_file = self
            .buffers
            .get(self.active_tab)
            .and_then(|b| b.file_path().map(|p| p.to_path_buf()));

        let theme_str = match self.theme {
            EditorTheme::Mariana => "Mariana".to_string(),
            EditorTheme::Monokai => "Monokai".to_string(),
        };

        let session_state = txt_core::SessionState {
            open_files,
            active_file,
            sidebar_visible: self.sidebar.is_visible,
            font_size: self.editor_view.font_size,
            theme: theme_str,
            word_wrap: self.editor_view.word_wrap,
        };

        let session_path = txt_core::SessionManager::default_session_path();
        let _ = txt_core::SessionManager::save_session(&session_path, &session_state);
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
            self.save_session();
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if self.buffers.len() > 1 {
            self.buffers.remove(idx);
            self.selections.remove(idx);
            if self.active_tab >= self.buffers.len() {
                self.active_tab = self.buffers.len().saturating_sub(1);
            }
            self.save_session();
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
        let alt = ctx.input(|i| i.modifiers.alt);

        if cmd && shift && ctx.input(|i| i.key_pressed(Key::P)) {
            self.palette.open(&self.registry);
        } else if cmd && !shift && ctx.input(|i| i.key_pressed(Key::R)) {
            self.execute_command("goto.symbol");
        } else if (cmd && !shift && ctx.input(|i| i.key_pressed(Key::G)))
            || (ctx.input(|i| i.modifiers.ctrl) && ctx.input(|i| i.key_pressed(Key::G)))
        {
            self.execute_command("goto.line");
        } else if cmd && !shift && !alt && ctx.input(|i| i.key_pressed(Key::F)) {
            self.execute_command("find.find");
        } else if (cmd && alt && ctx.input(|i| i.key_pressed(Key::F)))
            || (cmd && !shift && ctx.input(|i| i.key_pressed(Key::H)))
        {
            self.execute_command("find.replace");
        } else if ctx.input(|i| i.key_pressed(Key::F3)) {
            if shift {
                self.execute_command("find.prev");
            } else {
                self.execute_command("find.next");
            }
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
        } else if cmd && shift && ctx.input(|i| i.key_pressed(Key::F)) {
            self.execute_command("find.in_files");
        } else if cmd && shift && ctx.input(|i| i.key_pressed(Key::D)) {
            self.execute_command("edit.duplicate_line");
        } else if cmd && shift && ctx.input(|i| i.key_pressed(Key::K)) {
            self.execute_command("edit.delete_line");
        } else if cmd && alt && ctx.input(|i| i.key_pressed(Key::R)) {
            self.execute_command("file.reveal_in_sidebar");
        }

        // Handle Command Palette execution (with symbol & line jumping)
        let active_lang = self.active_language();
        let lang_str = active_lang.display_name();
        let active_buf = self.buffers.get(self.active_tab);

        if let Some(action) = self.palette.show(ctx, self.theme, &self.registry, active_buf, lang_str) {
            match action {
                crate::palette::PaletteAction::ExecuteCommand(cmd_id) => {
                    self.execute_command(&cmd_id);
                }
                crate::palette::PaletteAction::GotoPosition { line, column } => {
                    if let (Some(buf), Some(sels)) = (
                        self.buffers.get_mut(self.active_tab),
                        self.selections.get_mut(self.active_tab),
                    ) {
                        let char_offset = buf.pos_to_char(CursorPosition::new(line, column));
                        sels.set_primary(Selection::point(char_offset));
                        self.editor_view.scroll_to_line(line);
                    }
                }
            }
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
                    StatusBar::show(
                        ui,
                        self.theme,
                        buf,
                        sels,
                        lang,
                        sync_status,
                        self.last_notification.as_deref(),
                    );
                }
            });

        // Find & Replace Bottom Overlay Panel
        if self.find_replace.is_open {
            egui::TopBottomPanel::bottom("find_replace_bottom_panel")
                .frame(egui::Frame::none().fill(self.theme.status_bar_bg()))
                .show(ctx, |ui| {
                    if let (Some(buf), Some(sels)) = (
                        self.buffers.get_mut(self.active_tab),
                        self.selections.get_mut(self.active_tab),
                    ) {
                        self.find_replace.show(ui, self.theme, buf, sels);
                    }
                });
        }

        let mut opened_file = None;
        let mut close_tab_from_sidebar = None;
        if self.sidebar.is_visible {
            egui::SidePanel::left("left_sidebar")
                .frame(egui::Frame::none().fill(self.theme.sidebar_bg()))
                .resizable(true)
                .default_width(200.0)
                .width_range(140.0..=360.0)
                .show(ctx, |ui| {
                    self.sidebar.show(
                        ui,
                        self.theme,
                        &self.buffers,
                        &mut self.active_tab,
                        &mut close_tab_from_sidebar,
                        &mut opened_file,
                    );
                });
        }

        if let Some(idx) = close_tab_from_sidebar {
            self.close_tab(idx);
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
                    self.editor_view.show(
                        ui,
                        self.theme,
                        buf,
                        sels,
                        lang,
                        &self.find_replace.matches,
                        self.find_replace.active_match_index,
                    );
                }
            });
    }
}
