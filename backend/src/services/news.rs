use std::time::Duration;

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

/// Fetch and parse a single RSS feed
async fn fetch_feed(name: &str, url: &str) -> Result<Vec<NewsArticle>, String> {
    // Build HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    // Fetch feed content
    let response = client.get(url).await
        .map_err(|e| format!("Failed to fetch: {}", friendly_error(&e)))?;

    let bytes = response.bytes().await
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
                    .and_then(|t| t.content.as_ref())
                    .map(|s| strip_html(s))
                    .unwrap_or_else(|| "(No title)".to_string()),
                link: entry.links.first()
                    .map(|link| link.href.clone())
                    .unwrap_or_default(),
                snippet: entry.summary.as_ref()
                    .and_then(|s| s.content.as_ref())
                    .map(|s| strip_html(s))
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
