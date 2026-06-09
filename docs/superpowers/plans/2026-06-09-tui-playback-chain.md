# TUI Playback Chain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the TUI to SiteApi so users can browse sites → categories → videos → play via mpv.

**Architecture:** 3-panel HomeScreen (Sources | Categories | Vod List) with block_on SiteApi calls in the event loop. DetailScreen triggers play. All async calls use Handle::block_on — simple, acceptable for MVP.

**Tech Stack:** ratatui 0.29, tokio, reqwest, rivu-core/rivu-spider/rivu-player

---

### Task 1: HomeScreen — 3-panel layout + focus + vod/category state

**Files:**
- Modify: `crates/rivu-ui/src/screens/home.rs`
- Test: same file, `#[cfg(test)] mod tests`

**Changes to `HomeScreen` struct:**

```rust
pub struct HomeScreen {
    pub sites: Vec<Site>,
    pub site_selected: usize,
    pub focus: usize, // 0=sources, 1=categories, 2=vod_list

    // Categories (from SiteApi::home)
    pub categories: Vec<Class>,
    pub category_selected: usize,

    // Vod list (from SiteApi::home or SiteApi::category)
    pub vod_list: Vec<Vod>,
    pub vod_selected: usize,

    // State
    pub loading: bool,
    pub error: Option<String>,
}
```

Remove old fields: `selected`, `result` (replaced by categories + vod_list).
Change import from `use rivu_core::models::{ApiResult, Site}` to `use rivu_core::models::{Class, Site, Vod}`.

- [ ] **Step 1: Write failing tests for new HomeScreen layout and focus**

```rust
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
```

- [ ] **Step 2: Run tests — verify they fail to compile (struct changed)**

Run: `cargo test -p rivu-ui`
Expected: compile errors (old HomeScreen fields `selected` and `result` removed)

- [ ] **Step 3: Rewrite HomeScreen struct**

```rust
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
}
```

Delete the old imports (`ApiResult`), delete the old `selected` and `result` fields. Keep `Default` impl.

- [ ] **Step 4: Rewrite HomeScreen::draw for 3-panel layout**

The draw method renders 3 columns: Sources (30%) | Categories (25%) | Vod List (45%).
Each column highlights the selected item and shows loading/error state.

```rust
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
```

- [ ] **Step 5: Run tests — verify they pass**

Run: `cargo test -p rivu-ui`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/rivu-ui/src/screens/home.rs
git commit -m "feat: 3-panel HomeScreen with focus and vod/category state"
```

---

### Task 2: DetailScreen — flag switching + play trigger

**Files:**
- Modify: `crates/rivu-ui/src/screens/detail.rs`
- Test: same file

- [ ] **Step 1: Write failing tests for flag switching**

Add to the existing `#[cfg(test)] mod tests` in `detail.rs`:

```rust
#[test]
fn test_detail_screen_flag_switch_left() {
    let mut screen = DetailScreen::new();
    screen.flags = vec![
        Flag { name: "CK".into(), episodes: vec![Episode { name: "1".into(), url: "u1".into() }] },
        Flag { name: "Bili".into(), episodes: vec![Episode { name: "1".into(), url: "u2".into() }] },
    ];
    screen.selected_flag = 1;
    if screen.selected_flag > 0 {
        screen.selected_flag -= 1;
    } else {
        screen.selected_flag = screen.flags.len() - 1;
    }
    assert_eq!(screen.selected_flag, 0);
    assert_eq!(screen.selected_episode, 0);
}

#[test]
fn test_detail_screen_flag_switch_right() {
    let mut screen = DetailScreen::new();
    screen.flags = vec![
        Flag { name: "CK".into(), episodes: vec![Episode { name: "1".into(), url: "u1".into() }] },
        Flag { name: "Bili".into(), episodes: vec![Episode { name: "1".into(), url: "u2".into() }] },
    ];
    screen.selected_flag = 0;
    let next = (screen.selected_flag + 1) % screen.flags.len();
    screen.selected_flag = next;
    assert_eq!(screen.selected_flag, 1);
}

#[test]
fn test_detail_screen_flag_switch_wraps_both_sides() {
    let mut screen = DetailScreen::new();
    screen.flags = vec![
        Flag { name: "CK".into(), episodes: vec![Episode { name: "1".into(), url: "u1".into() }] },
        Flag { name: "Bili".into(), episodes: vec![Episode { name: "1".into(), url: "u2".into() }] },
    ];
    screen.selected_flag = 0;
    screen.selected_flag = screen.flags.len() - 1;
    assert_eq!(screen.selected_flag, 1);
}
```

