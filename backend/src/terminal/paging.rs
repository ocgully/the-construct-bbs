use crate::terminal::ansi::{AnsiWriter, Color};

/// A page of output for paginated display
#[derive(Debug, Clone)]
pub struct Page {
    pub lines: Vec<String>,
    pub is_last: bool,
}

impl Page {
    /// Convert page to ANSI string
    pub fn to_ansi(&self) -> String {
        self.lines.join("\r\n")
    }
}

/// Pager for handling paginated output with [More] prompts
pub struct Pager {
    terminal_rows: u16,
    reserved_rows: u16,
    lines_shown: u16,
}

impl Pager {
    /// Create a new pager with specified terminal height
    pub fn new(terminal_rows: u16) -> Self {
        Self {
            terminal_rows,
            reserved_rows: 2,
            lines_shown: 0,
        }
    }

    /// Calculate the number of lines that fit in a page
    pub fn page_size(&self) -> u16 {
        self.terminal_rows.saturating_sub(self.reserved_rows)
    }

    /// Reset the pager state
    pub fn reset(&mut self) {
        self.lines_shown = 0;
    }

    /// Check if a pause is needed before showing more content
    pub fn needs_pause(&self) -> bool {
        self.lines_shown >= self.page_size()
    }

    /// Add lines to the count
    pub fn add_lines(&mut self, count: u16) {
        self.lines_shown = self.lines_shown.saturating_add(count);
    }

    /// Split text into pages based on terminal size
    pub fn paginate(&mut self, text: &str) -> Vec<Page> {
        let page_size = self.page_size() as usize;
        if page_size == 0 {
            return vec![Page {
                lines: vec![],
                is_last: true,
            }];
        }

        let all_lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();

        if all_lines.is_empty() {
            return vec![Page {
                lines: vec![],
                is_last: true,
            }];
        }

        let mut pages = Vec::new();
        let chunks: Vec<&[String]> = all_lines.chunks(page_size).collect();
        let total_pages = chunks.len();

        for (idx, chunk) in chunks.into_iter().enumerate() {
            pages.push(Page {
                lines: chunk.to_vec(),
                is_last: idx == total_pages - 1,
            });
        }

        pages
    }
}

/// Generate a [More] prompt with ANSI styling
pub fn more_prompt() -> String {
    let mut writer = AnsiWriter::new();
    writer.set_color(Color::Yellow, Color::Blue);
    writer.bold();
    writer.write_str(" [More] ");
    writer.reset_color();
    writer.flush()
}

/// Generate escape sequence to clear the [More] prompt
pub fn clear_more_prompt(cols: u16) -> String {
    let mut writer = AnsiWriter::new();
    writer.write_str("\r");
    writer.write_str(&" ".repeat(cols as usize));
    writer.write_str("\r");
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size() {
        let pager = Pager::new(25);
        assert_eq!(pager.page_size(), 23);
    }

    #[test]
    fn test_paginate_short_text() {
        let mut pager = Pager::new(25);
        let text = "Line 1\nLine 2\nLine 3";
        let pages = pager.paginate(text);

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].lines.len(), 3);
        assert!(pages[0].is_last);
    }

    #[test]
    fn test_paginate_multiple_pages() {
        let mut pager = Pager::new(10); // 8 lines per page (10 - 2 reserved)

        // Create text with 20 lines
        let lines: Vec<String> = (1..=20).map(|i| format!("Line {}", i)).collect();
        let text = lines.join("\n");

        let pages = pager.paginate(&text);

        assert_eq!(pages.len(), 3); // 8 + 8 + 4 lines
        assert_eq!(pages[0].lines.len(), 8);
        assert!(!pages[0].is_last);
        assert_eq!(pages[1].lines.len(), 8);
        assert!(!pages[1].is_last);
        assert_eq!(pages[2].lines.len(), 4);
        assert!(pages[2].is_last);
    }

    #[test]
    fn test_paginate_empty() {
        let mut pager = Pager::new(25);
        let pages = pager.paginate("");

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].lines.len(), 0);
        assert!(pages[0].is_last);
    }

    #[test]
    fn test_more_prompt_has_ansi() {
        let prompt = more_prompt();
        assert!(prompt.contains("\x1B["));
        assert!(prompt.contains("[More]"));
    }

    #[test]
    fn test_needs_pause() {
        let mut pager = Pager::new(10);
        assert!(!pager.needs_pause());

        pager.add_lines(8);
        assert!(pager.needs_pause());
    }

    #[test]
    fn test_reset() {
        let mut pager = Pager::new(10);
        pager.add_lines(8);
        assert!(pager.needs_pause());

        pager.reset();
        assert!(!pager.needs_pause());
    }

    #[test]
    fn test_page_to_ansi() {
        let page = Page {
            lines: vec!["Line 1".to_string(), "Line 2".to_string()],
            is_last: true,
        };

        let output = page.to_ansi();
        assert_eq!(output, "Line 1\r\nLine 2");
    }
}
