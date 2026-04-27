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
    pub feed_title: String,
    pub content: String,
    pub scroll: u16,
    pub content_height: u16,
    pub starred: bool,
}

impl ReaderPanel {
    pub fn new() -> Self {
        Self {
            article: None,
            feed_title: String::new(),
            content: String::new(),
            scroll: 0,
            content_height: 0,
            starred: false,
        }
    }

    pub fn set_article(&mut self, article: Article, feed_title: String, content: String, starred: bool) {
        self.article = Some(article);
        self.feed_title = feed_title;
        self.content = content;
        self.scroll = 0;
        self.starred = starred;
    }

    pub fn scroll_down(&mut self) { self.scroll = self.scroll.saturating_add(1); }
    pub fn scroll_up(&mut self) { self.scroll = self.scroll.saturating_sub(1); }

    pub fn page_down(&mut self, height: u16) {
        self.scroll = self.scroll.saturating_add(height.saturating_sub(4));
    }
    pub fn page_up(&mut self, height: u16) {
        self.scroll = self.scroll.saturating_sub(height.saturating_sub(4));
    }

    // full-screen reader layout
    pub fn render_fullscreen(&mut self, frame: &mut Frame, area: Rect) {
        let Some(article) = &self.article else { return; };

        // center column: max 82 chars wide
        let col_width = area.width.min(86);
        let h_pad = (area.width.saturating_sub(col_width)) / 2;
        let center = Rect {
            x: area.x + h_pad,
            y: area.y,
            width: col_width,
            height: area.height,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // breadcrumb
                Constraint::Length(1), // spacer
                Constraint::Length(2), // feed + date
                Constraint::Length(1), // spacer
                Constraint::Length(3), // title
                Constraint::Length(1), // spacer
                Constraint::Min(0),    // body
            ])
            .split(center);

        // breadcrumb: feeds > articles > reader
        let breadcrumb = Paragraph::new(Line::from(vec![
            Span::styled("feeds", Style::default().fg(theme::TX4)),
            Span::styled("  /  ", Style::default().fg(theme::TX4)),
            Span::styled(&self.feed_title, Style::default().fg(theme::TX3)),
            Span::styled("  /  ", Style::default().fg(theme::TX4)),
            Span::styled("reader", Style::default().fg(theme::TX2)),
        ]));
        frame.render_widget(breadcrumb, chunks[0]);

        // feed + date line
        let star = if self.starred {
            Span::styled("★  ", Style::default().fg(theme::ACCENT))
        } else {
            Span::raw("")
        };
        let date = article.published.as_deref().unwrap_or("");
        let meta = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(&self.feed_title, Style::default().fg(theme::ACCENT)),
            ]),
            Line::from(vec![
                star,
                Span::styled(date, Style::default().fg(theme::TX3)),
            ]),
        ]);
        frame.render_widget(meta, chunks[2]);

        // title — bold, full text weight
        let title_text = article.title.clone();
        let title = Paragraph::new(
            Span::styled(
                &title_text,
                Style::default()
                    .fg(theme::TX)
                    .add_modifier(Modifier::BOLD),
            )
        )
        .wrap(Wrap { trim: false });
        frame.render_widget(title, chunks[4]);

        // body
        let body_area = chunks[6];
        let inner_height = body_area.height;
        let line_count = self.content.lines().count() as u16;
        self.content_height = line_count;

        let max_scroll = line_count.saturating_sub(inner_height);
        if self.scroll > max_scroll { self.scroll = max_scroll; }

        let body = Paragraph::new(self.content.as_str())
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme::TX2));
        frame.render_widget(body, body_area);

        // scrollbar on right edge of full area
        if line_count > inner_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme::BORDER2));
            let mut state = ScrollbarState::new(max_scroll as usize).position(self.scroll as usize);
            frame.render_stateful_widget(
                scrollbar,
                area.inner(Margin { horizontal: 0, vertical: 0 }),
                &mut state,
            );
        }
    }

    // compact reader used inside 3-panel layout (articles view)
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let Some(article) = &self.article else {
            let empty = Paragraph::new(Line::from(Span::styled(
                "select an article",
                Style::default().fg(theme::TX4),
            )))
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
                Span::styled(&self.feed_title, Style::default().fg(theme::ACCENT)),
            ]),
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
        if self.scroll > max_scroll { self.scroll = max_scroll; }

        let progress = if line_count > inner_height {
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
                    .title(Span::styled(progress, Style::default().fg(theme::TX4)))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::BORDER))
                    .style(Style::default().bg(theme::BG)),
            );
        frame.render_widget(body, content_area);

        if line_count > inner_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme::BORDER2));
            let mut state =
                ScrollbarState::new(max_scroll as usize).position(self.scroll as usize);
            frame.render_stateful_widget(
                scrollbar,
                content_area.inner(Margin { horizontal: 0, vertical: 1 }),
                &mut state,
            );
        }
    }
}
