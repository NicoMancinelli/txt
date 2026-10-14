use egui::{self, Color32, Key, Rect, Rounding, Stroke, Vec2};
use txt_core::command::{CommandRegistry, MatchedCommand};
use crate::theme::EditorTheme;

pub struct CommandPalette {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub matched_commands: Vec<MatchedCommand>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            matched_commands: Vec::new(),
        }
    }

    pub fn open(&mut self, registry: &CommandRegistry) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
        self.update_matches(registry);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn update_matches(&mut self, registry: &CommandRegistry) {
        self.matched_commands = registry.search(&self.query);
        if self.selected_index >= self.matched_commands.len() {
            self.selected_index = self.matched_commands.len().saturating_sub(1);
        }
    }

    /// Renders the modal command palette overlay.
    /// Returns Some(command_id) if a command was activated.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        theme: EditorTheme,
        registry: &CommandRegistry,
    ) -> Option<String> {
        if !self.is_open {
            return None;
        }

        let mut executed_id = None;

        // Dim background overlay
        let screen_rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("palette_dim_layer"),
        ));
        painter.rect_filled(screen_rect, Rounding::ZERO, Color32::from_black_alpha(120));

        let modal_width = 560.0_f32.min(screen_rect.width() - 40.0);
        let modal_top = 60.0;
        let modal_rect = Rect::from_min_size(
            egui::pos2(
                (screen_rect.width() - modal_width) / 2.0,
                modal_top,
            ),
            Vec2::new(modal_width, 400.0),
        );

        egui::Area::new(egui::Id::new("command_palette_modal"))
            .fixed_pos(modal_rect.min)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(theme.modal_bg())
                    .rounding(Rounding::same(8.0))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x40, 0x48, 0x59)))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(modal_width - 24.0);

                        // Header with search input
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🔍").size(16.0));
                            let text_edit = egui::TextEdit::singleline(&mut self.query)
                                .hint_text("Type a command (Sublime fuzzy search)...")
                                .desired_width(ui.available_width())
                                .lock_focus(true);

                            let response = ui.add(text_edit);
                            response.request_focus();

                            if response.changed() {
                                self.update_matches(registry);
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Keyboard navigation handling
                        if ui.input(|i| i.key_pressed(Key::Escape)) {
                            self.close();
                        }
                        if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                            if !self.matched_commands.is_empty() {
                                self.selected_index = (self.selected_index + 1) % self.matched_commands.len();
                            }
                        }
                        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                            if !self.matched_commands.is_empty() {
                                self.selected_index = if self.selected_index == 0 {
                                    self.matched_commands.len().saturating_sub(1)
                                } else {
                                    self.selected_index - 1
                                };
                            }
                        }
                        if ui.input(|i| i.key_pressed(Key::Enter)) {
                            if let Some(matched) = self.matched_commands.get(self.selected_index) {
                                executed_id = Some(matched.descriptor.id.clone());
                                self.close();
                            }
                        }

                        // Scrollable list of matched commands
                        egui::ScrollArea::vertical()
                            .max_height(320.0)
                            .show(ui, |ui| {
                                if self.matched_commands.is_empty() {
                                    ui.add_space(16.0);
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new("No matching commands")
                                                .color(Color32::from_rgb(0x8a, 0x93, 0xa0)),
                                        );
                                    });
                                    return;
                                }

                                for (idx, item) in self.matched_commands.iter().enumerate() {
                                    let is_selected = idx == self.selected_index;
                                    let bg = if is_selected {
                                        theme.modal_item_hover()
                                    } else {
                                        Color32::TRANSPARENT
                                    };

                                    let resp = egui::Frame::none()
                                        .fill(bg)
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Category tag
                                                ui.label(
                                                    egui::RichText::new(&item.descriptor.category)
                                                        .color(Color32::from_rgb(0x5f, 0xb3, 0xb3))
                                                        .size(11.0),
                                                );
                                                ui.label(
                                                    egui::RichText::new(":")
                                                        .color(Color32::from_rgb(0x8a, 0x93, 0xa0))
                                                        .size(11.0),
                                                );

                                                // Title
                                                let title_color = if is_selected {
                                                    Color32::WHITE
                                                } else {
                                                    Color32::from_rgb(0xd8, 0xde, 0xe9)
                                                };
                                                ui.label(
                                                    egui::RichText::new(&item.descriptor.title)
                                                        .color(title_color)
                                                        .size(13.0)
                                                        .strong(),
                                                );

                                                // Shortcut badge aligned right
                                                if let Some(shortcut) = &item.descriptor.shortcut {
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(egui::Align::Center),
                                                        |ui| {
                                                            egui::Frame::none()
                                                                .fill(Color32::from_rgb(0x18, 0x1a, 0x1f))
                                                                .rounding(Rounding::same(3.0))
                                                                .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                                                .show(ui, |ui| {
                                                                    ui.label(
                                                                        egui::RichText::new(shortcut)
                                                                            .color(Color32::from_rgb(0xfa, 0xc8, 0x63))
                                                                            .size(10.0),
                                                                    );
                                                                });
                                                        },
                                                    );
                                                }
                                            });
                                        });

                                    if resp.response.clicked() {
                                        executed_id = Some(item.descriptor.id.clone());
                                    }
                                }
                            });
                    });
            });

        if executed_id.is_some() {
            self.close();
        }

        executed_id
    }
}
