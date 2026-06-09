use rivu_core::error::Result;
use rivu_core::models::PlayInfo;

pub struct SourceExtractor;

impl SourceExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, url: &str) -> String {
        let url = url.trim();
        if let Some(stripped) = url.strip_prefix("video://") {
            stripped.to_string()
        } else if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            url.to_string()
        }
    }

    pub fn extract(&self, info: &PlayInfo) -> Result<PlayInfo> {
        let url = self.resolve(&info.url);
        Ok(PlayInfo {
            url,
            headers: info.headers.clone(),
            user_agent: info.user_agent.clone(),
            referer: info.referer.clone(),
        })
    }
}

impl Default for SourceExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_strip_video_prefix() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("video://http://example.com/stream.m3u8"), "http://example.com/stream.m3u8");
        assert_eq!(ext.resolve("http://example.com/video.mp4"), "http://example.com/video.mp4");
        assert_eq!(ext.resolve("magnet:?xt=urn:btih:abc"), "magnet:?xt=urn:btih:abc");
    }

    #[test]
    fn test_strip_video_prefix_https() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("video://https://s.com/play.m3u8"), "https://s.com/play.m3u8");
    }

    #[test]
    fn test_resolve_https_passthrough() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("https://cdn.com/video.mp4"), "https://cdn.com/video.mp4");
    }

    #[test]
    fn test_resolve_ed2k_passthrough() {
        let ext = SourceExtractor::new();
        let ed2k = "ed2k://|file|movie.avi|1234567890|hash|/";
        assert_eq!(ext.resolve(ed2k), ed2k);
    }

    #[test]
    fn test_resolve_whitespace_trimmed() {
        let ext = SourceExtractor::new();
        assert_eq!(ext.resolve("  http://a.com/v.mp4  "), "http://a.com/v.mp4");
    }

    #[test]
    fn test_extract_preserves_headers() {
        let ext = SourceExtractor::new();
        let mut headers = HashMap::new();
        headers.insert("Referer".into(), "http://ref.com".into());
        let info = PlayInfo {
            url: "video://http://real.com/stream".into(),
            headers: headers.clone(),
            user_agent: Some("test-agent".into()),
            referer: Some("http://ref.com".into()),
        };
        let result = ext.extract(&info).unwrap();
        assert_eq!(result.url, "http://real.com/stream");
        assert_eq!(result.headers.get("Referer").unwrap(), "http://ref.com");
        assert_eq!(result.user_agent.as_deref(), Some("test-agent"));
    }

    #[test]
    fn test_extract_empty_url() {
        let ext = SourceExtractor::new();
        let info = PlayInfo {
            url: "".into(),
            headers: HashMap::new(),
            user_agent: None,
            referer: None,
        };
        let result = ext.extract(&info).unwrap();
        assert_eq!(result.url, "");
    }
}
