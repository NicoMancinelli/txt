use egui::{self, Color32};
use txt_core::buffer::TextBuffer;
use txt_core::cursor::SelectionSet;
use txt_core::syntax::Language;
use txt_sync::adapter::SyncStatus;
use crate::theme::EditorTheme;

pub struct StatusBar;

impl StatusBar {
    pub fn show(
        ui: &mut egui::Ui,
        theme: EditorTheme,
        buffer: &TextBuffer,
        selections: &SelectionSet,
        language: Language,
        sync_status: SyncStatus,
        notification: Option<&str>,
    ) {
        egui::Frame::none()
            .fill(theme.status_bar_bg())
            .inner_margin(egui::Margin::symmetric(12.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let prim = selections.primary();
                    let pos = buffer.char_to_pos(prim.head);

                    // Line and Column
                    let selection_text = if prim.is_empty() {
                        format!("Line {}, Column {}", pos.line + 1, pos.column + 1)
                    } else {
                        format!(
                            "Line {}, Column {} ({} selected)",
                            pos.line + 1,
                            pos.column + 1,
                            prim.len()
                        )
                    };

                    ui.label(
                        egui::RichText::new(selection_text)
                            .color(theme.status_bar_text())
                            .size(11.0),
                    );

                    // Multi-cursor count if more than 1
                    if selections.selections().len() > 1 {
                        ui.label(
                            egui::RichText::new(format!("{} cursors", selections.selections().len()))
                                .color(Color32::from_rgb(0xfa, 0xc8, 0x63))
                                .size(11.0),
                        );
                    }

                    // Notification banner if active
                    if let Some(msg) = notification {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("⚡ {}", msg))
                                .color(Color32::from_rgb(0x5f, 0xb3, 0xb3))
                                .size(11.0),
                        );
                    }

                    // Spacer to right-aligned items
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Language
                        ui.label(
                            egui::RichText::new(language.display_name())
                                .color(theme.status_bar_text())
                                .size(11.0),
                        );
                        ui.separator();

                        // Sync Status
                        let (sync_color, sync_text) = match sync_status {
                            SyncStatus::Synced => (Color32::from_rgb(0x99, 0xc7, 0x94), sync_status.display_badge()),
                            SyncStatus::Syncing => (Color32::from_rgb(0x66, 0x99, 0xcc), sync_status.display_badge()),
                            SyncStatus::Offline => (Color32::from_rgb(0x75, 0x71, 0x5e), sync_status.display_badge()),
                            SyncStatus::Conflict => (Color32::from_rgb(0xf9, 0x91, 0x57), sync_status.display_badge()),
                            SyncStatus::Error => (Color32::from_rgb(0xec, 0x5f, 0x67), sync_status.display_badge()),
                        };
                        ui.label(egui::RichText::new(sync_text).color(sync_color).size(11.0));
                        ui.separator();

                        // Encoding & Line ending
                        ui.label(
                            egui::RichText::new(format!(
                                "{}  {}",
                                buffer.encoding(),
                                buffer.line_ending().as_str().trim_start_matches('\r').replace('\n', "LF")
                            ))
                            .color(Color32::from_rgb(0x8a, 0x92, 0x9e))
                            .size(11.0),
                        );
                        ui.separator();

                        // Spaces: 4
                        ui.label(
                            egui::RichText::new("Spaces: 4")
                                .color(Color32::from_rgb(0x8a, 0x92, 0x9e))
                                .size(11.0),
                        );
                        ui.separator();

                        // Memory stats
                        let mem_kb = (buffer.estimated_memory_usage() as f64) / 1024.0;
                        ui.label(
                            egui::RichText::new(format!("{:.1} KB", mem_kb))
                                .color(Color32::from_rgb(0x5f, 0xb3, 0xb3))
                                .size(11.0),
                        );
                    });
                });
            });
    }
}
