pub mod bili;
pub mod generic;
pub mod ygp;
pub mod wogg;

use std::collections::HashMap;
use async_trait::async_trait;
use rivu_core::error::Result;
use rivu_core::models::{ApiResult, PlayInfo, Site};

#[async_trait]
pub trait Spider: Send + Sync {
    fn name(&self) -> &str;
    async fn home(&self, site: &Site) -> Result<ApiResult>;
    async fn category(&self, site: &Site, tid: &str, pg: i32, filters: &[(&str, &str)]) -> Result<ApiResult>;
    async fn detail(&self, site: &Site, ids: &[String]) -> Result<ApiResult>;
    async fn play(&self, site: &Site, flag: &str, id: &str) -> Result<PlayInfo>;
    async fn search(&self, site: &Site, keyword: &str, pg: i32) -> Result<ApiResult>;
}

pub struct SpiderRegistry {
    spiders: HashMap<String, Box<dyn Spider>>,
}

impl SpiderRegistry {
    pub fn new() -> Self {
        Self { spiders: HashMap::new() }
    }

    pub fn register(&mut self, spider: Box<dyn Spider>) {
        self.spiders.insert(spider.name().to_string(), spider);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Spider> {
        self.spiders.get(name).map(|s| s.as_ref())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.spiders.contains_key(name)
    }

    pub fn names(&self) -> Vec<String> {
        self.spiders.keys().cloned().collect()
    }

    fn register_generic(&mut self, names: &[&str]) {
        for name in names {
            self.spiders.insert(name.to_string(), Box::new(generic::GenericSpider::new(name)));
        }
    }

    pub fn register_builtin(&mut self) {
        self.register(Box::new(bili::BiliSpider::new()));
        self.register(Box::new(ygp::YGPSpider::new()));
        self.register(Box::new(wogg::WoGGSpider::new()));

        let csp_names = &[
            "csp_DouDouGuard", "csp_MyDriveGuard", "csp_MusicGuard",
            "csp_SeedhubGuard", "csp_S_zpsGuard", "csp_T4Guard",
            "csp_YCyzGuard", "csp_NewCzGuard", "csp_BttwooGuard",
            "csp_JPJGuard", "csp_LibvioGuard", "csp_NmyswvGuard",
            "csp_JpysGuard", "csp_AppTTGuard", "csp_AppSxGuard",
            "csp_AueteGuard", "csp_SixVGuard", "csp_Dm84Guard",
            "csp_Anime1Guard", "csp_KanqiuGuard", "csp_LiveGzGuard",
            "csp_AllliveGuard", "csp_Tingshu275Guard", "csp_FirstAidGuard",
            "csp_KkSsGuard", "csp_UuSsGuard", "csp_MIPanSoGuard",
            "csp_YpanSoGuard", "csp_BpanSoGuard", "csp_PushGuard",
            "csp_XPathGuard",
        ];
        self.register_generic(csp_names);

        let js_urls = &[
            "https://gh-proxy.com/https://raw.githubusercontent.com/fantaiying7/EXT/refs/heads/main/drpy2.min.js",
            "https://git.yylx.win/https://raw.githubusercontent.com/fantaiying7/EXT/refs/heads/main/drpy2.min.js",
        ];
        self.register_generic(js_urls);
    }
}

impl Default for SpiderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestSpider;

    #[async_trait]
    impl Spider for TestSpider {
        fn name(&self) -> &str { "csp_TestGuard" }
        async fn home(&self, _site: &Site) -> Result<ApiResult> {
            Ok(ApiResult::default())
        }
        async fn category(&self, _site: &Site, _tid: &str, _pg: i32, _filters: &[(&str, &str)]) -> Result<ApiResult> {
            Ok(ApiResult::default())
        }
        async fn detail(&self, _site: &Site, _ids: &[String]) -> Result<ApiResult> {
            Ok(ApiResult::default())
        }
        async fn play(&self, _site: &Site, _flag: &str, _id: &str) -> Result<PlayInfo> {
            Ok(PlayInfo { url: "".into(), headers: HashMap::new(), user_agent: None, referer: None })
        }
        async fn search(&self, _site: &Site, _keyword: &str, _pg: i32) -> Result<ApiResult> {
            Ok(ApiResult::default())
        }
    }

    #[tokio::test]
    async fn test_registry_register_and_get() {
        let mut reg = SpiderRegistry::new();
        reg.register(Box::new(TestSpider));
        assert!(reg.contains("csp_TestGuard"));
        assert!(reg.get("csp_TestGuard").is_some());
    }

    #[tokio::test]
    async fn test_registry_unknown_returns_none() {
        let reg = SpiderRegistry::new();
        assert!(!reg.contains("csp_Nonexistent"));
        assert!(reg.get("csp_Nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_registry_empty_names() {
        let reg = SpiderRegistry::new();
        assert!(reg.names().is_empty());
    }

    #[test]
    fn test_register_builtin_includes_all_csp_names() {
        let mut reg = SpiderRegistry::new();
        reg.register_builtin();
        assert!(reg.contains("csp_BiliGuard"));
        assert!(reg.contains("csp_YGPGuard"));
        assert!(reg.contains("csp_WoGGGuard"));
        assert!(reg.contains("csp_T4Guard"));
        assert!(reg.contains("csp_AppSxGuard"));
        assert!(reg.contains("csp_DouDouGuard"));
        assert!(reg.contains("csp_MusicGuard"));
    }

    #[test]
    fn test_register_builtin_includes_js_urls() {
        let mut reg = SpiderRegistry::new();
        reg.register_builtin();
        assert!(reg.contains("https://gh-proxy.com/https://raw.githubusercontent.com/fantaiying7/EXT/refs/heads/main/drpy2.min.js"));
    }
}
