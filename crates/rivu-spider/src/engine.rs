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
        let base = if path.is_empty() {
            base.to_string()
        } else {
            format!("{}/{}", base, path.trim_start_matches('/'))
        };
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        if query.is_empty() {
            return base;
        }
        let separator = if base.contains('?') { "&" } else { "?" };
        format!("{}{}{}", base, separator, query)
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
    fn test_build_url_no_params_returns_base() {
        let engine = SpiderEngine::new();
        let site = test_site();
        let url = engine.build_url(&site, "", &[]);
        assert_eq!(url, "http://example.com/api");
    }

    #[test]
    fn test_build_url_path_and_params() {
        let engine = SpiderEngine::new();
        let mut site = test_site();
        site.api = "http://example.com/".into();
        let url = engine.build_url(&site, "proxy", &[("do", "get")]);
        assert_eq!(url, "http://example.com/proxy?do=get");
    }
}
