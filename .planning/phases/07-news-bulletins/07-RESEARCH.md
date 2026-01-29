# Phase 7: News & Bulletins - Research

**Researched:** 2026-01-28
**Domain:** RSS feed parsing and terminal list navigation
**Confidence:** MEDIUM

## Summary

This phase implements RSS news feed display accessed via N hotkey on the main menu. User decisions specify multiple RSS feeds merged and grouped by source, fresh fetching on every access (no caching), list navigation with arrow keys, and full article view with [More] paging.

The standard Rust RSS parsing stack is feed-rs (v2.3.1) for unified RSS/Atom/JSON Feed parsing, combined with reqwest for async HTTP fetching. The existing terminal infrastructure (AnsiWriter, Pager) provides ANSI rendering and pagination. Navigation requires stateful list selection logic, following the render pattern established by the mail inbox (render functions return String, session state manages selection).

Key technical challenges: RSS feed parsing is notoriously fragile (10% of feeds are malformed XML at any given time), requiring lenient parsers and comprehensive error handling. Network failures must be handled gracefully with user-friendly error messages. List navigation requires tracking selected index and rendering highlighted items.

**Primary recommendation:** Use feed-rs for parsing with reqwest for fetching. Build a NewsService following the existing Service trait pattern. Create render functions that return String (matching mail.rs pattern), with session state managing list selection and article viewing.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| feed-rs | 2.3.1 | RSS/Atom/JSON Feed parsing | Unified API for all feed formats, lenient parser handles malformed feeds, supports common extensions (Dublin Core, iTunes, Media RSS) |
| reqwest | Latest (0.12+) | HTTP client for fetching feeds | De facto standard async HTTP client in Rust, integrates with tokio runtime already in use |
| tokio | 1.x (already present) | Async runtime | Already required by axum, provides async/await support for feed fetching |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4 (already present) | Date/time formatting for publication dates | Already in use for mail timestamps, reuse for article dates |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| feed-rs | rss + atom_syndication | Separate crates for RSS and Atom require format detection and dual code paths |
| feed-rs | rss_parser | Newer streaming parser but RSS-only (no Atom support), less mature |

**Installation:**
```bash
cargo add feed-rs
# reqwest already present via existing dependencies
# tokio already present
# chrono already present
```

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── services/
│   └── news.rs         # NewsService implementation + render functions
├── config.rs           # Add NewsConfig struct with RSS feed URLs
└── terminal/
    └── (existing)      # Reuse AnsiWriter, Pager
