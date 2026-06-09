use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use rivu_core::models::{Flag, Vod};

pub struct DetailScreen {
    pub vod: Option<Vod>,
    pub flags: Vec<Flag>,
    pub selected_episode: usize,
    pub selected_flag: usize,
}

impl DetailScreen {
    pub fn new() -> Self {
        Self {
            vod: None,
            flags: Vec::new(),
            selected_episode: 0,
            selected_flag: 0,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(1)])
            .split(area);

        if let Some(vod) = &self.vod {
            let info = Text::from(vec![
                Line::from(Span::styled(
                    &vod.vod_name,
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(format!(
                    "Year: {} | Area: {} | Score: {}",
                    vod.vod_year.as_deref().unwrap_or("-"),
                    vod.vod_area.as_deref().unwrap_or("-"),
                    vod.vod_score.as_deref().unwrap_or("-")
                )),
                Line::from(format!(
                    "Director: {}",
                    vod.vod_director.as_deref().unwrap_or("-")
                )),
            ]);
            let info_widget =
                Paragraph::new(info).block(Block::default().borders(Borders::ALL)).wrap(Wrap { trim: false });
            frame.render_widget(info_widget, chunks[0]);

            let episodes = self.build_episode_list();
            let ep_list = List::new(episodes)
                .block(Block::default().title(" Episodes ").borders(Borders::ALL));
            frame.render_widget(ep_list, chunks[1]);
        }
    }

    fn build_episode_list(&self) -> Vec<ListItem<'static>> {
        if let Some(flag) = self.flags.get(self.selected_flag) {
            flag.episodes
                .iter()
                .enumerate()
                .map(|(i, ep)| {
                    let style = if i == self.selected_episode {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(ep.name.clone(), style)))
                })
                .collect()
        } else {
            vec![ListItem::new(Line::from(Span::raw("No episodes")))]
        }
    }
}

impl Default for DetailScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rivu_core::models::Episode;

    #[test]
    fn test_detail_screen_new_has_no_vod() {
        let screen = DetailScreen::new();
        assert!(screen.vod.is_none());
        assert!(screen.flags.is_empty());
    }

    #[test]
    fn test_detail_screen_with_vod_sets_metadata() {
        let mut screen = DetailScreen::new();
        screen.vod = Some(Vod {
            vod_id: "100".into(), vod_name: "Test Movie".into(),
            vod_year: Some("2024".into()), vod_area: Some("CN".into()),
            vod_score: Some("8.5".into()), vod_director: Some("Dir".into()),
            ..Default::default()
        });
        let vod = screen.vod.as_ref().unwrap();
        assert_eq!(vod.vod_year.as_deref(), Some("2024"));
        assert_eq!(vod.vod_score.as_deref(), Some("8.5"));
    }

    #[test]
    fn test_detail_screen_build_episode_list() {
        let mut screen = DetailScreen::new();
        screen.flags = vec![Flag {
            name: "CK".into(),
            episodes: vec![
                Episode { name: "1".into(), url: "http://a.com/1.mp4".into() },
                Episode { name: "2".into(), url: "http://a.com/2.mp4".into() },
            ],
        }];
        screen.selected_flag = 0;
        let items = screen.build_episode_list();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_detail_screen_build_episode_list_no_flags() {
        let screen = DetailScreen::new();
        let items = screen.build_episode_list();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_detail_screen_episode_selection_highlight() {
        let mut screen = DetailScreen::new();
        screen.flags = vec![Flag {
            name: "CK".into(),
            episodes: vec![
                Episode { name: "1".into(), url: "http://a.com/1.mp4".into() },
                Episode { name: "2".into(), url: "http://a.com/2.mp4".into() },
            ],
        }];
        screen.selected_flag = 0;
        screen.selected_episode = 1;
        let items = screen.build_episode_list();
        assert_eq!(items.len(), 2);
    }
}
