use crate::fetch::Article;
use crate::theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};

pub struct ReaderPanel {
    pub article: Option<Article>,
    pub content: String,
    pub scroll: u16,
    pub content_height: u16,
    pub starred: bool,
}

impl ReaderPanel {
    pub fn new() -> Self {
        Self {
            article: None,
            content: String::new(),
            scroll: 0,
            content_height: 0,
            starred: false,
        }
    }

    pub fn set_article(&mut self, article: Article, content: String, starred: bool) {
        self.article = Some(article);
        self.content = content;
        self.scroll = 0;
        self.starred = starred;
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn page_down(&mut self, height: u16) {
        self.scroll = self.scroll.saturating_add(height.saturating_sub(4));
    }

    pub fn page_up(&mut self, height: u16) {
        self.scroll = self.scroll.saturating_sub(height.saturating_sub(4));
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let Some(article) = &self.article else {
            let empty = Paragraph::new(
                Line::from(Span::styled("select an article", Style::default().fg(theme::TX4))),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::BORDER))
                    .style(Style::default().bg(theme::BG)),
            );
            frame.render_widget(empty, area);
            return;
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        let star = if self.starred {
            Span::styled("★  ", Style::default().fg(theme::ACCENT))
        } else {
            Span::raw("")
        };

        let date = article.published.as_deref().unwrap_or("");

        let header = Paragraph::new(vec![
            Line::from(vec![
                star,
                Span::styled(
                    article.title.clone(),
                    Style::default().fg(theme::TX).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled(date, Style::default().fg(theme::TX3)),
                if !date.is_empty() { Span::styled("  ·  ", Style::default().fg(theme::TX4)) }
                else { Span::raw("") },
                Span::styled(
                    truncate_url(&article.url, 60),
                    Style::default().fg(theme::TX4),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER2))
                .style(Style::default().bg(theme::BG2)),
        );

        frame.render_widget(header, chunks[0]);

        let content_area = chunks[1];
        let inner_height = content_area.height.saturating_sub(2);
        let line_count = self.content.lines().count() as u16;
        self.content_height = line_count;

        let max_scroll = line_count.saturating_sub(inner_height);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        let progress_title = if line_count > inner_height {
            let pct = (self.scroll as f32 / max_scroll.max(1) as f32 * 100.0) as u16;
            format!(" {}% ", pct)
        } else {
            String::new()
        };

        let body = Paragraph::new(self.content.as_str())
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme::TX2))
            .block(
                Block::default()
                    .title(Span::styled(progress_title, Style::default().fg(theme::TX4)))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::BORDER))
                    .style(Style::default().bg(theme::BG)),
            );

        frame.render_widget(body, content_area);

        if line_count > inner_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme::BORDER2));
            let mut scrollbar_state =
                ScrollbarState::new(max_scroll as usize).position(self.scroll as usize);
            frame.render_stateful_widget(
                scrollbar,
                content_area.inner(Margin { horizontal: 0, vertical: 1 }),
                &mut scrollbar_state,
            );
        }
    }
}

fn truncate_url(url: &str, max: usize) -> String {
    if url.len() <= max {
        url.to_string()
    } else {
        format!("{}…", &url[..max])
    }
}
