use async_trait::async_trait;
use rivu_core::error::{Result, CoreError};
use rivu_core::models::{ApiResult, PlayInfo, Site, Class, Vod};
use serde_json::Value;
use crate::spider::Spider;

pub struct WoGGSpider {
    client: reqwest::Client,
}

impl WoGGSpider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .build().unwrap(),
        }
    }

    fn extract_cloud_path(ext: &Option<Value>) -> Option<(String, String)> {
        let ext = ext.as_ref()?;
        let base = ext.get("_source_base")?.as_str()?.to_string();
        let path = ext.get("Cloud-drive")?.as_str()?.to_string();
        Some((base, path))
    }

    fn extract_categories(entries: &[(String, String, String)]) -> Vec<Class> {
        let mut cats: Vec<String> = entries.iter()
            .map(|(_, _, t)| t.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats.into_iter().enumerate()
            .map(|(i, name)| Class {
                type_id: (i + 1).to_string(),
                type_name: name,
                type_flag: None,
                filters: None,
            })
            .collect()
    }
}

#[async_trait]
impl Spider for WoGGSpider {
    fn name(&self) -> &str {
        "csp_WoGGGuard"
    }

    async fn home(&self, site: &Site) -> Result<ApiResult> {
        let (base, path) = Self::extract_cloud_path(&site.ext)
            .ok_or_else(|| CoreError::Spider("WoGGSpider requires ext with Cloud-drive and _source_base".into()))?;

        let url = format!("{}{}", base.trim_end_matches('/'), path);
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        let mut entries: Vec<(String, String, String)> = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.split("$$").collect();
            if parts.len() >= 2 {
                entries.push((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts.get(2).map(|s| s.to_string()).unwrap_or_default(),
                ));
            }
        }

        let categories = Self::extract_categories(&entries);

        let list: Vec<Vod> = entries.iter().map(|(name, url, type_name)| Vod {
            vod_id: url.clone(),
            vod_name: name.clone(),
            vod_pic: Some(String::new()),
            vod_remarks: Some(type_name.clone()),
            vod_play_from: Some("Cloud".into()),
            vod_play_url: Some(format!("1${}", url)),
            ..Default::default()
        }).collect();

        Ok(ApiResult {
            class: Some(categories),
            list: Some(list),
            ..Default::default()
        })
    }

    async fn category(&self, site: &Site, tid: &str, _pg: i32, _filters: &[(&str, &str)]) -> Result<ApiResult> {
        let result = self.home(site).await?;
        let filtered: Vec<Vod> = result.list.unwrap_or_default().into_iter()
            .filter(|v| v.vod_remarks.as_deref() == Some(tid))
            .collect();
        Ok(ApiResult { class: None, list: Some(filtered), ..Default::default() })
    }

    async fn detail(&self, _site: &Site, ids: &[String]) -> Result<ApiResult> {
        let url = ids.first().map(|s| s.as_str()).unwrap_or("");
        let vod = Vod {
            vod_id: url.to_string(),
            vod_name: "Cloud Drive".into(),
            vod_pic: Some(String::new()),
            vod_play_from: Some("Cloud".into()),
            vod_play_url: Some(format!("1${}", url)),
            ..Default::default()
        };
        Ok(ApiResult { class: None, list: Some(vec![vod]), ..Default::default() })
    }

    async fn play(&self, _site: &Site, _flag: &str, id: &str) -> Result<PlayInfo> {
        Ok(PlayInfo {
            url: id.to_string(),
            headers: std::collections::HashMap::new(),
            user_agent: None,
            referer: None,
        })
    }

    async fn search(&self, _site: &Site, _keyword: &str, _pg: i32) -> Result<ApiResult> {
        Ok(ApiResult { class: None, list: None, ..Default::default() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rivu_core::models::Site;

    #[test]
    fn test_wogg_name() {
        let spider = WoGGSpider::new();
        assert_eq!(spider.name(), "csp_WoGGGuard");
    }

    #[test]
    fn test_extract_cloud_path_missing() {
        let site = Site { ext: None, ..Default::default() };
        assert!(WoGGSpider::extract_cloud_path(&site.ext).is_none());
    }

    #[test]
    fn test_extract_categories() {
        let entries = vec![
            ("a".into(), "u1".into(), "movie".into()),
            ("b".into(), "u2".into(), "tv".into()),
            ("c".into(), "u3".into(), "movie".into()),
        ];
        let cats = WoGGSpider::extract_categories(&entries);
        assert_eq!(cats.len(), 2);
    }
}
