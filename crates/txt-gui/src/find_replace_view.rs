use egui::{self, Color32, Key, Stroke};
use txt_core::buffer::TextBuffer;
use txt_core::cursor::{Selection, SelectionSet};
use txt_core::find_replace::{FindMatch, FindOptions, FindReplaceEngine};
use crate::theme::EditorTheme;

pub struct FindReplacePanel {
    pub is_open: bool,
    pub show_replace: bool,
    pub find_query: String,
    pub replace_query: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub is_regex: bool,
    pub matches: Vec<FindMatch>,
    pub active_match_index: Option<usize>,
    pub status_text: String,
}

impl Default for FindReplacePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl FindReplacePanel {
    pub fn new() -> Self {
        Self {
            is_open: false,
            show_replace: false,
            find_query: String::new(),
            replace_query: String::new(),
            case_sensitive: false,
            whole_word: false,
            is_regex: false,
            matches: Vec::new(),
            active_match_index: None,
            status_text: String::new(),
        }
    }

    pub fn open_find(&mut self, initial_text: Option<String>, buffer: &TextBuffer) {
        self.is_open = true;
        self.show_replace = false;
        if let Some(text) = initial_text {
            if !text.is_empty() && !text.contains('\n') {
                self.find_query = text;
            }
        }
        self.update_matches(buffer);
    }

    pub fn open_replace(&mut self, initial_text: Option<String>, buffer: &TextBuffer) {
        self.is_open = true;
        self.show_replace = true;
        if let Some(text) = initial_text {
            if !text.is_empty() && !text.contains('\n') {
                self.find_query = text;
            }
        }
        self.update_matches(buffer);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.matches.clear();
        self.active_match_index = None;
        self.status_text.clear();
    }

    pub fn options(&self) -> FindOptions {
        FindOptions {
            case_sensitive: self.case_sensitive,
            whole_word: self.whole_word,
            is_regex: self.is_regex,
        }
    }

    pub fn update_matches(&mut self, buffer: &TextBuffer) {
        if self.find_query.is_empty() {
            self.matches.clear();
            self.active_match_index = None;
            self.status_text = String::new();
            return;
        }

        let opts = self.options();
        self.matches = FindReplaceEngine::find_all(buffer, &self.find_query, &opts);

        if self.matches.is_empty() {
            self.active_match_index = None;
            self.status_text = "No matches".to_string();
        } else {
            if self.active_match_index.map_or(true, |idx| idx >= self.matches.len()) {
                self.active_match_index = Some(0);
            }
            let idx = self.active_match_index.unwrap_or(0);
            self.status_text = format!("{} of {} matches", idx + 1, self.matches.len());
        }
    }

