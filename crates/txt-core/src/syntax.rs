use std::ops::Range;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    Keyword,
    Type,
    Function,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    PlainText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxToken {
    pub token_type: TokenType,
    pub range: Range<usize>, // Character offset range within the line
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    Json,
    Markdown,
    PlainText,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "json" => Language::Json,
            "md" | "markdown" => Language::Markdown,
            _ => Language::PlainText,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::Json => "JSON",
            Language::Markdown => "Markdown",
            Language::PlainText => "Plain Text",
        }
    }
}

pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    /// Tokenizes a single line of text for rendering.
    pub fn tokenize_line(line: &str, language: Language) -> Vec<SyntaxToken> {
        match language {
            Language::Rust => Self::tokenize_rust(line),
            Language::Python => Self::tokenize_python(line),
            Language::Json => Self::tokenize_json(line),
            Language::Markdown => Self::tokenize_markdown(line),
            Language::PlainText => vec![SyntaxToken {
                token_type: TokenType::PlainText,
                range: 0..line.chars().count(),
            }],
        }
    }

    fn tokenize_rust(line: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Line comment //
            if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
                tokens.push(SyntaxToken {
                    token_type: TokenType::Comment,
                    range: i..len,
                });
                break;
            }

            // String literal "..."
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < len {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    token_type: TokenType::String,
                    range: start..i,
                });
                continue;
            }

            // Identifiers / Keywords / Types
            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();

                let token_type = if Self::is_rust_keyword(&word) {
                    TokenType::Keyword
                } else if Self::is_rust_type(&word) {
                    TokenType::Type
                } else if i < len && chars[i] == '(' {
                    TokenType::Function
                } else {
                    TokenType::PlainText
                };

                tokens.push(SyntaxToken {
                    token_type,
                    range: start..i,
                });
                continue;
            }

            // Numbers
            if chars[i].is_ascii_digit() {
                let start = i;
                while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '.') {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    token_type: TokenType::Number,
                    range: start..i,
                });
                continue;
            }

            // Operators & Punctuation
            if "+-*/%=<>!&|^~".contains(chars[i]) {
                tokens.push(SyntaxToken {
                    token_type: TokenType::Operator,
                    range: i..i + 1,
                });
                i += 1;
                continue;
            }

            if "(){}[];:,".contains(chars[i]) {
                tokens.push(SyntaxToken {
                    token_type: TokenType::Punctuation,
                    range: i..i + 1,
                });
                i += 1;
                continue;
            }

            // Whitespace or unclassified
            let start = i;
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i > start {
                tokens.push(SyntaxToken {
                    token_type: TokenType::PlainText,
                    range: start..i,
                });
            } else {
                tokens.push(SyntaxToken {
                    token_type: TokenType::PlainText,
                    range: i..i + 1,
                });
                i += 1;
            }
        }

        tokens
    }

    fn tokenize_python(line: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            if chars[i] == '#' {
                tokens.push(SyntaxToken {
                    token_type: TokenType::Comment,
                    range: i..len,
                });
                break;
            }

            if chars[i] == '"' || chars[i] == '\'' {
                let quote = chars[i];
                let start = i;
                i += 1;
                while i < len && chars[i] != quote {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                        continue;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    token_type: TokenType::String,
                    range: start..i,
                });
                continue;
            }

            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let token_type = if Self::is_python_keyword(&word) {
                    TokenType::Keyword
                } else if i < len && chars[i] == '(' {
                    TokenType::Function
                } else {
                    TokenType::PlainText
                };
                tokens.push(SyntaxToken {
                    token_type,
                    range: start..i,
                });
                continue;
            }

            if chars[i].is_ascii_digit() {
                let start = i;
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    token_type: TokenType::Number,
                    range: start..i,
                });
                continue;
            }

            tokens.push(SyntaxToken {
                token_type: TokenType::PlainText,
                range: i..i + 1,
            });
            i += 1;
        }

        tokens
    }

    fn tokenize_json(line: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < len && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2;
                        continue;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                // Check if key or value by looking ahead for colon
                let mut is_key = false;
                let mut peek = i;
                while peek < len && chars[peek].is_whitespace() {
                    peek += 1;
                }
                if peek < len && chars[peek] == ':' {
                    is_key = true;
                }

                tokens.push(SyntaxToken {
                    token_type: if is_key { TokenType::Keyword } else { TokenType::String },
                    range: start..i,
                });
                continue;
            }

            if chars[i].is_ascii_digit() || chars[i] == '-' {
                let start = i;
                i += 1;
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == 'e') {
                    i += 1;
                }
                tokens.push(SyntaxToken {
                    token_type: TokenType::Number,
                    range: start..i,
                });
                continue;
            }

            tokens.push(SyntaxToken {
                token_type: TokenType::PlainText,
                range: i..i + 1,
            });
            i += 1;
        }

        tokens
    }

    fn tokenize_markdown(line: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();

        if line.starts_with('#') {
            tokens.push(SyntaxToken {
                token_type: TokenType::Keyword,
                range: 0..len,
            });
            return tokens;
        }

        tokens.push(SyntaxToken {
            token_type: TokenType::PlainText,
            range: 0..len,
        });
        tokens
    }

    fn is_rust_keyword(word: &str) -> bool {
        matches!(
            word,
            "fn" | "let" | "mut" | "pub" | "struct" | "enum" | "impl" | "trait" | "type"
                | "match" | "if" | "else" | "while" | "for" | "in" | "loop" | "return" | "break"
                | "continue" | "use" | "mod" | "crate" | "super" | "self" | "Self" | "const"
                | "static" | "ref" | "move" | "where" | "as" | "async" | "await" | "unsafe"
        )
    }

    fn is_rust_type(word: &str) -> bool {
        matches!(
            word,
            "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32"
                | "i64" | "i128" | "isize" | "f32" | "f64" | "char" | "str" | "String"
                | "Option" | "Result" | "Vec" | "Box" | "Rc" | "Arc"
        ) || (word.starts_with(char::is_uppercase) && word.chars().any(char::is_lowercase))
    }

    fn is_python_keyword(word: &str) -> bool {
        matches!(
            word,
            "def" | "class" | "return" | "if" | "elif" | "else" | "for" | "while" | "try"
                | "except" | "finally" | "with" | "as" | "import" | "from" | "raise" | "pass"
                | "lambda" | "yield" | "break" | "continue" | "in" | "is" | "not" | "and" | "or"
                | "True" | "False" | "None" | "async" | "await"
        )
    }
}