```

### Pattern 1: Render Functions Return String
**What:** All rendering logic lives in pure functions that build ANSI strings using AnsiWriter, matching the pattern established in mail.rs and chat.rs.

**When to use:** All news display views (list, article view, error messages).

**Example:**
```rust
// Source: Existing pattern from backend/src/services/mail.rs
pub fn render_news_list(articles: &[NewsArticle], selected_idx: usize) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    // Header
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("═══ THE WIRE ═══");
    w.reset_color();

    // Articles with selection highlight
    for (idx, article) in articles.iter().enumerate() {
        if idx == selected_idx {
            w.set_bg(Color::Blue);  // Highlight selected
        }
        w.write_str(&format!("[{}] {}", article.source, article.title));
        w.reset_color();
        w.writeln("");
    }

    w.flush()
}
```

### Pattern 2: Config-Driven Feed List
**What:** RSS feed URLs configured in config.toml, loaded at startup into Config struct.

**When to use:** Sysop-configurable feed sources.

**Example:**
```rust
// Source: Existing pattern from backend/src/config.rs
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewsConfig {
    #[serde(default = "default_feeds")]
    pub feeds: Vec<NewsFeed>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewsFeed {
    pub name: String,
    pub url: String,
}

fn default_feeds() -> Vec<NewsFeed> {
    vec![
        NewsFeed {
            name: "Hacker News".to_string(),
            url: "https://hnrss.org/newest".to_string(),
        },
    ]
}

// config.toml:
// [news]
// [[news.feeds]]
// name = "Hacker News"
// url = "https://hnrss.org/newest"
```

### Pattern 3: Stateful Session for Navigation
**What:** Session stores current view state (list selection index, current article, page number), while render functions remain pure.

**When to use:** List navigation with arrow keys.

**Example:**
```rust
// Session state (in websocket/session.rs)
struct NewsState {
    articles: Vec<NewsArticle>,
    selected_idx: usize,
    viewing_article: Option<usize>,
}

// Input handling
fn handle_news_input(&mut self, input: &str) {
    match input {
        "UP" => {
            if self.news_state.selected_idx > 0 {
                self.news_state.selected_idx -= 1;
                self.redraw_news_list();
            }
        }
        "DOWN" => {
            if self.news_state.selected_idx < self.news_state.articles.len() - 1 {
                self.news_state.selected_idx += 1;
                self.redraw_news_list();
            }
        }
        // ...
    }
}
```

### Pattern 4: Fresh Fetch on Every Access
**What:** No caching layer. Each time user accesses news, fetch fresh data from RSS feeds.

**When to use:** User decision specifies fresh fetching, no caching.

**Example:**
```rust
async fn fetch_all_feeds(feeds: &[NewsFeed]) -> Vec<NewsArticle> {
    let mut articles = Vec::new();

    for feed in feeds {
        match fetch_and_parse_feed(&feed.url).await {
            Ok(feed_articles) => {
                // Take 10 most recent from this feed
                articles.extend(feed_articles.into_iter().take(10));
            }
            Err(e) => {
                eprintln!("Failed to fetch {}: {}", feed.name, e);
                // Continue with other feeds
            }
        }
    }

    articles
}

async fn fetch_and_parse_feed(url: &str) -> Result<Vec<NewsArticle>, Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let feed = feed_rs::parser::parse(&bytes[..])?;

    // Convert feed entries to NewsArticle structs
    Ok(convert_feed_to_articles(&feed))
}
```

### Anti-Patterns to Avoid
- **Custom RSS parser:** Don't hand-roll XML parsing for RSS. Use feed-rs which handles malformed feeds gracefully.
- **Synchronous HTTP:** Don't use blocking HTTP calls. Use reqwest's async API with tokio.
- **Rendering in Service trait methods:** Don't write ANSI codes directly in on_enter/handle_input. Call render functions that return String.
- **Caching without expiry:** User explicitly wants fresh data. Don't add caching.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| RSS/Atom parsing | Custom XML parser with regex or string manipulation | feed-rs library | RSS feeds are frequently malformed (10% invalid XML at any time). feed-rs handles edge cases: unescaped entities, missing required fields, namespace variations, character encoding issues. |
| HTTP fetching | Manual TCP socket + HTTP protocol | reqwest | Handles redirects, TLS, connection pooling, timeouts, proper error types. |
| Date parsing from feeds | Custom date string parsing | feed-rs feed.updated field + chrono for formatting | RSS date formats vary wildly (RFC 822, RFC 3339, ISO 8601, custom). feed-rs normalizes these. |
| Feed format detection | Check XML tags to distinguish RSS vs Atom | feed-rs automatic detection | feed-rs detects format automatically, provides unified API. |

**Key insight:** RSS/Atom parsing is a domain where real-world data quality is terrible. ~10% of feeds are invalid XML. Parsers must be lenient, handle missing required fields, and provide fallbacks. feed-rs is battle-tested for this.

## Common Pitfalls

### Pitfall 1: Assuming Well-Formed XML
**What goes wrong:** Parser crashes or fails on real-world RSS feeds that contain unclosed tags, unescaped ampersands, or invalid characters.

**Why it happens:** RSS feeds are generated by various tools, many of which don't properly escape content. Blog posts contain user-generated content with special characters.

**How to avoid:** Use feed-rs which is lenient by design. Always handle parse errors gracefully and show user-friendly error messages. Test with real-world feeds, not just examples.

**Warning signs:** Parser fails on actual news sites but works on example feeds.

### Pitfall 2: Network Failure Handling
**What goes wrong:** Feed fetch hangs indefinitely, or network errors crash the BBS session.

**Why it happens:** External RSS feeds can be slow, down, or behind firewalls. No timeout on HTTP requests.

**How to avoid:** Set reasonable timeouts on reqwest (5-10 seconds). Handle all error cases:
- Connection timeout
- DNS failure
- HTTP 404/500 errors
- Empty response body
- Redirect loops

Show user-friendly error per feed (e.g., "CNN feed unavailable") and continue with other feeds.

**Warning signs:** BBS hangs when accessing news menu, sessions timeout waiting for slow feeds.

### Pitfall 3: Missing Required Fields
**What goes wrong:** Panic or error when accessing feed.title, entry.link, etc. because feed is missing required fields.

**Why it happens:** Many feeds omit supposedly "required" fields. feed-rs represents these as Option<T>.

**How to avoid:** Always use .unwrap_or() or match on Option fields. Provide sensible defaults:
- Missing title: "(No title)"
- Missing link: Don't make item clickable
- Missing description: Show empty snippet

**Warning signs:** Code panics on unwrap() when parsing feeds from certain sources.

### Pitfall 4: Character Encoding Issues
**What goes wrong:** Articles display with garbled characters (e.g., "â€™" instead of apostrophe).

**Why it happens:** RSS feeds may declare incorrect encoding, or contain characters not properly encoded as UTF-8.

**How to avoid:** feed-rs handles encoding internally. Ensure terminal output uses UTF-8. Test with international news sources.

**Warning signs:** Non-ASCII characters (quotes, dashes, accents) render incorrectly.

### Pitfall 5: Blocking UI on Fetch
**What goes wrong:** Terminal freezes while fetching feeds. User can't interact with BBS.

**Why it happens:** Fetching feeds in the input handler thread blocks the session.

**How to avoid:** Two approaches:
1. **Pre-fetch on menu enter:** Fetch all feeds async when user presses N, show "Loading..." message, then display list when ready.
2. **Background task:** Spawn tokio task for fetching, poll for completion.

For this phase: Pre-fetch on menu access is simpler and acceptable (fresh fetch per user decision).

**Warning signs:** Session becomes unresponsive when accessing news.

### Pitfall 6: Article List Ordering
**What goes wrong:** Articles from different feeds are interleaved, making it hard to see which feed they're from.

**Why it happens:** Merging articles chronologically mixes sources.

**How to avoid:** User decision specifies "grouped by feed". Keep articles grouped:
```
[Hacker News] Article 1
[Hacker News] Article 2
...
[Ars Technica] Article 1
[Ars Technica] Article 2
```

Don't merge chronologically across feeds.

**Warning signs:** Source attribution gets lost in mixed chronological list.

## Code Examples

Verified patterns from official sources:

### Fetching and Parsing RSS Feed
```rust
// Source: feed-rs docs at https://docs.rs/feed-rs
// Note: Adapted from documentation patterns

