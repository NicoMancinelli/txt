use egui::Color32;
use txt_core::syntax::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTheme {
    Mariana,
    Monokai,
}

impl EditorTheme {
    pub fn bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x1e, 0x22, 0x27), // #1e2227
            EditorTheme::Monokai => Color32::from_rgb(0x27, 0x28, 0x22), // #272822
        }
    }

    pub fn sidebar_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x18, 0x1a, 0x1f),
            EditorTheme::Monokai => Color32::from_rgb(0x1e, 0x1f, 0x1c),
        }
    }

    pub fn tab_bar_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x14, 0x16, 0x1a),
            EditorTheme::Monokai => Color32::from_rgb(0x18, 0x19, 0x16),
        }
    }

    pub fn tab_active_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x1e, 0x22, 0x27),
            EditorTheme::Monokai => Color32::from_rgb(0x27, 0x28, 0x22),
        }
    }

    pub fn gutter_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x1a, 0x1d, 0x21),
            EditorTheme::Monokai => Color32::from_rgb(0x23, 0x24, 0x1f),
        }
    }

    pub fn gutter_text(&self) -> Color32 {
        Color32::from_rgb(0x5c, 0x63, 0x70)
    }

    pub fn current_line_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x25, 0x2b, 0x32),
            EditorTheme::Monokai => Color32::from_rgb(0x32, 0x33, 0x2d),
        }
    }

    pub fn selection_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x3e, 0x48, 0x57),
            EditorTheme::Monokai => Color32::from_rgb(0x49, 0x48, 0x3e),
        }
    }

    pub fn cursor_color(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0xf9, 0x91, 0x57), // Sublime orange cursor
            EditorTheme::Monokai => Color32::from_rgb(0xf8, 0xf8, 0xf0),
        }
    }

    pub fn status_bar_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x15, 0x18, 0x1c),
            EditorTheme::Monokai => Color32::from_rgb(0x19, 0x1a, 0x17),
        }
    }

    pub fn modal_bg(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x21, 0x26, 0x2d),
            EditorTheme::Monokai => Color32::from_rgb(0x2e, 0x2f, 0x29),
        }
    }

    pub fn modal_item_hover(&self) -> Color32 {
        match self {
            EditorTheme::Mariana => Color32::from_rgb(0x2d, 0x34, 0x3e),
            EditorTheme::Monokai => Color32::from_rgb(0x3e, 0x3f, 0x37),
        }
    }

    pub fn syntax_color(&self, token_type: TokenType) -> Color32 {
        match self {
            EditorTheme::Mariana => match token_type {
                TokenType::Keyword => Color32::from_rgb(0xc5, 0x94, 0xc5), // purple
                TokenType::Type => Color32::from_rgb(0x5f, 0xb3, 0xb3),    // teal
                TokenType::Function => Color32::from_rgb(0x66, 0x99, 0xcc),// blue
                TokenType::String => Color32::from_rgb(0x99, 0xc7, 0x94),  // green
                TokenType::Number => Color32::from_rgb(0xfa, 0xc8, 0x63),  // yellow
                TokenType::Comment => Color32::from_rgb(0x65, 0x73, 0x7e), // gray
                TokenType::Operator => Color32::from_rgb(0xf9, 0x91, 0x57),// orange
                TokenType::Punctuation => Color32::from_rgb(0xc0, 0xc5, 0xce),
                TokenType::PlainText => Color32::from_rgb(0xd8, 0xde, 0xe9),
            },
            EditorTheme::Monokai => match token_type {
                TokenType::Keyword => Color32::from_rgb(0xf9, 0x26, 0x72), // pink
                TokenType::Type => Color32::from_rgb(0x66, 0xd9, 0xef),    // cyan
                TokenType::Function => Color32::from_rgb(0xa6, 0xe2, 0x2e),// green
                TokenType::String => Color32::from_rgb(0xe6, 0xdb, 0x74),  // yellow
                TokenType::Number => Color32::from_rgb(0xae, 0x81, 0xff),  // purple
                TokenType::Comment => Color32::from_rgb(0x75, 0x71, 0x5e), // olive
                TokenType::Operator => Color32::from_rgb(0xf9, 0x26, 0x72),
                TokenType::Punctuation => Color32::from_rgb(0xf8, 0xf8, 0xf2),
                TokenType::PlainText => Color32::from_rgb(0xf8, 0xf8, 0xf2),
            },
        }
    }
}
