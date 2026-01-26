/// ANSI escape sequence buffering for WebSocket frames
///
/// Ensures complete ANSI sequences are sent to the client, preventing
/// partial sequences from causing rendering artifacts in xterm.js.
pub struct AnsiBuffer {
    buffer: Vec<u8>,
    in_escape: bool,
}

impl AnsiBuffer {
    /// Create a new empty ANSI buffer
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            in_escape: false,
        }
    }

    /// Process incoming bytes and return complete sequences/text chunks
    ///
    /// ANSI escape sequences:
    /// - ESC (0x1B) starts a sequence
    /// - Sequence ends with alphabetic character (A-Z, a-z) or ~
    /// - CSI sequences: ESC [ ... <letter>
    /// - OSC sequences: ESC ] ... <terminator>
    ///
    /// Incomplete sequences remain buffered until complete.
    pub fn push(&mut self, data: &[u8]) -> Vec<String> {
        let mut results = Vec::new();
        let mut current_chunk = Vec::new();

        for &byte in data {
            if self.in_escape {
                self.buffer.push(byte);

                // Check if this byte terminates the escape sequence
                if self.is_sequence_terminator(byte) {
                    // Complete sequence - flush buffer to current chunk
                    current_chunk.extend_from_slice(&self.buffer);
                    self.buffer.clear();
                    self.in_escape = false;
                }
            } else if byte == 0x1B {
                // Start of escape sequence
                // Flush any accumulated text first
                if !current_chunk.is_empty() {
                    if let Ok(s) = String::from_utf8(current_chunk.clone()) {
                        results.push(s);
                    }
                    current_chunk.clear();
                }

                self.buffer.push(byte);
                self.in_escape = true;
            } else {
                // Regular character
                current_chunk.push(byte);
            }
        }

        // Flush any remaining complete text
        if !current_chunk.is_empty() {
            if let Ok(s) = String::from_utf8(current_chunk) {
                results.push(s);
            }
        }

        results
    }

    /// Check if a byte terminates an ANSI escape sequence
    fn is_sequence_terminator(&self, byte: u8) -> bool {
        // After ESC, check if we have enough context
        if self.buffer.len() < 2 {
            return false;
        }

        // Get the character after ESC
        let second = self.buffer[1];

        match second {
            b'[' => {
                // CSI sequence: ESC [ <params> <letter>
                // Terminates with alphabetic character (A-Z, a-z)
                byte.is_ascii_alphabetic()
            }
            b']' => {
                // OSC sequence: ESC ] <text> BEL or ESC \
                byte == 0x07 || (byte == b'\\' && self.buffer.len() > 2 && self.buffer[self.buffer.len() - 2] == 0x1B)
            }
            b'(' | b')' => {
                // Character set selection: ESC ( <char>
                byte.is_ascii_alphabetic()
            }
            _ => {
                // Simple ESC sequences: ESC <letter>
                byte.is_ascii_alphabetic()
            }
        }
    }

    /// Force flush remaining buffered content (use on disconnect)
    pub fn flush(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }

        let result = String::from_utf8_lossy(&self.buffer).into_owned();
        self.buffer.clear();
        self.in_escape = false;
        Some(result)
    }
}

impl Default for AnsiBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_text_passes_through() {
        let mut buffer = AnsiBuffer::new();
        let results = buffer.push(b"Hello, world!");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "Hello, world!");
    }

    #[test]
    fn test_partial_escape_buffered() {
        let mut buffer = AnsiBuffer::new();

        // Send partial ESC sequence
        let results = buffer.push(b"\x1B[3");
        assert_eq!(results.len(), 0, "Partial sequence should be buffered");

        // Complete the sequence
        let results = buffer.push(b"1m");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "\x1B[31m");
    }

    #[test]
    fn test_complete_escape_sequence() {
        let mut buffer = AnsiBuffer::new();

        // Send complete ESC[31m (set red foreground)
        let results = buffer.push(b"\x1B[31m");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "\x1B[31m");
    }

    #[test]
    fn test_multiple_sequences() {
        let mut buffer = AnsiBuffer::new();

        // Send text + color + text in one push
        let results = buffer.push(b"Hello \x1B[31mRed\x1B[0m World");
        assert_eq!(results.len(), 5);
        assert_eq!(results[0], "Hello ");
        assert_eq!(results[1], "\x1B[31m");
        assert_eq!(results[2], "Red");
        assert_eq!(results[3], "\x1B[0m");
        assert_eq!(results[4], " World");
    }

    #[test]
    fn test_clear_screen_sequence() {
        let mut buffer = AnsiBuffer::new();

        // ESC[2J (clear screen) + ESC[H (home cursor)
        let results = buffer.push(b"\x1B[2J\x1B[H");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "\x1B[2J");
        assert_eq!(results[1], "\x1B[H");
    }

    #[test]
    fn test_synchronized_rendering() {
        let mut buffer = AnsiBuffer::new();

        // DECSET 2026: ESC[?2026h
        let results = buffer.push(b"\x1B[?2026h");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "\x1B[?2026h");
    }

    #[test]
    fn test_flush_remaining() {
        let mut buffer = AnsiBuffer::new();

        // Push partial sequence
        buffer.push(b"\x1B[3");

        // Flush should return the incomplete sequence
        let flushed = buffer.flush();
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap(), "\x1B[3");

        // Buffer should now be empty
        let flushed2 = buffer.flush();
        assert!(flushed2.is_none());
    }

    #[test]
    fn test_cursor_positioning() {
        let mut buffer = AnsiBuffer::new();

        // ESC[10;25H (move cursor to row 10, col 25)
        let results = buffer.push(b"\x1B[10;25H");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "\x1B[10;25H");
    }

    #[test]
    fn test_split_across_pushes() {
        let mut buffer = AnsiBuffer::new();

        // Split "Hello \x1B[31mWorld" across multiple pushes
        let r1 = buffer.push(b"Hel");
        let r2 = buffer.push(b"lo ");
        let r3 = buffer.push(b"\x1B[");
        let r4 = buffer.push(b"31");
        let r5 = buffer.push(b"m");
        let r6 = buffer.push(b"World");

        // Text accumulates until ESC starts
        assert_eq!(r1.len(), 1);
        assert_eq!(r1[0], "Hel");
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0], "lo ");

        // ESC sequence buffered
        assert_eq!(r3.len(), 0);
        assert_eq!(r4.len(), 0);

        // Sequence completes
        assert_eq!(r5.len(), 1);
        assert_eq!(r5[0], "\x1B[31m");

        // More text
        assert_eq!(r6.len(), 1);
        assert_eq!(r6[0], "World");
    }
}