use feed_rs::parser;
use reqwest;

async fn fetch_feed(url: &str) -> Result<feed_rs::model::Feed, Box<dyn std::error::Error>> {
    // Fetch with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(url).await?;
    let bytes = response.bytes().await?;

    // Parse feed (handles RSS, Atom, JSON Feed automatically)
    let feed = parser::parse(&bytes[..])?;

    Ok(feed)
}

// Accessing feed data
fn extract_articles(feed: &feed_rs::model::Feed) -> Vec<NewsArticle> {
    feed.entries.iter()
        .map(|entry| NewsArticle {
            title: entry.title.as_ref()
                .and_then(|t| t.content.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "(No title)".to_string()),
            link: entry.links.first()
                .map(|link| link.href.clone())
                .unwrap_or_default(),
            description: entry.summary.as_ref()
                .and_then(|s| s.content.as_ref())
                .map(|s| s.to_string())
                .or_else(|| entry.content.as_ref()
                    .and_then(|c| c.body.clone()))
                .unwrap_or_default(),
            published: entry.published.or(entry.updated),
        })
        .collect()
}
```

### List Navigation with Selection Highlight
```rust
// Source: Existing ANSI pattern from backend/src/services/mail.rs

pub fn render_news_list(
    articles: &[NewsArticle],
    selected_idx: usize,
    source_names: &HashMap<String, String>,
) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();

    // Header
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("╔════════════════════════════════════════════════════════════════════════════╗");
    w.write_str("║");
    w.write_str(&format!("{:^78}", "THE WIRE"));
    w.writeln("║");
    w.writeln("╚════════════════════════════════════════════════════════════════════════════╝");
    w.reset_color();
    w.writeln("");

    // Article list
    for (idx, article) in articles.iter().enumerate() {
        let is_selected = idx == selected_idx;

        if is_selected {
            w.set_bg(Color::Blue);
            w.set_fg(Color::White);
            w.bold();
        } else {
            w.set_fg(Color::LightGray);
        }

        // Source tag
        w.write_str(&format!("[{}] ", article.source));
        w.set_fg(Color::White);
        w.writeln(&truncate(&article.title, 60));

        // Snippet on second line, indented
        if is_selected {
            w.reset_color();
            w.set_fg(Color::LightGray);
        }
        w.write_str("    ");
        w.writeln(&truncate(&article.snippet, 70));

        w.reset_color();
    }

    // Footer
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [↑↓] Select  [Enter] Read  [N] Next page  [P] Prev page  [Q] Quit");
    w.reset_color();

    w.flush()
}
```

### Error Handling for Feed Fetching
```rust
// Pattern for graceful degradation

