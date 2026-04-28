use crate::fetch::Article;

fn truncate_str(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let mut t: String = chars[..max.saturating_sub(1)].iter().collect();
        t.push('…');
        t
    }
}
use crate::theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};

pub struct ReaderPanel {
    pub article: Option<Article>,
    pub feed_title: String,
    pub lines: Vec<Line<'static>>,
    pub scroll: u16,
    pub content_height: u16,
    pub starred: bool,
}

impl ReaderPanel {
    pub fn new() -> Self {
        Self {
            article: None,
            feed_title: String::new(),
            lines: Vec::new(),
            scroll: 0,
            content_height: 0,
            starred: false,
        }
    }

    pub fn set_article(
        &mut self,
        article: Article,
        feed_title: String,
        lines: Vec<Line<'static>>,
        starred: bool,
    ) {
        self.article = Some(article);
        self.feed_title = feed_title;
        self.lines = lines;
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

    pub fn render_fullscreen(&mut self, frame: &mut Frame, area: Rect) {
        let Some(article) = &self.article else { return };

        // center column
        let col_width = area.width.min(86);
        let h_pad = (area.width.saturating_sub(col_width)) / 2;
        let center = Rect { x: area.x + h_pad, y: area.y, width: col_width, height: area.height };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // topbar
                Constraint::Length(1), // divider
                Constraint::Min(0),    // body
            ])
            .split(center);

        // topbar: feed · title (truncated) ··· ★ date
        let star = if self.starred {
            Span::styled(" ★", Style::default().fg(theme::ACCENT))
        } else {
            Span::raw("")
        };
        let date = article.published.as_deref().unwrap_or("");
        let date_span = if date.is_empty() {
            Span::raw("")
        } else {
            Span::styled(format!("  {}", date), Style::default().fg(theme::TX4))
        };

        // available width for title: col - feed - sep - star - date - padding
        let meta_len = self.feed_title.chars().count()
            + 3  // " · "
            + if self.starred { 2 } else { 0 }
            + date.chars().count()
            + if date.is_empty() { 0 } else { 2 };
        let title_max = (col_width as usize).saturating_sub(meta_len + 2);
        let title_short = truncate_str(&article.title, title_max);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(self.feed_title.clone(), Style::default().fg(theme::TX3)),
                Span::styled("  ·  ", Style::default().fg(theme::TX4)),
                Span::styled(title_short, Style::default().fg(theme::TX2)),
                star,
                date_span,
            ])),
            chunks[0],
        );

        // divider
        frame.render_widget(
            Paragraph::new(Span::styled(
                "─".repeat(col_width as usize),
                Style::default().fg(theme::BORDER),
            )),
            chunks[1],
        );

        // body
        let body_area = chunks[2];
        let inner_height = body_area.height;
        self.content_height = self.lines.len() as u16;

        let max_scroll = self.content_height.saturating_sub(inner_height);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        let visible: Vec<Line<'static>> = self
            .lines
            .iter()
            .skip(self.scroll as usize)
            .take(inner_height as usize)
            .cloned()
            .collect();

        frame.render_widget(Paragraph::new(Text::from(visible)), body_area);

        // scrollbar
        if self.content_height > inner_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme::BORDER2));
            let mut state =
                ScrollbarState::new(max_scroll as usize).position(self.scroll as usize);
            frame.render_stateful_widget(
                scrollbar,
                area.inner(Margin { horizontal: 0, vertical: 0 }),
                &mut state,
            );
        }
    }

    // empty-state widget for when no article is open (3-panel layout removed but kept for safety)
    pub fn render_empty(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "select an article",
                Style::default().fg(theme::TX4),
            )))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::BORDER))
                    .style(Style::default().bg(theme::BG)),
            ),
            area,
        );
    }
}
