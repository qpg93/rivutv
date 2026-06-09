use rivu_core::error::Result;
use rivu_core::models::{ApiResult, Vod};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct XmlVideo {
    id: Option<String>,
    name: Option<String>,
    pic: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XmlList {
    #[serde(default)]
    video: Vec<XmlVideo>,
}

pub struct Parser;

impl Parser {
    pub fn parse_json(data: &str) -> Result<ApiResult> {
        let result: ApiResult = serde_json::from_str(data)?;
        Ok(result)
    }

    pub fn parse_xml(data: &str) -> Result<ApiResult> {
        let mut result = ApiResult::default();

        if let Ok(list) = quick_xml::de::from_str::<XmlList>(data) {
            let vods: Vec<Vod> = list.video.into_iter().map(|v| Vod {
                vod_id: v.id.unwrap_or_default(),
                vod_name: v.name.unwrap_or_default(),
                vod_pic: v.pic,
                vod_remarks: v.note,
                ..Default::default()
            }).collect();
            result.list = Some(vods);
        } else {
            result.list = Some(Vec::new());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_response() {
        let json = r#"{
            "class": [{"type_id": "1", "type_name": "Movie"}],
            "list": [{"vod_id": "123", "vod_name": "Test Movie", "vod_pic": "http://example.com/pic.jpg"}]
        }"#;
        let result = Parser::parse_json(json).unwrap();
        assert!(result.class.is_some());
        assert!(result.list.is_some());
        assert_eq!(result.list.unwrap()[0].vod_id, "123");
    }

    #[test]
    fn test_parse_json_empty() {
        let json = r#"{"class":[],"list":[],"page":1,"pagecount":1,"limit":20,"total":0}"#;
        let result = Parser::parse_json(json).unwrap();
        assert!(result.list.unwrap().is_empty());
    }

    #[test]
    fn test_parse_json_with_filters() {
        let json = r#"{
            "class": [{"type_id":"1","type_name":"Movie"}],
            "filters": {
                "1": [
                    {"key":"area","name":"Region","value":[{"v":"","n":"All"},{"v":"CN","n":"China"}]}
                ]
            },
            "list": []
        }"#;
        let result = Parser::parse_json(json).unwrap();
        let filters = result.filters.unwrap();
        let area_filters = filters.get("1").unwrap();
        assert_eq!(area_filters[0].key, "area");
        assert_eq!(area_filters[0].value[1].n, "China");
    }

    #[test]
    fn test_parse_json_malformed_returns_error() {
        let result = Parser::parse_json("this is not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_null_values_accepted() {
        let json = r#"{"class":null,"list":null,"page":null,"total":null}"#;
        let result = Parser::parse_json(json).unwrap();
        assert!(result.class.is_none());
        assert!(result.list.is_none());
        assert!(result.page.is_none());
    }

    #[test]
    fn test_parse_xml_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <list>
            <video>
                <id>100</id>
                <name>Movie A</name>
                <pic>http://pic/a.jpg</pic>
                <note>HD</note>
            </video>
            <video>
                <id>101</id>
                <name>Movie B</name>
                <pic>http://pic/b.jpg</pic>
                <note>4K</note>
            </video>
        </list>"#;
        let result = Parser::parse_xml(xml).unwrap();
        let vods = result.list.unwrap();
        assert_eq!(vods.len(), 2);
        assert_eq!(vods[0].vod_id, "100");
        assert_eq!(vods[1].vod_name, "Movie B");
    }

    #[test]
    fn test_parse_xml_empty_returns_empty_list() {
        let xml = r#"<?xml version="1.0"?><list></list>"#;
        let result = Parser::parse_xml(xml).unwrap();
        assert!(result.list.unwrap().is_empty());
    }

    #[test]
    fn test_parse_xml_invalid_returns_empty_list() {
        let result = Parser::parse_xml("not xml").unwrap();
        assert!(result.list.unwrap().is_empty());
    }
}
