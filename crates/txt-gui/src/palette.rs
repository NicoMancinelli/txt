use egui::{self, Color32, Key, Rect, Rounding, Stroke, Vec2};
use txt_core::buffer::TextBuffer;
use txt_core::command::{CommandRegistry, MatchedCommand};
use txt_core::fuzzy::FuzzyMatcher;
use txt_core::symbols::{DocumentSymbol, SymbolExtractor};
use crate::theme::EditorTheme;

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteAction {
    ExecuteCommand(String),
    GotoPosition { line: usize, column: usize },
}

#[derive(Debug, Clone)]
pub struct MatchedSymbol {
    pub symbol: DocumentSymbol,
    pub score: i64,
    pub matched_indices: Vec<usize>,
}

pub struct CommandPalette {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub matched_commands: Vec<MatchedCommand>,
    pub matched_symbols: Vec<MatchedSymbol>,
    pub goto_line_target: Option<usize>,
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
            matched_symbols: Vec::new(),
            goto_line_target: None,
        }
    }

    pub fn open(&mut self, registry: &CommandRegistry) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
        self.update_matches_with_buffer(registry, None, "rust");
    }

    pub fn open_symbols(&mut self, registry: &CommandRegistry, buffer: &TextBuffer, language: &str) {
        self.is_open = true;
        self.query = "@".to_string();
        self.selected_index = 0;
        self.update_matches_with_buffer(registry, Some(buffer), language);
    }

    pub fn open_goto_line(&mut self, registry: &CommandRegistry, buffer: &TextBuffer) {
        self.is_open = true;
        self.query = ":".to_string();
        self.selected_index = 0;
        self.update_matches_with_buffer(registry, Some(buffer), "rust");
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.selected_index = 0;
        self.matched_commands.clear();
        self.matched_symbols.clear();
        self.goto_line_target = None;
    }

    #[allow(dead_code)]
    pub fn update_matches(&mut self, registry: &CommandRegistry) {
        self.update_matches_with_buffer(registry, None, "rust");
    }

    pub fn update_matches_with_buffer(
        &mut self,
        registry: &CommandRegistry,
        buffer: Option<&TextBuffer>,
        language: &str,
    ) {
        if self.query.starts_with('@') {
            // Goto Symbol mode
            self.matched_commands.clear();
            self.goto_line_target = None;

            if let Some(buf) = buffer {
                let symbol_query = self.query[1..].trim();
                let symbols = SymbolExtractor::extract_symbols(buf, language);

                if symbol_query.is_empty() {
                    self.matched_symbols = symbols
                        .into_iter()
                        .map(|s| MatchedSymbol {
                            symbol: s,
                            score: 0,
                            matched_indices: Vec::new(),
                        })
                        .collect();
                } else {
                    let mut matched = Vec::new();
                    for s in symbols {
                        if let Some(m) = FuzzyMatcher::match_str(symbol_query, &s.name) {
                            matched.push(MatchedSymbol {
                                symbol: s,
                                score: m.score,
                                matched_indices: m.matched_indices,
                            });
                        }
                    }
                    matched.sort_by(|a, b| b.score.cmp(&a.score));
                    self.matched_symbols = matched;
                }
            } else {
                self.matched_symbols.clear();
            }

            if self.selected_index >= self.matched_symbols.len() {
                self.selected_index = self.matched_symbols.len().saturating_sub(1);
            }
        } else if self.query.starts_with(':') {
            // Goto Line mode
            self.matched_commands.clear();
            self.matched_symbols.clear();
            let line_str = self.query[1..].trim();
            if let Ok(line_num) = line_str.parse::<usize>() {
                self.goto_line_target = Some(line_num.saturating_sub(1));
            } else {
                self.goto_line_target = None;
            }
            self.selected_index = 0;
        } else {
            // Standard Command Search mode
            self.matched_symbols.clear();
            self.goto_line_target = None;
            self.matched_commands = registry.search(&self.query);
            if self.selected_index >= self.matched_commands.len() {
                self.selected_index = self.matched_commands.len().saturating_sub(1);
            }
        }
    }

    /// Renders the modal command palette overlay.
    /// Returns Some(PaletteAction) if an action was confirmed.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        theme: EditorTheme,
        registry: &CommandRegistry,
        buffer: Option<&TextBuffer>,
        language: &str,
    ) -> Option<PaletteAction> {
        if !self.is_open {
            return None;
        }

        let mut executed_action = None;

        // Dim background overlay
        let screen_rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("palette_dim_layer"),
        ));
        painter.rect_filled(screen_rect, Rounding::ZERO, Color32::from_black_alpha(120));

        let modal_width = 580.0_f32.min(screen_rect.width() - 40.0);
        let modal_top = 60.0;
        let modal_rect = Rect::from_min_size(
            egui::pos2((screen_rect.width() - modal_width) / 2.0, modal_top),
            Vec2::new(modal_width, 420.0),
        );

        egui::Area::new(egui::Id::new("command_palette_modal"))
            .fixed_pos(modal_rect.min)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(theme.modal_bg())
                    .rounding(Rounding::same(8.0))
                    .stroke(Stroke::new(1.0_f32, theme.border()))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(modal_width - 24.0);

                        let hint = if self.query.starts_with('@') {
                            "Goto Symbol in file (@symbol_name)..."
                        } else if self.query.starts_with(':') {
                            "Goto Line (:line_number)..."
                        } else {
                            "Type a command (@ for symbols, : for line)..."
                        };

                        let icon = if self.query.starts_with('@') {
                            "🏷"
                        } else if self.query.starts_with(':') {
                            "📍"
                        } else {
                            "🔍"
                        };

                        // Header with search input
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).size(16.0));
                            let text_edit = egui::TextEdit::singleline(&mut self.query)
                                .hint_text(hint)
                                .desired_width(ui.available_width())
                                .lock_focus(true);

                            let response = ui.add(text_edit);
                            response.request_focus();

                            if response.changed() {
                                self.update_matches_with_buffer(registry, buffer, language);
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Keyboard navigation handling
                        if ui.input(|i| i.key_pressed(Key::Escape)) {
                            self.close();
                        }

                        let total_items = if self.query.starts_with('@') {
                            self.matched_symbols.len()
                        } else if self.query.starts_with(':') {
                            if self.goto_line_target.is_some() { 1 } else { 0 }
                        } else {
                            self.matched_commands.len()
                        };

                        if ui.input(|i| i.key_pressed(Key::ArrowDown)) && total_items > 0 {
                            self.selected_index = (self.selected_index + 1) % total_items;
                        }
                        if ui.input(|i| i.key_pressed(Key::ArrowUp)) && total_items > 0 {
                            self.selected_index = if self.selected_index == 0 {
                                total_items.saturating_sub(1)
                            } else {
                                self.selected_index - 1
                            };
                        }

                        if ui.input(|i| i.key_pressed(Key::Enter)) {
                            if self.query.starts_with('@') {
                                if let Some(m) = self.matched_symbols.get(self.selected_index) {
                                    executed_action = Some(PaletteAction::GotoPosition {
                                        line: m.symbol.line,
                                        column: m.symbol.column,
                                    });
                                    self.close();
                                }
                            } else if self.query.starts_with(':') {
                                if let Some(line) = self.goto_line_target {
                                    executed_action = Some(PaletteAction::GotoPosition {
                                        line,
                                        column: 0,
                                    });
                                    self.close();
                                }
                            } else if let Some(matched) = self.matched_commands.get(self.selected_index) {
                                executed_action = Some(PaletteAction::ExecuteCommand(
                                    matched.descriptor.id.clone(),
                                ));
                                self.close();
                            }
                        }

                        // Scrollable list
                        egui::ScrollArea::vertical()
                            .max_height(320.0)
                            .show(ui, |ui| {
                                if self.query.starts_with('@') {
                                    // Symbol Results
                                    if self.matched_symbols.is_empty() {
                                        ui.add_space(16.0);
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                egui::RichText::new("No matching symbols found in current buffer")
                                                    .color(Color32::from_rgb(0x8a, 0x93, 0xa0)),
                                            );
                                        });
                                        return;
                                    }

                                    for (idx, item) in self.matched_symbols.iter().enumerate() {
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
                                                    // Symbol kind badge
                                                    let kind_text = format!("[{}]", item.symbol.kind.as_str());
                                                    ui.label(
                                                        egui::RichText::new(kind_text)
                                                            .size(11.0)
                                                            .color(Color32::from_rgb(0xf9, 0x91, 0x57)),
                                                    );

                                                    // Symbol name with fuzzy highlights
                                                    let matched_set: std::collections::HashSet<usize> =
                                                        item.matched_indices.iter().copied().collect();

                                                    let mut job = egui::text::LayoutJob::default();
                                                    for (char_idx, ch) in item.symbol.name.chars().enumerate() {
                                                        let color = if matched_set.contains(&char_idx) {
                                                            Color32::from_rgb(0xf9, 0x91, 0x57)
                                                        } else {
                                                            Color32::from_rgb(0xd8, 0xde, 0xe9)
                                                        };
                                                        job.append(
                                                            &ch.to_string(),
                                                            0.0,
                                                            egui::TextFormat {
                                                                color,
                                                                font_id: egui::FontId::proportional(13.0),
                                                                ..Default::default()
                                                            },
                                                        );
                                                    }
                                                    ui.label(job);

                                                    // Detail / Line on right
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.label(
                                                            egui::RichText::new(format!("line {}", item.symbol.line + 1))
                                                                .size(11.0)
                                                                .color(Color32::from_rgb(0x75, 0x71, 0x5e)),
                                                        );
                                                        if let Some(detail) = &item.symbol.detail {
                                                            ui.label(
                                                                egui::RichText::new(detail)
                                                                    .size(11.0)
                                                                    .color(Color32::from_rgb(0x8a, 0x93, 0xa0)),
                                                            );
                                                        }
                                                    });
                                                });
                                            });

                                        if resp.response.interact(egui::Sense::click()).clicked() {
                                            executed_action = Some(PaletteAction::GotoPosition {
                                                line: item.symbol.line,
                                                column: item.symbol.column,
                                            });
                                        }
                                    }
                                } else if self.query.starts_with(':') {
                                    // Goto Line preview
                                    if let Some(line) = self.goto_line_target {
                                        let max_lines = buffer.map_or(1, |b| b.len_lines());
                                        let target_display = (line + 1).min(max_lines);
                                        let resp = egui::Frame::none()
                                            .fill(theme.modal_item_hover())
                                            .rounding(Rounding::same(4.0))
                                            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!("Jump to Line {}", target_display))
                                                            .size(14.0)
                                                            .color(Color32::from_rgb(0xa6, 0xe2, 0x2e))
                                                            .strong(),
                                                    );
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.label(
                                                            egui::RichText::new(format!("(buffer has {} lines)", max_lines))
                                                                .size(12.0)
                                                                .color(Color32::from_rgb(0x8a, 0x93, 0xa0)),
                                                        );
                                                    });
                                                });
                                            });
                                        if resp.response.interact(egui::Sense::click()).clicked() {
                                            executed_action = Some(PaletteAction::GotoPosition {
                                                line,
                                                column: 0,
                                            });
                                            self.close();
                                        }
                                    } else {
                                        ui.add_space(16.0);
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                egui::RichText::new("Enter a line number to jump to (e.g. :42)")
                                                    .color(Color32::from_rgb(0x8a, 0x93, 0xa0)),
                                            );
                                        });
                                    }
                                } else {
                                    // Command results
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
                                                    // Highlight matched letters
                                                    let title = &item.descriptor.title;
                                                    let matched_set: std::collections::HashSet<usize> =
                                                        item.match_result.matched_indices.iter().copied().collect();

                                                    let mut job = egui::text::LayoutJob::default();
                                                    for (char_idx, ch) in title.chars().enumerate() {
                                                        let color = if matched_set.contains(&char_idx) {
                                                            Color32::from_rgb(0xf9, 0x91, 0x57) // Highlight matched
                                                        } else {
                                                            Color32::from_rgb(0xd8, 0xde, 0xe9)
                                                        };
                                                        job.append(
                                                            &ch.to_string(),
                                                            0.0,
                                                            egui::TextFormat {
                                                                color,
                                                                font_id: egui::FontId::proportional(13.0),
                                                                ..Default::default()
                                                            },
                                                        );
                                                    }
                                                    ui.label(job);

                                                    // Right side: category or shortcut
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(egui::Align::Center),
                                                        |ui| {
                                                            ui.label(
                                                                egui::RichText::new(&item.descriptor.category)
                                                                    .size(11.0)
                                                                    .color(Color32::from_rgb(0x8a, 0x93, 0xa0)),
                                                            );
                                                        },
                                                    );
                                                });
                                            });

                                        if resp.response.interact(egui::Sense::click()).clicked() {
                                            executed_action = Some(PaletteAction::ExecuteCommand(
                                                item.descriptor.id.clone(),
                                            ));
                                        }
                                    }
                                }
                            });
                    });
            });

        if executed_action.is_some() {
            self.close();
        }

        executed_action
    }
}
