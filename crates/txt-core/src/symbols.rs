use std::borrow::Cow;
use std::sync::OnceLock;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::buffer::TextBuffer;

/// Classification of a document symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Class,
    Heading,
    Constant,
    Variable,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "Function",
            SymbolKind::Struct => "Struct",
            SymbolKind::Enum => "Enum",
            SymbolKind::Trait => "Trait",
            SymbolKind::Class => "Class",
            SymbolKind::Heading => "Heading",
            SymbolKind::Constant => "Constant",
            SymbolKind::Variable => "Variable",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SymbolKind::Function => "ƒ",
            SymbolKind::Struct => "S",
            SymbolKind::Enum => "E",
            SymbolKind::Trait => "T",
            SymbolKind::Class => "C",
            SymbolKind::Heading => "#",
            SymbolKind::Constant => "c",
            SymbolKind::Variable => "v",
        }
    }
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A recognized symbol within a document buffer with line, column, and optional detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub column: usize,
    pub detail: Option<String>,
}

impl DocumentSymbol {
    pub fn new(name: impl Into<String>, kind: SymbolKind, line: usize, column: usize) -> Self {
        Self {
            name: name.into(),
            kind,
            line,
            column,
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Fast lexical document symbol extractor.
pub struct SymbolExtractor;

impl SymbolExtractor {
    /// Extracts symbols from a text buffer based on the specified language or file extension.
    pub fn extract_symbols(buffer: &TextBuffer, language: &str) -> Vec<DocumentSymbol> {
        let lang_str = if language.is_empty() || language.eq_ignore_ascii_case("auto") {
            buffer
                .file_path()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .unwrap_or("")
        } else {
            language
        };

        let lang_lower = lang_str.trim().to_ascii_lowercase();
        match lang_lower.as_str() {
            "rust" | "rs" => Self::extract_rust(buffer),
            "python" | "py" => Self::extract_python(buffer),
            "markdown" | "md" => Self::extract_markdown(buffer),
            "javascript" | "js" | "jsx" | "typescript" | "ts" | "tsx" => {
                Self::extract_javascript(buffer)
            }
            _ => Vec::new(),
        }
    }

    /// Extracts Rust functions, structs, enums, traits, impl blocks, and constants.
    pub fn extract_rust(buffer: &TextBuffer) -> Vec<DocumentSymbol> {
        static FN_RE: OnceLock<Regex> = OnceLock::new();
        static STRUCT_RE: OnceLock<Regex> = OnceLock::new();
        static ENUM_RE: OnceLock<Regex> = OnceLock::new();
        static TRAIT_RE: OnceLock<Regex> = OnceLock::new();
        static IMPL_RE: OnceLock<Regex> = OnceLock::new();
        static CONST_RE: OnceLock<Regex> = OnceLock::new();
        static STATIC_RE: OnceLock<Regex> = OnceLock::new();

        let fn_re = FN_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:default\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern(?:\s*"[^"]*")?\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });
        let struct_re = STRUCT_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?struct\s+([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });
        let enum_re = ENUM_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?enum\s+([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });
        let trait_re = TRAIT_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:unsafe\s+)?trait\s+([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });
        let impl_re = IMPL_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:unsafe\s+)?impl(?:<[^>]*>)?\s+([^{]+)"#).unwrap()
        });
        let const_re = CONST_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?const\s+([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });
        let static_re = STATIC_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?static\s+(?:mut\s+)?([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });

        let mut symbols = Vec::new();
        for line_idx in 0..buffer.len_lines() {
            let line_slice = match buffer.line(line_idx) {
                Some(slice) => slice,
                None => continue,
            };
            let line_cow: Cow<str> = match line_slice.as_str() {
                Some(s) => Cow::Borrowed(s),
                None => Cow::Owned(line_slice.to_string()),
            };
            let line = line_cow.trim_end_matches(['\r', '\n']);
            let trimmed = line.trim_start();

            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
            {
                continue;
            }

            // Check fn (before const, to handle `const fn`)
            if let Some(caps) = fn_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Function, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check struct
            if let Some(caps) = struct_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Struct, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check enum
            if let Some(caps) = enum_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Enum, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check trait
            if let Some(caps) = trait_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Trait, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check impl
            if let Some(caps) = impl_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let raw_header = m.as_str().trim();
                    let cleaned = raw_header
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .split("where")
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !cleaned.is_empty() {
                        let is_trait_impl = cleaned.contains(" for ");
                        let kind = if is_trait_impl {
                            SymbolKind::Trait
                        } else {
                            SymbolKind::Class
                        };
                        let name = cleaned.to_string();
                        let col = line.find(cleaned).unwrap_or(m.start());
                        let detail = format!("impl {}", cleaned);
                        symbols.push(
                            DocumentSymbol::new(name, kind, line_idx, col).with_detail(detail),
                        );
                        continue;
                    }
                }
            }

