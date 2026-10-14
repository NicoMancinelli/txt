use std::path::{Path, PathBuf};
use egui::{self, Color32};
use crate::theme::EditorTheme;

pub struct Sidebar {
    pub is_visible: bool,
    pub current_directory: PathBuf,
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
        }
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: EditorTheme,
        on_open_file: &mut Option<PathBuf>,
    ) {
        if !self.is_visible {
            return;
        }

        egui::Frame::none()
            .fill(theme.sidebar_bg())
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.set_width(220.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("FOLDERS")
                            .color(Color32::from_rgb(0x7a, 0x82, 0x8e))
                            .size(11.0)
                            .strong(),
                    );
                });
                ui.add_space(4.0);

                let root_name = self
                    .current_directory
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Project".to_string());

                egui::CollapsingHeader::new(format!("📁 {}", root_name))
                    .default_open(true)
                    .show(ui, |ui| {
                        Self::render_dir_contents(ui, &self.current_directory, on_open_file);
                    });
            });
    }

    fn render_dir_contents(
        ui: &mut egui::Ui,
        dir: &Path,
        on_open_file: &mut Option<PathBuf>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut items: Vec<PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    // Ignore hidden files and build output folders
                    p.file_name().map_or(false, |name| {
                        let s = name.to_string_lossy();
                        !s.starts_with('.') && s != "target"
                    })
                })
                .collect();

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

            for item in items.into_iter().take(40) {
                let file_name = item
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if item.is_dir() {
                    let sub_dir = item.clone();
                    egui::CollapsingHeader::new(format!("📁 {}", file_name))
                        .default_open(false)
                        .show(ui, |ui| {
                            Self::render_dir_contents(ui, &sub_dir, on_open_file);
                        });
                } else {
                    let icon = match item.extension().and_then(|e| e.to_str()).unwrap_or("") {
                        "rs" => "🦀",
                        "toml" => "⚙",
                        "md" => "📝",
                        "json" => "{}",
                        "py" => "🐍",
                        _ => "📄",
                    };

                    if ui.selectable_label(false, format!("{} {}", icon, file_name)).clicked() {
                        *on_open_file = Some(item);
                    }
                }
            }
        }
    }
}
