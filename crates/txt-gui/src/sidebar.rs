use std::path::{Path, PathBuf};
use egui::{self, Color32, Rounding};
use txt_core::buffer::TextBuffer;
use crate::theme::EditorTheme;

pub struct Sidebar {
    pub is_visible: bool,
    pub current_directory: PathBuf,
    pub show_open_files: bool,
    pub search_filter: String,
    pub show_search_filter: bool,
    pub new_file_dialog_open: bool,
    pub new_file_name: String,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        let current_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            is_visible: true,
            current_directory,
            show_open_files: true,
            search_filter: String::new(),
            show_search_filter: false,
            new_file_dialog_open: false,
            new_file_name: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn toggle_open_files(&mut self) {
        self.show_open_files = !self.show_open_files;
    }

    pub fn refresh(&mut self) {
        if let Ok(dir) = std::env::current_dir() {
            self.current_directory = dir;
        }
    }

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

        let active_file_path = buffers.get(*active_tab).and_then(|b| b.file_path().map(|p| p.to_path_buf()));

        egui::Frame::none()
            .fill(theme.sidebar_bg())
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.set_width(220.0);

                // Top Toolbar & Actions
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("EXPLORER")
                            .color(Color32::from_rgb(0x7a, 0x82, 0x8e))
                            .size(11.0)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Quick filter button
                        let filter_btn = ui.small_button("🔍");
                        if filter_btn.clicked() {
                            self.show_search_filter = !self.show_search_filter;
                            if !self.show_search_filter {
                                self.search_filter.clear();
                            }
                        }

                        // New file button
                        let new_btn = ui.small_button("+");
                        if new_btn.clicked() {
                            self.new_file_dialog_open = !self.new_file_dialog_open;
                            self.new_file_name.clear();
                        }
                    });
                });

                // Inline New File prompt
                if self.new_file_dialog_open {
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(theme.modal_bg())
                        .rounding(Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(6.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("New File Name:").size(10.0).color(Color32::from_rgb(0xfa, 0xc8, 0x63)));
                            let text_edit = egui::TextEdit::singleline(&mut self.new_file_name)
                                .hint_text("e.g. main.rs")
                                .desired_width(ui.available_width());
                            ui.add(text_edit);

                            ui.horizontal(|ui| {
                                if ui.small_button("Create").clicked() && !self.new_file_name.trim().is_empty() {
                                    let new_path = self.current_directory.join(self.new_file_name.trim());
                                    if !new_path.exists() {
                                        let _ = std::fs::write(&new_path, "");
                                    }
                                    *on_open_file = Some(new_path);
                                    self.new_file_dialog_open = false;
                                    self.new_file_name.clear();
                                }
                                if ui.small_button("Cancel").clicked() {
                                    self.new_file_dialog_open = false;
                                    self.new_file_name.clear();
                                }
                            });
                        });
                }

                // Quick file search filter bar
                if self.show_search_filter {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔎").size(10.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_filter)
                                .hint_text("Filter files...")
                                .desired_width(ui.available_width()),
                        );
                    });
                }

                ui.add_space(6.0);

                // Section 1: OPEN FILES (Sublime Text hallmark)
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
                                    ui.label(format!("{} {}", icon, buf.file_name()));

                                    if ui.small_button("×").clicked() {
                                        *on_close_tab = Some(i);
                                    }
                                });
                            });

                            if resp.response.clicked() {
                                *active_tab = i;
                            }
                        }
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                }

                // Section 2: FOLDERS Project Tree
                let root_name = self
                    .current_directory
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Project".to_string());

                egui::CollapsingHeader::new(format!("📁 {}", root_name))
                    .default_open(true)
                    .show(ui, |ui| {
                        Self::render_dir_contents(
                            ui,
                            theme,
                            &self.current_directory,
                            &self.search_filter,
                            active_file_path.as_deref(),
                            on_open_file,
                        );
                    });
            });
    }

    fn render_dir_contents(
        ui: &mut egui::Ui,
        theme: EditorTheme,
        dir: &Path,
        filter: &str,
        active_file: Option<&Path>,
        on_open_file: &mut Option<PathBuf>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut items: Vec<PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.file_name().map_or(false, |name| {
                        let s = name.to_string_lossy();
                        !s.starts_with('.') && s != "target"
                    })
                })
                .collect();

            // Directory first sorting
            items.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                if a_is_dir == b_is_dir {
                    a.cmp(b)
                } else if a_is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            for item in items.into_iter().take(50) {
                let file_name = item
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Apply quick filter if active
                if !filter.is_empty() && !item.is_dir() && !file_name.to_lowercase().contains(&filter.to_lowercase()) {
                    continue;
                }

                if item.is_dir() {
                    let sub_dir = item.clone();
                    egui::CollapsingHeader::new(format!("📁 {}", file_name))
                        .default_open(false)
                        .show(ui, |ui| {
                            Self::render_dir_contents(ui, theme, &sub_dir, filter, active_file, on_open_file);
                        });
                } else {
                    let icon = Self::file_icon(&item);
                    let is_current = active_file.map_or(false, |af| af == item);

                    let bg = if is_current {
                        theme.selection_bg()
                    } else {
                        Color32::TRANSPARENT
                    };

                    let frame = egui::Frame::none()
                        .fill(bg)
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(4.0, 2.0));

                    let resp = frame.show(ui, |ui| {
                        let label_color = if is_current {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(0xd8, 0xde, 0xe9)
                        };
                        ui.label(
                            egui::RichText::new(format!("{} {}", icon, file_name))
                                .color(label_color)
                                .size(12.0),
                        );
                    });

                    // Tooltip with file size
                    if let Ok(meta) = item.metadata() {
                        let kb = (meta.len() as f64) / 1024.0;
                        resp.response.clone().on_hover_text(format!("{} ({:.1} KB)", file_name, kb));
                    }

                    if resp.response.clicked() {
                        *on_open_file = Some(item);
                    }
                }
            }
        }
    }

    /// File extension icon mapping matching modern editor aesthetics
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
