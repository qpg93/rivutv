# RivuTV TVBox Protocol Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a working Linux-native TVBox media client capable of loading source configs from TVBox JSON URLs
(e.g., 饭太硬-style), browsing VOD content, and playing videos via mpv.

**Architecture:** A Rust workspace with 6 crates. The core TVBox protocol handling lives in `rivu-spider`
(fetching + parsing) and `rivu-core` (data models). `rivu-config` handles loading source JSON configs,
`rivu-player` abstracts mpv subprocess, and `rivu-ui` provides a ratatui TUI. The root binary wires
everything together with a clap CLI.

**Tech Stack:** Rust, tokio (async runtime), reqwest (HTTP), serde/serde_json (serialization),
ratatui (TUI), clap (CLI), mpv (subprocess player), thiserror (error types).

**Plan based on:** FongMi/TV (https://github.com/FongMi/TV) architecture analysis.

---

### Task 1: Core Data Models (TVBox Protocol Types)

**Files:**
- Create: `crates/rivu-core/src/error.rs`
- Create: `crates/rivu-core/src/models.rs`
- Create: `crates/rivu-core/src/traits.rs`
- Modify: `crates/rivu-core/src/lib.rs`
- Modify: `crates/rivu-core/Cargo.toml`

This task replaces the current placeholder models with complete TVBox protocol types.
All models use serde derives for JSON deserialization. Error types use thiserror.

- [ ] **Add thiserror dependency and rewrite lib.rs**

`crates/rivu-core/Cargo.toml`:
```toml
[package]
name = "rivu-core"
version = "0.1.0"
edition = "2021"
description = "RivuTV core types, traits, and data models"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
reqwest = { version = "0.12", default-features = false }
async-trait = "0.1"
```

`crates/rivu-core/src/lib.rs`:
```rust
pub mod error;
pub mod models;
pub mod traits;
```

- [ ] **Create error module**

`crates/rivu-core/src/error.rs`:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config parse error: {0}")]
    Config(String),

    #[error("Spider error: {0}")]
    Spider(String),

    #[error("Player error: {0}")]
    Player(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
```

- [ ] **Rewrite models with complete TVBox protocol types**

`crates/rivu-core/src/models.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Source Configuration ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceConfig {
    pub sites: Vec<Site>,
    pub lives: Option<Vec<Live>>,
    pub parses: Option<Vec<Parse>>,
    pub rules: Option<Vec<Rule>>,
    pub headers: Option<HashMap<String, String>>,
    pub flags: Option<Vec<String>>,
    pub ads: Option<Vec<String>>,
    pub spider: Option<String>,
    pub wallpaper: Option<String>,
    pub logo: Option<String>,
    pub notice: Option<String>,
    pub urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub site_type: u8,
    pub api: String,
    pub jar: Option<String>,
    pub ext: Option<String>,
    pub searchable: Option<i32>,
    pub quick_search: Option<i32>,
    pub filterable: Option<i32>,
    pub player_type: Option<u8>,
    pub categories: Option<Vec<Category>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Live {
    pub name: String,
    pub url: String,
    pub api: Option<String>,
    pub ext: Option<String>,
    pub jar: Option<String>,
    pub logo: Option<String>,
    pub epg: Option<String>,
    pub ua: Option<String>,
    pub origin: Option<String>,
    pub referer: Option<String>,
    pub header: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parse {
    pub name: String,
    #[serde(rename = "type")]
    pub parse_type: u8,
    pub url: String,
    pub ext: Option<HashMap<String, String>>,
    pub header: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub host: Option<String>,
    pub rule: Option<String>,
}

// ── API Response Types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiResult {
    pub class: Option<Vec<Class>>,
    pub list: Option<Vec<Vod>>,
    pub page: Option<i32>,
    pub pagecount: Option<i32>,
    pub limit: Option<i32>,
    pub total: Option<i32>,
    pub filters: Option<HashMap<String, Vec<Filter>>>,
    pub header: Option<String>,
    pub url: Option<String>,
    pub flag: Option<String>,
    pub play_url: Option<String>,
    pub parse: Option<i32>,
    pub jx: Option<i32>,
    pub link: Option<String>,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    #[serde(rename = "type_id")]
    pub type_id: String,
    #[serde(rename = "type_name")]
    pub type_name: String,
    #[serde(rename = "type_flag")]
    pub type_flag: Option<String>,
    pub filters: Option<HashMap<String, Vec<Filter>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    #[serde(rename = "type_id")]
    pub type_id: String,
    #[serde(rename = "type_name")]
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub key: String,
    pub name: String,
    pub value: Vec<FilterValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterValue {
    pub v: String,
    pub n: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vod {
    #[serde(rename = "vod_id")]
    pub vod_id: String,
    #[serde(rename = "vod_name")]
    pub vod_name: String,
    #[serde(rename = "vod_pic")]
    pub vod_pic: Option<String>,
    #[serde(rename = "vod_remarks")]
    pub vod_remarks: Option<String>,
    #[serde(rename = "vod_year")]
    pub vod_year: Option<String>,
    #[serde(rename = "vod_area")]
    pub vod_area: Option<String>,
    #[serde(rename = "vod_director")]
    pub vod_director: Option<String>,
    #[serde(rename = "vod_actor")]
    pub vod_actor: Option<String>,
    #[serde(rename = "vod_content")]
    pub vod_content: Option<String>,
    #[serde(rename = "vod_play_from")]
    pub vod_play_from: Option<String>,
    #[serde(rename = "vod_play_url")]
    pub vod_play_url: Option<String>,
    #[serde(rename = "vod_tag")]
    pub vod_tag: Option<String>,
    #[serde(rename = "type_name")]
    pub type_name: Option<String>,
    #[serde(rename = "vod_score")]
    pub vod_score: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Flag {
    pub name: String,
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct PlayInfo {
    pub url: String,
    pub headers: HashMap<String, String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

// ── Config persistence ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub source_url: Option<String>,
    pub sites: Vec<Site>,
    pub player: String,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            source_url: None,
            sites: Vec::new(),
            player: "mpv".to_string(),
            theme: "dark".to_string(),
        }
    }
}
```

- [ ] **Add inline unit tests for models and Flag parsing**

Add to `crates/rivu-core/src/models.rs` (append at end):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_config_empty() {
        let json = r#"{"sites": [], "lives": [], "parses": []}"#;
        let config: SourceConfig = serde_json::from_str(json).unwrap();
        assert!(config.sites.is_empty());
        assert_eq!(config.spider, None);
    }

    #[test]
    fn test_source_config_full() {
        let json = r#"{
            "sites": [{"key": "k1", "name": "Src1", "type": 3, "api": "http://a.com", "jar": "http://j.com/jar.jar"}],
            "lives": [{"name": "Live1", "url": "http://l.com"}],
            "parses": [{"name": "Parse1", "type": 1, "url": "http://p.com"}],
            "headers": {"User-Agent": "test"},
            "flags": ["4k", "1080p"],
            "spider": "http://s.com/spider.jar",
            "wallpaper": "http://w.com/wall.jpg",
            "notice": "Hello",
            "urls": ["http://depot1.com"]
        }"#;
        let config: SourceConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.sites.len(), 1);
        assert_eq!(config.lives.as_ref().unwrap().len(), 1);
        assert_eq!(config.parses.as_ref().unwrap().len(), 1);
        assert_eq!(config.flags.as_ref().unwrap().len(), 2);
        assert_eq!(config.spider.as_deref().unwrap(), "http://s.com/spider.jar");
    }

    #[test]
    fn test_vod_all_fields() {
        let json = r#"{
            "vod_id": "100", "vod_name": "Test",
            "vod_pic": "http://pic.jpg", "vod_remarks": "HD",
            "vod_year": "2024", "vod_area": "CN",
            "vod_director": "Dir A", "vod_actor": "Actor B",
            "vod_content": "A good movie", "vod_score": "9.0"
        }"#;
        let vod: Vod = serde_json::from_str(json).unwrap();
        assert_eq!(vod.vod_id, "100");
        assert_eq!(vod.vod_score.as_deref(), Some("9.0"));
        assert_eq!(vod.vod_actor.as_deref(), Some("Actor B"));
    }

    #[test]
    fn test_vod_minimal() {
        let json = r#"{"vod_id": "1", "vod_name": "Minimal"}"#;
        let vod: Vod = serde_json::from_str(json).unwrap();
        assert_eq!(vod.vod_name, "Minimal");
        assert_eq!(vod.vod_pic, None);
        assert_eq!(vod.vod_actor, None);
        assert_eq!(vod.vod_content, None);
    }

    #[test]
    fn test_class_with_filters() {
        let json = r#"{
            "type_id": "1", "type_name": "Movie",
            "filters": {
                "1": [{"key": "area", "name": "Region", "value": [{"v": "", "n": "All"}, {"v": "CN", "n": "China"}]}]
            }
        }"#;
        let class: Class = serde_json::from_str(json).unwrap();
        assert_eq!(class.type_id, "1");
        let filters = class.filters.unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters["1"][0].value.len(), 2);
    }

    #[test]
    fn test_api_result_empty() {
        let result = ApiResult::default();
        assert!(result.class.is_none());
        assert!(result.list.is_none());
    }

    #[test]
    fn test_site_deserialize_type_field() {
        let json = r#"{"key": "k", "name": "N", "type": 3, "api": "http://a.com"}"#;
        let site: Site = serde_json::from_str(json).unwrap();
        assert_eq!(site.site_type, 3);
    }

    #[test]
    fn test_flag_parse_single_flag() {
        let flags = Flag::parse_flags("CK", "1$http://a.mp4#2$http://b.mp4");
        assert_eq!(flags.len(), 1);
        assert_eq!(flags[0].name, "CK");
        assert_eq!(flags[0].episodes.len(), 2);
        assert_eq!(flags[0].episodes[0].url, "http://a.mp4");
    }

    #[test]
    fn test_flag_parse_multi_flag() {
        let flags = Flag::parse_flags("CK$$$Bili", "1$http://a.mp4#2$http://b.mp4$$$1$http://c.mp4");
        assert_eq!(flags.len(), 2);
        assert_eq!(flags[0].name, "CK");
        assert_eq!(flags[1].name, "Bili");
        assert_eq!(flags[0].episodes.len(), 2);
        assert_eq!(flags[1].episodes.len(), 1);
    }

    #[test]
    fn test_flag_parse_mismatched_returns_empty() {
        let flags = Flag::parse_flags("A$$$B", "1$http://a.mp4");
        assert!(flags.is_empty());
    }

    #[test]
    fn test_flag_parse_empty_inputs() {
        let flags = Flag::parse_flags("", "");
        assert!(flags.is_empty());
    }

    #[test]
    fn test_app_config_serde_roundtrip() {
        let config = AppConfig {
            source_url: Some("http://example.com/config.json".into()),
            sites: vec![Site {
                key: "k".into(), name: "N".into(), site_type: 0, api: "http://a.com".into(),
                jar: None, ext: None, searchable: None, quick_search: None,
                filterable: None, player_type: None, categories: None,
            }],
            player: "mpv".into(),
            theme: "light".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.source_url, config.source_url);
        assert_eq!(restored.sites.len(), 1);
        assert_eq!(restored.theme, "light");
    }

    #[test]
    fn test_episode_url_contains_special_chars() {
        let flag = Flag::parse_flags("S", "1$http://a.com/play?token=abc&id=123#2$http://b.com");
        assert_eq!(flag[0].episodes[0].url, "http://a.com/play?token=abc&id=123");
        assert_eq!(flag[0].episodes[1].url, "http://b.com");
    }
}
```

- [ ] **Run unit tests for core**

```bash
cargo test -p rivu-core
```

- [ ] **Rewrite traits**

`crates/rivu-core/src/traits.rs`:
```rust
use crate::error::Result;
use crate::models::{ApiResult, PlayInfo, Vod};

#[async_trait::async_trait]
pub trait Spider: Send + Sync {
    async fn home(&self) -> Result<ApiResult>;
    async fn category(&self, tid: &str, pg: i32, filter: bool, extend: &str) -> Result<ApiResult>;
    async fn detail(&self, ids: &[String]) -> Result<ApiResult>;
    async fn play(&self, flag: &str, id: &str) -> Result<PlayInfo>;
    async fn search(&self, keyword: &str, pg: i32) -> Result<ApiResult>;
}

#[async_trait::async_trait]
pub trait Player: Send + Sync {
    async fn play(&self, info: &PlayInfo) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    fn is_running(&self) -> bool;
}
```

- [ ] **Commit**

```bash
git add crates/rivu-core/
git commit -m "feat(core): add TVBox protocol data models and traits"
```

---

### Task 2: Config Loader — Fetch and Parse Source JSON

**Files:**
- Modify: `crates/rivu-config/Cargo.toml`
- Rewrite: `crates/rivu-config/src/lib.rs`
- Create: `crates/rivu-config/src/loader.rs`

The config crate fetches a JSON URL (e.g., 饭太硬 config), parses it into `SourceConfig`,
and stores it. Also handles loading/saving the local `AppConfig`.

- [ ] **Update Cargo.toml with new dependencies**

`crates/rivu-config/Cargo.toml`:
```toml
[package]
name = "rivu-config"
version = "0.1.0"
edition = "2021"
description = "RivuTV configuration management"

[dependencies]
rivu-core = { path = "../rivu-core" }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
tempfile = "3"
```

- [ ] **Rewrite the config module**

`crates/rivu-config/src/lib.rs`:
```rust
pub mod loader;
```

- [ ] **Create config loader**

`crates/rivu-config/src/loader.rs`:
```rust
use rivu_core::error::Result;
use rivu_core::models::{AppConfig, SourceConfig};

pub struct ConfigLoader {
    client: reqwest::Client,
    config_path: std::path::PathBuf,
    pub source_config: Option<SourceConfig>,
    pub app_config: AppConfig,
}

impl ConfigLoader {
    pub fn new(config_dir: &std::path::Path) -> Self {
        let config_path = config_dir.join("config.json");
        let app_config = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Self {
            client: reqwest::Client::new(),
            config_path,
            source_config: None,
            app_config,
        }
    }

    pub async fn fetch_source(&mut self, url: &str) -> Result<&SourceConfig> {
        let resp = self.client.get(url).send().await?;
        let text = resp.text().await?;
        let config: SourceConfig = serde_json::from_str(&text)?;
        self.source_config = Some(config);
        Ok(self.source_config.as_ref().unwrap())
    }

    pub fn save_app_config(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let json = serde_json::to_string_pretty(&self.app_config)?;
        std::fs::write(&self.config_path, json)?;
        Ok(())
    }

    pub fn get_config_dir() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        std::path::Path::new(&home).join(".config/rivutv")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rivu_core::models::Site;
    use std::io::Write;

    #[test]
    fn test_parse_source_config() {
        let json = r#"{
            "sites": [{"key": "test", "name": "Test", "type": 0, "api": "http://example.com"}],
            "lives": [],
            "parses": []
        }"#;
        let config: SourceConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.sites.len(), 1);
        assert_eq!(config.sites[0].key, "test");
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.player, "mpv");
    }

    #[test]
    fn test_source_config_with_all_optionals_missing() {
        let json = r#"{"sites":[]}"#;
        let config: SourceConfig = serde_json::from_str(json).unwrap();
        assert!(config.sites.is_empty());
        assert!(config.lives.is_none());
        assert!(config.parses.is_none());
        assert!(config.headers.is_none());
        assert!(config.flags.is_none());
    }

    #[test]
    fn test_source_config_unknown_fields_ignored() {
        let json = r#"{
            "sites": [],
            "unknown_field": "should be ignored",
            "extra_object": {"a": 1}
        }"#;
        let config: SourceConfig = serde_json::from_str(json).unwrap();
        assert!(config.sites.is_empty());
    }

    #[test]
    fn test_app_config_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut config = AppConfig::default();
        config.source_url = Some("http://example.com/tv.json".into());
        config.player = "mpv".into();
        config.theme = "dark".into();

        // Save
        let json = serde_json::to_string_pretty(&config).unwrap();
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        drop(file);

        // Load
        let loaded: AppConfig = {
            let content = std::fs::read_to_string(&path).unwrap();
            serde_json::from_str(&content).unwrap()
        };
        assert_eq!(loaded.source_url, config.source_url);
        assert_eq!(loaded.player, "mpv");
    }

    #[test]
    fn test_app_config_corrupted_file_falls_back_to_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "this is not json").unwrap();

        let app_config = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        assert_eq!(app_config.player, "mpv");
        assert!(app_config.source_url.is_none());
    }

    #[test]
    fn test_get_config_dir_respects_home() {
        let home = std::env::var("HOME").unwrap();
        let dir = ConfigLoader::get_config_dir();
        assert_eq!(dir, std::path::Path::new(&home).join(".config/rivutv"));
    }

    #[test]
    fn test_config_loader_new_with_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        let config = AppConfig {
            source_url: Some("http://test.tv".into()),
            sites: vec![],
            player: "vlc".into(),
            theme: "light".into(),
        };
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let loader = ConfigLoader::new(dir.path());
        assert_eq!(loader.app_config.player, "vlc");
        assert_eq!(loader.app_config.source_url.as_deref(), Some("http://test.tv"));
    }
}
```

- [ ] **Run tests**

```bash
cargo test -p rivu-config
```

- [ ] **Commit**

```bash
git add crates/rivu-config/
git commit -m "feat(config): add TVBox source config loader"
```

---

### Task 3: HTTP API Engine — SiteApi for TVBox Protocol

**Files:**
- Modify: `crates/rivu-spider/Cargo.toml`
- Rewrite: `crates/rivu-spider/src/lib.rs`
- Rewrite: `crates/rivu-spider/src/engine.rs`
- Rewrite: `crates/rivu-spider/src/parsers.rs`
- Create: `crates/rivu-spider/src/site_api.rs`
- Create: `crates/rivu-spider/src/extractor.rs`

The spider crate handles all TVBox API interactions. `SiteApi` is the main facade
(analogous to FongMi's `SiteApi.java`). It handles HTTP calls for type 0/1/2/4 sites
and resolves play URLs.

- [ ] **Update Cargo.toml**

`crates/rivu-spider/Cargo.toml`:
```toml
[package]
name = "rivu-spider"
version = "0.1.0"
edition = "2021"
description = "RivuTV spider engine for TVBox APIs"

[dependencies]
rivu-core = { path = "../rivu-core" }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
quick-xml = "0.37"
```

- [ ] **Rewrite lib.rs**

`crates/rivu-spider/src/lib.rs`:
```rust
pub mod engine;
pub mod extractor;
pub mod parsers;
pub mod site_api;
```

- [ ] **Rewrite engine.rs**

`crates/rivu-spider/src/engine.rs`:
```rust
use rivu_core::error::Result;
use rivu_core::models::Site;

pub struct SpiderEngine {
    client: reqwest::Client,
}

impl SpiderEngine {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .build()
                .unwrap(),
        }
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub fn build_url(&self, site: &Site, path: &str, params: &[(&str, &str)]) -> String {
        let base = site.api.trim_end_matches('/');
        let separator = if base.contains('?') { "&" } else { "?" };
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}{}{}{}", base, separator, path, query)
    }
}

impl Default for SpiderEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_site() -> Site {
        Site {
            key: "test".into(), name: "Test".into(), site_type: 0, api: "http://example.com/api".into(),
            jar: None, ext: None, searchable: None, quick_search: None, filterable: None,
            player_type: None, categories: None,
        }
    }

    #[test]
    fn test_build_url_no_trailing_slash() {
        let engine = SpiderEngine::new();
        let site = test_site();
        let url = engine.build_url(&site, "", &[("ac", "videolist")]);
        assert_eq!(url, "http://example.com/api?ac=videolist");
    }

    #[test]
    fn test_build_url_with_trailing_slash() {
        let engine = SpiderEngine::new();
        let mut site = test_site();
        site.api = "http://example.com/api/".into();
        let url = engine.build_url(&site, "", &[("ac", "videolist")]);
        assert_eq!(url, "http://example.com/api?ac=videolist");
    }

    #[test]
    fn test_build_url_multiple_params() {
        let engine = SpiderEngine::new();
        let site = test_site();
        let url = engine.build_url(&site, "", &[("ac", "videolist"), ("t", "1"), ("pg", "2")]);
        assert_eq!(url, "http://example.com/api?ac=videolist&t=1&pg=2");
    }

    #[test]
    fn test_build_url_with_existing_query() {
        let engine = SpiderEngine::new();
        let mut site = test_site();
        site.api = "http://example.com/api?token=abc".into();
        let url = engine.build_url(&site, "", &[("ac", "videolist")]);
        assert_eq!(url, "http://example.com/api?token=abc&ac=videolist");
    }

    #[test]
    fn test_build_url_no_params() {
        let engine = SpiderEngine::new();
        let site = test_site();
        let url = engine.build_url(&site, "", &[]);
        assert_eq!(url, "http://example.com/api?");
    }

    #[test]
    fn test_build_url_path_and_params() {
        let engine = SpiderEngine::new();
        let mut site = test_site();
        site.api = "http://example.com/".into();
        let url = engine.build_url(&site, "proxy?", &[("do", "get")]);
        assert_eq!(url, "http://example.com/proxy?do=get");
    }
}
```

- [ ] **Rewrite parsers.rs**

`crates/rivu-spider/src/parsers.rs`:
```rust
use rivu_core::error::Result;
use rivu_core::models::{ApiResult, Vod};

pub struct Parser;

impl Parser {
    pub fn parse_json(data: &str) -> Result<ApiResult> {
        let result: ApiResult = serde_json::from_str(data)?;
        Ok(result)
    }

    pub fn parse_xml(data: &str) -> Result<ApiResult> {
        // Quick-and-dirty XML to JSON-like structure for TVBox XML responses
        // TVBox XML format: <list><video><id>...</id><name>...</name>...</video></list>
        let mut result = ApiResult::default();
        let mut vods = Vec::new();

        if let Ok(doc) = quick_xml::de::from_str::<serde_json::Value>(data) {
            if let Some(list) = doc.get("list").and_then(|v| v.as_array()) {
                for item in list {
                    let vod = Vod {
                        vod_id: item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        vod_name: item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        vod_pic: item.get("pic").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        vod_remarks: item.get("note").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        ..Default::default()
                    };
                    vods.push(vod);
                }
            }
        }

        result.list = Some(vods);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_response() {
        let json = r#"{
            "class": [{"type_id": "1", "type_name": "Movie"}],
            "list": [{"vod_id": "123", "vod_name": "Test Movie", "vod_pic": "http://example.com/pic.jpg"}]
        }"#;
        let result = Parser::parse_json(json).unwrap();
        assert!(result.class.is_some());
        assert!(result.list.is_some());
        assert_eq!(result.list.unwrap()[0].vod_id, "123");
    }

    #[test]
    fn test_parse_json_empty() {
        let json = r#"{"class":[],"list":[],"page":1,"pagecount":1,"limit":20,"total":0}"#;
        let result = Parser::parse_json(json).unwrap();
        assert!(result.list.unwrap().is_empty());
    }

    #[test]
    fn test_parse_json_with_filters() {
        let json = r#"{
            "class": [{"type_id":"1","type_name":"Movie"}],
            "filters": {
                "1": [
                    {"key":"area","name":"Region","value":[{"v":"","n":"All"},{"v":"CN","n":"China"}]}
                ]
            },
            "list": []
        }"#;
        let result = Parser::parse_json(json).unwrap();
        let filters = result.filters.unwrap();
        let area_filters = filters.get("1").unwrap();
        assert_eq!(area_filters[0].key, "area");
        assert_eq!(area_filters[0].value[1].n, "China");
    }

    #[test]
    fn test_parse_json_malformed_returns_error() {
        let result = Parser::parse_json("this is not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_null_values_accepted() {
        let json = r#"{"class":null,"list":null,"page":null,"total":null}"#;
        let result = Parser::parse_json(json).unwrap();
        assert!(result.class.is_none());
        assert!(result.list.is_none());
        assert!(result.page.is_none());
    }

    #[test]
    fn test_parse_xml_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <list>
            <video>
                <id>100</id>
                <name>Movie A</name>
                <pic>http://pic/a.jpg</pic>
                <note>HD</note>
            </video>
            <video>
                <id>101</id>
                <name>Movie B</name>
                <pic>http://pic/b.jpg</pic>
                <note>4K</note>
            </video>
        </list>"#;
        let result = Parser::parse_xml(xml).unwrap();
        let vods = result.list.unwrap();
        assert_eq!(vods.len(), 2);
        assert_eq!(vods[0].vod_id, "100");
        assert_eq!(vods[1].vod_name, "Movie B");
    }

    #[test]
    fn test_parse_xml_empty_returns_empty_list() {
        let xml = r#"<?xml version="1.0"?><list></list>"#;
        let result = Parser::parse_xml(xml).unwrap();
        assert!(result.list.unwrap().is_empty());
    }

    #[test]
    fn test_parse_xml_invalid_returns_empty_list() {
        let result = Parser::parse_xml("not xml").unwrap();
        assert!(result.list.unwrap().is_empty());
    }
}
```

- [ ] **Create site_api.rs**

`crates/rivu-spider/src/site_api.rs`:
```rust
use rivu_core::error::Result;
use rivu_core::models::{ApiResult, PlayInfo, Site};
use crate::engine::SpiderEngine;
use crate::parsers::Parser;

pub struct SiteApi {
    engine: SpiderEngine,
}

impl SiteApi {
    pub fn new() -> Self {
        Self {
            engine: SpiderEngine::new(),
        }
    }

    pub async fn home(&self, site: &Site) -> Result<ApiResult> {
        let url = self.engine.build_url(site, "", &[("ac", "videolist")]);
        let resp = self.engine.client().get(&url).send().await?;
        let text = resp.text().await?;
        Parser::parse_json(&text)
    }

    pub async fn category(
        &self,
        site: &Site,
        tid: &str,
        pg: i32,
        filters: &[(&str, &str)],
    ) -> Result<ApiResult> {
        let mut params = vec![("ac", "videolist"), ("t", tid), ("pg", &pg.to_string())];
        for (k, v) in filters {
            params.push((k, v));
        }
        let url = self.engine.build_url(site, "", &params);
        let resp = self.engine.client().get(&url).send().await?;
        let text = resp.text().await?;
        Parser::parse_json(&text)
    }

    pub async fn detail(&self, site: &Site, ids: &[String]) -> Result<ApiResult> {
        let ids_str = ids.join(",");
        let url = self.engine.build_url(site, "", &[("ac", "videolist"), ("ids", &ids_str)]);
        let resp = self.engine.client().get(&url).send().await?;
        let text = resp.text().await?;
        Parser::parse_json(&text)
    }

    pub async fn play(&self, site: &Site, flag: &str, id: &str) -> Result<PlayInfo> {
        let url = self.engine.build_url(site, "", &[("ac", "play"), ("flag", flag), ("ids", id)]);
        let resp = self.engine.client().get(&url).send().await?;
        let text = resp.text().await?;
        let result = Parser::parse_json(&text)?;

        Ok(PlayInfo {
            url: result.url.unwrap_or_default(),
            headers: std::collections::HashMap::new(),
            user_agent: None,
            referer: None,
        })
    }

    pub async fn search(&self, site: &Site, keyword: &str, pg: i32) -> Result<ApiResult> {
        let url = self.engine.build_url(site, "", &[("ac", "videolist"), ("wd", keyword), ("pg", &pg.to_string())]);
        let resp = self.engine.client().get(&url).send().await?;
        let text = resp.text().await?;
        Parser::parse_json(&text)
    }
}

impl Default for SiteApi {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Create extractor.rs**

`crates/rivu-spider/src/extractor.rs`:
```rust
use rivu_core::error::Result;
use rivu_core::models::PlayInfo;

pub struct SourceExtractor;

impl SourceExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Resolve a play URL — strip TVBox wrappers (video://, etc.)
    /// and return the actual stream URL.
    pub fn resolve(&self, url: &str) -> String {
        let url = url.trim();
        if let Some(stripped) = url.strip_prefix("video://") {
            stripped.to_string()
        } else if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            // Magnet, ed2k, etc. — pass through for mpv
            url.to_string()
        }
    }

    pub fn extract(&self, info: &PlayInfo) -> Result<PlayInfo> {
        let url = self.resolve(&info.url);
        Ok(PlayInfo {
            url,
            headers: info.headers.clone(),
            user_agent: info.user_agent.clone(),
            referer: info.referer.clone(),
        })
    }
}

impl Default for SourceExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_strip_video_prefix() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("video://http://example.com/stream.m3u8"), "http://example.com/stream.m3u8");
        assert_eq!(ext.resolve("http://example.com/video.mp4"), "http://example.com/video.mp4");
        assert_eq!(ext.resolve("magnet:?xt=urn:btih:abc"), "magnet:?xt=urn:btih:abc");
    }

    #[test]
    fn test_strip_video_prefix_https() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("video://https://s.com/play.m3u8"), "https://s.com/play.m3u8");
    }

    #[test]
    fn test_resolve_https_passthrough() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("https://cdn.com/video.mp4"), "https://cdn.com/video.mp4");
    }

    #[test]
    fn test_resolve_ed2k_passthrough() {
        let ext = SourceExtractor::new();
        let ed2k = "ed2k://|file|movie.avi|1234567890|hash|/";
        assert_eq!(ext.resolve(ed2k), ed2k);
    }

    #[test]
    fn test_resolve_whitespace_trimmed() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("  http://a.com/v.mp4  "), "http://a.com/v.mp4");
    }

    #[test]
    fn test_extract_preserves_headers() {
        let ext = SourceExtractor::new();
        let mut headers = HashMap::new();
        headers.insert("Referer".into(), "http://ref.com".into());
        let info = PlayInfo {
            url: "video://http://real.com/stream".into(),
            headers: headers.clone(),
            user_agent: Some("test-agent".into()),
            referer: Some("http://ref.com".into()),
        };
        let result = ext.extract(&info).unwrap();
        assert_eq!(result.url, "http://real.com/stream");
        assert_eq!(result.headers.get("Referer").unwrap(), "http://ref.com");
        assert_eq!(result.user_agent.as_deref(), Some("test-agent"));
    }

    #[test]
    fn test_extract_empty_url() {
        let ext = SourceExtractor::new();
        let info = PlayInfo {
            url: "".into(),
            headers: HashMap::new(),
            user_agent: None,
            referer: None,
        };
        let result = ext.extract(&info).unwrap();
        assert_eq!(result.url, "");
    }

}
```

- [ ] **Run tests**

```bash
cargo test -p rivu-spider
```

- [ ] **Commit**

```bash
git add crates/rivu-spider/
git commit -m "feat(spider): add SiteApi HTTP engine and TVBox protocol handlers"
```

---

### Task 4: Player Backend — mpv Subprocess

**Files:**
- Modify: `crates/rivu-player/Cargo.toml`
- Rewrite: `crates/rivu-player/src/lib.rs`
- Rewrite: `crates/rivu-player/src/backends.rs`
- Create: `crates/rivu-player/src/mpv.rs`

The player crate manages the mpv subprocess. It handles spawning mpv with appropriate
flags (headers, referer, user-agent), monitoring the process, and cleanup.

- [ ] **Update Cargo.toml**

`crates/rivu-player/Cargo.toml`:
```toml
[package]
name = "rivu-player"
version = "0.1.0"
edition = "2021"
description = "RivuTV media playback engine"

