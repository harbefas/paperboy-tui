mod fetch;
mod storage;
mod theme;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::{
    collections::HashSet,
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use ui::{articles::ArticlesPanel, feeds::FeedsPanel, reader::ReaderPanel};

#[derive(PartialEq)]
enum Focus {
    Feeds,
    Articles,
    Reader,
}

struct App {
    focus: Focus,
    feeds: FeedsPanel,
    articles: ArticlesPanel,
    reader: ReaderPanel,
    status: String,
    reader_height: u16,
}

impl App {
    fn new() -> Result<Self> {
        let feed_urls = storage::load_feeds()?;
        let history = storage::load_history()?;
        let starred = storage::load_starred()?;

        let read_urls: HashSet<String> = history.into_iter().map(|h| h.url).collect();
        let starred_urls: HashSet<String> = starred.into_iter().map(|s| s.url).collect();

        let mut articles = ArticlesPanel::new();
        articles.read_urls = read_urls;
        articles.starred_urls = starred_urls;

        Ok(Self {
            focus: Focus::Feeds,
            feeds: FeedsPanel::new(feed_urls),
            articles,
            reader: ReaderPanel::new(),
            status: String::from("j/k navigate  Enter open  r refresh  q quit"),
            reader_height: 0,
        })
    }

    fn load_feed(&mut self, url: &str) {
        self.feeds.loading.push(url.to_string());
        self.feeds.errors.retain(|u| u != url);
        self.status = format!("fetching {}…", fetch::url_to_label(url));

        match fetch::fetch_feed(url) {
            Ok(result) => {
                self.feeds.loading.retain(|u| u != url);
                self.feeds.set_title(url, result.title);
                self.articles.set_articles(result.articles);
                self.status = format!(
                    "{} articles  —  j/k navigate  Enter read  s star  Esc back",
                    self.articles.articles.len()
                );
                self.focus = Focus::Articles;
            }
            Err(e) => {
                self.feeds.loading.retain(|u| u != url);
                self.feeds.errors.push(url.to_string());
                self.status = format!("error: {}", e);
            }
        }
    }

    fn open_article(&mut self) {
        let Some(article) = self.articles.selected().cloned() else { return; };

        let starred = self.articles.starred_urls.contains(&article.url);
        let text = fetch::fetch_article_text(&article);
        let feed_title = self
            .feeds
            .selected()
            .and_then(|url| self.feeds.titles.get(url))
            .cloned()
            .unwrap_or_else(|| fetch::url_to_label(article.feed_url.as_str()));

        if !self.articles.read_urls.contains(&article.url) {
            self.articles.read_urls.insert(article.url.clone());
            let entry = storage::HistoryEntry {
                url: article.url.clone(),
                title: article.title.clone(),
                feed_url: article.feed_url.clone(),
                read_at: now_secs(),
            };
            let _ = storage::append_history(&entry);
        }

        self.reader.set_article(article, feed_title, text, starred);
        self.focus = Focus::Reader;
        self.status = String::from("j/k scroll  d/u page  b browser  s star  Esc back  q quit");
    }

    fn toggle_star(&mut self) {
        let url = match self.focus {
            Focus::Articles => self.articles.selected().map(|a| a.url.clone()),
            Focus::Reader => self.reader.article.as_ref().map(|a| a.url.clone()),
            _ => None,
        };
        let Some(url) = url else { return };

        if self.articles.starred_urls.contains(&url) {
            self.articles.starred_urls.remove(&url);
            if let Focus::Reader = self.focus { self.reader.starred = false; }
            self.status = String::from("unstarred");
        } else {
            self.articles.starred_urls.insert(url.clone());
            if let Focus::Reader = self.focus { self.reader.starred = true; }
            let article = self.articles.articles.iter().find(|a| a.url == url)
                .or(self.reader.article.as_ref());
            if let Some(a) = article {
                let entry = storage::StarredEntry {
                    url: a.url.clone(),
                    title: a.title.clone(),
                    feed_url: a.feed_url.clone(),
                    starred_at: now_secs(),
                };
                let _ = storage::append_starred(&entry);
            }
            self.status = String::from("starred ★");
        }
    }

