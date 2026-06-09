use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use rivu_core::models::{Class, Site, Vod};

pub struct HomeScreen {
    pub sites: Vec<Site>,
    pub site_selected: usize,
    pub focus: usize,
    pub categories: Vec<Class>,
    pub category_selected: usize,
    pub vod_list: Vec<Vod>,
    pub vod_selected: usize,
    pub loading: bool,
    pub error: Option<String>,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            sites: Vec::new(),
            site_selected: 0,
            focus: 0,
            categories: Vec::new(),
            category_selected: 0,
            vod_list: Vec::new(),
            vod_selected: 0,
            loading: false,
            error: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(25),
                Constraint::Percentage(45),
            ])
            .split(area);

        // Left: Sources
        let src_items: Vec<ListItem> = self
            .sites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if self.focus == 0 && i == self.site_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if i == self.site_selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(&s.name, style)))
            })
            .collect();
        let src_list = List::new(src_items)
            .block(Block::default().title(" Sources ").borders(Borders::ALL));
        frame.render_widget(src_list, chunks[0]);

        // Middle: Categories
        let cat_items: Vec<ListItem> = if self.loading && self.focus == 1 {
            vec![ListItem::new(Line::from(Span::raw("Loading...")))]
        } else if let Some(ref err) = self.error {
            vec![ListItem::new(Line::from(Span::styled(
                format!("Error: {}", err),
                Style::default().fg(Color::Red),
            )))]
        } else if self.categories.is_empty() {
            vec![ListItem::new(Line::from(Span::raw("(empty)")))]
        } else {
            self.categories.iter().enumerate().map(|(i, c)| {
                let style = if self.focus == 1 && i == self.category_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if i == self.category_selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(&c.type_name, style)))
            }).collect()
        };
        let cat_list = List::new(cat_items)
            .block(Block::default().title(" Categories ").borders(Borders::ALL));
        frame.render_widget(cat_list, chunks[1]);

        // Right: Vod List
        let vod_items: Vec<ListItem> = if self.loading && self.focus == 2 {
            vec![ListItem::new(Line::from(Span::raw("Loading...")))]
        } else if self.vod_list.is_empty() {
            vec![ListItem::new(Line::from(Span::raw("(empty)")))]
        } else {
            self.vod_list.iter().enumerate().map(|(i, v)| {
                let style = if self.focus == 2 && i == self.vod_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if i == self.vod_selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let remarks = v.vod_remarks.as_deref().unwrap_or("");
                ListItem::new(Line::from(vec![
                    Span::styled(&v.vod_name, style),
                    Span::styled(
                        format!(" [{}]", remarks),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            }).collect()
        };
        let vod_list = List::new(vod_items)
            .block(Block::default().title(" Videos ").borders(Borders::ALL));
        frame.render_widget(vod_list, chunks[2]);
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
    fn test_home_screen_new_has_3_panel_state() {
        let screen = HomeScreen::new();
        assert!(screen.sites.is_empty());
        assert!(screen.categories.is_empty());
        assert!(screen.vod_list.is_empty());
        assert_eq!(screen.focus, 0);
        assert_eq!(screen.site_selected, 0);
        assert!(!screen.loading);
        assert!(screen.error.is_none());
    }

    #[test]
    fn test_home_screen_with_sites_selects_first() {
        let mut screen = HomeScreen::new();
        screen.sites = vec![
            Site { key: "a".into(), name: "Site A".into(), site_type: 0, api: "http://a.com".into(), jar: None, ext: None, searchable: None, quick_search: None, filterable: None, player_type: None, categories: None },
            Site { key: "b".into(), name: "Site B".into(), site_type: 1, api: "http://b.com".into(), jar: None, ext: None, searchable: None, quick_search: None, filterable: None, player_type: None, categories: None },
        ];
        assert_eq!(screen.site_selected, 0);
    }

    #[test]
    fn test_home_screen_with_categories() {
        let mut screen = HomeScreen::new();
        screen.categories = vec![
            Class { type_id: "1".into(), type_name: "Movie".into(), type_flag: None, filters: None },
            Class { type_id: "2".into(), type_name: "TV Series".into(), type_flag: None, filters: None },
        ];
        assert_eq!(screen.categories.len(), 2);
        assert_eq!(screen.categories[0].type_name, "Movie");
    }

    #[test]
    fn test_home_screen_with_vod_list() {
        let mut screen = HomeScreen::new();
        screen.vod_list = vec![
            Vod { vod_id: "1".into(), vod_name: "Film A".into(), vod_remarks: Some("HD".into()), ..Default::default() },
            Vod { vod_id: "2".into(), vod_name: "Film B".into(), vod_remarks: Some("4K".into()), ..Default::default() },
        ];
        assert_eq!(screen.vod_list.len(), 2);
    }

    #[test]
    fn test_home_screen_focus_switching() {
        let mut screen = HomeScreen::new();
        screen.focus = 1;
        assert_eq!(screen.focus, 1);
    }
}
