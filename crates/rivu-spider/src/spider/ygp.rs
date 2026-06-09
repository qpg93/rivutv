use std::collections::HashMap;
use async_trait::async_trait;
use rivu_core::error::Result;
use rivu_core::models::{ApiResult, PlayInfo, Site, Class, Vod};
use crate::spider::Spider;

pub struct YGPSpider {
    client: reqwest::Client,
}

impl YGPSpider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .build().unwrap(),
        }
    }

    fn base_url() -> &'static str {
        "https://www.ygdy8.com"
    }

    async fn fetch_page(&self, url: &str) -> Result<String> {
        let resp = self.client.get(url).send().await?;
        Ok(resp.text().await?)
    }

    fn parse_list_html(html: &str) -> Vec<Vod> {
        use scraper::{Html, Selector};
        let doc = Html::parse_document(html);
        let link_sel = Selector::parse("table.tbspan a.ulink").unwrap();
        let td_sel = Selector::parse("table.tbspan").unwrap();

        let mut list = Vec::new();
        for table in doc.select(&td_sel) {
            if let Some(link) = table.select(&link_sel).next() {
                let name = link.text().collect::<String>().trim().to_string();
                let href = link.value().attr("href").unwrap_or("").to_string();
                if !name.is_empty() && !href.is_empty() {
                    list.push(Vod {
                        vod_id: href.clone(),
                        vod_name: name,
                        ..Default::default()
                    });
                }
            }
        }
        list
    }
}

#[async_trait]
impl Spider for YGPSpider {
    fn name(&self) -> &str {
        "csp_YGPGuard"
    }

    async fn home(&self, _site: &Site) -> Result<ApiResult> {
        let html = self.fetch_page(&format!("{}/html/gndy/dyzz/index.html", Self::base_url())).await?;
        let list = Self::parse_list_html(&html);

        let classes = vec![
            Class { type_id: "1".into(), type_name: "电影".into(), type_flag: None, filters: None },
            Class { type_id: "2".into(), type_name: "连续剧".into(), type_flag: None, filters: None },
        ];

        Ok(ApiResult { class: Some(classes), list: Some(list), ..Default::default() })
    }

    async fn category(&self, _site: &Site, tid: &str, pg: i32, _filters: &[(&str, &str)]) -> Result<ApiResult> {
        let url = if tid == "1" {
            format!("{}/html/gndy/dyzz/list_1_{}.html", Self::base_url(), pg)
        } else {
            format!("{}/html/gndy/oumei/list_2_{}.html", Self::base_url(), pg)
        };
        let html = self.fetch_page(&url).await?;
        let list = Self::parse_list_html(&html);
        Ok(ApiResult { class: None, list: Some(list), ..Default::default() })
    }

    async fn detail(&self, _site: &Site, ids: &[String]) -> Result<ApiResult> {
        let tom = ids.first().map(|s| s.as_str()).unwrap_or("");
        let url = format!("{}{}", Self::base_url(), tom);
        let html = self.fetch_page(&url).await?;

        use scraper::{Html, Selector};
        let doc = Html::parse_document(&html);

        let title_sel = Selector::parse("div.title_all h1").unwrap();
        let title = doc.select(&title_sel).next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let zoom_sel = Selector::parse("div#Zoom").unwrap();
        let content = doc.select(&zoom_sel).next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();

        let mut play_url = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("magnet:") || line.starts_with("ed2k:") {
                play_url = line.to_string();
                break;
            }
        }

        let vod = Vod {
            vod_id: tom.to_string(),
            vod_name: title,
            vod_content: Some(content),
            vod_play_from: Some("ygdy8".into()),
            vod_play_url: Some(format!("1${}", play_url)),
            ..Default::default()
        };

        Ok(ApiResult { class: None, list: Some(vec![vod]), ..Default::default() })
    }

    async fn play(&self, _site: &Site, _flag: &str, id: &str) -> Result<PlayInfo> {
        let mut headers = HashMap::new();
        headers.insert("Referer".into(), Self::base_url().into());

        Ok(PlayInfo {
            url: id.to_string(),
            headers,
            user_agent: None,
            referer: Some(Self::base_url().into()),
        })
    }

    async fn search(&self, _site: &Site, keyword: &str, _pg: i32) -> Result<ApiResult> {
        let url = format!("https://s.ygdy8.com/plus/s0.php?typeid=1&keyword={}", keyword);
        let html = self.fetch_page(&url).await?;
        let list = Self::parse_list_html(&html);
        Ok(ApiResult { class: None, list: Some(list), ..Default::default() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rivu_core::models::Site;

    fn test_site() -> Site {
        Site {
            key: "YGP".into(), name: "YGP".into(), site_type: 3,
            api: "csp_YGPGuard".into(), jar: None, ext: None,
            searchable: None, quick_search: None, filterable: None,
            player_type: None, categories: None,
        }
    }

    #[tokio::test]
    async fn test_ygp_name() {
        let spider = YGPSpider::new();
        assert_eq!(spider.name(), "csp_YGPGuard");
    }

    #[tokio::test]
    async fn test_ygp_home_returns_list() {
        let spider = YGPSpider::new();
        let result = spider.home(&test_site()).await;
        assert!(result.is_ok());
    }
}