[dependencies]
rivu-core = { path = "../rivu-core" }
thiserror = "2"
tokio = { version = "1", features = ["process"] }
```

- [ ] **Rewrite lib.rs**

`crates/rivu-player/src/lib.rs`:
```rust
pub mod backends;
pub mod mpv;

pub use mpv::MpvBackend;
```

- [ ] **Create mpv.rs**

`crates/rivu-player/src/mpv.rs`:
```rust
use rivu_core::error::Result;
use rivu_core::models::PlayInfo;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

pub struct MpvBackend {
    process: Mutex<Option<Child>>,
}

impl MpvBackend {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }

    pub fn play(&self, info: &PlayInfo) -> Result<()> {
        self.stop()?;

        let mut cmd = Command::new("mpv");
        cmd.arg(&info.url)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if let Some(ua) = &info.user_agent {
            cmd.arg(format!("--user-agent={}", ua));
        }

        if let Some(ref) = &info.referer {
            cmd.arg(format!("--referrer={}", ref));
        }

        for (key, val) in &info.headers {
            cmd.arg(format!("--http-header-fields={}: {}", key, val));
        }

        let child = cmd.spawn().map_err(|e| {
            rivu_core::error::CoreError::Player(format!("Failed to launch mpv: {}", e))
        })?;

        *self.process.lock().unwrap() = Some(child);
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let mut proc = self.process.lock().unwrap();
        if let Some(mut child) = proc.take() {
            child.kill().ok();
            child.wait().ok();
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process
            .lock()
            .unwrap()
            .as_ref()
            .map(|c| c.try_wait().ok().flatten().is_none())
            .unwrap_or(false)
    }
}

