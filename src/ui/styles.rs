#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
    Rgb(u8, u8, u8),
}

impl Color {
    fn ansi_code(self) -> String {
        match self {
            Color::Black => "30".to_string(),
            Color::Red => "31".to_string(),
            Color::Green => "32".to_string(),
            Color::Yellow => "33".to_string(),
            Color::Blue => "34".to_string(),
            Color::Magenta => "35".to_string(),
            Color::Cyan => "36".to_string(),
            Color::White => "37".to_string(),
            Color::Default => "39".to_string(),
            Color::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
    pub dim: bool,
    pub underline: bool,
}

impl Style {
    pub fn new() -> Self {
        Self {
            fg: None,
            bold: false,
            dim: false,
            underline: false,
        }
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    fn apply_to(&self, text: &str, use_ansi: bool) -> String {
        if !use_ansi {
            return text.to_string();
        }

        let mut codes = Vec::new();

        if let Some(color) = self.fg {
            codes.push(color.ansi_code());
        }

        if self.bold {
            codes.push("1".to_string());
        }

        if self.dim {
            codes.push("2".to_string());
        }

        if self.underline {
            codes.push("4".to_string());
        }

        if codes.is_empty() {
            return text.to_string();
        }

        format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

pub fn styled(text: &str, style: Style) -> StyledString {
    StyledString {
        text: text.to_string(),
        style,
    }
}

pub fn dim(text: &str) -> StyledString {
    styled(text, Style::new().dim())
}

pub struct StyledString {
    text: String,
    style: Style,
}

impl StyledString {
    pub fn render(&self, use_ansi: bool) -> String {
        self.style.apply_to(&self.text, use_ansi)
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

static NO_COLOR_FLAG: AtomicBool = AtomicBool::new(false);

pub fn force_no_color() {
    NO_COLOR_FLAG.store(true, Ordering::Relaxed);
}

pub fn use_color() -> bool {
    use std::io::IsTerminal;
    if NO_COLOR_FLAG.load(Ordering::Relaxed) {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_default() {
        let style = Style::new();
        assert!(!style.bold);
        assert!(!style.dim);
        assert!(style.fg.is_none());
    }

    #[test]
    fn test_style_builder() {
        let style = Style::new().bold().dim().fg(Color::Red);
        assert!(style.bold);
        assert!(style.dim);
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(Color::from_hex("#00ff00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(Color::from_hex("0000ff"), Some(Color::Rgb(0, 0, 255)));
        assert_eq!(Color::from_hex("#abc"), None);
        assert_eq!(Color::from_hex("invalid"), None);
    }
}
