/// Text manipulation routines modeled after Sublime Text's transform engine.
pub struct TextTransform;

impl TextTransform {
    pub fn to_uppercase(text: &str) -> String {
        text.to_uppercase()
    }

    pub fn to_lowercase(text: &str) -> String {
        text.to_lowercase()
    }

    pub fn to_title_case(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut capitalize_next = true;

        for c in text.chars() {
            if c.is_whitespace() || c == '_' || c == '-' {
                capitalize_next = true;
                result.push(c);
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        }
        result
    }

    pub fn to_snake_case(text: &str) -> String {
        let mut result = String::with_capacity(text.len() + 4);
        let mut prev_is_lower = false;

        for (i, c) in text.chars().enumerate() {
            if c.is_whitespace() || c == '-' {
                result.push('_');
                prev_is_lower = false;
            } else if c.is_uppercase() {
                if prev_is_lower && i > 0 {
                    result.push('_');
                }
                result.extend(c.to_lowercase());
                prev_is_lower = false;
            } else {
                result.push(c);
                prev_is_lower = true;
            }
        }
        result
    }

    pub fn to_kebab_case(text: &str) -> String {
        Self::to_snake_case(text).replace('_', "-")
    }

    pub fn to_camel_case(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut capitalize_next = false;

        for (i, c) in text.chars().enumerate() {
            if c.is_whitespace() || c == '_' || c == '-' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else if i == 0 {
                result.extend(c.to_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    pub fn convert_tabs_to_spaces(text: &str, tab_size: usize) -> String {
        let spaces = " ".repeat(tab_size);
        text.replace('\t', &spaces)
    }

    pub fn convert_spaces_to_tabs(text: &str, tab_size: usize) -> String {
        let spaces = " ".repeat(tab_size);
        text.replace(&spaces, "\t")
    }
}
