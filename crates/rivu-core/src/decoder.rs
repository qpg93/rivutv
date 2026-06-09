use crate::error::{CoreError, Result};

pub struct SourceDecoder;

const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
const BMP_MAGIC: &[u8] = &[0x42, 0x4D];

impl SourceDecoder {
    /// Decode source bytes into a JSON string.
    ///
    /// Handles formats:
    /// - Plain JSON (with optional `//` comments)
    /// - Base64-encoded JSON
    /// - JPEG/BMP image with appended base64 data
    pub fn decode(data: &[u8]) -> Result<String> {
        // 1. Try plain text with comment stripping
        if let Some(text) = try_as_utf8_json(data) {
            return Ok(text);
        }

        // 2. Check for image-embedded base64 (JPEG/BMP)
        if data.starts_with(JPEG_MAGIC) || data.starts_with(BMP_MAGIC) {
            if let Some(b64) = extract_appended_base64(data) {
                let decoded = decode_base64(&b64)?;
                if let Some(json) = try_clean_json(&decoded) {
                    return Ok(json);
                }
            }
        }

        // 3. Try as direct base64
        if let Ok(text) = std::str::from_utf8(data) {
            if looks_like_base64(text) {
                let decoded = decode_base64(text)?;
                if let Some(json) = try_clean_json(&decoded) {
                    return Ok(json);
                }
            }
        }

        // 4. Fallback: strip comments from whatever text we have
        if let Ok(text) = std::str::from_utf8(data) {
            let cleaned = strip_json_comments(text);
            if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
                return Ok(cleaned);
            }
            return Err(CoreError::Config(format!(
                "JSON parse error after decoding: {}",
                cleaned.chars().take(100).collect::<String>()
            )));
        }

        Err(CoreError::Config(
            "source data is not valid UTF-8 or recognized format".into(),
        ))
    }
}

fn try_as_utf8_json(data: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(data).ok()?;
    let cleaned = strip_json_comments(text);
    serde_json::from_str::<serde_json::Value>(&cleaned).ok()?;
    Some(cleaned)
}

fn try_clean_json(text: &str) -> Option<String> {
    let cleaned = strip_json_comments(text);
    serde_json::from_str::<serde_json::Value>(&cleaned).ok()?;
    Some(cleaned)
}

