use crate::fetch::Article;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
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
        self.scroll = self.scroll.saturating_add(height.saturating_sub(3));
    }

    pub fn page_up(&mut self, height: u16) {
        self.scroll = self.scroll.saturating_sub(height.saturating_sub(3));
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let Some(article) = &self.article else {
            let empty = Paragraph::new("no article selected")
                .style(Style::default().fg(Color::DarkGray))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                );
            frame.render_widget(empty, area);
            return;
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let star_indicator = if self.starred { "★ " } else { "" };
        let date = article.published.as_deref().unwrap_or("");

        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(star_indicator, Style::default().fg(Color::Yellow)),
                Span::styled(
                    article.title.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(date, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(&article.url, Style::default().fg(Color::DarkGray)),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
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

        let progress = if line_count > inner_height {
            let pct = (self.scroll as f32 / max_scroll as f32 * 100.0) as u16;
            format!(" {}% ", pct)
        } else {
            String::new()
        };

        let body = Paragraph::new(self.content.as_str())
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title(progress)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        frame.render_widget(body, content_area);

        if line_count > inner_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
                .position(self.scroll as usize);
            frame.render_stateful_widget(
                scrollbar,
                content_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }),
                &mut scrollbar_state,
            );
        }
    }
}
