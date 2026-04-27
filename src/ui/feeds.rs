use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

pub struct FeedsPanel {
    pub feeds: Vec<String>,
    pub state: ListState,
    pub loading: Vec<String>,
    pub errors: Vec<String>,
}

impl FeedsPanel {
    pub fn new(feeds: Vec<String>) -> Self {
        let mut state = ListState::default();
        if !feeds.is_empty() {
            state.select(Some(0));
        }
        Self {
            feeds,
            state,
            loading: vec![],
            errors: vec![],
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.state.selected().and_then(|i| self.feeds.get(i)).map(|s| s.as_str())
    }

    pub fn next(&mut self) {
        let len = self.feeds.len();
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
        if !self.feeds.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn last(&mut self) {
        if !self.feeds.is_empty() {
            self.state.select(Some(self.feeds.len() - 1));
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let items: Vec<ListItem> = self
            .feeds
            .iter()
            .map(|url| {
                let label = feed_label(url);
                let is_loading = self.loading.contains(url);
                let is_error = self.errors.contains(url);

                let indicator = if is_loading {
                    Span::styled("~ ", Style::default().fg(Color::Yellow))
                } else if is_error {
                    Span::styled("! ", Style::default().fg(Color::Red))
                } else {
                    Span::raw("  ")
                };

                ListItem::new(Line::from(vec![indicator, Span::raw(label)]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" feeds ")
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

fn feed_label(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string()
}
