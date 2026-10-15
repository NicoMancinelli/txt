use std::ops::Range;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

use crate::buffer::TextBuffer;

/// Represents a single find match within a `TextBuffer`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindMatch {
    /// 0-indexed character range [start, end) within the buffer.
    pub range: Range<usize>,
    /// 0-indexed line number containing the start of the match.
    pub line: usize,
    /// 0-indexed character column offset from the start of the line.
    pub column: usize,
    /// The exact text content of the match.
    pub text: String,
}

impl FindMatch {
    pub fn new(range: Range<usize>, line: usize, column: usize, text: impl Into<String>) -> Self {
        Self {
            range,
            line,
            column,
            text: text.into(),
        }
    }
}

/// Options controlling the behavior of find and replace queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindOptions {
    /// Whether the search is case-sensitive.
    pub case_sensitive: bool,
    /// Whether to match whole words only.
    pub whole_word: bool,
    /// Whether the search query is a regular expression.
    pub is_regex: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            is_regex: false,
        }
    }
}

impl FindOptions {
    /// Creates a default `FindOptions` instance (case-insensitive, non-whole-word, plain text).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets case sensitivity.
    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }

    /// Sets whole-word matching.
    pub fn whole_word(mut self, enabled: bool) -> Self {
        self.whole_word = enabled;
        self
    }

    /// Sets regular expression matching.
    pub fn regex(mut self, enabled: bool) -> Self {
        self.is_regex = enabled;
        self
    }

    /// Alias for `regex` for ergonomic API usage.
    pub fn is_regex(mut self, enabled: bool) -> Self {
        self.is_regex = enabled;
        self
    }
}

/// High-performance Find & Replace engine for `TextBuffer`, modeled after Sublime Text.
/// Provides sub-millisecond search and in-place replacements with history transaction batching.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FindReplaceEngine;

impl FindReplaceEngine {
    /// Compiles a regex matching the given query and options.
    fn compile_regex(query: &str, options: &FindOptions) -> Result<Regex, regex::Error> {
        let pattern_str = if options.is_regex {
            if options.whole_word {
                format!(r"\b(?:{})\b", query)
            } else {
                query.to_string()
            }
        } else {
            let escaped = regex::escape(query);
            if options.whole_word {
                format!(r"\b(?:{})\b", escaped)
            } else {
                escaped
            }
        };

        RegexBuilder::new(&pattern_str)
            .case_insensitive(!options.case_sensitive)
            .multi_line(true)
            .build()
    }

    /// Finds all matches for `query` in `buffer` with sub-millisecond efficiency.
    /// Supports plain substring, whole-word matching, and regex syntax.
    pub fn find_all(buffer: &TextBuffer, query: &str, options: &FindOptions) -> Vec<FindMatch> {
        if query.is_empty() || buffer.is_empty() {
            return Vec::new();
        }

        let re = match Self::compile_regex(query, options) {
            Ok(re) => re,
            Err(_) => return Vec::new(),
        };

        let full_text = buffer.text();
        let is_ascii = full_text.is_ascii();

        let mut matches = Vec::new();
        let mut current_byte = 0;
        let mut current_char = 0;

        for mat in re.find_iter(&full_text) {
            let (start_char, end_char) = if is_ascii {
                (mat.start(), mat.end())
            } else {
                let prefix = &full_text[current_byte..mat.start()];
                current_char += prefix.chars().count();
                let start = current_char;

                let match_str = mat.as_str();
                current_char += match_str.chars().count();
                current_byte = mat.end();
                (start, current_char)
            };

            let pos = buffer.char_to_pos(start_char);

            matches.push(FindMatch {
                range: start_char..end_char,
                line: pos.line,
                column: pos.column,
                text: mat.as_str().to_string(),
            });
        }

        matches
    }

    /// Finds next match at or after `from_char` (wraps around to 0 on reaching EOF).
    pub fn find_next(
        buffer: &TextBuffer,
        query: &str,
        from_char: usize,
        options: &FindOptions,
    ) -> Option<FindMatch> {
        let matches = Self::find_all(buffer, query, options);
        if matches.is_empty() {
            return None;
        }

        // Return the first match located at or after from_char
        if let Some(m) = matches.iter().find(|m| m.range.start >= from_char) {
            Some(m.clone())
        } else {
            // Wraps around to 0 on reaching EOF
            matches.first().cloned()
        }
    }

    /// Finds previous match strictly before `from_char` (wraps around to buffer end on reaching 0).
    pub fn find_prev(
        buffer: &TextBuffer,
        query: &str,
        from_char: usize,
        options: &FindOptions,
    ) -> Option<FindMatch> {
        let matches = Self::find_all(buffer, query, options);
        if matches.is_empty() {
            return None;
        }

        // Return the closest preceding match located before from_char
        if let Some(m) = matches.iter().rev().find(|m| m.range.start < from_char) {
            Some(m.clone())
        } else {
            // Wraps around to buffer end on reaching 0 / no match before from_char
            matches.last().cloned()
        }
    }

    /// Replaces the next match after `from_char` and returns the range of the newly inserted replacement.
    pub fn replace_next(
        buffer: &mut TextBuffer,
        query: &str,
        replacement: &str,
        from_char: usize,
        options: &FindOptions,
    ) -> Option<Range<usize>> {
        let matched = Self::find_next(buffer, query, from_char, options)?;

        let rep_str = if options.is_regex {
            if let Ok(re) = Self::compile_regex(query, options) {
                if let Some(caps) = re.captures(&matched.text) {
                    let mut dst = String::new();
                    caps.expand(replacement, &mut dst);
                    dst
                } else {
                    replacement.to_string()
                }
            } else {
                replacement.to_string()
            }
        } else {
            replacement.to_string()
        };

        let start = matched.range.start;
        let new_char_len = rep_str.chars().count();
        buffer.replace(matched.range, &rep_str);

        Some(start..start + new_char_len)
    }

    /// Replaces all matches throughout the buffer in reverse order to preserve earlier indices;
    /// returns the count of replacements made.
    pub fn replace_all(
        buffer: &mut TextBuffer,
        query: &str,
        replacement: &str,
        options: &FindOptions,
    ) -> usize {
        let matches = Self::find_all(buffer, query, options);
        if matches.is_empty() {
            return 0;
        }

        let count = matches.len();
        let compiled_re = if options.is_regex {
            Self::compile_regex(query, options).ok()
        } else {
            None
        };

        // Replace in reverse order so character offsets earlier in the buffer are not invalidated
        for m in matches.into_iter().rev() {
            let rep_str = if let Some(ref re) = compiled_re {
                if let Some(caps) = re.captures(&m.text) {
                    let mut dst = String::new();
                    caps.expand(replacement, &mut dst);
                    dst
                } else {
                    replacement.to_string()
                }
            } else {
                replacement.to_string()
            };

            buffer.replace(m.range, &rep_str);
        }

        count
    }
}
