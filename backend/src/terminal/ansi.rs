use codepage_437::{FromCp437, CP437_CONTROL};

/// CGA 16-color palette
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Color {
    Black,
    Red,
    Green,
    Brown,
    Blue,
    Magenta,
    Cyan,
    LightGray,
    DarkGray,
    LightRed,
    LightGreen,
    Yellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

impl Color {
    /// Returns the ANSI foreground color code
    pub fn fg_code(&self) -> u8 {
        match self {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Brown => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::LightGray => 37,
            Color::DarkGray => 90,
            Color::LightRed => 91,
            Color::LightGreen => 92,
            Color::Yellow => 93,
            Color::LightBlue => 94,
            Color::LightMagenta => 95,
            Color::LightCyan => 96,
            Color::White => 97,
        }
    }

    /// Returns the ANSI background color code
    pub fn bg_code(&self) -> u8 {
        match self {
            Color::Black => 40,
            Color::Red => 41,
            Color::Green => 42,
            Color::Brown => 43,
            Color::Blue => 44,
            Color::Magenta => 45,
            Color::Cyan => 46,
            Color::LightGray => 47,
            Color::DarkGray => 100,
            Color::LightRed => 101,
            Color::LightGreen => 102,
            Color::Yellow => 103,
            Color::LightBlue => 104,
            Color::LightMagenta => 105,
            Color::LightCyan => 106,
            Color::White => 107,
        }
    }
}

/// ANSI escape sequence writer for terminal output
pub struct AnsiWriter {
    buffer: String,
}

impl AnsiWriter {
    /// Create a new AnsiWriter
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Begin synchronized rendering (DECSET 2026)
    pub fn begin_sync(&mut self) {
        self.buffer.push_str("\x1B[?2026h");
    }

    /// End synchronized rendering (DECSET 2026)
    pub fn end_sync(&mut self) {
        self.buffer.push_str("\x1B[?2026l");
    }

    /// Clear screen and move cursor to home position
    pub fn clear_screen(&mut self) {
        self.buffer.push_str("\x1B[2J\x1B[H");
    }

    /// Move cursor to specified position (1-based)
    #[allow(dead_code)]
    pub fn move_cursor(&mut self, row: u16, col: u16) {
        self.buffer.push_str(&format!("\x1B[{};{}H", row, col));
    }

    /// Set foreground and background colors
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.buffer.push_str(&format!("\x1B[{};{}m", fg.fg_code(), bg.bg_code()));
    }

    /// Set foreground color only
    pub fn set_fg(&mut self, fg: Color) {
        self.buffer.push_str(&format!("\x1B[{}m", fg.fg_code()));
    }

    /// Set background color only
    pub fn set_bg(&mut self, bg: Color) {
        self.buffer.push_str(&format!("\x1B[{}m", bg.bg_code()));
    }

    /// Reset colors to default
    pub fn reset_color(&mut self) {
        self.buffer.push_str("\x1B[0m");
    }

    /// Enable bold text
    pub fn bold(&mut self) {
        self.buffer.push_str("\x1B[1m");
    }

    /// Write raw text to buffer
    pub fn write_str(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    /// Write text with line ending
    pub fn writeln(&mut self, text: &str) {
        self.buffer.push_str(text);
        self.buffer.push_str("\r\n");
    }

    /// Convert CP437 bytes to UTF-8 and write to buffer
    pub fn write_cp437(&mut self, bytes: &[u8]) {
        let text = cp437_to_utf8(bytes);
        self.buffer.push_str(&text);
    }

    /// Hide cursor
    #[allow(dead_code)]
    pub fn hide_cursor(&mut self) {
        self.buffer.push_str("\x1B[?25l");
    }

    /// Show cursor
    pub fn show_cursor(&mut self) {
        self.buffer.push_str("\x1B[?25h");
    }

    /// Flush buffer and return contents, leaving buffer empty
    pub fn flush(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }

    /// Get current buffer length
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for AnsiWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert CP437 bytes to UTF-8 string
pub fn cp437_to_utf8(bytes: &[u8]) -> String {
    String::from_cp437(bytes.to_vec(), &CP437_CONTROL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp437_box_drawing() {
        // Test common box-drawing characters
        assert_eq!(cp437_to_utf8(&[0xB3]), "│"); // Vertical line
        assert_eq!(cp437_to_utf8(&[0xC4]), "─"); // Horizontal line
        assert_eq!(cp437_to_utf8(&[0xDA]), "┌"); // Top-left corner
        assert_eq!(cp437_to_utf8(&[0xBF]), "┐"); // Top-right corner
        assert_eq!(cp437_to_utf8(&[0xC0]), "└"); // Bottom-left corner
        assert_eq!(cp437_to_utf8(&[0xD9]), "┘"); // Bottom-right corner
    }

    #[test]
    fn test_ansi_writer_colors() {
        let mut writer = AnsiWriter::new();
        writer.set_color(Color::LightCyan, Color::Blue);
        assert_eq!(writer.flush(), "\x1B[96;44m");
    }

    #[test]
    fn test_ansi_writer_clear_screen() {
        let mut writer = AnsiWriter::new();
        writer.clear_screen();
        assert_eq!(writer.flush(), "\x1B[2J\x1B[H");
    }

    #[test]
    fn test_ansi_writer_synchronized_rendering() {
        let mut writer = AnsiWriter::new();
        writer.begin_sync();
        writer.write_str("test");
        writer.end_sync();
        assert_eq!(writer.flush(), "\x1B[?2026htest\x1B[?2026l");
    }

    #[test]
    fn test_ansi_writer_flush() {
        let mut writer = AnsiWriter::new();
        writer.write_str("hello");
        assert_eq!(writer.len(), 5);
        assert!(!writer.is_empty());

        let content = writer.flush();
        assert_eq!(content, "hello");
        assert!(writer.is_empty());
        assert_eq!(writer.len(), 0);
    }

    #[test]
    fn test_color_codes() {
        // Test foreground codes
        assert_eq!(Color::Black.fg_code(), 30);
        assert_eq!(Color::Brown.fg_code(), 33);
        assert_eq!(Color::Yellow.fg_code(), 93);
        assert_eq!(Color::White.fg_code(), 97);

        // Test background codes
        assert_eq!(Color::Black.bg_code(), 40);
        assert_eq!(Color::Brown.bg_code(), 43);
        assert_eq!(Color::Yellow.bg_code(), 103);
        assert_eq!(Color::White.bg_code(), 107);
    }

    #[test]
    fn test_cursor_movement() {
        let mut writer = AnsiWriter::new();
        writer.move_cursor(10, 25);
        assert_eq!(writer.flush(), "\x1B[10;25H");
    }

    #[test]
    fn test_writeln() {
        let mut writer = AnsiWriter::new();
        writer.writeln("test line");
        assert_eq!(writer.flush(), "test line\r\n");
    }
}