/// Extract base64 data appended after image bytes.
///
/// Works at byte level since image data (0xFF etc.) is invalid UTF-8.
/// Scans for the first long base64-like ASCII run in the trailing portion.
fn extract_appended_base64(data: &[u8]) -> Option<String> {
    // Scan for base64-encoded JSON start markers at byte level.
    // "eyJ" = base64 of `{"`, "ew0K" = base64 of `{\r\n`, "ewogIC" = base64 of `{\n  `.
    let start_markers: &[&[u8]] = &[b"eyJ", b"ew0K", b"ewogIC"];

    let start = start_markers
        .iter()
        .filter_map(|m| {
            // Search after the first 100 bytes (past the image header)
            data.windows(m.len())
                .position(|w| w == *m)
        })
        .min()?;

    // From the start position, collect continuous base64 characters
    let b64_bytes: Vec<u8> = data[start..]
        .iter()
        .take_while(|&&b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=')
        .copied()
        .collect();

    if b64_bytes.len() > 50 {
        String::from_utf8(b64_bytes).ok()
    } else {
        None
    }
}

fn decode_base64(input: &str) -> Result<String> {
    // Add padding if needed
    let input = input.trim();
    let mut b64 = input.to_string();
    let rem = b64.len() % 4;
    if rem != 0 {
        b64.push_str(&"=".repeat(4 - rem));
    }

    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    let bytes = engine
        .decode(&b64)
        .map_err(|e| CoreError::Config(format!("base64 decode error: {e}")))?;
    String::from_utf8(bytes).map_err(|e| CoreError::Config(format!("UTF-8 decode error: {e}")))
}

/// Heuristic: check if text looks like base64 (mostly alphanumeric + / + =)
fn looks_like_base64(text: &str) -> bool {
    let total = text.len();
    if total < 10 {
        return false;
    }
    let b64_chars = text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=')
        .count();
    b64_chars as f64 / total as f64 > 0.85
}

/// Strip JavaScript-style comments from JSON text.
///
/// Handles:
/// - Single-line `//` comments
/// - Multi-line `/* */` comments
/// - Respects string boundaries (no stripping inside "...")
pub fn strip_json_comments(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = json.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if ch == '\\' {
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                }
            } else if ch == '"' {
                in_string = false;
            }
        } else if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                out.push(ch);
            }
        } else if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
        } else if ch == '/' {
            match chars.peek() {
                Some(&'/') => {
                    chars.next();
                    in_line_comment = true;
                }
                Some(&'*') => {
                    chars.next();
                    in_block_comment = true;
                }
                _ => {
                    out.push(ch);
                }
            }
        } else {
            if ch == '"' {
                in_string = true;
            }
            out.push(ch);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_json() {
        let json = r#"{"sites":[{"key":"k","name":"N","type":0,"api":"http://a.com"}]}"#;
        let result = SourceDecoder::decode(json.as_bytes()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["key"], "k");
    }

    #[test]
    fn test_json_with_line_comments() {
        let json = "{\n// site list\n\"sites\":[{\"key\":\"k\",\"name\":\"N\",\"type\":0,\"api\":\"http://a.com\"}]}";
        let result = SourceDecoder::decode(json.as_bytes()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["key"], "k");
    }

    #[test]
    fn test_json_with_block_comments() {
        let json = "{\n/* site list */\n\"sites\":[{\"key\":\"k\",\"name\":\"N\",\"type\":0,\"api\":\"http://a.com\"}]}";
        let result = SourceDecoder::decode(json.as_bytes()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["key"], "k");
    }

    #[test]
    fn test_json_with_inline_comment() {
        let json = r#"{"sites":[{"key":"k","name":"N","type":0,"api":"http://a.com"}]}// end"#;
        let result = SourceDecoder::decode(json.as_bytes()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["api"], "http://a.com");
    }

    #[test]
    fn test_base64_encoded_json() {
        let json = r#"{"sites":[{"key":"k","name":"N","type":0,"api":"http://a.com"}]}"#;
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        let b64 = engine.encode(json);
        let result = SourceDecoder::decode(b64.as_bytes()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["key"], "k");
    }

    #[test]
    fn test_base64_json_with_comments() {
        let json = "{\n// comment\n\"sites\":[{\"key\":\"k\",\"name\":\"N\",\"type\":0,\"api\":\"http://a.com\"}]}";
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        let b64 = engine.encode(json);
        let result = SourceDecoder::decode(b64.as_bytes()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["name"], "N");
    }

    #[test]
    fn test_jpeg_with_appended_base64() {
        // Construct: JPEG header + base64 of a valid JSON
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0];
        let json = r#"{"sites":[{"key":"k","name":"N","type":0,"api":"http://a.com"}],"lives":[],"parses":[]}"#;
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        let b64 = engine.encode(json);

        let mut data = jpeg_header;
        // Pad with enough noise to simulate image data
        data.extend(std::iter::repeat(0x00).take(200));
        data.extend(b64.as_bytes());

        let result = SourceDecoder::decode(&data).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["key"], "k");
    }

    #[test]
    fn test_bmp_with_appended_base64() {
        let bmp_header = vec![0x42, 0x4D];
        let json = r#"{"sites":[{"key":"k","name":"N","type":1,"api":"http://b.com"}]}"#;
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        let b64 = engine.encode(json);

        let mut data = bmp_header;
        data.extend(std::iter::repeat(0xFF).take(100));
        data.extend(b64.as_bytes());

        let result = SourceDecoder::decode(&data).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["api"], "http://b.com");
    }

    #[test]
    fn test_strip_comment_does_not_affect_strings() {
        let input = r#"{"url": "http://example.com//path"}"#;
        let result = strip_json_comments(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_strip_block_comment_in_string() {
        let input = r#"{"comment": "/* not a comment */"}"#;
        let result = strip_json_comments(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_invalid_utf8_returns_error() {
        let data = vec![0xFF, 0xFE, 0x00, 0x01];
        let result = SourceDecoder::decode(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let result = SourceDecoder::decode(b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_looks_like_base64() {
        assert!(looks_like_base64("SGVsbG8gV29ybGQ="));
        assert!(looks_like_base64("eyJ0ZXN0IjogImRhdGEifQ=="));
        assert!(!looks_like_base64("Hello, 世界!"));
        assert!(!looks_like_base64(""));
    }

    #[test]
    fn test_decode_strips_padding_issue() {
        // Base64 without padding
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        let json = r#"{"key":"value"}"#;
        let b64 = engine.encode(json);
        let no_pad = b64.trim_end_matches('=');
        let result = SourceDecoder::decode(no_pad.as_bytes()).unwrap();
        assert_eq!(result, json);
    }

    #[test]
    fn test_real_style_source_with_all_features() {
        let json = "{\n\"spider\":\"http://example.com/spider.jar\",\n\"wallpaper\":\"http://example.com/wall.jpg\",\n\"sites\":[\n{\"key\":\"k1\",\"name\":\"Source One\",\"type\":3,\"api\":\"csp_Test\"}\n],\n// some live\n\"lives\":[\n{\"name\":\"CCTV1\",\"url\":\"http://live.com/cctv1\"}\n],\n\"parses\":[],\n\"rules\":[]\n}";
        let result = SourceDecoder::decode(json.as_bytes()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["sites"][0]["name"], "Source One");
        assert_eq!(v["lives"][0]["name"], "CCTV1");
    }

    #[test]
    fn test_plain_json_no_trailing_garbage() {
        let json = r#"{"sites":[{"key":"k","name":"N","type":0,"api":"http://a.com"}]}"#;
        let result = SourceDecoder::decode(json.as_bytes()).unwrap();
        // Parse back and check no extra fields added
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("sites"));
    }
}
