use serde::de::{self, Deserializer, Unexpected};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Source Configuration ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Site {
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub site_type: u8,
    pub api: String,
    pub jar: Option<String>,
    pub ext: Option<serde_json::Value>,
    pub searchable: Option<i32>,
    #[serde(rename = "quickSearch")]
    pub quick_search: Option<i32>,
    pub filterable: Option<i32>,
    #[serde(rename = "playerType", deserialize_with = "de_opt_u8_str")]
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
#[serde(default)]
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
#[serde(default)]
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

impl Flag {
    pub fn parse_flags(play_from: &str, play_url: &str) -> Vec<Flag> {
        if play_from.is_empty() || play_url.is_empty() {
            return Vec::new();
        }
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

/// Deserialize an `Option<u8>` that accepts both integers and strings.
fn de_opt_u8_str<'de, D>(deserializer: D) -> std::result::Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptU8Visitor;
    impl<'de> de::Visitor<'de> for OptU8Visitor {
        type Value = Option<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a u8 integer or string")
        }

        fn visit_none<E: de::Error>(self) -> Result<Option<u8>, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Option<u8>, E> {
            Ok(None)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<u8>, E> {
            Ok(Some(v as u8))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<u8>, E> {
            Ok(Some(v as u8))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<u8>, E> {
            v.parse::<u8>()
                .map(Some)
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &"a u8 integer as string"))
        }
    }

    deserializer.deserialize_any(OptU8Visitor)
}

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
