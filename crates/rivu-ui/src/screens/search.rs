use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use rivu_core::models::Vod;

pub struct SearchScreen {
    pub query: String,
    pub results: Vec<Vod>,
    pub selected: usize,
}

impl SearchScreen {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected: 0,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let input = Paragraph::new(Line::from(Span::raw(&self.query)))
            .block(Block::default().title(" Search ").borders(Borders::ALL));
        frame.render_widget(input, chunks[0]);

        let items: Vec<ListItem> = self
            .results
            .iter()
            .map(|v| {
                let remarks = v.vod_remarks.as_deref().unwrap_or("");
                ListItem::new(Line::from(vec![
                    Span::raw(v.vod_name.clone()),
                    Span::styled(
                        format!(" [{}]", remarks),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL));
        frame.render_widget(list, chunks[1]);
    }
}

impl Default for SearchScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_screen_new_is_empty() {
        let screen = SearchScreen::new();
        assert!(screen.query.is_empty());
        assert!(screen.results.is_empty());
        assert_eq!(screen.selected, 0);
    }

    #[test]
    fn test_search_screen_with_results() {
        let mut screen = SearchScreen::new();
        screen.query = "test".into();
        screen.results = vec![
            Vod { vod_id: "1".into(), vod_name: "Result A".into(), vod_remarks: Some("HD".into()), ..Default::default() },
            Vod { vod_id: "2".into(), vod_name: "Result B".into(), vod_remarks: Some("4K".into()), ..Default::default() },
        ];
        assert_eq!(screen.results.len(), 2);
        assert_eq!(screen.results[0].vod_name, "Result A");
    }

    #[test]
    fn test_search_screen_empty_results() {
        let screen = SearchScreen::new();
        assert!(screen.results.is_empty());
    }
}
