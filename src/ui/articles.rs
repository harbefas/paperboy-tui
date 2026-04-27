use crate::fetch::Article;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

pub struct ArticlesPanel {
    pub articles: Vec<Article>,
    pub state: ListState,
    pub read_urls: std::collections::HashSet<String>,
    pub starred_urls: std::collections::HashSet<String>,
}

impl ArticlesPanel {
    pub fn new() -> Self {
        Self {
            articles: vec![],
            state: ListState::default(),
            read_urls: std::collections::HashSet::new(),
            starred_urls: std::collections::HashSet::new(),
        }
    }

    pub fn set_articles(&mut self, articles: Vec<Article>) {
        self.articles = articles;
        if !self.articles.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    pub fn selected(&self) -> Option<&Article> {
        self.state.selected().and_then(|i| self.articles.get(i))
    }

    pub fn next(&mut self) {
        let len = self.articles.len();
        if len == 0 {
            return;
        }
        let i = self.state.selected().map(|i| (i + 1).min(len - 1)).unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = self.state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn first(&mut self) {
        if !self.articles.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn last(&mut self) {
        if !self.articles.is_empty() {
            self.state.select(Some(self.articles.len() - 1));
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let items: Vec<ListItem> = self
            .articles
            .iter()
            .map(|a| {
                let is_read = self.read_urls.contains(&a.url);
                let is_starred = self.starred_urls.contains(&a.url);

                let date = a.published.as_deref().unwrap_or("          ");

                let star = if is_starred {
                    Span::styled("★ ", Style::default().fg(Color::Yellow))
                } else {
                    Span::raw("  ")
                };

                let title_style = if is_read {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    star,
                    Span::styled(
                        format!("{} ", date),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(a.title.clone(), title_style),
                ]))
            })
            .collect();

        let title = format!(" articles ({}) ", self.articles.len());
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}