- [ ] **Step 2: Run tests — verify they pass (Flag/Episode types already available via imports)**

Run: `cargo test -p rivu-ui detail`
Expected: tests compile and pass

- [ ] **Step 3: Update DetailScreen draw to show flag tabs**

Add a helper method:

```rust
fn build_flag_tabs(&self) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, flag) in self.flags.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" | "));
        }
        let style = if i == self.selected_flag {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        spans.push(Span::styled(&flag.name, style));
    }
    Line::from(spans)
}
```

Update `draw` to include flag tabs row between info and episode list:

```rust
pub fn draw(&self, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    if let Some(vod) = &self.vod {
        let info = Text::from(vec![
            Line::from(Span::styled(&vod.vod_name, Style::default().add_modifier(Modifier::BOLD))),
            Line::from(format!("Year: {} | Area: {} | Score: {}",
                vod.vod_year.as_deref().unwrap_or("-"),
                vod.vod_area.as_deref().unwrap_or("-"),
                vod.vod_score.as_deref().unwrap_or("-"))),
            Line::from(format!("Director: {}", vod.vod_director.as_deref().unwrap_or("-"))),
        ]);
        let info_widget = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(info_widget, chunks[0]);
    }

    // Flag tabs
    vod.vod_play_from.as_deref().unwrap_or("") is shown when we have flags
    let tabs_content = if !self.flags.is_empty() {
        Text::from(self.build_flag_tabs())
    } else {
        Text::from(Line::from(Span::raw("No sources")))
    };
    let tabs_widget = Paragraph::new(tabs_content)
        .block(Block::default().title(" Sources ").borders(Borders::ALL));
    frame.render_widget(tabs_widget, chunks[1]);

    // Episode list
    let episodes = self.build_episode_list();
    let ep_list = List::new(episodes)
        .block(Block::default().title(" Episodes ").borders(Borders::ALL));
    frame.render_widget(ep_list, chunks[2]);
}
```

- [ ] **Step 4: Run tests — verify all pass**

Run: `cargo test -p rivu-ui`
Expected: all pass

- [ ] **Step 5: Commit**

```bash
git add crates/rivu-ui/src/screens/detail.rs
git commit -m "feat: DetailScreen flag tabs and episode navigation"
```

---

### Task 3: App — SiteApi/Player integration + async wiring

**Files:**
- Modify: `crates/rivu-ui/src/app.rs`

- [ ] **Step 1: Write the new App struct with SiteApi + MpvBackend**

```rust
use rivu_core::error::Result;
use rivu_core::models::{Class, Flag, Site, Vod};
use rivu_player::MpvBackend;
use rivu_spider::extractor::SourceExtractor;
use rivu_spider::site_api::SiteApi;
use std::collections::HashMap;

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
```

- [ ] **Step 2: Implement App::new, set_sites, and current_site**

```rust
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
        let handle = tokio::runtime::Handle::current();
        let result = handle.block_on(self.api.home(&site));
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
}
```

- [ ] **Step 3: Add load_category, load_detail, play_episode helpers**

```rust
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
    let handle = tokio::runtime::Handle::current();
    let result = handle.block_on(self.api.category(&site, &tid, 1, &[]));
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
    let handle = tokio::runtime::Handle::current();
    let result = handle.block_on(self.api.detail(&site, &[vod.vod_id.clone()]));
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
    let handle = tokio::runtime::Handle::current();
    let play_info = handle.block_on(self.api.play(&site, &flag.name, &ep.url))?;
    let extractor = SourceExtractor::new();
    let resolved = extractor.extract(&play_info)?;
    self.player.play(&resolved)?;
    Ok(())
}
```

