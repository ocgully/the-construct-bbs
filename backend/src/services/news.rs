use std::time::Duration;
use crate::terminal::{AnsiWriter, Color};

/// A single news article extracted from an RSS feed
#[derive(Debug, Clone)]
pub struct NewsArticle {
    pub source: String,
    pub title: String,
    pub link: String,
    pub snippet: String,
    pub published: Option<String>,
}

/// Result of fetching all configured feeds
#[derive(Debug)]
pub struct FetchResult {
    pub articles: Vec<NewsArticle>,
    pub errors: Vec<String>,
}

/// Session state for news navigation
#[derive(Debug, Clone)]
pub struct NewsState {
    pub articles: Vec<NewsArticle>,
    pub errors: Vec<String>,
    pub selected_idx: usize,
    pub viewing_article: bool,
    pub page_offset: usize,
}

impl NewsState {
    /// Create new news state from fetch result
    pub fn new(result: FetchResult) -> Self {
        Self {
            articles: result.articles,
            errors: result.errors,
            selected_idx: 0,
            viewing_article: false,
            page_offset: 0,
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
            // Adjust page offset if needed
            if self.selected_idx < self.page_offset {
                self.page_offset = self.selected_idx;
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_idx < self.articles.len().saturating_sub(1) {
            self.selected_idx += 1;
            // Adjust page offset if needed (show 15 items per page)
            if self.selected_idx >= self.page_offset + 15 {
                self.page_offset = self.selected_idx.saturating_sub(14);
            }
        }
    }

    /// Enter article view for selected item
    pub fn enter_article(&mut self) {
        if !self.articles.is_empty() {
            self.viewing_article = true;
        }
    }

    /// Return to list view
    pub fn exit_article(&mut self) {
        self.viewing_article = false;
    }

    /// Get currently selected article
    pub fn current_article(&self) -> Option<&NewsArticle> {
        self.articles.get(self.selected_idx)
    }

    /// Check if there are any articles
    pub fn has_articles(&self) -> bool {
        !self.articles.is_empty()
    }
}

/// Fetch and parse a single RSS feed
async fn fetch_feed(name: &str, url: &str) -> Result<Vec<NewsArticle>, String> {
    // Build HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    // Fetch feed content
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch: {}", friendly_error(&e)))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Parse feed (handles RSS, Atom, JSON Feed automatically)
    let feed = feed_rs::parser::parse(&bytes[..])
        .map_err(|e| format!("Failed to parse feed: {}", e))?;

    // Extract articles (10 most recent per user decision)
    let articles: Vec<NewsArticle> = feed.entries.iter()
        .take(10)
        .map(|entry| {
            NewsArticle {
                source: name.to_string(),
                title: entry.title.as_ref()
                    .map(|t| strip_html(&t.content))
                    .unwrap_or_else(|| "(No title)".to_string()),
                link: entry.links.first()
                    .map(|link| link.href.clone())
                    .unwrap_or_default(),
                snippet: entry.summary.as_ref()
                    .map(|s| strip_html(&s.content))
                    .or_else(|| entry.content.as_ref()
                        .and_then(|c| c.body.as_ref())
                        .map(|s| strip_html(s)))
                    .unwrap_or_default(),
                published: entry.published.or(entry.updated)
                    .map(|dt| dt.to_rfc3339()),
            }
        })
        .collect();

    Ok(articles)
}

/// Fetch all configured feeds, returning articles grouped by source
pub async fn fetch_feeds(feeds: &[crate::config::NewsFeed]) -> FetchResult {
    let mut articles = Vec::new();
    let mut errors = Vec::new();

    for feed in feeds {
        match fetch_feed(&feed.name, &feed.url).await {
            Ok(feed_articles) => {
                articles.extend(feed_articles);
            }
            Err(e) => {
                eprintln!("Feed fetch failed for {}: {}", feed.name, e);
                errors.push(format!("{}: {}", feed.name, e));
            }
        }
    }

    FetchResult { articles, errors }
}

/// Convert HTTP errors to user-friendly messages
fn friendly_error(err: &reqwest::Error) -> String {
    let msg = err.to_string();
    if msg.contains("timeout") || msg.contains("timed out") {
        "Connection timeout".to_string()
    } else if msg.contains("dns") || msg.contains("resolve") {
        "Site not found".to_string()
    } else if err.status().map(|s| s.as_u16() == 404).unwrap_or(false) {
        "Feed not found".to_string()
    } else {
        "Unavailable".to_string()
    }
}

/// Strip HTML tags from text (simple regex-free approach)
fn strip_html(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .trim()
        .to_string()
}

// ============================================================================
// RENDER FUNCTIONS
// ============================================================================

/// Truncate a string to max length, appending "..." if truncated
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max.saturating_sub(3)).collect::<String>())
    }
}

