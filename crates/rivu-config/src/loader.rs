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

        let json = serde_json::to_string_pretty(&config).unwrap();
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        drop(file);

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

        let app_config: AppConfig = std::fs::read_to_string(&path)
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