            // Check const
            if let Some(caps) = const_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('=')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Constant, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check static
            if let Some(caps) = static_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('=')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Variable, line_idx, col)
                            .with_detail(detail),
                    );
                }
            }
        }

        symbols
    }

    /// Extracts Python functions and classes.
    pub fn extract_python(buffer: &TextBuffer) -> Vec<DocumentSymbol> {
        static DEF_RE: OnceLock<Regex> = OnceLock::new();
        static CLASS_RE: OnceLock<Regex> = OnceLock::new();
        static VAR_RE: OnceLock<Regex> = OnceLock::new();

        let def_re = DEF_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:async\s+)?def\s+([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });
        let class_re = CLASS_RE.get_or_init(|| {
            Regex::new(r#"^\s*class\s+([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap()
        });
        let var_re = VAR_RE.get_or_init(|| {
            Regex::new(r#"^([a-zA-Z_][a-zA-Z0-9_]*)\s*="#).unwrap()
        });

        let mut symbols = Vec::new();
        for line_idx in 0..buffer.len_lines() {
            let line_slice = match buffer.line(line_idx) {
                Some(slice) => slice,
                None => continue,
            };
            let line_cow: Cow<str> = match line_slice.as_str() {
                Some(s) => Cow::Borrowed(s),
                None => Cow::Owned(line_slice.to_string()),
            };
            let line = line_cow.trim_end_matches(['\r', '\n']);
            let trimmed = line.trim_start();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(caps) = def_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line.trim().trim_end_matches(':').trim().to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Function, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            if let Some(caps) = class_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line.trim().trim_end_matches(':').trim().to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Class, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            if let Some(caps) = var_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let is_const = name.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
                    let kind = if is_const {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    };
                    let detail = line.trim().to_string();
                    symbols.push(
                        DocumentSymbol::new(name, kind, line_idx, col).with_detail(detail),
                    );
                }
            }
        }

        symbols
    }

    /// Extracts Markdown ATX headings (# Heading).
    pub fn extract_markdown(buffer: &TextBuffer) -> Vec<DocumentSymbol> {
        static HEADING_RE: OnceLock<Regex> = OnceLock::new();
        let heading_re = HEADING_RE.get_or_init(|| {
            Regex::new(r#"^\s*(#{1,6})\s+(.+)$"#).unwrap()
        });

        let mut symbols = Vec::new();
        for line_idx in 0..buffer.len_lines() {
            let line_slice = match buffer.line(line_idx) {
                Some(slice) => slice,
                None => continue,
            };
            let line_cow: Cow<str> = match line_slice.as_str() {
                Some(s) => Cow::Borrowed(s),
                None => Cow::Owned(line_slice.to_string()),
            };
            let line = line_cow.trim_end_matches(['\r', '\n']);

            if let Some(caps) = heading_re.captures(line) {
                if let (Some(hashes), Some(title_match)) = (caps.get(1), caps.get(2)) {
                    let level = hashes.as_str().len();
                    let title = title_match.as_str().trim().to_string();
                    let col = title_match.start();
                    let detail = format!("H{}", level);
                    symbols.push(
                        DocumentSymbol::new(title, SymbolKind::Heading, line_idx, col)
                            .with_detail(detail),
                    );
                }
            }
        }

        symbols
    }

    /// Extracts JavaScript / TypeScript functions, classes, arrow functions, constants, and variables.
    pub fn extract_javascript(buffer: &TextBuffer) -> Vec<DocumentSymbol> {
        static FN_RE: OnceLock<Regex> = OnceLock::new();
        static CLASS_RE: OnceLock<Regex> = OnceLock::new();
        static ARROW_FN_RE: OnceLock<Regex> = OnceLock::new();
        static EXPR_FN_RE: OnceLock<Regex> = OnceLock::new();
        static CONST_RE: OnceLock<Regex> = OnceLock::new();
        static VAR_RE: OnceLock<Regex> = OnceLock::new();

        let fn_re = FN_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:export\s+(?:default\s+)?)?(?:async\s+)?function(?:\s*\*|\s+)\s*([a-zA-Z_$][a-zA-Z0-9_$]*)"#).unwrap()
        });
        let class_re = CLASS_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:export\s+(?:default\s+)?)?class\s+([a-zA-Z_$][a-zA-Z0-9_$]*)"#).unwrap()
        });
        let arrow_fn_re = ARROW_FN_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:export\s+)?(?:const|let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*(?:async\s*)?(?:\([^)]*\)|[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=>"#).unwrap()
        });
        let expr_fn_re = EXPR_FN_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:export\s+)?(?:const|let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*(?:async\s*)?function"#).unwrap()
        });
        let const_re = CONST_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:export\s+)?const\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*="#).unwrap()
        });
        let var_re = VAR_RE.get_or_init(|| {
            Regex::new(r#"^\s*(?:export\s+)?(?:let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*="#).unwrap()
        });

        let mut symbols = Vec::new();
        for line_idx in 0..buffer.len_lines() {
            let line_slice = match buffer.line(line_idx) {
                Some(slice) => slice,
                None => continue,
            };
            let line_cow: Cow<str> = match line_slice.as_str() {
                Some(s) => Cow::Borrowed(s),
                None => Cow::Owned(line_slice.to_string()),
            };
            let line = line_cow.trim_end_matches(['\r', '\n']);
            let trimmed = line.trim_start();

            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
            {
                continue;
            }

            // Check standard function
            if let Some(caps) = fn_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Function, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check class
            if let Some(caps) = class_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Class, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check arrow function
            if let Some(caps) = arrow_fn_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Function, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check function expression
            if let Some(caps) = expr_fn_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Function, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check const
            if let Some(caps) = const_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('=')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Constant, line_idx, col)
                            .with_detail(detail),
                    );
                    continue;
                }
            }

            // Check let/var
            if let Some(caps) = var_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().to_string();
                    let col = m.start();
                    let detail = line
                        .trim()
                        .split('=')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    symbols.push(
                        DocumentSymbol::new(name, SymbolKind::Variable, line_idx, col)
                            .with_detail(detail),
                    );
                }
            }
        }

        symbols
    }
}
