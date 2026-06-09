use async_trait::async_trait;
use rivu_core::error::Result;
use rivu_core::models::{ApiResult, PlayInfo, Site, Class, Vod};
use serde_json::Value;
use crate::spider::Spider;

pub struct GenericSpider {
    client: reqwest::Client,
    spider_name: String,
}

impl GenericSpider {
    pub fn new(name: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build reqwest client"),
            spider_name: name.to_string(),
        }
    }

    fn ext_value(ext: &Option<Value>, key: &str) -> Option<String> {
        let ext = ext.as_ref()?;
        match ext {
            Value::Object(map) => map.get(key)?.as_str().map(|s| s.to_string()),
            _ => None,
        }
    }

    fn ext_text(ext: &Option<Value>) -> Option<String> {
        let ext = ext.as_ref()?;
        match ext {
            Value::String(s) => {
                let s = s.trim();
                if s.is_empty() { None } else { Some(s.to_string()) }
            }
            _ => None,
        }
    }

    fn is_js_url(api: &str) -> bool {
        api.ends_with(".js") || api.contains("drpy2") || api.contains(".min.js")
    }

    async fn cloud_drive_home(&self, site: &Site) -> Result<ApiResult> {
        let base = Self::ext_value(&site.ext, "_source_base").unwrap_or_default();
        let path = match Self::ext_value(&site.ext, "Cloud-drive") {
            Some(p) => p,
            None => return Ok(ApiResult::default()),
        };

        let url = format!("{}{}", base, path);
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

        let mut cats: Vec<String> = entries.iter()
            .map(|(_, _, t)| t.clone())
            .collect();
        cats.sort();
        cats.dedup();
        let categories: Vec<Class> = cats.into_iter().enumerate()
            .map(|(i, name)| Class {
                type_id: (i + 1).to_string(),
                type_name: name,
                type_flag: None,
                filters: None,
            })
            .collect();

        let list: Vec<Vod> = entries.iter().map(|(name, url, type_name)| Vod {
            vod_id: url.clone(),
            vod_name: name.clone(),
            vod_pic: Some(String::new()),
            vod_remarks: Some(type_name.clone()),
            vod_play_from: Some("Cloud".into()),
            vod_play_url: Some(format!("1${}", url)),
            ..Default::default()
        }).collect();

        Ok(ApiResult { class: Some(categories), list: Some(list), ..Default::default() })
    }

    async fn site_api_home(&self, base_url: &str) -> Result<ApiResult> {
        let url = format!("{}?ac=list", base_url.trim_end_matches('/'));
        let resp = self.client.get(&url).send().await;
        let resp = match resp {
            Ok(r) => r,
            Err(_) => return Ok(ApiResult::default()),
        };
        let text = resp.text().await.unwrap_or_default();
        if text.is_empty() { return Ok(ApiResult::default()); }
        Ok(serde_json::from_str(&text).ok().unwrap_or_default())
    }

    async fn site_api_search(&self, base_url: &str, keyword: &str) -> Result<ApiResult> {
        let url = format!("{}?ac=detail&wd={}", base_url.trim_end_matches('/'), keyword);
        let resp = self.client.get(&url).send().await;
        let resp = match resp {
            Ok(r) => r,
            Err(_) => return Ok(ApiResult::default()),
        };
        let text = resp.text().await.unwrap_or_default();
        if text.is_empty() { return Ok(ApiResult::default()); }
        Ok(serde_json::from_str(&text).ok().unwrap_or_default())
    }

    async fn json_config_home(&self, ext: &Option<Value>) -> Result<ApiResult> {
        let json_url = match Self::ext_value(ext, "json") {
            Some(u) => u,
            None => return Ok(ApiResult::default()),
        };
        let resp = self.client.get(&json_url).send().await;
        let resp = match resp {
            Ok(r) => r,
            Err(_) => return Ok(ApiResult::default()),
        };
        let text = resp.text().await.unwrap_or_default();
        if text.is_empty() { return Ok(ApiResult::default()); }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            let list: Vec<Vod> = v.as_array().map(|arr| {
                arr.iter().filter_map(|item| {
                    let name = item["name"].as_str().unwrap_or("").to_string();
                    if name.is_empty() { return None; }
                    Some(Vod {
                        vod_id: item["id"].as_str().or_else(|| item["url"].as_str()).unwrap_or("").to_string(),
                        vod_name: name,
                        vod_pic: item["pic"].as_str().map(|s| s.to_string()),
                        vod_remarks: item["remarks"].as_str().map(|s| s.to_string()),
                        vod_play_from: Some("Source".into()),
                        vod_play_url: Some(format!("1${}",
                            item["url"].as_str().or_else(|| item["id"].as_str()).unwrap_or(""))),
                        ..Default::default()
                    })
                }).collect()
            }).unwrap_or_default();

            return Ok(ApiResult { list: Some(list), ..Default::default() });
        }

        Ok(ApiResult::default())
    }
}

