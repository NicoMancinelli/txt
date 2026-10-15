use egui::Color32;
use txt_core::syntax::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTheme {
    Mariana,
    Monokai,
    SolarizedDark,
    SolarizedLight,
    OneDark,
    Dracula,
    Nord,
}

#[allow(dead_code)]
impl EditorTheme {
    pub fn all() -> &'static [EditorTheme] {
        &[
            EditorTheme::Mariana,
            EditorTheme::Monokai,
            EditorTheme::SolarizedDark,
            EditorTheme::SolarizedLight,
            EditorTheme::OneDark,
            EditorTheme::Dracula,
            EditorTheme::Nord,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EditorTheme::Mariana => "Mariana",
            EditorTheme::Monokai => "Monokai",
            EditorTheme::SolarizedDark => "Solarized Dark",
            EditorTheme::SolarizedLight => "Solarized Light",
            EditorTheme::OneDark => "One Dark",
            EditorTheme::Dracula => "Dracula",
            EditorTheme::Nord => "Nord",
        }
    }

    pub fn from_name(name: &str) -> Option<EditorTheme> {
        let clean = name.trim().to_lowercase().replace(['-', '_', ' '], "");
        match clean.as_str() {
            "mariana" => Some(EditorTheme::Mariana),
            "monokai" => Some(EditorTheme::Monokai),
            "solarizeddark" => Some(EditorTheme::SolarizedDark),
            "solarizedlight" => Some(EditorTheme::SolarizedLight),
            "onedark" => Some(EditorTheme::OneDark),
            "dracula" => Some(EditorTheme::Dracula),
            "nord" => Some(EditorTheme::Nord),
            _ => None,
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            EditorTheme::SolarizedLight => false,
            _ => true,
        }
    }

    pub fn bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x1e, 0x22, 0x27),
            EditorTheme::Monokai => Color32::from_rgb(0x27, 0x28, 0x22),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x00, 0x2b, 0x36),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xfd, 0xf6, 0xe3),
            EditorTheme::OneDark => Color32::from_rgb(0x28, 0x2c, 0x34),
            EditorTheme::Dracula => Color32::from_rgb(0x28, 0x2a, 0x36),
            EditorTheme::Nord => Color32::from_rgb(0x2e, 0x34, 0x40),
        }
    }

    pub fn sidebar_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x18, 0x1a, 0x1f),
            EditorTheme::Monokai => Color32::from_rgb(0x1e, 0x1f, 0x1c),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x00, 0x21, 0x2b),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xee, 0xe8, 0xd5),
            EditorTheme::OneDark => Color32::from_rgb(0x21, 0x25, 0x2b),
            EditorTheme::Dracula => Color32::from_rgb(0x21, 0x22, 0x2c),
            EditorTheme::Nord => Color32::from_rgb(0x27, 0x2c, 0x36),
        }
    }

    pub fn tab_bar_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x14, 0x16, 0x1a),
            EditorTheme::Monokai => Color32::from_rgb(0x18, 0x19, 0x16),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x00, 0x1b, 0x24),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xe6, 0xdf, 0xcb),
            EditorTheme::OneDark => Color32::from_rgb(0x1e, 0x22, 0x27),
            EditorTheme::Dracula => Color32::from_rgb(0x19, 0x1a, 0x21),
            EditorTheme::Nord => Color32::from_rgb(0x23, 0x27, 0x30),
        }
    }

    pub fn tab_active_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x1e, 0x22, 0x27),
            EditorTheme::Monokai => Color32::from_rgb(0x27, 0x28, 0x22),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x00, 0x2b, 0x36),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xfd, 0xf6, 0xe3),
            EditorTheme::OneDark => Color32::from_rgb(0x28, 0x2c, 0x34),
            EditorTheme::Dracula => Color32::from_rgb(0x28, 0x2a, 0x36),
            EditorTheme::Nord => Color32::from_rgb(0x2e, 0x34, 0x40),
        }
    }

    pub fn tab_inactive_text(&self) -> Color32 {
        match self {
            EditorTheme::SolarizedLight => Color32::from_rgb(0x93, 0xa1, 0xa1),
            EditorTheme::Monokai => Color32::from_rgb(0x8f, 0x90, 0x8a),
            EditorTheme::Dracula => Color32::from_rgb(0x62, 0x72, 0xa4),
            EditorTheme::Nord => Color32::from_rgb(0x61, 0x6e, 0x88),
            _ => Color32::from_rgb(0x9a, 0xa3, 0xb0),
        }
    }

    pub fn tab_active_text(&self) -> Color32 {
        match self {
            EditorTheme::SolarizedLight => Color32::from_rgb(0x58, 0x6e, 0x75),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x93, 0xa1, 0xa1),
            _ => Color32::WHITE,
        }
    }

    pub fn gutter_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x1a, 0x1d, 0x21),
            EditorTheme::Monokai => Color32::from_rgb(0x23, 0x24, 0x1f),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x00, 0x25, 0x30),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xee, 0xe8, 0xd5),
            EditorTheme::OneDark => Color32::from_rgb(0x21, 0x25, 0x2b),
            EditorTheme::Dracula => Color32::from_rgb(0x21, 0x22, 0x2c),
            EditorTheme::Nord => Color32::from_rgb(0x27, 0x2c, 0x36),
        }
    }

    pub fn gutter_text(&self) -> Color32 {
        match self {
            EditorTheme::SolarizedLight => Color32::from_rgb(0x93, 0xa1, 0xa1),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x58, 0x6e, 0x75),
            EditorTheme::Monokai => Color32::from_rgb(0x75, 0x71, 0x5e),
            EditorTheme::Dracula => Color32::from_rgb(0x62, 0x72, 0xa4),
            EditorTheme::Nord => Color32::from_rgb(0x4c, 0x56, 0x6a),
            _ => Color32::from_rgb(0x5c, 0x63, 0x70),
        }
    }

    pub fn gutter_active_text(&self) -> Color32 {
        match self {
            EditorTheme::SolarizedLight => Color32::from_rgb(0x58, 0x6e, 0x75),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x93, 0xa1, 0xa1),
            EditorTheme::Monokai => Color32::from_rgb(0xf8, 0xf8, 0xf2),
            EditorTheme::Dracula => Color32::from_rgb(0xf8, 0xf8, 0xf2),
            EditorTheme::Nord => Color32::from_rgb(0xd8, 0xde, 0xe9),
            _ => Color32::from_rgb(0xc0, 0xc5, 0xce),
        }
    }

    pub fn current_line_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x25, 0x2b, 0x32),
            EditorTheme::Monokai => Color32::from_rgb(0x32, 0x33, 0x2d),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x07, 0x36, 0x42),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xee, 0xe8, 0xd5),
            EditorTheme::OneDark => Color32::from_rgb(0x2c, 0x31, 0x3a),
            EditorTheme::Dracula => Color32::from_rgb(0x44, 0x47, 0x5a),
            EditorTheme::Nord => Color32::from_rgb(0x3b, 0x42, 0x52),
        }
    }

    pub fn selection_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x3e, 0x48, 0x57),
            EditorTheme::Monokai => Color32::from_rgb(0x49, 0x48, 0x3e),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x07, 0x36, 0x42),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xd6, 0xd0, 0xbf),
            EditorTheme::OneDark => Color32::from_rgb(0x3e, 0x44, 0x51),
            EditorTheme::Dracula => Color32::from_rgb(0x44, 0x47, 0x5a),
            EditorTheme::Nord => Color32::from_rgb(0x43, 0x4c, 0x5e),
        }
    }

    pub fn cursor_color(&self) -> Color32 {
        match self {
            EditorTheme::SolarizedLight => Color32::from_rgb(0x65, 0x7b, 0x83),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x83, 0x94, 0x96),
            EditorTheme::OneDark => Color32::from_rgb(0x52, 0x8b, 0xff),
            EditorTheme::Nord => Color32::from_rgb(0xd8, 0xde, 0xe9),
            _ => Color32::from_rgb(0xf8, 0xf8, 0xf0),
        }
    }

    pub fn status_bar_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x14, 0x16, 0x1a),
            EditorTheme::Monokai => Color32::from_rgb(0x18, 0x19, 0x16),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x00, 0x1b, 0x24),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xee, 0xe8, 0xd5),
            EditorTheme::OneDark => Color32::from_rgb(0x21, 0x25, 0x2b),
            EditorTheme::Dracula => Color32::from_rgb(0x19, 0x1a, 0x21),
            EditorTheme::Nord => Color32::from_rgb(0x23, 0x27, 0x30),
        }
    }

    pub fn status_bar_text(&self) -> Color32 {
        match self {
            EditorTheme::SolarizedLight => Color32::from_rgb(0x65, 0x7b, 0x83),
            EditorTheme::Monokai => Color32::from_rgb(0x8f, 0x90, 0x8a),
            EditorTheme::Dracula => Color32::from_rgb(0xf8, 0xf8, 0xf2),
            EditorTheme::Nord => Color32::from_rgb(0xd8, 0xde, 0xe9),
            _ => Color32::from_rgb(0xa0, 0xa8, 0xb6),
        }
    }

    pub fn modal_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x1e, 0x22, 0x27),
            EditorTheme::Monokai => Color32::from_rgb(0x27, 0x28, 0x22),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x00, 0x2b, 0x36),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xfd, 0xf6, 0xe3),
            EditorTheme::OneDark => Color32::from_rgb(0x28, 0x2c, 0x34),
            EditorTheme::Dracula => Color32::from_rgb(0x28, 0x2a, 0x36),
            EditorTheme::Nord => Color32::from_rgb(0x2e, 0x34, 0x40),
        }
    }

    pub fn modal_item_hover(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x2b, 0x31, 0x3c),
            EditorTheme::Monokai => Color32::from_rgb(0x3e, 0x3d, 0x32),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x07, 0x36, 0x42),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xee, 0xe8, 0xd5),
            EditorTheme::OneDark => Color32::from_rgb(0x2c, 0x31, 0x3a),
            EditorTheme::Dracula => Color32::from_rgb(0x44, 0x47, 0x5a),
            EditorTheme::Nord => Color32::from_rgb(0x3b, 0x42, 0x52),
        }
    }

    pub fn border(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x2f, 0x36, 0x40),
            EditorTheme::Monokai => Color32::from_rgb(0x3e, 0x3d, 0x32),
            EditorTheme::SolarizedDark => Color32::from_rgb(0x07, 0x36, 0x42),
            EditorTheme::SolarizedLight => Color32::from_rgb(0xd6, 0xd0, 0xbf),
            EditorTheme::OneDark => Color32::from_rgb(0x18, 0x1a, 0x1f),
            EditorTheme::Dracula => Color32::from_rgb(0x62, 0x72, 0xa4),
            EditorTheme::Nord => Color32::from_rgb(0x43, 0x4c, 0x5e),
        }
    }

    pub fn menu_bar_bg(&self) -> Color32 {
        self.tab_bar_bg()
    }

    pub fn menu_item_hover(&self) -> Color32 {
        self.modal_item_hover()
    }

    pub fn syntax_color(&self, token_type: TokenType) -> Color32 {
        match self {
            EditorTheme::Mariana => match token_type {
                TokenType::Keyword => Color32::from_rgb(0xc6, 0x78, 0xdd),
                TokenType::Type => Color32::from_rgb(0xe5, 0xc0, 0x7b),
                TokenType::Function => Color32::from_rgb(0x61, 0xaf, 0xef),
                TokenType::String => Color32::from_rgb(0x98, 0xc3, 0x79),
                TokenType::Number => Color32::from_rgb(0xd1, 0x9a, 0x66),
                TokenType::Comment => Color32::from_rgb(0x5c, 0x63, 0x70),
                TokenType::Operator => Color32::from_rgb(0x56, 0xb6, 0xc2),
                TokenType::Punctuation | TokenType::PlainText => {
                    Color32::from_rgb(0xab, 0xb2, 0xbf)
                }
            },
            EditorTheme::Monokai => match token_type {
                TokenType::Keyword => Color32::from_rgb(0xf9, 0x26, 0x72),
                TokenType::Type => Color32::from_rgb(0x66, 0xd9, 0xef),
                TokenType::Function => Color32::from_rgb(0xa6, 0xe2, 0x2e),
                TokenType::String => Color32::from_rgb(0xe6, 0xdb, 0x74),
                TokenType::Number => Color32::from_rgb(0xae, 0x81, 0xff),
                TokenType::Comment => Color32::from_rgb(0x75, 0x71, 0x5e),
                TokenType::Operator => Color32::from_rgb(0xf9, 0x26, 0x72),
                TokenType::Punctuation | TokenType::PlainText => {
                    Color32::from_rgb(0xf8, 0xf8, 0xf2)
                }
            },
            EditorTheme::SolarizedDark => match token_type {
                TokenType::Keyword => Color32::from_rgb(0x85, 0x99, 0x00),
                TokenType::Type => Color32::from_rgb(0xb5, 0x89, 0x00),
                TokenType::Function => Color32::from_rgb(0x26, 0x8b, 0xd2),
                TokenType::String => Color32::from_rgb(0x2a, 0xa1, 0x98),
                TokenType::Number => Color32::from_rgb(0xd3, 0x36, 0x82),
                TokenType::Comment => Color32::from_rgb(0x58, 0x6e, 0x75),
                TokenType::Operator => Color32::from_rgb(0xcb, 0x4b, 0x16),
                TokenType::Punctuation | TokenType::PlainText => {
                    Color32::from_rgb(0x83, 0x94, 0x96)
                }
            },
            EditorTheme::SolarizedLight => match token_type {
                TokenType::Keyword => Color32::from_rgb(0x85, 0x99, 0x00),
                TokenType::Type => Color32::from_rgb(0xb5, 0x89, 0x00),
                TokenType::Function => Color32::from_rgb(0x26, 0x8b, 0xd2),
                TokenType::String => Color32::from_rgb(0x2a, 0xa1, 0x98),
                TokenType::Number => Color32::from_rgb(0xd3, 0x36, 0x82),
                TokenType::Comment => Color32::from_rgb(0x93, 0xa1, 0xa1),
                TokenType::Operator => Color32::from_rgb(0xcb, 0x4b, 0x16),
                TokenType::Punctuation | TokenType::PlainText => {
                    Color32::from_rgb(0x65, 0x7b, 0x83)
                }
            },
            EditorTheme::OneDark => match token_type {
                TokenType::Keyword => Color32::from_rgb(0xc6, 0x78, 0xdd),
                TokenType::Type => Color32::from_rgb(0xe5, 0xc0, 0x7b),
                TokenType::Function => Color32::from_rgb(0x61, 0xaf, 0xef),
                TokenType::String => Color32::from_rgb(0x98, 0xc3, 0x79),
                TokenType::Number => Color32::from_rgb(0xd1, 0x9a, 0x66),
                TokenType::Comment => Color32::from_rgb(0x5c, 0x63, 0x70),
                TokenType::Operator => Color32::from_rgb(0x56, 0xb6, 0xc2),
                TokenType::Punctuation | TokenType::PlainText => {
                    Color32::from_rgb(0xab, 0xb2, 0xbf)
                }
            },
            EditorTheme::Dracula => match token_type {
                TokenType::Keyword => Color32::from_rgb(0xff, 0x79, 0xc6),
                TokenType::Type => Color32::from_rgb(0x8b, 0xe9, 0xfd),
                TokenType::Function => Color32::from_rgb(0x50, 0xfa, 0x7b),
                TokenType::String => Color32::from_rgb(0xf1, 0xfa, 0x8c),
                TokenType::Number => Color32::from_rgb(0xbd, 0x93, 0xf9),
                TokenType::Comment => Color32::from_rgb(0x62, 0x72, 0xa4),
                TokenType::Operator => Color32::from_rgb(0xff, 0x79, 0xc6),
                TokenType::Punctuation | TokenType::PlainText => {
                    Color32::from_rgb(0xf8, 0xf8, 0xf2)
                }
            },
            EditorTheme::Nord => match token_type {
                TokenType::Keyword => Color32::from_rgb(0x81, 0xa1, 0xc1),
                TokenType::Type => Color32::from_rgb(0x8f, 0xbc, 0xbb),
                TokenType::Function => Color32::from_rgb(0x88, 0xc0, 0xd0),
                TokenType::String => Color32::from_rgb(0xa3, 0xbe, 0x8c),
                TokenType::Number => Color32::from_rgb(0xb4, 0x8e, 0xad),
                TokenType::Comment => Color32::from_rgb(0x4c, 0x56, 0x6a),
                TokenType::Operator => Color32::from_rgb(0x81, 0xa1, 0xc1),
                TokenType::Punctuation | TokenType::PlainText => {
                    Color32::from_rgb(0xec, 0xef, 0xf4)
                }
            },
        }
    }
}