    fn open_in_browser(&self) {
        let url = match self.focus {
            Focus::Articles => self.articles.selected().map(|a| a.url.as_str()),
            Focus::Reader => self.reader.article.as_ref().map(|a| a.url.as_str()),
            _ => None,
        };
        if let Some(url) = url {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run(&mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new()?;
    let mut gg_pending = false;

    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(size);

            let main_area = outer[0];
            let status_area = outer[1];

            if app.focus == Focus::Reader {
                app.reader_height = main_area.height;
                app.reader.render_fullscreen(frame, main_area);
            } else {
                let panels = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(20),
                        Constraint::Percentage(40),
                        Constraint::Percentage(40),
                    ])
                    .split(main_area);

                app.reader_height = panels[2].height;
                app.feeds.render(frame, panels[0], app.focus == Focus::Feeds);
                app.articles.render(frame, panels[1], app.focus == Focus::Articles);
                app.reader.render(frame, panels[2]);
            }

            let status_line = Line::from(vec![
                Span::styled(" ■ ", Style::default().fg(theme::ORANGE)),
                Span::styled("paperboy  ", Style::default().fg(theme::TX3)),
                Span::styled(app.status.as_str(), Style::default().fg(theme::TX4)),
            ]);
            frame.render_widget(
                Paragraph::new(status_line).style(Style::default().bg(theme::BG2)),
                status_area,
            );
        })?;

        if !event::poll(Duration::from_millis(200))? { continue; }

        if let Event::Key(key) = event::read()? {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                break;
            }

            match app.focus {
                Focus::Feeds => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => { gg_pending = false; app.feeds.next(); }
                    KeyCode::Char('k') | KeyCode::Up => { gg_pending = false; app.feeds.prev(); }
                    KeyCode::Char('g') => {
                        if gg_pending { app.feeds.first(); gg_pending = false; }
                        else { gg_pending = true; }
                    }
                    KeyCode::Char('G') => { gg_pending = false; app.feeds.last(); }
                    KeyCode::Enter | KeyCode::Char('r') => {
                        gg_pending = false;
                        if let Some(url) = app.feeds.selected().map(|s| s.to_string()) {
                            app.load_feed(&url);
                        }
                    }
                    KeyCode::Tab => {
                        gg_pending = false;
                        if !app.articles.articles.is_empty() {
                            app.focus = Focus::Articles;
                            app.status = String::from("j/k navigate  Enter read  s star  Esc back  q quit");
                        }
                    }
                    _ => { gg_pending = false; }
                },

                Focus::Articles => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => { gg_pending = false; app.articles.next(); }
                    KeyCode::Char('k') | KeyCode::Up => { gg_pending = false; app.articles.prev(); }
                    KeyCode::Char('g') => {
                        if gg_pending { app.articles.first(); gg_pending = false; }
                        else { gg_pending = true; }
                    }
                    KeyCode::Char('G') => { gg_pending = false; app.articles.last(); }
                    KeyCode::Enter => { gg_pending = false; app.open_article(); }
                    KeyCode::Char('s') => { gg_pending = false; app.toggle_star(); }
                    KeyCode::Char('b') => { gg_pending = false; app.open_in_browser(); }
                    KeyCode::Esc | KeyCode::BackTab => {
                        gg_pending = false;
                        app.focus = Focus::Feeds;
                        app.status = String::from("j/k navigate  Enter load  r refresh  q quit");
                    }
                    KeyCode::Tab => {
                        gg_pending = false;
                        if app.reader.article.is_some() {
                            app.open_article();
                        }
                    }
                    _ => { gg_pending = false; }
                },

                Focus::Reader => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => { app.reader.scroll_down(); }
                    KeyCode::Char('k') | KeyCode::Up => { app.reader.scroll_up(); }
                    KeyCode::Char('d') => { app.reader.page_down(app.reader_height); }
                    KeyCode::Char('u') => { app.reader.page_up(app.reader_height); }
                    KeyCode::Char('g') => {
                        if gg_pending { app.reader.scroll = 0; gg_pending = false; }
                        else { gg_pending = true; }
                    }
                    KeyCode::Char('G') => { gg_pending = false; app.reader.scroll = app.reader.content_height; }
                    KeyCode::Char('s') => { gg_pending = false; app.toggle_star(); }
                    KeyCode::Char('b') => { gg_pending = false; app.open_in_browser(); }
                    KeyCode::Esc | KeyCode::BackTab => {
                        gg_pending = false;
                        app.focus = Focus::Articles;
                        app.status = String::from("j/k navigate  Enter read  s star  b browser  Esc back");
                    }
                    _ => { gg_pending = false; }
                },
            }
        }
    }

    Ok(())
}
