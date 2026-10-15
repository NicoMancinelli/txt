use egui::{
    self, Color32, FontId, Key, Pos2, Rect, Rounding, Stroke, Vec2,
};
use txt_core::buffer::TextBuffer;
use txt_core::cursor::{CursorPosition, Selection, SelectionSet};
use txt_core::syntax::{Language, SyntaxHighlighter};
use crate::theme::EditorTheme;

pub struct EditorView {
    pub font_size: f32,
    pub line_height: f32,
    pub tab_size: usize,
    pub scroll_offset_y: f32,
    pub show_minimap: bool,
    pub word_wrap: bool,
}

impl Default for EditorView {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            line_height: 20.0,
            tab_size: 4,
            scroll_offset_y: 0.0,
            show_minimap: true,
            word_wrap: true,
        }
    }
}

impl EditorView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: EditorTheme,
        buffer: &mut TextBuffer,
        selections: &mut SelectionSet,
        language: Language,
    ) {
        let total_lines = buffer.len_lines().max(1);
        let gutter_width = 54.0;
        let font_id = FontId::monospace(self.font_size);
        let char_width = self.font_size * 0.6; // Approximate monospace char width

        // Handle keyboard input before rendering
        self.handle_input(ui, buffer, selections);

        // Fill whole available editor area with theme background
        let available_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // Main editor background
        painter.rect_filled(rect, Rounding::ZERO, theme.bg());

        // Gutter background
        let gutter_rect = Rect::from_min_size(
            rect.min,
            Vec2::new(gutter_width, rect.height()),
        );
        painter.rect_filled(gutter_rect, Rounding::ZERO, theme.gutter_bg());

        // Gutter separator line
        painter.line_segment(
            [
                Pos2::new(rect.min.x + gutter_width, rect.min.y),
                Pos2::new(rect.min.x + gutter_width, rect.max.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(0x28, 0x2c, 0x34)),
        );

        let content_start_x = rect.min.x + gutter_width + 12.0;
        let content_start_y = rect.min.y + 8.0 - self.scroll_offset_y;

        // Mouse click to set cursor
        if response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if mouse_pos.x > rect.min.x + gutter_width {
                    let clicked_line = (((mouse_pos.y - content_start_y) / self.line_height).floor() as isize).max(0) as usize;
                    let clicked_line = clicked_line.min(total_lines.saturating_sub(1));
                    let clicked_col = (((mouse_pos.x - content_start_x) / char_width).round() as isize).max(0) as usize;

                    let char_offset = buffer.pos_to_char(CursorPosition::new(clicked_line, clicked_col));
                    selections.set_primary(Selection::point(char_offset));
                }
            }
        }

        // Active line highlight for primary cursor
        let primary_pos = buffer.char_to_pos(selections.primary().head);
        let active_line_y = content_start_y + (primary_pos.line as f32) * self.line_height;
        if active_line_y >= rect.min.y && active_line_y < rect.max.y {
            let active_line_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + gutter_width + 1.0, active_line_y),
                Vec2::new(rect.width() - gutter_width - 1.0, self.line_height),
            );
            painter.rect_filled(active_line_rect, Rounding::ZERO, theme.current_line_bg());
        }

        // Visible line range for fast, memory-efficient viewport rendering
        let visible_start_line = ((rect.min.y - content_start_y) / self.line_height).floor().max(0.0) as usize;
        let visible_end_line = (((rect.max.y - content_start_y) / self.line_height).ceil() as usize).min(total_lines);

        for line_idx in visible_start_line..visible_end_line {
            let line_y = content_start_y + (line_idx as f32) * self.line_height;

            // Render line number in gutter
            let line_number_text = format!("{:4}", line_idx + 1);
            let line_num_color = if line_idx == primary_pos.line {
                Color32::from_rgb(0xc0, 0xc5, 0xce) // Active line number stands out
            } else {
                theme.gutter_text()
            };

            painter.text(
                Pos2::new(rect.min.x + gutter_width - 10.0, line_y + 2.0),
                egui::Align2::RIGHT_TOP,
                line_number_text,
                FontId::monospace(12.0),
                line_num_color,
            );

            // Fetch line text from rope buffer
            let line_str = buffer
                .line(line_idx)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let line_clean = line_str.trim_end_matches(['\r', '\n']);

            // Tokenize line for syntax highlighting
            let tokens = SyntaxHighlighter::tokenize_line(line_clean, language);

            let mut current_x = content_start_x;
            for token in tokens {
                let token_slice: String = line_clean.chars().skip(token.range.start).take(token.range.len()).collect();
                let token_color = theme.syntax_color(token.token_type);

                let text_pos = Pos2::new(current_x, line_y + 2.0);
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_TOP,
                    &token_slice,
                    font_id.clone(),
                    token_color,
                );
                current_x += (token.range.len() as f32) * char_width;
            }
        }

        // Render selections and cursors
        for sel in selections.selections() {
            let head_pos = buffer.char_to_pos(sel.head);
            let cursor_y = content_start_y + (head_pos.line as f32) * self.line_height;
            let cursor_x = content_start_x + (head_pos.column as f32) * char_width;

            // Render selection background if not empty
            if !sel.is_empty() {
                let min_pos = buffer.char_to_pos(sel.min_char());
                let max_pos = buffer.char_to_pos(sel.max_char());

                if min_pos.line == max_pos.line {
                    let sel_y = content_start_y + (min_pos.line as f32) * self.line_height;
                    let sel_x1 = content_start_x + (min_pos.column as f32) * char_width;
                    let sel_x2 = content_start_x + (max_pos.column as f32) * char_width;
                    let sel_rect = Rect::from_min_max(
                        Pos2::new(sel_x1, sel_y),
                        Pos2::new(sel_x2, sel_y + self.line_height),
                    );
                    painter.rect_filled(sel_rect, Rounding::ZERO, theme.selection_bg());
                }
            }

            // Draw cursor caret
            if cursor_y >= rect.min.y && cursor_y < rect.max.y {
                let caret_rect = Rect::from_min_size(
                    Pos2::new(cursor_x, cursor_y + 2.0),
                    Vec2::new(2.0, self.line_height - 4.0),
                );
                painter.rect_filled(caret_rect, Rounding::same(1.0), theme.cursor_color());
            }
        }

        // Skyline Minimap (Sublime Text style minimap)
        if self.show_minimap && rect.width() > 320.0 {
            let minimap_width = 85.0_f32;
            let minimap_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - minimap_width, rect.min.y),
                Vec2::new(minimap_width, rect.height()),
            );

            // Minimap background with subtle border
            painter.rect_filled(minimap_rect, Rounding::ZERO, Color32::from_black_alpha(40));
            painter.line_segment(
                [
                    Pos2::new(minimap_rect.min.x, minimap_rect.min.y),
                    Pos2::new(minimap_rect.min.x, minimap_rect.max.y),
                ],
                Stroke::new(1.0_f32, Color32::from_white_alpha(15)),
            );

            // Render mini line bars
            let mini_line_height = 2.5_f32;
            let mini_spacing = 1.0_f32;
            let max_visible_mini_lines = (minimap_rect.height() / (mini_line_height + mini_spacing)).floor() as usize;

            for i in 0..total_lines.min(max_visible_mini_lines) {
                let y = minimap_rect.min.y + 6.0 + (i as f32) * (mini_line_height + mini_spacing);
                let line_len = buffer.line(i).map_or(0, |l| l.len_chars());
                let bar_width = ((line_len as f32) * 1.8).min(minimap_width - 16.0).max(4.0);

                let bar_rect = Rect::from_min_size(
                    Pos2::new(minimap_rect.min.x + 8.0, y),
                    Vec2::new(bar_width, mini_line_height),
                );
                painter.rect_filled(bar_rect, Rounding::ZERO, Color32::from_white_alpha(35));
            }

            // Viewport indicator box
            let viewport_ratio = (rect.height() / (self.line_height * (total_lines as f32))).min(1.0);
            let indicator_height = (minimap_rect.height() * viewport_ratio).max(24.0);
            let scroll_ratio = if total_lines > 1 {
                self.scroll_offset_y / ((total_lines as f32) * self.line_height)
            } else {
                0.0
            };
            let indicator_y = minimap_rect.min.y + scroll_ratio * minimap_rect.height();

            let indicator_rect = Rect::from_min_size(
                Pos2::new(minimap_rect.min.x + 2.0, indicator_y),
                Vec2::new(minimap_width - 4.0, indicator_height),
            );
            painter.rect_filled(indicator_rect, Rounding::same(3.0), Color32::from_white_alpha(25));
            painter.rect_stroke(indicator_rect, Rounding::same(3.0), Stroke::new(1.0_f32, Color32::from_white_alpha(45)));
        }
    }

    fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        buffer: &mut TextBuffer,
        selections: &mut SelectionSet,
    ) {
        // Collect text typed
        let typed_text = ui.input(|i| i.events.iter().filter_map(|e| {
            if let egui::Event::Text(t) = e {
                Some(t.clone())
            } else {
                None
            }
        }).collect::<Vec<_>>());

        for text in typed_text {
            let new_sels = buffer.insert_at_selections(selections.selections(), &text);
            selections.set_all(new_sels);
        }

        // Duplicate Line (Sublime: Cmd+Shift+D / Ctrl+Shift+D)
        let cmd = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
        let shift = ui.input(|i| i.modifiers.shift);

        if cmd && shift && ui.input(|i| i.key_pressed(Key::D)) {
            let pos = buffer.char_to_pos(selections.primary().head);
            if let Some(line_slice) = buffer.line(pos.line) {
                let line_text = line_slice.to_string();
                let next_line_start = buffer.pos_to_char(CursorPosition::new(pos.line + 1, 0));
                buffer.insert(next_line_start, &line_text);
            }
        }

        // Delete Line (Sublime: Cmd+Shift+K / Ctrl+Shift+K)
        if cmd && shift && ui.input(|i| i.key_pressed(Key::K)) {
            let pos = buffer.char_to_pos(selections.primary().head);
            let line_start = buffer.pos_to_char(CursorPosition::new(pos.line, 0));
            let line_end = buffer.pos_to_char(CursorPosition::new(pos.line + 1, 0));
            if line_end > line_start {
                buffer.delete(line_start..line_end);
                selections.set_primary(Selection::point(line_start.min(buffer.len_chars())));
            }
        }

        // Backspace
        if ui.input(|i| i.key_pressed(Key::Backspace)) {
            let mut new_sels = Vec::new();
            for sel in selections.selections() {
                if !sel.is_empty() {
                    let min = sel.min_char();
                    let max = sel.max_char();
                    buffer.delete(min..max);
                    new_sels.push(Selection::point(min));
                } else if sel.head > 0 {
                    let delete_pos = sel.head - 1;
                    buffer.delete(delete_pos..sel.head);
                    new_sels.push(Selection::point(delete_pos));
                } else {
                    new_sels.push(*sel);
                }
            }
            selections.set_all(new_sels);
        }

        // Enter
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            let newline = buffer.line_ending().as_str();
            let new_sels = buffer.insert_at_selections(selections.selections(), newline);
            selections.set_all(new_sels);
        }

        // Tab (Sublime style: inserts spaces according to tab_size)
        if ui.input(|i| i.key_pressed(Key::Tab)) {
            let spaces = " ".repeat(self.tab_size);
            let new_sels = buffer.insert_at_selections(selections.selections(), &spaces);
            selections.set_all(new_sels);
        }

        // Arrow navigation
        if ui.input(|i| i.key_pressed(Key::ArrowLeft)) {
            let shift = ui.input(|i| i.modifiers.shift);
            let mut list = Vec::new();
            for sel in selections.selections() {
                let new_head = sel.head.saturating_sub(1);
                let new_anchor = if shift { sel.anchor } else { new_head };
                list.push(Selection::range(new_anchor, new_head));
            }
            selections.set_all(list);
        }

        if ui.input(|i| i.key_pressed(Key::ArrowRight)) {
            let shift = ui.input(|i| i.modifiers.shift);
            let max_len = buffer.len_chars();
            let mut list = Vec::new();
            for sel in selections.selections() {
                let new_head = (sel.head + 1).min(max_len);
                let new_anchor = if shift { sel.anchor } else { new_head };
                list.push(Selection::range(new_anchor, new_head));
            }
            selections.set_all(list);
        }

        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
            let shift = ui.input(|i| i.modifiers.shift);
            let mut list = Vec::new();
            for sel in selections.selections() {
                let pos = buffer.char_to_pos(sel.head);
                let target_pos = CursorPosition::new(pos.line.saturating_sub(1), pos.column);
                let new_head = buffer.pos_to_char(target_pos);
                let new_anchor = if shift { sel.anchor } else { new_head };
                list.push(Selection::range(new_anchor, new_head));
            }
            selections.set_all(list);
        }

        if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
            let shift = ui.input(|i| i.modifiers.shift);
            let total_lines = buffer.len_lines();
            let mut list = Vec::new();
            for sel in selections.selections() {
                let pos = buffer.char_to_pos(sel.head);
                let target_pos = CursorPosition::new((pos.line + 1).min(total_lines.saturating_sub(1)), pos.column);
                let new_head = buffer.pos_to_char(target_pos);
                let new_anchor = if shift { sel.anchor } else { new_head };
                list.push(Selection::range(new_anchor, new_head));
            }
            selections.set_all(list);
        }

        // Scroll
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            self.scroll_offset_y = (self.scroll_offset_y - scroll_delta).max(0.0);
        }
    }
}