    pub fn next_match(&mut self, buffer: &TextBuffer, selections: &mut SelectionSet) {
        if self.matches.is_empty() {
            self.update_matches(buffer);
        }
        if self.matches.is_empty() {
            return;
        }

        let next_idx = match self.active_match_index {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.active_match_index = Some(next_idx);
        let m = &self.matches[next_idx];
        selections.set_primary(Selection::range(m.range.start, m.range.end));
        self.status_text = format!("{} of {} matches", next_idx + 1, self.matches.len());
    }

    pub fn prev_match(&mut self, buffer: &TextBuffer, selections: &mut SelectionSet) {
        if self.matches.is_empty() {
            self.update_matches(buffer);
        }
        if self.matches.is_empty() {
            return;
        }

        let prev_idx = match self.active_match_index {
            Some(0) | None => self.matches.len().saturating_sub(1),
            Some(i) => i - 1,
        };
        self.active_match_index = Some(prev_idx);
        let m = &self.matches[prev_idx];
        selections.set_primary(Selection::range(m.range.start, m.range.end));
        self.status_text = format!("{} of {} matches", prev_idx + 1, self.matches.len());
    }

    pub fn replace_current(
        &mut self,
        buffer: &mut TextBuffer,
        selections: &mut SelectionSet,
    ) {
        if self.find_query.is_empty() {
            return;
        }
        let opts = self.options();
        let from_char = selections.primary().min_char();
        if let Some(new_range) = FindReplaceEngine::replace_next(
            buffer,
            &self.find_query,
            &self.replace_query,
            from_char,
            &opts,
        ) {
            self.update_matches(buffer);
            selections.set_primary(Selection::range(new_range.start, new_range.end));
        }
    }

    pub fn replace_all_matches(
        &mut self,
        buffer: &mut TextBuffer,
        selections: &mut SelectionSet,
    ) {
        if self.find_query.is_empty() {
            return;
        }
        let opts = self.options();
        let count = FindReplaceEngine::replace_all(buffer, &self.find_query, &self.replace_query, &opts);
        self.update_matches(buffer);
        self.status_text = format!("Replaced {} occurrences", count);
        let pos = buffer.len_chars().min(selections.primary().head);
        selections.set_primary(Selection::point(pos));
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: EditorTheme,
        buffer: &mut TextBuffer,
        selections: &mut SelectionSet,
    ) {
        if !self.is_open {
            return;
        }

        let panel_height = if self.show_replace { 74.0 } else { 42.0 };
        egui::Frame::none()
            .fill(theme.status_bar_bg())
            .stroke(Stroke::new(1.0_f32, theme.border()))
            .inner_margin(egui::Margin::symmetric(12.0, 6.0))
            .show(ui, |ui| {
                ui.set_height(panel_height);

                // Row 1: Find input, options, nav buttons, and close
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Find:").strong().size(13.0));

                    let find_edit = egui::TextEdit::singleline(&mut self.find_query)
                        .desired_width(220.0)
                        .hint_text("Find pattern...");
                    let resp = ui.add(find_edit);
                    if resp.changed() {
                        self.update_matches(buffer);
                    }

                    // Toggles
                    if ui.selectable_label(self.case_sensitive, egui::RichText::new("Aa").size(12.0)).clicked() {
                        self.case_sensitive = !self.case_sensitive;
                        self.update_matches(buffer);
                    }
                    if ui.selectable_label(self.whole_word, egui::RichText::new("\\b").size(12.0)).clicked() {
                        self.whole_word = !self.whole_word;
                        self.update_matches(buffer);
                    }
                    if ui.selectable_label(self.is_regex, egui::RichText::new(".*").size(12.0)).clicked() {
                        self.is_regex = !self.is_regex;
                        self.update_matches(buffer);
                    }

                    // Navigation buttons
                    if ui.button("◀ Prev").clicked() {
                        self.prev_match(buffer, selections);
                    }
                    if ui.button("Next ▶").clicked() {
                        self.next_match(buffer, selections);
                    }

                    // Toggle replace
                    let replace_icon = if self.show_replace { "▼ Replace" } else { "▶ Replace" };
                    if ui.small_button(replace_icon).clicked() {
                        self.show_replace = !self.show_replace;
                    }

                    // Status text
                    if !self.status_text.is_empty() {
                        ui.label(egui::RichText::new(&self.status_text).size(12.0).color(Color32::from_rgb(0xa6, 0xac, 0xcd)));
                    }

                    // Close button on far right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✕").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                            self.close();
                        }
                    });
                });

                // Row 2 (if replace mode active)
                if self.show_replace {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Replace:").strong().size(13.0));

                        let replace_edit = egui::TextEdit::singleline(&mut self.replace_query)
                            .desired_width(220.0)
                            .hint_text("Replace with...");
                        ui.add(replace_edit);

                        if ui.button("Replace").clicked() {
                            self.replace_current(buffer, selections);
                        }

                        if ui.button("Replace All").clicked() {
                            self.replace_all_matches(buffer, selections);
                        }
                    });
                }
            });
    }
}
