use rivu_core::decoder::SourceDecoder;
use rivu_core::models::*;
use rivu_spider::extractor::SourceExtractor;
use rivu_spider::parsers::Parser;
use std::collections::HashMap;

#[test]
fn test_full_parse_pipeline() {
    let json = r#"{
        "class": [
            {"type_id": "1", "type_name": "Movie"},
            {"type_id": "2", "type_name": "TV Series"}
        ],
        "list": [
            {"vod_id": "100", "vod_name": "Test A", "vod_pic": "http://a.jpg", "vod_remarks": "HD"},
            {"vod_id": "101", "vod_name": "Test B", "vod_pic": "http://b.jpg", "vod_remarks": "4K"}
        ],
        "filters": {
            "1": [{"key": "area", "name": "Region", "value": [{"v": "", "n": "All"}]}]
        }
    }"#;

    let result = Parser::parse_json(json).unwrap();
    assert_eq!(result.class.as_ref().unwrap().len(), 2);
    assert_eq!(result.list.as_ref().unwrap().len(), 2);
    assert_eq!(result.list.as_ref().unwrap()[0].vod_name, "Test A");
}

#[test]
fn test_parse_site_config() {
    let json = r#"{
        "sites": [
            {"key": "site1", "name": "Source 1", "type": 0, "api": "http://api1.com"}
        ]
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.sites.len(), 1);
    assert_eq!(config.sites[0].site_type, 0);
}

#[test]
fn test_parse_episodes_from_play_url() {
    let play_from = "ck$$$LianMeng";
    let play_url = "1$http://a.com/1.mp4#2$http://a.com/2.mp4$$$1$http://b.com/1.mp4";
    let flags = Flag::parse_flags(play_from, play_url);
    assert_eq!(flags.len(), 2);
    assert_eq!(flags[0].name, "ck");
    assert_eq!(flags[0].episodes.len(), 2);
    assert_eq!(flags[1].episodes.len(), 1);
}

#[test]
fn test_parse_then_extract_video_prefix() {
    let json = r#"{"url": "video://http://real.com/play.m3u8", "flag": "ck"}"#;
    let result = Parser::parse_json(json).unwrap();
    let play_info = PlayInfo {
        url: result.url.unwrap_or_default(),
        headers: HashMap::new(),
        user_agent: None,
        referer: None,
    };
    let ext = SourceExtractor::new();
    let resolved = ext.extract(&play_info).unwrap();
    assert_eq!(resolved.url, "http://real.com/play.m3u8");
}

#[test]
fn test_empty_source_config_does_not_panic() {
    let json = "{}";
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    assert!(config.sites.is_empty());
    assert!(config.lives.is_none());
}

#[test]
fn test_vod_with_all_empty_strings() {
    let json = r#"{
        "vod_id": "", "vod_name": "",
        "vod_pic": "", "vod_remarks": "",
        "vod_play_from": "", "vod_play_url": ""
    }"#;
    let vod: Vod = serde_json::from_str(json).unwrap();
    assert_eq!(vod.vod_id, "");
    assert_eq!(vod.vod_play_url.as_deref(), Some(""));
}

#[test]
fn test_source_config_with_full_lives_config() {
    let json = r#"{
        "sites": [],
        "lives": [{
            "name": "CCTV", "url": "http://live.com/cctv.m3u8",
            "epg": "http://epg.com", "ua": "Mozilla/5.0",
            "origin": "http://origin.com", "referer": "http://ref.com",
            "header": {"User-Agent": "test"}
        }]
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    let lives = config.lives.unwrap();
    assert_eq!(lives[0].name, "CCTV");
    assert_eq!(lives[0].epg.as_deref(), Some("http://epg.com"));
}

#[test]
fn test_source_config_with_parse_definitions() {
    let json = r#"{
        "sites": [],
        "parses": [
            {"name": "Parse1", "type": 1, "url": "http://parse1.com/api?url="},
            {"name": "Parse2", "type": 0, "url": "http://parse2.com/?url=",
             "ext": {"flag": "1"}, "header": {"Referer": "http://ref.com"}}
        ]
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    let parses = config.parses.unwrap();
    assert_eq!(parses.len(), 2);
    assert_eq!(parses[0].parse_type, 1);
    assert_eq!(parses[1].ext.as_ref().unwrap().get("flag").unwrap(), "1");
}

#[test]
fn test_flag_parse_single_episode() {
    let flags = Flag::parse_flags("CK", "1$http://a.mp4");
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].episodes.len(), 1);
    assert_eq!(flags[0].episodes[0].name, "1");
}

#[test]
fn test_flag_parse_three_flags() {
    let flags = Flag::parse_flags("A$$$B$$$C", "1$u1$$$1$u2$$$1$u3");
    assert_eq!(flags.len(), 3);
    assert_eq!(flags[0].episodes[0].url, "u1");
    assert_eq!(flags[2].episodes[0].url, "u3");
}