async fn fetch_all_feeds(feeds: &[NewsFeed]) -> FetchResult {
    let mut articles = Vec::new();
    let mut errors = Vec::new();

    for feed in feeds {
        match fetch_feed(&feed.url).await {
            Ok(parsed_feed) => {
                let feed_articles = extract_articles(&parsed_feed);
                // Take 10 most recent (user decision)
                articles.extend(
                    feed_articles.into_iter()
                        .take(10)
                        .map(|mut a| {
                            a.source = feed.name.clone();
                            a
                        })
                );
            }
            Err(e) => {
                // Log error but continue with other feeds
                eprintln!("Feed fetch failed for {}: {}", feed.name, e);
                errors.push(format!("{}: {}", feed.name, friendly_error(&e)));
            }
        }
    }

    FetchResult { articles, errors }
}

fn friendly_error(err: &dyn std::error::Error) -> String {
    // Convert technical errors to user-friendly messages
    let msg = err.to_string();
    if msg.contains("timeout") {
        "Connection timeout"
    } else if msg.contains("dns") {
        "Site not found"
    } else if msg.contains("404") {
        "Feed not found"
    } else {
        "Unavailable"
    }.to_string()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate RSS/Atom parsers | Unified feed-rs parser | feed-rs 1.0 (2020) | Single API for all feed formats, automatic detection |
| Strict XML validation | Lenient parsing with fallbacks | Modern parsers (2015+) | Can handle real-world malformed feeds |
| Synchronous HTTP | Async reqwest with tokio | reqwest 0.11 (2021) | Non-blocking I/O, better for BBS with multiple users |

**Deprecated/outdated:**
- **rss crate 1.x**: Older RSS-only parser, superseded by feed-rs for multi-format support
- **atom_syndication separate crate**: feed-rs now handles both RSS and Atom in one API

## Open Questions

Things that couldn't be fully resolved:

1. **Arrow key input handling**
   - What we know: Terminal sends ANSI escape sequences for arrow keys (e.g., `\x1B[A` for up)
   - What's unclear: Exact escape sequence handling in the existing WebSocket session code
   - Recommendation: Check how existing code handles special keys. May need to detect escape sequences in character input stream or handle as special input strings.

2. **Article content cleaning**
   - What we know: RSS feed descriptions often contain HTML tags, which should not be rendered in terminal
   - What's unclear: Whether feed-rs automatically strips HTML, or if manual cleaning is needed
   - Recommendation: Test with real feeds. If HTML present, use a simple strip_tags function or regex to remove `<.*?>` patterns.

3. **Feed URL validation**
   - What we know: Sysop configures feed URLs in config.toml
   - What's unclear: Should BBS validate URLs at startup or only show errors when fetching fails?
   - Recommendation: Validate at startup (check URL format), log warnings for invalid URLs but don't crash. Show errors per-feed at runtime.

## Sources

### Primary (HIGH confidence)
- feed-rs crate documentation - https://docs.rs/feed-rs (API patterns, parsing examples)
- reqwest GitHub repository - https://github.com/seanmonstar/reqwest (HTTP client usage)
- Existing codebase:
  - backend/src/services/mail.rs (render pattern, list display, ANSI formatting)
  - backend/src/terminal/ansi.rs (AnsiWriter API)
  - backend/src/terminal/paging.rs (Pager for [More] prompt)
  - backend/src/config.rs (config structure patterns)

### Secondary (MEDIUM confidence)
- [feed-rs crates.io page](https://crates.io/crates/feed-rs) - version 2.3.1, released Dec 2024
- [Making HTTP requests in Rust with Reqwest - LogRocket Blog](https://blog.logrocket.com/making-http-requests-rust-reqwest/)
- [Rust and TUI: Building a command-line interface in Rust - LogRocket Blog](https://blog.logrocket.com/rust-and-tui-building-a-command-line-interface-in-rust/)

### Tertiary (LOW confidence - WebSearch only)
- [Parsing RSS At All Costs - xml.com](https://www.xml.com/pub/a/2003/01/22/dive-into-xml.html) - RSS parsing challenges, ~10% malformed feeds statistic
- [RSS Not Working: Troubleshooting Common Issues - visualping.io](https://visualping.io/blog/rss-is-not-working) - Common RSS feed errors
- [GitHub - AMythicDev/minus](https://github.com/AMythicDev/minus/) - Terminal paging library (referenced for patterns, but using existing Pager)

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM - feed-rs verified via docs/crates.io, reqwest already in use. Version 2.3.1 confirmed but couldn't verify all API details due to docs.rs JavaScript requirement.
- Architecture: HIGH - Patterns directly verified from existing codebase (mail.rs, chat.rs, config.rs). Service trait pattern, render functions, ANSI writer all established.
- Pitfalls: MEDIUM - RSS parsing issues well-documented across multiple sources. Malformed feed statistics from older but authoritative source. Network handling patterns standard for Rust async.

**Research date:** 2026-01-28
**Valid until:** 2026-02-28 (30 days - feed-rs and reqwest are stable libraries)