#[async_trait]
impl Spider for GenericSpider {
    fn name(&self) -> &str {
        &self.spider_name
    }

    async fn home(&self, site: &Site) -> Result<ApiResult> {
        if Self::is_js_url(&site.api) {
            return Ok(ApiResult::default());
        }
        if Self::ext_value(&site.ext, "Cloud-drive").is_some() {
            return self.cloud_drive_home(site).await;
        }
        if Self::ext_value(&site.ext, "json").is_some() {
            return self.json_config_home(&site.ext).await;
        }
        if let Some(url) = Self::ext_value(&site.ext, "siteUrl") {
            return self.site_api_home(&url).await;
        }
        if let Some(text) = Self::ext_text(&site.ext) {
            if text.starts_with("http://") || text.starts_with("https://") {
                return self.site_api_home(&text).await;
            }
        }
        Ok(ApiResult::default())
    }

    async fn category(&self, site: &Site, tid: &str, _pg: i32, _filters: &[(&str, &str)]) -> Result<ApiResult> {
        if Self::ext_value(&site.ext, "Cloud-drive").is_some() {
            let result = self.cloud_drive_home(site).await?;
            let filtered: Vec<Vod> = result.list.unwrap_or_default().into_iter()
                .filter(|v| v.vod_remarks.as_deref() == Some(tid))
                .collect();
            return Ok(ApiResult { class: None, list: Some(filtered), ..Default::default() });
        }
        if let Some(url) = Self::ext_value(&site.ext, "siteUrl") {
            let url = format!("{}?ac=videolist&t={}", url.trim_end_matches('/'), tid);
            let resp = self.client.get(&url).send().await;
            if let Ok(r) = resp {
                let text = r.text().await.unwrap_or_default();
                if !text.is_empty() {
                    if let Ok(api) = serde_json::from_str::<ApiResult>(&text) {
                        return Ok(api);
                    }
                }
            }
        }
        Ok(ApiResult::default())
    }

    async fn detail(&self, _site: &Site, ids: &[String]) -> Result<ApiResult> {
        let id = ids.first().map(|s| s.as_str()).unwrap_or("");
        let vod = Vod {
            vod_id: id.to_string(),
            vod_name: "Media".into(),
            vod_play_from: Some("Source".into()),
            vod_play_url: Some(format!("1${}", id)),
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

    async fn search(&self, site: &Site, keyword: &str, _pg: i32) -> Result<ApiResult> {
        if Self::is_js_url(&site.api) {
            return Ok(ApiResult::default());
        }
        if Self::ext_value(&site.ext, "Cloud-drive").is_some() {
            let result = self.cloud_drive_home(site).await?;
            let kw = keyword.to_lowercase();
            let filtered: Vec<Vod> = result.list.unwrap_or_default().into_iter()
                .filter(|v| v.vod_name.to_lowercase().contains(&kw))
                .collect();
            return Ok(ApiResult { list: Some(filtered), ..Default::default() });
        }
        if let Some(url) = Self::ext_value(&site.ext, "siteUrl") {
            return self.site_api_search(&url, keyword).await;
        }
        if let Some(text) = Self::ext_text(&site.ext) {
            if text.starts_with("http://") || text.starts_with("https://") {
                return self.site_api_search(&text, keyword).await;
            }
        }
        Ok(ApiResult { list: None, ..Default::default() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_site() -> Site {
        Site {
            key: "generic".into(), name: "Generic".into(), site_type: 3,
            api: "csp_TestGuard".into(), jar: None, ext: None,
            searchable: None, quick_search: None, filterable: None,
            player_type: None, categories: None,
        }
    }

    #[tokio::test]
    async fn test_generic_empty_ext_returns_empty() {
        let spider = GenericSpider::new("csp_TestGuard");
        let result = spider.home(&test_site()).await.unwrap();
        assert!(result.class.is_none() || result.class.as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_generic_js_url_returns_empty() {
        let spider = GenericSpider::new("csp_JSGuard");
        let mut site = test_site();
        site.api = "https://example.com/drpy2.min.js".into();
        let result = spider.home(&site).await.unwrap();
        assert!(result.list.unwrap_or_default().is_empty());
    }

    #[test]
    fn test_generic_ext_value() {
        let ext = Some(serde_json::json!({"Cloud-drive": "test.txt"}));
        assert_eq!(GenericSpider::ext_value(&ext, "Cloud-drive"), Some("test.txt".into()));
        assert_eq!(GenericSpider::ext_value(&ext, "missing"), None);
    }

    #[test]
    fn test_generic_is_js_url() {
        assert!(GenericSpider::is_js_url("https://example.com/drpy2.min.js"));
        assert!(!GenericSpider::is_js_url("csp_TestGuard"));
    }
}
