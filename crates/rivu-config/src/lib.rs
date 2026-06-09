use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub sources: Vec<SourceConfig>,
    pub ui: UiConfig,
    pub player: PlayerConfig,
}

#[derive(Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub url: String,
    pub api: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub language: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerConfig {
    pub backend: String,
    pub hardware_accel: bool,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            api: String::new(),
            enabled: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "zh".to_string(),
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            backend: "mpv".to_string(),
            hardware_accel: true,
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(path, json).ok();
        }
    }
}
