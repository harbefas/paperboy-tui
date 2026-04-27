use crate::fetch::url_to_label;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};
use std::collections::HashMap;

pub struct FeedsPanel {
    pub feeds: Vec<String>,
    pub titles: HashMap<String, String>,
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
        Self { feeds, titles: HashMap::new(), state, loading: vec![], errors: vec![] }
    }

    pub fn selected(&self) -> Option<&str> {
        self.state.selected().and_then(|i| self.feeds.get(i)).map(|s| s.as_str())
    }

    pub fn set_title(&mut self, url: &str, title: String) {
        self.titles.insert(url.to_string(), title);
    }

    pub fn next(&mut self) {
        let len = self.feeds.len();
        if len == 0 { return; }
        let i = self.state.selected().map(|i| (i + 1).min(len - 1)).unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = self.state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn first(&mut self) {
        if !self.feeds.is_empty() { self.state.select(Some(0)); }
    }

    pub fn last(&mut self) {
        if !self.feeds.is_empty() { self.state.select(Some(self.feeds.len() - 1)); }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let items: Vec<ListItem> = self.feeds.iter().map(|url| {
            let is_loading = self.loading.contains(url);
            let is_error = self.errors.contains(url);

            let label = self.titles.get(url).cloned().unwrap_or_else(|| url_to_label(url));

            let (indicator, ind_style) = if is_loading {
                ("~ ", Style::default().fg(theme::ACCENT))
            } else if is_error {
                ("! ", Style::default().fg(theme::RED))
            } else {
                ("  ", Style::default())
            };

            ListItem::new(Line::from(vec![
                Span::styled(indicator, ind_style),
                Span::styled(label, Style::default().fg(theme::TX2)),
            ]))
        }).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Span::styled(" feeds ", Style::default().fg(theme::TX3)))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme::border(focused))
                    .style(Style::default().bg(theme::BG)),
            )
            .highlight_style(theme::highlight())
            .highlight_symbol("▸ ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}
