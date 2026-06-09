use std::io;
use std::sync::LazyLock;

use crossterm::event::KeyCode;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rivu_core::error::Result;
use rivu_core::models::{Flag, Site};
#[cfg(test)]
use rivu_core::models::Vod;
use rivu_player::MpvBackend;
use rivu_spider::extractor::SourceExtractor;
use rivu_spider::site_api::SiteApi;

static RT: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Runtime::new().expect("Failed to create tokio runtime for API calls")
});

use crate::screens::{detail::DetailScreen, home::HomeScreen, search::SearchScreen};

enum Screen {
    Home,
    Detail,
    Search,
}

pub struct App {
    pub home: HomeScreen,
    pub detail: DetailScreen,
    pub search: SearchScreen,
    pub api: SiteApi,
    pub player: MpvBackend,
    pub sites: Vec<Site>,
    pub current_site_index: usize,
    current: Screen,
}

impl App {
    pub fn new() -> Self {
        Self {
            home: HomeScreen::new(),
            detail: DetailScreen::new(),
            search: SearchScreen::new(),
            api: SiteApi::new(),
            player: MpvBackend::new(),
            sites: Vec::new(),
            current_site_index: 0,
            current: Screen::Home,
        }
    }

    pub fn set_sites(&mut self, sites: Vec<Site>) {
        self.sites = sites;
        self.home.sites = self.sites.clone();
    }

    pub fn current_site(&self) -> Option<&Site> {
        self.sites.get(self.current_site_index)
    }

    pub fn load_home(&mut self) {
        self.home.loading = true;
        self.home.error = None;
        let site = match self.current_site().cloned() {
            Some(s) => s,
            None => return,
        };
        let result = RT.block_on(self.api.home(&site));
        match result {
            Ok(api_result) => {
                self.home.categories = api_result.class.unwrap_or_default();
                self.home.vod_list = api_result.list.unwrap_or_default();
                self.home.loading = false;
            }
            Err(e) => {
                self.home.loading = false;
                self.home.error = Some(e.to_string());
            }
        }
    }

    fn load_category(&mut self) {
        self.home.loading = true;
        self.home.error = None;
        let site = match self.current_site().cloned() {
            Some(s) => s,
            None => return,
        };
        let tid = match self.home.categories.get(self.home.category_selected) {
            Some(c) => c.type_id.clone(),
            None => return,
        };
        let result = RT.block_on(self.api.category(&site, &tid, 1, &[]));
        match result {
            Ok(api_result) => {
                self.home.vod_list = api_result.list.unwrap_or_default();
                self.home.vod_selected = 0;
                self.home.loading = false;
            }
            Err(e) => {
                self.home.loading = false;
                self.home.error = Some(e.to_string());
            }
        }
    }

    fn load_detail(&mut self) {
        let site = match self.current_site().cloned() {
            Some(s) => s,
            None => return,
        };
        let vod = match self.home.vod_list.get(self.home.vod_selected) {
            Some(v) => v.clone(),
            None => return,
        };
        let result = RT.block_on(self.api.detail(&site, std::slice::from_ref(&vod.vod_id)));
        match result {
            Ok(api_result) => {
                if let Some(list) = api_result.list {
                    if let Some(detail_vod) = list.into_iter().next() {
                        self.detail.vod = Some(detail_vod.clone());
                        self.detail.flags = Flag::parse_flags(
                            &detail_vod.vod_play_from.unwrap_or_default(),
                            &detail_vod.vod_play_url.unwrap_or_default(),
                        );
                        self.detail.selected_flag = 0;
                        self.detail.selected_episode = 0;
                        self.current = Screen::Detail;
                    }
                }
            }
            Err(e) => {
                self.home.error = Some(e.to_string());
            }
        }
    }

