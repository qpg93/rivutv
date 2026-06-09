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
        let pg_str = pg.to_string();
        let mut params = vec![("ac", "videolist"), ("t", tid), ("pg", &pg_str)];
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
        let pg_str = pg.to_string();
        let url = self.engine.build_url(site, "", &[("ac", "videolist"), ("wd", keyword), ("pg", &pg_str)]);
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
