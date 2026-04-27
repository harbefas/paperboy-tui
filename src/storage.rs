use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub fn dir() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("paperboy")
}

pub fn load_feeds() -> Result<Vec<String>> {
    let path = dir().join("feeds.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub feed_url: String,
    pub read_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarredEntry {
    pub url: String,
    pub title: String,
    pub feed_url: String,
    pub starred_at: u64,
}

pub fn load_history() -> Result<Vec<HistoryEntry>> {
    read_jsonl(dir().join("history.jsonl"))
}

pub fn load_starred() -> Result<Vec<StarredEntry>> {
    read_jsonl(dir().join("starred.jsonl"))
}

pub fn append_history(entry: &HistoryEntry) -> Result<()> {
    append_jsonl(dir().join("history.jsonl"), entry)
}

pub fn append_starred(entry: &StarredEntry) -> Result<()> {
    append_jsonl(dir().join("starred.jsonl"), entry)
}

pub fn load_feed_titles() -> Result<std::collections::HashMap<String, String>> {
    let path = dir().join("feed-titles.json");
    if !path.exists() {
        return Ok(std::collections::HashMap::new());
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

pub fn save_feed_title(url: &str, title: &str) -> Result<()> {
    let mut titles = load_feed_titles().unwrap_or_default();
    titles.insert(url.to_string(), title.to_string());
    let path = dir().join("feed-titles.json");
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_string(&titles)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn read_jsonl<T: for<'de> Deserialize<'de>>(path: PathBuf) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut out = vec![];
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str(trimmed) {
            out.push(v);
        }
    }
    Ok(out)
}

fn append_jsonl<T: Serialize>(path: PathBuf, entry: &T) -> Result<()> {
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(entry)?)?;
    Ok(())
}
