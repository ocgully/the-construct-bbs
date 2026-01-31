//! Profanity filter for Acromania submissions
//!
//! Optional filter (default ON per GAME_DECISIONS.md) that checks
//! submissions for offensive content.

use std::collections::HashSet;

/// Profanity filter for submission validation
pub struct ProfanityFilter {
    enabled: bool,
    blocked_words: HashSet<String>,
}

impl ProfanityFilter {
    /// Create a new filter (enabled by default)
    pub fn new(enabled: bool) -> Self {
        let mut filter = Self {
            enabled,
            blocked_words: HashSet::new(),
        };
        filter.load_default_words();
        filter
    }

    /// Load default blocked words
    fn load_default_words(&mut self) {
        // Basic profanity list - kept minimal and focused on clearly offensive terms
        // This is a family-friendly BBS after all
        let words = [
            // Slurs and hate speech (absolutely blocked)
            "n1gger", "nigger", "n1gga", "nigga",
            "f4ggot", "faggot", "f4g", "fag",
            "k1ke", "kike",
            "sp1c", "spic",
            "ch1nk", "chink",
            "wetback",
            "retard", "retarded",

            // Strong profanity
            "fuck", "f*ck", "fu*k", "fuk", "fck",
            "shit", "sh1t", "sh!t",
            "cunt", "c*nt",
            "cock", "c0ck",
            "dick", "d1ck",
            "pussy", "puss1",
            "bitch", "b1tch", "b!tch",
            "whore", "wh0re",
            "slut", "sl*t",
            "ass", "a$$", "@ss", // Only blocks standalone, not "class" etc.
        ];

        for word in words {
            self.blocked_words.insert(word.to_lowercase());
        }
    }

    /// Check if text contains blocked content
    /// Returns true if the text is CLEAN (passes filter)
    pub fn is_clean(&self, text: &str) -> bool {
        if !self.enabled {
            return true;
        }

        let lower = text.to_lowercase();

        // Check for blocked words
        for word in self.blocked_words.iter() {
            // Check as whole word or with common separators
            if self.contains_word(&lower, word) {
                return false;
            }
        }

        true
    }

    /// Check if text contains a blocked word (with word boundary detection)
    fn contains_word(&self, text: &str, word: &str) -> bool {
        // Simple word boundary check
        let text_chars: Vec<char> = text.chars().collect();
        let word_chars: Vec<char> = word.chars().collect();

        if word_chars.is_empty() || text_chars.len() < word_chars.len() {
            return false;
        }

        for i in 0..=text_chars.len() - word_chars.len() {
            let potential_match = &text_chars[i..i + word_chars.len()];
            if potential_match == word_chars.as_slice() {
                // Check word boundaries
                let before_ok = i == 0 || !text_chars[i - 1].is_alphanumeric();
                let after_ok = i + word_chars.len() >= text_chars.len()
                    || !text_chars[i + word_chars.len()].is_alphanumeric();

                // Special case: "ass" should not match in "class", "bass", "pass", etc.
                if word == "ass" {
                    if !before_ok || !after_ok {
                        continue;
                    }
                }

                if before_ok && after_ok {
                    return true;
                }
            }
        }

        false
    }

    /// Get a cleaned version of text (replaces profanity with asterisks)
    #[allow(dead_code)]
    pub fn clean_text(&self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let mut result = text.to_string();

        for word in self.blocked_words.iter() {
            let replacement = "*".repeat(word.len());
            // Case-insensitive replacement (simple approach)
            let lower = result.to_lowercase();
            if let Some(pos) = lower.find(word) {
                result.replace_range(pos..pos + word.len(), &replacement);
            }
        }

        result
    }

    /// Enable/disable the filter
    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if filter is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for ProfanityFilter {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_enabled_by_default() {
        let filter = ProfanityFilter::default();
        assert!(filter.is_enabled());
    }

    #[test]
    fn test_clean_text_passes() {
        let filter = ProfanityFilter::default();
        assert!(filter.is_clean("Hello World"));
        assert!(filter.is_clean("This is a test"));
        assert!(filter.is_clean("Wonderful Tiny Frogs Live"));
    }

    #[test]
    fn test_profanity_blocked() {
        let filter = ProfanityFilter::default();
        assert!(!filter.is_clean("What the fuck"));
        assert!(!filter.is_clean("That is shit")); // Standalone "shit"
    }

    #[test]
    fn test_disabled_filter_allows_all() {
        let filter = ProfanityFilter::new(false);
        assert!(filter.is_clean("What the fuck"));
        assert!(filter.is_clean("Any text passes"));
    }

    #[test]
    fn test_ass_word_boundary() {
        let filter = ProfanityFilter::default();
        // Should pass - "ass" is part of longer words
        assert!(filter.is_clean("Classical music"));
        assert!(filter.is_clean("Pass the test"));
        assert!(filter.is_clean("Bass guitar"));
        assert!(filter.is_clean("Massive success"));

        // Should fail - standalone "ass"
        assert!(!filter.is_clean("What an ass"));
        assert!(!filter.is_clean("Ass is blocked"));
    }

    #[test]
    fn test_case_insensitive() {
        let filter = ProfanityFilter::default();
        assert!(!filter.is_clean("FUCK"));
        assert!(!filter.is_clean("FuCk"));
        assert!(!filter.is_clean("fuck"));
    }

    #[test]
    fn test_enable_disable() {
        let mut filter = ProfanityFilter::default();
        assert!(filter.is_enabled());

        filter.set_enabled(false);
        assert!(!filter.is_enabled());
        assert!(filter.is_clean("fuck")); // Now passes

        filter.set_enabled(true);
        assert!(!filter.is_clean("fuck")); // Now blocked
    }
}