/// Format date from RFC 3339 to "Mon DD" short format
fn format_date_short(rfc3339: &str) -> String {
    // Try to extract YYYY-MM-DD portion
    let date_part = if rfc3339.len() >= 10 { &rfc3339[..10] } else { rfc3339 };
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3 {
        let month = match parts[1] {
            "01" => "Jan", "02" => "Feb", "03" => "Mar", "04" => "Apr",
            "05" => "May", "06" => "Jun", "07" => "Jul", "08" => "Aug",
            "09" => "Sep", "10" => "Oct", "11" => "Nov", "12" => "Dec",
            _ => return rfc3339.to_string(),
        };
        format!("{} {}", month, parts[2])
    } else {
        rfc3339.to_string()
    }
}

/// Simple word wrap for text
fn word_wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.chars().count() + 1 + word.chars().count() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Render loading message while fetching feeds
pub fn render_news_loading() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    // Header
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
    w.write_str("\u{2551}");
    w.write_str(&format!("{:^38}", "THE WIRE"));
    w.writeln("\u{2551}");
    w.writeln("\u{255A}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255D}");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Connecting to news feeds...");
    w.reset_color();

    w.flush()
}

/// Render news list with selected item highlighted
///
/// Groups articles by source (as per user decision).
/// Shows 15 items per page, with navigation footer.
pub fn render_news_list(state: &NewsState) -> String {
    let mut w = AnsiWriter::new();
    let border_color = Color::Yellow;
    let title_color = Color::Yellow;
    let source_color = Color::LightCyan;
    let normal_color = Color::LightGray;
    let selected_bg = Color::Blue;
    let selected_fg = Color::White;

    let inner = 78;

    w.clear_screen();

    // Header with THE WIRE title
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(inner)));
    w.write_str("\u{2551}");
    w.set_fg(title_color);
    w.bold();
    w.write_str(&format!("{:^78}", "THE WIRE"));
    w.reset_color();
    w.set_fg(border_color);
    w.writeln("\u{2551}");
    w.writeln(&format!("\u{255A}{}\u{255D}", "\u{2550}".repeat(inner)));
    w.reset_color();
    w.writeln("");

    if state.articles.is_empty() {
        w.set_fg(Color::LightGray);
        w.writeln("  No news articles available.");
        if !state.errors.is_empty() {
            w.writeln("");
            w.set_fg(Color::LightRed);
            w.writeln("  Feed errors:");
            for err in &state.errors {
                w.writeln(&format!("    - {}", err));
            }
        }
        w.reset_color();
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln("  [Q] Return to menu");
        w.reset_color();
        return w.flush();
    }

    // Show articles (15 per visible page)
    let visible_count = 15;
    let start = state.page_offset;
    let end = (start + visible_count).min(state.articles.len());

    // Track current source for grouping
    let mut current_source: Option<&str> = None;

    for (display_idx, idx) in (start..end).enumerate() {
        let article = &state.articles[idx];
        let is_selected = idx == state.selected_idx;

        // Source header if changed
        if current_source != Some(&article.source) {
            current_source = Some(&article.source);
            if display_idx > 0 {
                w.writeln(""); // Blank line between sources
            }
            w.set_fg(source_color);
            w.bold();
            w.writeln(&format!("  [{}]", article.source));
            w.reset_color();
        }

        // Article line
        if is_selected {
            w.set_bg(selected_bg);
            w.set_fg(selected_fg);
            w.bold();
            w.write_str(" > ");
        } else {
            w.set_fg(normal_color);
            w.write_str("   ");
        }

        // Title (truncate to fit)
        let title = truncate(&article.title, 70);
        w.write_str(&title);

        // Pad to clear selection highlight
        let padding = 75_usize.saturating_sub(3 + title.chars().count());
        w.write_str(&" ".repeat(padding));

        w.reset_color();
        w.writeln("");

        // Snippet line for selected item
        if is_selected && !article.snippet.is_empty() {
            w.set_fg(Color::DarkGray);
            w.write_str("     ");
            w.writeln(&truncate(&article.snippet, 70));
            w.reset_color();
        }
    }

    // Page indicator
    let total_pages = (state.articles.len() + visible_count - 1) / visible_count;
    let current_page = state.page_offset / visible_count + 1;
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Page {} of {} ({} articles)", current_page, total_pages, state.articles.len()));

    // Show errors if any
    if !state.errors.is_empty() {
        w.set_fg(Color::LightRed);
        w.write_str("  (");
        w.write_str(&format!("{} feed{} unavailable", state.errors.len(), if state.errors.len() == 1 { "" } else { "s" }));
        w.writeln(")");
    }
    w.reset_color();

    // Navigation footer
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [\u{2191}\u{2193}] ");
    w.set_fg(Color::White);
    w.write_str("Navigate  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[Enter] ");
    w.set_fg(Color::White);
    w.write_str("Read  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[N/P] ");
    w.set_fg(Color::White);
    w.write_str("Page  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[Q] ");
    w.set_fg(Color::White);
    w.writeln("Quit");
    w.reset_color();

    w.flush()
}

