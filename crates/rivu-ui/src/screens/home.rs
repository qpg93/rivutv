use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use rivu_core::models::{ApiResult, Site};

pub struct HomeScreen {
    pub sites: Vec<Site>,
    pub selected: usize,
    pub categories: Vec<String>,
    pub result: Option<ApiResult>,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            sites: Vec::new(),
            selected: 0,
            categories: Vec::new(),
            result: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        let sites: Vec<ListItem> = self
            .sites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(&s.name, style)))
            })
            .collect();

        let sites_list = List::new(sites)
            .block(Block::default().title(" Sources ").borders(Borders::ALL));
        frame.render_widget(sites_list, chunks[0]);

        let categories: Vec<ListItem> = self
            .result
            .as_ref()
            .and_then(|r| r.class.as_ref())
            .map(|classes| {
                classes
                    .iter()
                    .map(|c| ListItem::new(Line::from(Span::raw(&c.type_name))))
                    .collect()
            })
            .unwrap_or_default();

        let cat_list = List::new(categories)
            .block(Block::default().title(" Categories ").borders(Borders::ALL));
        frame.render_widget(cat_list, chunks[1]);
    }
}

impl Default for HomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rivu_core::models::Class;

    #[test]
    fn test_home_screen_new_has_no_sites() {
        let screen = HomeScreen::new();
        assert!(screen.sites.is_empty());
        assert_eq!(screen.selected, 0);
    }

    #[test]
    fn test_home_screen_with_sites_selects_first() {
        let mut screen = HomeScreen::new();
        screen.sites = vec![
            Site { key: "a".into(), name: "Site A".into(), site_type: 0, api: "http://a.com".into(), jar: None, ext: None, searchable: None, quick_search: None, filterable: None, player_type: None, categories: None },
            Site { key: "b".into(), name: "Site B".into(), site_type: 1, api: "http://b.com".into(), jar: None, ext: None, searchable: None, quick_search: None, filterable: None, player_type: None, categories: None },
        ];
        assert_eq!(screen.sites.len(), 2);
        assert_eq!(screen.selected, 0);
    }

    #[test]
    fn test_home_screen_with_categories() {
        let mut screen = HomeScreen::new();
        screen.result = Some(ApiResult {
            class: Some(vec![
                Class { type_id: "1".into(), type_name: "Movie".into(), type_flag: None, filters: None },
                Class { type_id: "2".into(), type_name: "TV Series".into(), type_flag: None, filters: None },
            ]),
            ..Default::default()
        });
        let classes = screen.result.as_ref().and_then(|r| r.class.as_ref()).unwrap();
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].type_name, "Movie");
    }

    #[test]
    fn test_home_screen_no_categories_when_no_result() {
        let screen = HomeScreen::new();
        assert!(screen.result.is_none());
    }
}
