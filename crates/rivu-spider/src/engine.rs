use rivu_core::error::{CoreError, Result};
use rivu_core::models::{ApiResult, PlayInfo, Site};
use crate::site_api::SiteApi;
use crate::spider::{Spider, SpiderRegistry};
use crate::CHROME_UA;

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use async_trait::async_trait;
#[cfg(test)]
use rivu_core::models::{Class, Vod};

pub struct SpiderEngine {
    client: reqwest::Client,
}

impl SpiderEngine {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(CHROME_UA)
                .timeout(std::time::Duration::from_secs(10))
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

pub struct SpiderApi {
    site_api: SiteApi,
    registry: SpiderRegistry,
}

impl SpiderApi {
    pub fn new(site_api: SiteApi, registry: SpiderRegistry) -> Self {
        Self { site_api, registry }
    }

    fn spider(&self, site: &Site) -> Result<&dyn Spider> {
        self.registry.get(&site.api)
            .ok_or_else(|| CoreError::Spider(format!("Spider '{}' not implemented", site.api)))
    }

    pub async fn home(&self, site: &Site) -> Result<ApiResult> {
        match site.site_type {
            3 => self.spider(site)?.home(site).await,
            _ => self.site_api.home(site).await,
        }
    }

    pub async fn category(&self, site: &Site, tid: &str, pg: i32, filters: &[(&str, &str)]) -> Result<ApiResult> {
        match site.site_type {
            3 => self.spider(site)?.category(site, tid, pg, filters).await,
            _ => self.site_api.category(site, tid, pg, filters).await,
        }
    }

    pub async fn detail(&self, site: &Site, ids: &[String]) -> Result<ApiResult> {
        match site.site_type {
            3 => self.spider(site)?.detail(site, ids).await,
            _ => self.site_api.detail(site, ids).await,
        }
    }

    pub async fn play(&self, site: &Site, flag: &str, id: &str) -> Result<PlayInfo> {
        match site.site_type {
            3 => self.spider(site)?.play(site, flag, id).await,
            _ => self.site_api.play(site, flag, id).await,
        }
    }

    pub async fn search(&self, site: &Site, keyword: &str, pg: i32) -> Result<ApiResult> {
        match site.site_type {
            3 => self.spider(site)?.search(site, keyword, pg).await,
            _ => self.site_api.search(site, keyword, pg).await,
        }
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

    struct MockSpider;
    #[async_trait]
    impl Spider for MockSpider {
        fn name(&self) -> &str { "csp_MockGuard" }
        async fn home(&self, _site: &Site) -> Result<ApiResult> {
            Ok(ApiResult {
                class: Some(vec![Class { type_id: "1".into(), type_name: "Mock".into(), type_flag: None, filters: None }]),
                list: Some(vec![Vod { vod_id: "1".into(), vod_name: "Mock Movie".into(), ..Default::default() }]),
                ..Default::default()
            })
        }
        async fn category(&self, _site: &Site, _tid: &str, _pg: i32, _filters: &[(&str, &str)]) -> Result<ApiResult> {
            Ok(ApiResult::default())
        }
        async fn detail(&self, _site: &Site, _ids: &[String]) -> Result<ApiResult> {
            Ok(ApiResult::default())
        }
        async fn play(&self, _site: &Site, _flag: &str, _id: &str) -> Result<PlayInfo> {
            Ok(PlayInfo { url: "https://mock.play".into(), headers: HashMap::new(), user_agent: None, referer: None })
        }
        async fn search(&self, _site: &Site, _keyword: &str, _pg: i32) -> Result<ApiResult> {
            Ok(ApiResult::default())
        }
    }

    fn test_site_type_3() -> Site {
        Site {
            key: "mock".into(), name: "Mock".into(), site_type: 3,
            api: "csp_MockGuard".into(), jar: None, ext: None,
            searchable: None, quick_search: None, filterable: None,
            player_type: None, categories: None,
        }
    }

    #[tokio::test]
    async fn test_spider_api_routes_type_3_to_spider() {
        let mut registry = SpiderRegistry::new();
        registry.register(Box::new(MockSpider));
        let api = SpiderApi::new(SiteApi::new(), registry);
        let result = api.home(&test_site_type_3()).await.unwrap();
        assert_eq!(result.list.unwrap()[0].vod_name, "Mock Movie");
    }

    #[tokio::test]
    async fn test_spider_api_unknown_spider_returns_error() {
        let registry = SpiderRegistry::new();
        let api = SpiderApi::new(SiteApi::new(), registry);
        let result = api.home(&test_site_type_3()).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not implemented"));
    }
}
