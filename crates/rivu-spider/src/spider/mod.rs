pub mod bili;
pub mod ygp;

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

    pub fn register_builtin(&mut self) {
        self.register(Box::new(bili::BiliSpider::new()));
        self.register(Box::new(ygp::YGPSpider::new()));
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
}