/// Render full article view
pub fn render_news_article(article: &NewsArticle) -> String {
    let mut w = AnsiWriter::new();
    let border_color = Color::Yellow;
    let label_color = Color::LightGray;
    let title_color = Color::White;
    let source_color = Color::LightCyan;
    let body_color = Color::LightGray;

    let inner = 78;

    w.clear_screen();

    // Header box
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(inner)));

    // Source line
    w.write_str("\u{2551}  ");
    w.set_fg(label_color);
    w.write_str("Source: ");
    w.set_fg(source_color);
    w.bold();
    w.write_str(&article.source);
    w.reset_color();
    let source_len = 10 + article.source.len();
    w.write_str(&" ".repeat(inner.saturating_sub(source_len)));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Date line (if available)
    if let Some(ref published) = article.published {
        let date_str = format_date_short(published);
        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(label_color);
        w.write_str("Date: ");
        w.write_str(&date_str);
        let date_len = 8 + date_str.len();
        w.write_str(&" ".repeat(inner.saturating_sub(date_len)));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2500}".repeat(inner)));

    // Title (may wrap)
    w.set_fg(border_color);
    w.write_str("\u{2551}  ");
    w.set_fg(title_color);
    w.bold();
    // Word wrap title to 74 chars
    let title_lines = word_wrap(&article.title, 74);
    for (i, line) in title_lines.iter().enumerate() {
        if i > 0 {
            w.reset_color();
            w.set_fg(border_color);
            w.writeln("\u{2551}");
            w.write_str("\u{2551}  ");
            w.set_fg(title_color);
            w.bold();
        }
        w.write_str(line);
        let padding = inner.saturating_sub(2 + line.chars().count());
        w.reset_color();
        w.write_str(&" ".repeat(padding));
        w.set_fg(border_color);
    }
    w.writeln("\u{2551}");

    // Separator before body
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2500}".repeat(inner)));

    // Body/snippet
    w.reset_color();
    let body_text = if article.snippet.is_empty() {
        "(No content available)"
    } else {
        &article.snippet
    };

    let body_lines = word_wrap(body_text, 74);
    for line in &body_lines {
        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(body_color);
        w.write_str(line);
        let padding = inner.saturating_sub(2 + line.chars().count());
        w.write_str(&" ".repeat(padding));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Empty line in box
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.writeln("\u{2551}");

    // Bottom border
    w.writeln(&format!("\u{255A}{}\u{255D}", "\u{2550}".repeat(inner)));
    w.reset_color();

    // Link (if available)
    if !article.link.is_empty() {
        w.writeln("");
        w.set_fg(Color::LightGray);
        w.write_str("  Link: ");
        w.set_fg(Color::LightCyan);
        w.writeln(&truncate(&article.link, 68));
    }

    // Navigation
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.write_str("Back to list  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[N] ");
    w.set_fg(Color::White);
    w.write_str("Next article  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[P] ");
    w.set_fg(Color::White);
    w.writeln("Previous article");
    w.reset_color();

    w.flush()
}

/// Render error when all feeds failed
pub fn render_news_errors(errors: &[String]) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE WIRE");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightRed);
    w.writeln("  Unable to load news feeds:");
    w.writeln("");
    for err in errors {
        w.writeln(&format!("    - {}", err));
    }
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Press any key to return to menu.");
    w.reset_color();

    w.flush()
}
