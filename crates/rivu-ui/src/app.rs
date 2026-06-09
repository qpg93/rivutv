use std::io;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rivu_core::error::Result;
use rivu_core::models::Site;

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
    current: Screen,
}

impl App {
    pub fn new() -> Self {
        Self {
            home: HomeScreen::new(),
            detail: DetailScreen::new(),
            search: SearchScreen::new(),
            current: Screen::Home,
        }
    }

    pub fn set_sites(&mut self, sites: Vec<Site>) {
        self.home.sites = sites;
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
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('/') => self.current = Screen::Search,
                        KeyCode::Enter => self.current = Screen::Detail,
                        KeyCode::Esc => self.current = Screen::Home,
                        KeyCode::Down | KeyCode::Char('j') => self.navigate(1),
                        KeyCode::Up | KeyCode::Char('k') => self.navigate(-1),
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn navigate(&mut self, delta: i32) {
        let len = self.home.sites.len() as i32;
        if len > 0 {
            self.home.selected = ((self.home.selected as i32 + delta).rem_euclid(len)) as usize;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