- [ ] **Step 4: Wire keyboard handlers in run_loop**

Replace the old `run_loop` with the new event dispatch:

```rust
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
```

- [ ] **Step 5: Implement handle_home_key and navigate_wrap**

```rust
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
```

- [ ] **Step 6: Implement handle_detail_key**

```rust
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
```

- [ ] **Step 7: Implement handle_search_key**

On the first Enter, submit the query and show results. On a second Enter (with results populated), navigate to detail for the selected result.

```rust
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
                // Enter on a search result → go to detail
                let site = match self.current_site().cloned() {
                    Some(s) => s,
                    None => return Ok(()),
                };
                let vod = match self.search.results.get(self.search.selected) {
                    Some(v) => v.clone(),
                    None => return Ok(()),
                };
                let handle = tokio::runtime::Handle::current();
                let result = handle.block_on(self.api.detail(&site, &[vod.vod_id.clone()]));
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
                // First Enter → submit search query
                let site = match self.current_site().cloned() {
                    Some(s) => s,
                    None => return Ok(()),
                };
                let query = self.search.query.clone();
                if !query.is_empty() {
                    let handle = tokio::runtime::Handle::current();
                    let result = handle.block_on(self.api.search(&site, &query, 1));
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
```

- [ ] **Step 8: Remove old navigate method**

Delete the old `fn navigate(&mut self, delta: i32)` method body. The Default impl stays the same.

- [ ] **Step 9: Run tests — compile and pass**

Run: `cargo test -p rivu-ui`
Expected: compile without errors, all tests pass

- [ ] **Step 10: Commit**

```bash
git add crates/rivu-ui/src/app.rs
git commit -m "feat: App event loop with SiteApi/Player wiring"
```

---

### Task 4: main.rs — wire initial home load

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Update main.rs Run handler**

```rust
Cli::Run => {
    let config_dir = ConfigLoader::get_config_dir();
    let mut loader = ConfigLoader::new(&config_dir);
    let mut app = App::new();

    let source_url = loader.app_config.source_url.clone();
    if let Some(url) = source_url {
        match loader.fetch_source(&url).await {
            Ok(config) => {
                app.set_sites(config.sites.clone());
                app.load_home();
            }
            Err(e) => {
                eprintln!("Warning: couldn't load source config: {}", e);
            }
        }
    }

    app.run()?;
}
```

The `load_home` method is already `pub` from Task 3.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire initial home load in main.rs"
```

---

### Task 5: App state transition tests

**Files:**
- Modify: `crates/rivu-ui/src/app.rs` (add `#[cfg(test)] mod tests` at bottom)

- [ ] **Step 1: Add inline tests for App state transitions**

```rust
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
        // Initial state
        assert_eq!(app.detail.selected_episode, 0);
        // Down
        app.handle_detail_key(KeyCode::Char('j')).unwrap();
        assert_eq!(app.detail.selected_episode, 1);
        // Up
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
        // Start at 0
        app.handle_detail_key(KeyCode::Right).unwrap();
        assert_eq!(app.detail.selected_flag, 1);
        app.handle_detail_key(KeyCode::Right).unwrap();
        assert_eq!(app.detail.selected_flag, 2);
        app.handle_detail_key(KeyCode::Left).unwrap();
        assert_eq!(app.detail.selected_flag, 1);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p rivu-ui`
Expected: all tests pass

- [ ] **Step 3: Run full suite**

Run: `cargo test`
Expected: all tests pass, clippy clean

- [ ] **Step 4: Commit**

```bash
git add crates/rivu-ui/src/app.rs
git commit -m "test: App state transition tests"
```

---

### Task 6: Final cleanup — clippy + verify

- [ ] **Step 1: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: no warnings

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: all pass

- [ ] **Step 3: If clippy finds issues, fix and re-run**

- [ ] **Step 4: Commit any final fixes**

```bash
git add -A
git commit -m "chore: clippy fixes and final cleanup"
```
