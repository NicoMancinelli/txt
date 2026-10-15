use egui::{self, Color32, Rounding};
use txt_core::buffer::TextBuffer;
use crate::theme::EditorTheme;

pub struct TabBar;

impl TabBar {
    pub fn show(
        ui: &mut egui::Ui,
        theme: EditorTheme,
        buffers: &[TextBuffer],
        active_tab: &mut usize,
        on_close: &mut Option<usize>,
    ) {
        egui::Frame::none()
            .fill(theme.tab_bar_bg())
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (i, buf) in buffers.iter().enumerate() {
                        let is_active = i == *active_tab;
                        let bg = if is_active {
                            theme.tab_active_bg()
                        } else {
                            Color32::TRANSPARENT
                        };

                        let frame = egui::Frame::none()
                            .fill(bg)
                            .rounding(Rounding {
                                nw: 4.0,
                                ne: 4.0,
                                sw: 0.0,
                                se: 0.0,
                            })
                            .inner_margin(egui::Margin::symmetric(10.0, 6.0));

                        let resp = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Dirty indicator dot
                                if buf.is_dirty() {
                                    ui.label(
                                        egui::RichText::new("●")
                                            .color(Color32::from_rgb(0xf9, 0x91, 0x57))
                                            .size(9.0),
                                    );
                                }

                                let text_color = if is_active {
                                    theme.tab_active_text()
                                } else {
                                    theme.tab_inactive_text()
                                };

                                ui.label(
                                    egui::RichText::new(buf.file_name())
                                        .color(text_color)
                                        .size(12.0),
                                );

                                // Close tab button 'x'
                                if ui.small_button("×").clicked() {
                                    *on_close = Some(i);
                                }
                            });
                        });

                        if resp.response.clicked() {
                            *active_tab = i;
                        }
                    }
                });
            });
    }
}