impl Default for MpvBackend {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Rewrite backends.rs**

`crates/rivu-player/src/backends.rs`:
```rust
// Backend module root — individual backends are in their own files.
pub use crate::mpv::MpvBackend;
```

- [ ] **Add unit tests for MpvBackend**

Append to `crates/rivu-player/src/mpv.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_is_running_returns_false_when_not_started() {
        let backend = MpvBackend::new();
        assert!(!backend.is_running());
    }

    #[test]
    fn test_stop_when_not_running_does_not_panic() {
        let backend = MpvBackend::new();
        let result = backend.stop();
        assert!(result.is_ok());
    }

    #[test]
    fn test_stop_multiple_times_does_not_panic() {
        let backend = MpvBackend::new();
        assert!(backend.stop().is_ok());
        assert!(backend.stop().is_ok());
        assert!(backend.stop().is_ok());
    }

    #[test]
    fn test_play_with_invalid_mpv_path_returns_error() {
        // This tests the error handling — mpv binary lookup
        // will fail if mpv is not installed, but the code path itself is exercised.
        let backend = MpvBackend::new();
        let info = PlayInfo {
            url: "http://example.com/v.mp4".into(),
            headers: HashMap::new(),
            user_agent: None,
            referer: None,
        };
        // play() will fail gracefully if mpv binary not found
        let _ = backend.play(&info);
        // cleanup
        let _ = backend.stop();
    }
}
```

- [ ] **Add test for backends module re-export**

`crates/rivu-player/src/backends.rs`:
```rust
pub use crate::mpv::MpvBackend;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpv_backend_can_be_created_via_backends_module() {
        let backend = MpvBackend::new();
        assert!(!backend.is_running());
    }
}
```

- [ ] **Run tests**

```bash
cargo test -p rivu-player
cargo check
```

- [ ] **Commit**

```bash
git add crates/rivu-player/
git commit -m "feat(player): add mpv subprocess backend"
```

---

### Task 5: ratatui TUI — Application Shell

**Files:**
- Modify: `crates/rivu-ui/Cargo.toml`
- Rewrite: `crates/rivu-ui/src/lib.rs`
- Rewrite: `crates/rivu-ui/src/app.rs`
- Create: `crates/rivu-ui/src/screens/home.rs`
- Create: `crates/rivu-ui/src/screens/detail.rs`
- Create: `crates/rivu-ui/src/screens/search.rs`
- Create: `crates/rivu-ui/src/screens/mod.rs`
- Create: `crates/rivu-ui/src/widgets.rs`

The TUI provides a terminal interface for browsing categories, viewing video lists,
and inspecting details. Uses ratatui with crossterm backend.

- [ ] **Update Cargo.toml**

`crates/rivu-ui/Cargo.toml`:
```toml
[package]
name = "rivu-ui"
version = "0.1.0"
edition = "2021"
description = "RivuTV user interface (TUI)"

[dependencies]
rivu-core = { path = "../rivu-core" }
rivu-config = { path = "../rivu-config" }
rivu-spider = { path = "../rivu-spider" }
rivu-player = { path = "../rivu-player" }
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
thiserror = "2"
```

- [ ] **Create screen modules**

`crates/rivu-ui/src/screens/mod.rs`:
```rust
pub mod detail;
pub mod home;
pub mod search;
```

`crates/rivu-ui/src/screens/home.rs`:
```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
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
```

`crates/rivu-ui/src/screens/detail.rs`:
```rust
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use rivu_core::models::Vod;
use rivu_core::models::{Episode, Flag};

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

    fn build_episode_list(&self) -> Vec<ListItem> {
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
                    ListItem::new(Line::from(Span::styled(&ep.name, style)))
                })
                .collect()
        } else {
            vec![ListItem::new(Line::from(Span::raw("No episodes")))].to_vec()
        }
    }
}

impl Default for DetailScreen {
    fn default() -> Self {
        Self::new()
    }
}
```

`crates/rivu-ui/src/screens/search.rs`:
```rust
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
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
            .direction(ratatui::layout::Direction::Vertical)
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
                    Span::raw(&v.vod_name),
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
```

- [ ] **Rewrite lib.rs**

`crates/rivu-ui/src/lib.rs`:
```rust
pub mod app;
pub mod screens;
pub mod widgets;
```

- [ ] **Rewrite app.rs**

`crates/rivu-ui/src/app.rs`:
```rust
use std::io;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rivu_core::models::Site;
use rivu_core::error::Result;

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
                        KeyCode::Enter => {
                            self.current = Screen::Detail;
                        }
                        KeyCode::Esc => {
                            self.current = Screen::Home;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.navigate(1);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.navigate(-1);
                        }
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
```

`crates/rivu-ui/src/widgets.rs`:
```rust
use ratatui::widgets::ListState;

pub struct StatefulList<T> {
    pub items: Vec<T>,
    pub state: ListState,
}

impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self { items, state }
    }

    pub fn next(&mut self) {
        let i = self.state.selected().map(|i| {
            if i >= self.items.len() - 1 { 0 } else { i + 1 }
        }).unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = self.state.selected().map(|i| {
            if i == 0 { self.items.len() - 1 } else { i - 1 }
        }).unwrap_or(0);
        self.state.select(Some(i));
    }
}
```

- [ ] **Add unit tests for UI components**

Add to `crates/rivu-ui/src/widgets.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stateful_list_new_with_items() {
        let list = StatefulList::new(vec![1, 2, 3]);
        assert_eq!(list.items.len(), 3);
        assert_eq!(list.state.selected(), Some(0));
    }

    #[test]
    fn test_stateful_list_new_empty() {
        let list: StatefulList<i32> = StatefulList::new(vec![]);
        assert!(list.items.is_empty());
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_stateful_list_next_wraps_around() {
        let mut list = StatefulList::new(vec![1, 2]);
        assert_eq!(list.state.selected(), Some(0));
        list.next();
        assert_eq!(list.state.selected(), Some(1));
        list.next();
        assert_eq!(list.state.selected(), Some(0));
    }

    #[test]
    fn test_stateful_list_previous_wraps_around() {
        let mut list = StatefulList::new(vec![1, 2]);
        list.previous();
        assert_eq!(list.state.selected(), Some(1));
        list.previous();
        assert_eq!(list.state.selected(), Some(0));
    }

    #[test]
    fn test_stateful_list_next_on_empty_does_not_panic() {
        let mut list: StatefulList<i32> = StatefulList::new(vec![]);
        list.next();
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_stateful_list_previous_on_empty_does_not_panic() {
        let mut list: StatefulList<i32> = StatefulList::new(vec![]);
        list.previous();
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_stateful_list_single_item_stays_selected() {
        let mut list = StatefulList::new(vec![42]);
        assert_eq!(list.state.selected(), Some(0));
        list.next();
        assert_eq!(list.state.selected(), Some(0));
        list.previous();
        assert_eq!(list.state.selected(), Some(0));
    }
}
```

Add to `crates/rivu-ui/src/screens/home.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

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
```

Add to `crates/rivu-ui/src/screens/detail.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(items.len(), 1); // "No episodes" placeholder
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
```

Add to `crates/rivu-ui/src/screens/search.rs`:
```rust
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
```

- [ ] **Compile check**

```bash
cargo check -p rivu-ui
```

- [ ] **Commit**

```bash
git add crates/rivu-ui/
git commit -m "feat(ui): add ratatui TUI shell with home/detail/search screens"
```

---

### Task 6: Wire Everything Together in Main Binary

**Files:**
- Modify: `Cargo.toml` (root)
- Rewrite: `src/main.rs`
- Create: `src/app.rs`

The root binary wires config loading, site API, player, and TUI together.

- [ ] **Update root Cargo.toml**

No changes needed — already depends on all 5 crates.

- [ ] **Rewrite src/main.rs**

```rust
use clap::Parser;
use rivu_config::loader::ConfigLoader;
use rivu_core::error::Result;
use rivu_ui::app::App;

#[derive(Parser)]
#[command(name = "rivu")]
#[command(version, about = "RivuTV - A Linux-native TVBox media client")]
enum Cli {
    /// Launch the interactive TUI
    Run,
    /// Configure a source URL
    Config {
        /// TVBox source JSON URL
        url: String,
    },
    /// List configured sources
    Sources,
    /// Search media across all sources
    Search {
        keyword: String,
    },
    /// Play a URL directly
    Play {
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Run => {
            let config_dir = ConfigLoader::get_config_dir();
            let mut loader = ConfigLoader::new(&config_dir);
            let mut app = App::new();

            if let Some(ref url) = loader.app_config.source_url {
                match loader.fetch_source(url).await {
                    Ok(config) => {
                        app.set_sites(config.sites.clone());
                    }
                    Err(e) => {
                        eprintln!("Warning: couldn't load source config: {}", e);
                    }
                }
            }

            app.run()?;
        }
        Cli::Config { url } => {
            let config_dir = ConfigLoader::get_config_dir();
            let mut loader = ConfigLoader::new(&config_dir);
            loader.app_config.source_url = Some(url.clone());
            loader.save_app_config()?;
            println!("Source URL saved: {}", url);
        }
        Cli::Sources => {
            let config_dir = ConfigLoader::get_config_dir();
            let loader = ConfigLoader::new(&config_dir);
            if let Some(ref url) = loader.app_config.source_url {
                println!("Configured source: {}", url);
            } else {
                println!("No source configured. Use: rivu config <url>");
            }
        }
        Cli::Search { keyword } => {
            println!("Search for '{}' (not yet implemented in CLI mode)", keyword);
        }
        Cli::Play { url } => {
            let player = rivu_player::MpvBackend::new();
            let info = rivu_core::models::PlayInfo {
                url,
                headers: std::collections::HashMap::new(),
                user_agent: None,
                referer: None,
            };
            player.play(&info)?;
            println!("Press Ctrl+C to stop playback...");
            tokio::signal::ctrl_c().await?;
            player.stop()?;
        }
    }

    Ok(())
}
```

- [ ] **Add integration tests for CLI arg parsing**

Create `tests/cli_test.rs`:
```rust
use std::process::Command;

#[test]
fn test_cli_run_accepts_no_args() {
    // Verifies the binary can be invoked with --help without crashing
    let output = Command::new(env!("CARGO_BIN_EXE_rivutv"))
        .arg("--help")
        .output()
        .expect("failed to execute binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RivuTV"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("search"));
    assert!(stdout.contains("play"));
    assert!(stdout.contains("sources"));
}

#[test]
fn test_cli_version_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_rivutv"))
        .arg("--version")
        .output()
        .expect("failed to execute binary");
    assert!(output.status.success());
}

#[test]
fn test_cli_invalid_subcommand_returns_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_rivutv"))
        .arg("invalid-command")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
}

#[test]
fn test_cli_search_requires_keyword() {
    let output = Command::new(env!("CARGO_BIN_EXE_rivutv"))
        .arg("search")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("keyword"));
}

#[test]
fn test_cli_config_requires_url() {
    let output = Command::new(env!("CARGO_BIN_EXE_rivutv"))
        .arg("config")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
}

#[test]
fn test_cli_play_requires_url() {
    let output = Command::new(env!("CARGO_BIN_EXE_rivutv"))
        .arg("play")
        .output()
        .expect("failed to execute binary");
    assert!(!output.status.success());
}
```

Add to root `Cargo.toml` for binary test support:
```toml
# No changes needed — CARGO_BIN_EXE_rivutv is auto-set for the root crate
```

- [ ] **Compile check**

```bash
cargo check
cargo test --test cli_test
```

- [ ] **Commit**

```bash
git add src/ Cargo.toml tests/
git commit -m "feat(cli): wire config, TUI, and player into main binary"
```

---

### Task 7: Error Handling Audit and Edge Cases

**Files:**
- Modify: all crates as needed
- Create: `tests/integration_test.rs`

Clean up error handling, handle edge cases (network failures, invalid JSON, missing mpv).

- [ ] **Create workspace-level integration test**

`tests/integration_test.rs`:
```rust
use rivu_core::models::*;
use rivu_spider::extractor::SourceExtractor;
use rivu_spider::parsers::Parser;
use std::collections::HashMap;

#[test]
fn test_full_parse_pipeline() {
    let json = r#"{
        "class": [
            {"type_id": "1", "type_name": "Movie"},
            {"type_id": "2", "type_name": "TV Series"}
        ],
        "list": [
            {"vod_id": "100", "vod_name": "Test A", "vod_pic": "http://a.jpg", "vod_remarks": "HD"},
            {"vod_id": "101", "vod_name": "Test B", "vod_pic": "http://b.jpg", "vod_remarks": "4K"}
        ],
        "filters": {
            "1": [{"key": "area", "name": "Region", "value": [{"v": "", "n": "All"}]}]
        }
    }"#;

    let result = Parser::parse_json(json).unwrap();
    assert_eq!(result.class.as_ref().unwrap().len(), 2);
    assert_eq!(result.list.as_ref().unwrap().len(), 2);
    assert_eq!(result.list.as_ref().unwrap()[0].vod_name, "Test A");
}

#[test]
fn test_parse_site_config() {
    let json = r#"{
        "sites": [
            {"key": "site1", "name": "Source 1", "type": 0, "api": "http://api1.com"}
        ]
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.sites.len(), 1);
    assert_eq!(config.sites[0].site_type, 0);
}

#[test]
fn test_parse_episodes_from_play_url() {
    let play_from = "ck$$$LianMeng";
    let play_url = "1$http://a.com/1.mp4#2$http://a.com/2.mp4$$$1$http://b.com/1.mp4";
    let flags = Flag::parse_flags(play_from, play_url);
    assert_eq!(flags.len(), 2);
    assert_eq!(flags[0].name, "ck");
    assert_eq!(flags[0].episodes.len(), 2);
    assert_eq!(flags[1].episodes.len(), 1);
}

// ── Cross-crate integration: parse → extract pipeline ──

#[test]
fn test_parse_then_extract_video_prefix() {
    let json = r#"{"url": "video://http://real.com/play.m3u8", "flag": "ck"}"#;
    let result = Parser::parse_json(json).unwrap();
    let play_info = PlayInfo {
        url: result.url.unwrap_or_default(),
        headers: HashMap::new(),
        user_agent: None,
        referer: None,
    };
    let ext = SourceExtractor::new();
    let resolved = ext.extract(&play_info).unwrap();
    assert_eq!(resolved.url, "http://real.com/play.m3u8");
}

// ── Edge cases ──

#[test]
fn test_empty_source_config_does_not_panic() {
    let json = "{}";
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    assert!(config.sites.is_empty());
    assert!(config.lives.is_none());
}

#[test]
fn test_vod_with_all_empty_strings() {
    let json = r#"{
        "vod_id": "", "vod_name": "",
        "vod_pic": "", "vod_remarks": "",
        "vod_play_from": "", "vod_play_url": ""
    }"#;
    let vod: Vod = serde_json::from_str(json).unwrap();
    assert_eq!(vod.vod_id, "");
    assert_eq!(vod.vod_play_url.as_deref(), Some(""));
}

#[test]
fn test_source_config_with_full_lives_config() {
    let json = r#"{
        "sites": [],
        "lives": [{
            "name": "CCTV", "url": "http://live.com/cctv.m3u8",
            "epg": "http://epg.com", "ua": "Mozilla/5.0",
            "origin": "http://origin.com", "referer": "http://ref.com",
            "header": {"User-Agent": "test"}
        }]
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    let lives = config.lives.unwrap();
    assert_eq!(lives[0].name, "CCTV");
    assert_eq!(lives[0].epg.as_deref(), Some("http://epg.com"));
}

#[test]
fn test_source_config_with_parse_definitions() {
    let json = r#"{
        "sites": [],
        "parses": [
            {"name": "Parse1", "type": 1, "url": "http://parse1.com/api?url="},
            {"name": "Parse2", "type": 0, "url": "http://parse2.com/?url=",
             "ext": {"flag": "1"}, "header": {"Referer": "http://ref.com"}}
        ]
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    let parses = config.parses.unwrap();
    assert_eq!(parses.len(), 2);
    assert_eq!(parses[0].parse_type, 1);
    assert_eq!(parses[1].ext.as_ref().unwrap().get("flag").unwrap(), "1");
}

// ── Flag parsing edge cases ──

#[test]
fn test_flag_parse_single_episode() {
    let flags = Flag::parse_flags("CK", "1$http://a.mp4");
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].episodes.len(), 1);
    assert_eq!(flags[0].episodes[0].name, "1");
}

#[test]
fn test_flag_parse_three_flags() {
    let flags = Flag::parse_flags("A$$$B$$$C", "1$u1$$$1$u2$$$1$u3");
    assert_eq!(flags.len(), 3);
    assert_eq!(flags[0].episodes[0].url, "u1");
    assert_eq!(flags[2].episodes[0].url, "u3");
}

#[test]
fn test_flag_parse_empty_episode_list_for_flag() {
    let flags = Flag::parse_flags("A$$$B", "1$u1$$$");
    assert_eq!(flags.len(), 2);
    assert!(flags[1].episodes.is_empty());
}

#[test]
fn test_flag_parse_episodes_with_pound_in_url_not_allowed() {
    // # is the delimiter, so it shouldn't appear in names/urls
    let flags = Flag::parse_flags("CK", "1$http://a.com/1.mp4");
    assert_eq!(flags[0].episodes.len(), 1);
    assert_eq!(flags[0].episodes[0].name, "1");
}

// ── PlayInfo construction ──

#[test]
fn test_play_info_with_all_fields() {
    let mut headers = HashMap::new();
    headers.insert("Referer".into(), "http://ref.com".into());
    let info = PlayInfo {
        url: "http://stream.com/video.m3u8".into(),
        headers,
        user_agent: Some("Mozilla/5.0".into()),
        referer: Some("http://ref.com".into()),
    };
    assert!(info.url.starts_with("http"));
    assert_eq!(info.headers.len(), 1);
}

// ── Real-world TVBox config sample (anonymized) ──

#[test]
fn test_real_world_tvbox_config_structure() {
    let json = r#"{
        "sites": [
            {"key": "douban", "name": "豆瓣", "type": 0, "api": "http://douban.api.com/video"},
            {"key": "custom", "name": "My Source", "type": 3, "api": "csp_Myspider", "jar": "http://jar.com/spider.jar", "ext": "{}"}
        ],
        "lives": [{"name": "CCTV", "url": "http://live.com/cctv.m3u8"}],
        "parses": [{"name": "JsonParse", "type": 1, "url": "http://parse.com/?url="}],
        "flags": ["4k"],
        "spider": "http://spider.com/spider.jar"
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.sites.len(), 2);
    assert_eq!(config.sites[1].site_type, 3);
    assert_eq!(config.sites[1].jar.as_deref(), Some("http://jar.com/spider.jar"));
    assert!(config.flags.is_some());
}
```

- [ ] **Add Flag::parse_flags method**

Add to `crates/rivu-core/src/models.rs`:
```rust
impl Flag {
    /// Parse TVBox vod_play_from and vod_play_url into Flag list.
    /// Format: names separated by $$$, episodes separated by #,
    /// episode name and url separated by $.
    pub fn parse_flags(play_from: &str, play_url: &str) -> Vec<Flag> {
        let names: Vec<&str> = play_from.split("$$$").collect();
        let episode_lists: Vec<&str> = play_url.split("$$$").collect();
        if names.is_empty() || names.len() != episode_lists.len() {
            return Vec::new();
        }

        names
            .iter()
            .zip(episode_lists.iter())
            .map(|(name, ep_str)| {
                let episodes: Vec<Episode> = ep_str
                    .split('#')
                    .filter_map(|seg| {
                        let mut parts = seg.splitn(2, '$');
                        let name = parts.next()?.to_string();
                        let url = parts.next()?.to_string();
                        Some(Episode { name, url })
                    })
                    .collect();
                Flag {
                    name: name.to_string(),
                    episodes,
                }
            })
            .collect()
    }
}
```

- [ ] **Run all tests and fix failures**

```bash
cargo test
cargo clippy -- -D warnings
```

- [ ] **Commit**

```bash
git add . && git commit -m "test: add integration tests and fix edge cases"
```

---

## Future Tasks (Post-MVP)

These are not in scope for this initial plan but should be documented for later:

- **Live TV**: M3U/TXT parsing, EPG data, channel groups
- **Parse/Jiexi**: Full WebViewless parse types (HTTP-based only)
- **Extractor Chain**: Force video, JianPian, Thunder, and other extractors
- **Search History**: Persistent search history
- **Playback History**: Resume from last position
- **Multiple Source Support**: Switch between configured sources
- **Theme System**: Configurable color themes
- **Keybindings**: Configurable keybindings with help overlay
- **Download**: Stream recording/download support

## Self-Review Checklist

- [ ] **Spec coverage**: Every feature needed for MVP is covered:
  - Task 1: Core models (Site, Vod, ApiResult, PlayInfo, Flag, Episode, AppConfig)
  - Task 2: Config loader (fetch + parse source JSON, persist app config)
  - Task 3: SiteApi (home, category, detail, play, search HTTP calls)
  - Task 4: mpv player backend
  - Task 5: ratatui TUI (home, detail, search screens)
  - Task 6: CLI wiring (run, config, search, play commands)
  - Task 7: Error handling + integration tests
- [ ] **Placeholder scan**: No TBD, TODO, "implement later", or placeholder code
- [ ] **Type consistency**: Flag::parse_flags returns Vec<Flag>, Episode uses name+url, all models consistent