#[test]
fn test_flag_parse_empty_episode_list_for_flag() {
    let flags = Flag::parse_flags("A$$$B", "1$u1$$$");
    assert_eq!(flags.len(), 2);
    assert!(flags[1].episodes.is_empty());
}

#[test]
fn test_play_info_with_all_fields() {
    let mut headers = HashMap::new();
    headers.insert("Referer".into(), "http://ref.com".into());
    let info = PlayInfo {
        url: "http://stream.com/video.m3u8".into(),
        headers,
        user_agent: Some("Mozilla/5.0".into()),
        referer: Some("http://ref.com".into()),
    };
    assert!(info.url.starts_with("http"));
    assert_eq!(info.headers.len(), 1);
}

#[test]
fn test_real_world_tvbox_config_structure() {
    let json = r#"{
        "sites": [
            {"key": "douban", "name": "豆瓣", "type": 0, "api": "http://douban.api.com/video"},
            {"key": "custom", "name": "My Source", "type": 3, "api": "csp_Myspider", "jar": "http://jar.com/spider.jar", "ext": "{}"}
        ],
        "lives": [{"name": "CCTV", "url": "http://live.com/cctv.m3u8"}],
        "parses": [{"name": "JsonParse", "type": 1, "url": "http://parse.com/?url="}],
        "flags": ["4k"],
        "spider": "http://spider.com/spider.jar"
    }"#;
    let config: SourceConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.sites.len(), 2);
    assert_eq!(config.sites[1].site_type, 3);
    assert_eq!(config.sites[1].jar.as_deref(), Some("http://jar.com/spider.jar"));
    assert!(config.flags.is_some());
}

#[test]
fn test_decoder_full_pipeline_jpeg_embedded() {
    // Pre-computed base64 of: {"sites":[{"key":"k","name":"N","type":0,"api":"http://a.com"}]}
    let b64 = "eyJzaXRlcyI6W3sia2V5IjoiayIsIm5hbWUiOiJOIiwidHlwZSI6MCwiYXBpIjoiaHR0cDovL2EuY29tIn1dfQ==";

    let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    data.extend(std::iter::repeat(0x00).take(256));
    data.extend(b64.as_bytes());

    let decoded = SourceDecoder::decode(&data).unwrap();
    let config: SourceConfig = serde_json::from_str(&decoded).unwrap();
    assert_eq!(config.sites[0].key, "k");
    assert_eq!(config.sites[0].api, "http://a.com");
}

#[test]
fn test_decoder_base64_direct() {
    // Pre-computed base64 of: {"sites":[{"key":"k","name":"N","type":0,"api":"http://a.com"}]}
    let b64 = "eyJzaXRlcyI6W3sia2V5IjoiayIsIm5hbWUiOiJOIiwidHlwZSI6MCwiYXBpIjoiaHR0cDovL2EuY29tIn1dfQ==";
    let decoded = SourceDecoder::decode(b64.as_bytes()).unwrap();
    let config: SourceConfig = serde_json::from_str(&decoded).unwrap();
    assert_eq!(config.sites[0].key, "k");
}

#[test]
fn test_decoder_json_with_comments_to_source_config() {
    let json = "{\"sites\":[{\"key\":\"k\",\"name\":\"N\",\"type\":0,\"api\":\"http://a.com\"}],\"lives\":[],\"parses\":[]}// trailing comment";
    let decoded = SourceDecoder::decode(json.as_bytes()).unwrap();
    let config: SourceConfig = serde_json::from_str(&decoded).unwrap();
    assert_eq!(config.sites[0].key, "k");
}

#[test]
fn test_decoder_bmp_embedded() {
    let b64 = "eyJzaXRlcyI6W3sia2V5IjoiayIsIm5hbWUiOiJOIiwidHlwZSI6MCwiYXBpIjoiaHR0cDovL2EuY29tIn1dfQ==";
    let mut data = vec![0x42, 0x4D];
    data.extend(std::iter::repeat(0xFF).take(128));
    data.extend(b64.as_bytes());

    let decoded = SourceDecoder::decode(&data).unwrap();
    let config: SourceConfig = serde_json::from_str(&decoded).unwrap();
    assert_eq!(config.sites[0].name, "N");
}

#[test]
fn test_decoder_complex_source_with_comments() {
    let json = "{\n\"spider\":\"http://spider.jar\",\n\"sites\":[\n{\"key\":\"k1\",\"name\":\"S1\",\"type\":3,\"api\":\"csp_T\"}\n],\n// live section\n\"lives\":[\n{\"name\":\"CCTV\",\"url\":\"http://live.tv\"}\n],\n\"parses\":[]\n}";
    let decoded = SourceDecoder::decode(json.as_bytes()).unwrap();
    let config: SourceConfig = serde_json::from_str(&decoded).unwrap();
    assert_eq!(config.sites.len(), 1);
    assert_eq!(config.sites[0].name, "S1");
    assert!(config.lives.is_some());
}
