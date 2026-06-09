use std::collections::HashMap;
use async_trait::async_trait;
use rivu_core::error::Result;
use rivu_core::models::{ApiResult, Class, PlayInfo, Site, Vod};
use serde_json::Value;
use crate::spider::Spider;

pub struct BiliSpider {
    client: reqwest::Client,
}

impl BiliSpider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .build()
                .expect("Failed to build reqwest client"),
        }
    }
}

impl Default for BiliSpider {
    fn default() -> Self {
        Self::new()
    }
}

impl BiliSpider {
    fn tid_to_rid(tid: &str) -> &str {
        match tid {
            "1" => "1",   "2" => "3",   "3" => "4",
            "4" => "5",   "5" => "11",  "6" => "21",
            "7" => "23",  "8" => "24",
            _ => tid,
        }
    }

    fn parse_video(item: &Value) -> Vod {
        Vod {
            vod_id: item["aid"].as_i64().unwrap_or(0).to_string(),
            vod_name: item["title"].as_str().unwrap_or("").to_string(),
            vod_pic: Some(item["pic"].as_str().unwrap_or("").to_string()),
            vod_remarks: Some(format!(
                "播放:{} 弹幕:{}",
                item["stat"]["view"].as_i64().unwrap_or(0),
                item["stat"]["danmaku"].as_i64().unwrap_or(0),
            )),
            vod_actor: Some(item["owner"]["name"].as_str().unwrap_or("").to_string()),
            ..Default::default()
        }
    }
}

#[async_trait]
impl Spider for BiliSpider {
    fn name(&self) -> &str { "csp_BiliGuard" }

    async fn home(&self, _site: &Site) -> Result<ApiResult> {
        let classes = vec![
            Class { type_id: "1".into(), type_name: "动画".into(), type_flag: None, filters: None },
            Class { type_id: "2".into(), type_name: "音乐".into(), type_flag: None, filters: None },
            Class { type_id: "3".into(), type_name: "游戏".into(), type_flag: None, filters: None },
            Class { type_id: "4".into(), type_name: "知识".into(), type_flag: None, filters: None },
            Class { type_id: "5".into(), type_name: "影视".into(), type_flag: None, filters: None },
            Class { type_id: "6".into(), type_name: "纪录片".into(), type_flag: None, filters: None },
            Class { type_id: "7".into(), type_name: "电影".into(), type_flag: None, filters: None },
            Class { type_id: "8".into(), type_name: "电视剧".into(), type_flag: None, filters: None },
        ];

        let resp = self.client.get("https://api.bilibili.com/x/web-interface/popular")
            .send().await?;
        let body: Value = resp.json().await?;

        let list: Vec<Vod> = body["data"]["list"].as_array()
            .map(|arr| arr.iter().map(Self::parse_video).collect())
            .unwrap_or_default();

        Ok(ApiResult { class: Some(classes), list: Some(list), ..Default::default() })
    }

    async fn category(&self, _site: &Site, tid: &str, pg: i32, _filters: &[(&str, &str)]) -> Result<ApiResult> {
        let rid = Self::tid_to_rid(tid);
        let url = format!("https://api.bilibili.com/x/web-interface/newlist?rid={}&pn={}", rid, pg);
        let resp = self.client.get(&url).send().await?;
        let body: Value = resp.json().await?;

        let list: Vec<Vod> = body["data"]["archives"].as_array()
            .map(|arr| arr.iter().map(Self::parse_video).collect())
            .unwrap_or_default();

        Ok(ApiResult { class: None, list: Some(list), ..Default::default() })
    }

    async fn detail(&self, _site: &Site, ids: &[String]) -> Result<ApiResult> {
        let aid = ids.first().map(|s| s.as_str()).unwrap_or("");
        let url = format!("https://api.bilibili.com/x/web-interface/view?aid={}", aid);
        let resp = self.client.get(&url).send().await?;
        let body: Value = resp.json().await?;
        let data = &body["data"];

        let cid = data["cid"].as_i64().unwrap_or(0);
        let vod = Vod {
            vod_id: aid.to_string(),
            vod_name: data["title"].as_str().unwrap_or("").to_string(),
            vod_pic: Some(data["pic"].as_str().unwrap_or("").to_string()),
            vod_content: Some(data["desc"].as_str().unwrap_or("").to_string()),
            vod_play_from: Some("Bili".into()),
            vod_play_url: Some(format!("1${}_{}", aid, cid)),
            ..Default::default()
        };

        Ok(ApiResult { class: None, list: Some(vec![vod]), ..Default::default() })
    }

    async fn play(&self, _site: &Site, _flag: &str, id: &str) -> Result<PlayInfo> {
        let parts: Vec<&str> = id.split('_').collect();
        let aid = parts.first().unwrap_or(&"");
        let cid = parts.get(1).unwrap_or(&"0");
        let url = format!("https://api.bilibili.com/x/player/playurl?avid={}&cid={}&qn=80", aid, cid);
        let resp = self.client.get(&url).send().await?;
        let body: Value = resp.json().await?;

        let play_url = body["data"]["durl"][0]["url"]
            .as_str().unwrap_or("").to_string();

        let mut headers = HashMap::new();
        headers.insert("Referer".into(), "https://www.bilibili.com".into());

        Ok(PlayInfo {
            url: play_url,
            headers,
            user_agent: Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".into()),
            referer: Some("https://www.bilibili.com".into()),
        })
    }

    async fn search(&self, _site: &Site, keyword: &str, pg: i32) -> Result<ApiResult> {
        let resp = self.client
            .get("https://api.bilibili.com/x/web-interface/search/all/v2")
            .query(&[("keyword", keyword), ("page", &pg.to_string())])
            .send().await?;
        let body: Value = resp.json().await?;

        let list: Vec<Vod> = body["data"]["result"][0]["data"].as_array()
            .map(|arr| arr.iter().map(|item| Vod {
                vod_id: item["aid"].as_i64().unwrap_or(0).to_string(),
                vod_name: item["title"].as_str().unwrap_or("").to_string(),
                vod_pic: Some(item["pic"].as_str().unwrap_or("").to_string()),
                ..Default::default()
            }).collect())
            .unwrap_or_default();

        Ok(ApiResult { class: None, list: Some(list), ..Default::default() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rivu_core::models::Site;

    fn test_site() -> Site {
        Site {
            key: "Bili".into(), name: "Bilibili".into(), site_type: 3,
            api: "csp_BiliGuard".into(), jar: None, ext: None,
            searchable: None, quick_search: None, filterable: None,
            player_type: None, categories: None,
        }
    }

    #[tokio::test]
    async fn test_bili_name() {
        let spider = BiliSpider::new();
        assert_eq!(spider.name(), "csp_BiliGuard");
    }

    #[tokio::test]
    async fn test_bili_home_returns_data() {
        let spider = BiliSpider::new();
        let result = spider.home(&test_site()).await;
        assert!(result.is_ok());
        let api_result = result.unwrap();
        assert!(api_result.class.is_some());
        assert!(!api_result.class.as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_bili_category_returns_list() {
        let spider = BiliSpider::new();
        let result = spider.category(&test_site(), "1", 1, &[]).await;
        assert!(result.is_ok());
    }
}
