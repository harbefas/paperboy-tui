use anyhow::Result;
use feed_rs::parser;

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub url: String,
    pub feed_url: String,
    pub html: String,
    pub published: Option<String>,
}

pub struct FeedResult {
    pub title: String,
    pub articles: Vec<Article>,
}

pub fn fetch_feed(url: &str) -> Result<FeedResult> {
    let body = reqwest::blocking::get(url)?.bytes()?;
    let feed = parser::parse(body.as_ref())?;

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| url_to_label(url));

    let articles = feed
        .entries
        .into_iter()
        .map(|e| {
            let title = e
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "(no title)".into());
            let link = e.links.first().map(|l| l.href.clone()).unwrap_or_default();
            let html = e
                .content
                .and_then(|c| c.body)
                .or_else(|| e.summary.map(|s| s.content))
                .unwrap_or_default();
            let published = e.published.map(|d| d.format("%Y-%m-%d").to_string());
            Article { title, url: link, feed_url: url.to_string(), html, published }
        })
        .collect();

    Ok(FeedResult { title, articles })
}

pub fn url_to_label(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string()
}