    fn play_episode(&mut self) -> Result<()> {
        let site = match self.current_site().cloned() {
            Some(s) => s,
            None => return Ok(()),
        };
        let flag = match self.detail.flags.get(self.detail.selected_flag) {
            Some(f) => f,
            None => return Ok(()),
        };
        let ep = match flag.episodes.get(self.detail.selected_episode) {
            Some(e) => e.clone(),
            None => return Ok(()),
        };
        let play_info = RT.block_on(self.api.play(&site, &flag.name, &ep.url))?;
        let extractor = SourceExtractor::new();
        let resolved = extractor.extract(&play_info)?;
        self.player.play(&resolved)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_loop(&mut terminal);

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        if let Err(e) = &res {
            eprintln!("Error: {}", e);
        }

        res
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        use crossterm::event::{self, Event, KeyCode, KeyEventKind};

        loop {
            terminal.draw(|frame| {
                let area = frame.area();
                match self.current {
                    Screen::Home => self.home.draw(frame, area),
                    Screen::Detail => self.detail.draw(frame, area),
                    Screen::Search => self.search.draw(frame, area),
                }
            })?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.current {
                        Screen::Home => self.handle_home_key(key.code)?,
                        Screen::Detail => self.handle_detail_key(key.code)?,
                        Screen::Search => self.handle_search_key(key.code)?,
                    }
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_home_key(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                match self.home.focus {
                    0 => self.home.site_selected = self.navigate_wrap(self.home.site_selected, 1, self.home.sites.len()),
                    1 => self.home.category_selected = self.navigate_wrap(self.home.category_selected, 1, self.home.categories.len()),
                    2 => self.home.vod_selected = self.navigate_wrap(self.home.vod_selected, 1, self.home.vod_list.len()),
                    _ => {}
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.home.focus {
                    0 => self.home.site_selected = self.navigate_wrap(self.home.site_selected, -1, self.home.sites.len()),
                    1 => self.home.category_selected = self.navigate_wrap(self.home.category_selected, -1, self.home.categories.len()),
                    2 => self.home.vod_selected = self.navigate_wrap(self.home.vod_selected, -1, self.home.vod_list.len()),
                    _ => {}
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.home.focus = (self.home.focus + 1).min(2);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.home.focus = self.home.focus.saturating_sub(1);
            }
            KeyCode::Enter => {
                match self.home.focus {
                    0 => {
                        self.current_site_index = self.home.site_selected;
                        self.home.categories.clear();
                        self.home.vod_list.clear();
                        self.load_home();
                    }
                    1 => {
                        self.load_category();
                    }
                    2 => {
                        self.load_detail();
                    }
                    _ => {}
                }
            }
            KeyCode::Char('/') => {
                self.current = Screen::Search;
            }
            _ => {}
        }
        Ok(())
    }

    fn navigate_wrap(&self, current: usize, delta: i32, len: usize) -> usize {
        if len == 0 { return 0; }
        ((current as i32 + delta).rem_euclid(len as i32)) as usize
    }

    fn handle_detail_key(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(flag) = self.detail.flags.get(self.detail.selected_flag) {
                    if !flag.episodes.is_empty() {
                        self.detail.selected_episode = (self.detail.selected_episode + 1) % flag.episodes.len();
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(flag) = self.detail.flags.get(self.detail.selected_flag) {
                    if !flag.episodes.is_empty() {
                        let len = flag.episodes.len();
                        self.detail.selected_episode = (self.detail.selected_episode + len - 1) % len;
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if !self.detail.flags.is_empty() {
                    self.detail.selected_flag = (self.detail.selected_flag + 1) % self.detail.flags.len();
                    self.detail.selected_episode = 0;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if !self.detail.flags.is_empty() {
                    let len = self.detail.flags.len();
                    self.detail.selected_flag = (self.detail.selected_flag + len - 1) % len;
                    self.detail.selected_episode = 0;
                }
            }
            KeyCode::Enter => {
                self.play_episode()?;
            }
            KeyCode::Esc => {
                self.detail.vod = None;
                self.detail.flags.clear();
                self.current = Screen::Home;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_key(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Char(c) if c.is_alphanumeric() || c.is_ascii_punctuation() || c == ' ' => {
                self.search.query.push(c);
            }
            KeyCode::Backspace => {
                self.search.query.pop();
            }
            KeyCode::Enter => {
                if !self.search.results.is_empty() {
                    let site = match self.current_site().cloned() {
                        Some(s) => s,
                        None => return Ok(()),
                    };
                    let vod = match self.search.results.get(self.search.selected) {
                        Some(v) => v.clone(),
                        None => return Ok(()),
                    };
                    let result = RT.block_on(self.api.detail(&site, std::slice::from_ref(&vod.vod_id)));
                    match result {
                        Ok(api_result) => {
                            if let Some(list) = api_result.list {
                                if let Some(detail_vod) = list.into_iter().next() {
                                    self.detail.vod = Some(detail_vod.clone());
                                    self.detail.flags = Flag::parse_flags(
                                        &detail_vod.vod_play_from.unwrap_or_default(),
                                        &detail_vod.vod_play_url.unwrap_or_default(),
                                    );
                                    self.detail.selected_flag = 0;
                                    self.detail.selected_episode = 0;
                                    self.current = Screen::Detail;
                                }
                            }
                        }
                        Err(e) => {
                            self.home.error = Some(e.to_string());
                        }
                    }
                } else {
                    let site = match self.current_site().cloned() {
                        Some(s) => s,
                        None => return Ok(()),
                    };
                    let query = self.search.query.clone();
                    if !query.is_empty() {
                        let result = RT.block_on(self.api.search(&site, &query, 1));
                        match result {
                            Ok(api_result) => {
                                self.search.results = api_result.list.unwrap_or_default();
                                self.search.selected = 0;
                            }
                            Err(e) => {
                                self.home.error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.search.results.is_empty() {
                    self.search.selected = (self.search.selected + 1) % self.search.results.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.search.results.is_empty() {
                    let len = self.search.results.len();
                    self.search.selected = (self.search.selected + len - 1) % len;
                }
            }
            KeyCode::Esc => {
                self.search.query.clear();
                self.search.results.clear();
                self.current = Screen::Home;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new_has_no_sites() {
        let app = App::new();
        assert!(app.sites.is_empty());
        assert!(app.home.sites.is_empty());
    }

    #[test]
    fn test_app_set_sites_updates_both() {
        let mut app = App::new();
        let sites = vec![
            Site { key: "a".into(), name: "A".into(), site_type: 0, api: "http://a.com".into(), ..Default::default() },
        ];
        app.set_sites(sites);
        assert_eq!(app.sites.len(), 1);
        assert_eq!(app.home.sites.len(), 1);
    }

    #[test]
    fn test_app_current_site_returns_none_when_empty() {
        let app = App::new();
        assert!(app.current_site().is_none());
    }

    #[test]
    fn test_app_current_site_returns_selected() {
        let mut app = App::new();
        app.sites = vec![
            Site { key: "a".into(), name: "A".into(), site_type: 0, api: "http://a.com".into(), ..Default::default() },
            Site { key: "b".into(), name: "B".into(), site_type: 1, api: "http://b.com".into(), ..Default::default() },
        ];
        app.current_site_index = 1;
        assert_eq!(app.current_site().unwrap().key, "b");
    }

    #[test]
    fn test_navigate_wrap_basic() {
        let app = App::new();
        assert_eq!(app.navigate_wrap(0, 1, 3), 1);
        assert_eq!(app.navigate_wrap(2, 1, 3), 0);
        assert_eq!(app.navigate_wrap(0, -1, 3), 2);
    }

    #[test]
    fn test_navigate_wrap_empty() {
        let app = App::new();
        assert_eq!(app.navigate_wrap(0, 1, 0), 0);
    }

    #[test]
    fn test_home_keys_move_selection() {
        let mut app = App::new();
        app.home.sites = vec![
            Site { key: "a".into(), name: "A".into(), site_type: 0, api: "http://a.com".into(), ..Default::default() },
            Site { key: "b".into(), name: "B".into(), site_type: 1, api: "http://b.com".into(), ..Default::default() },
            Site { key: "c".into(), name: "C".into(), site_type: 2, api: "http://c.com".into(), ..Default::default() },
        ];
        app.home.focus = 0;
        app.handle_home_key(KeyCode::Char('j')).unwrap();
        assert_eq!(app.home.site_selected, 1);
        app.handle_home_key(KeyCode::Char('j')).unwrap();
        assert_eq!(app.home.site_selected, 2);
        app.handle_home_key(KeyCode::Char('k')).unwrap();
        assert_eq!(app.home.site_selected, 1);
    }

    #[test]
    fn test_home_focus_switching() {
        let mut app = App::new();
        app.home.focus = 0;
        app.handle_home_key(KeyCode::Right).unwrap();
        assert_eq!(app.home.focus, 1);
        app.handle_home_key(KeyCode::Right).unwrap();
        assert_eq!(app.home.focus, 2);
        app.handle_home_key(KeyCode::Right).unwrap();
        assert_eq!(app.home.focus, 2);
        app.handle_home_key(KeyCode::Left).unwrap();
        assert_eq!(app.home.focus, 1);
    }

    #[test]
    fn test_detail_keys_cycle_episodes() {
        let mut app = App::new();
        app.detail.flags = vec![Flag {
            name: "CK".into(),
            episodes: vec![
                rivu_core::models::Episode { name: "1".into(), url: "u1".into() },
                rivu_core::models::Episode { name: "2".into(), url: "u2".into() },
            ],
        }];
        assert_eq!(app.detail.selected_episode, 0);
        app.handle_detail_key(KeyCode::Char('j')).unwrap();
        assert_eq!(app.detail.selected_episode, 1);
        app.handle_detail_key(KeyCode::Char('k')).unwrap();
        assert_eq!(app.detail.selected_episode, 0);
    }

    #[test]
    fn test_detail_esc_clears_and_goes_home() {
        let mut app = App::new();
        app.detail.vod = Some(Vod { vod_id: "1".into(), vod_name: "T".into(), ..Default::default() });
        app.detail.flags = vec![Flag {
            name: "CK".into(),
            episodes: vec![rivu_core::models::Episode { name: "1".into(), url: "u1".into() }],
        }];
        app.handle_detail_key(KeyCode::Esc).unwrap();
        assert!(app.detail.vod.is_none());
        assert!(app.detail.flags.is_empty());
    }

    #[test]
    fn test_search_key_accepts_input() {
        let mut app = App::new();
        app.handle_search_key(KeyCode::Char('t')).unwrap();
        app.handle_search_key(KeyCode::Char('e')).unwrap();
        app.handle_search_key(KeyCode::Char('s')).unwrap();
        app.handle_search_key(KeyCode::Char('t')).unwrap();
        assert_eq!(app.search.query, "test");
    }

    #[test]
    fn test_search_backspace() {
        let mut app = App::new();
        app.search.query = "test".into();
        app.handle_search_key(KeyCode::Backspace).unwrap();
        assert_eq!(app.search.query, "tes");
    }

    #[test]
    fn test_search_esc_clears_and_goes_home() {
        let mut app = App::new();
        app.search.query = "hello".into();
        app.handle_search_key(KeyCode::Esc).unwrap();
        assert!(app.search.query.is_empty());
    }

    #[test]
    fn test_detail_flag_switching_with_keys() {
        let mut app = App::new();
        app.detail.flags = vec![
            Flag { name: "CK".into(), episodes: vec![] },
            Flag { name: "Bili".into(), episodes: vec![] },
            Flag { name: "QQ".into(), episodes: vec![] },
        ];
        app.handle_detail_key(KeyCode::Right).unwrap();
        assert_eq!(app.detail.selected_flag, 1);
        app.handle_detail_key(KeyCode::Right).unwrap();
        assert_eq!(app.detail.selected_flag, 2);
        app.handle_detail_key(KeyCode::Left).unwrap();
        assert_eq!(app.detail.selected_flag, 1);
    }
}
