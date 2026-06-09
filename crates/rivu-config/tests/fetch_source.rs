use base64::Engine as _;
use rivu_core::decoder::SourceDecoder;
use rivu_core::models::SourceConfig;

fn looks_like_base64(s: &str) -> bool {
    let clean: String = s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=').collect();
    clean.len() > 10 && (clean.len() as f64 / s.len() as f64) > 0.85
}

fn analyze_ext(ext: &serde_json::Value) {
    match ext {
        serde_json::Value::Null => {
            println!("    ext → null");
        }
        serde_json::Value::String(s) => {
            if s.is_empty() {
                println!("    ext → (empty string)");
            } else if looks_like_base64(s) {
                // Try to decode and then parse as JSON
                let engine = base64::engine::general_purpose::STANDARD;
                match engine.decode(s) {
                    Ok(decoded) => {
                        let dlen = decoded.len();
                        if let Ok(text) = String::from_utf8(decoded) {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                println!("    ext → base64 JSON:");
                                for line in serde_json::to_string_pretty(&parsed).unwrap().lines() {
                                    println!("      {}", line);
                                }
                            } else {
                                println!("    ext → base64, decoded to non-JSON text (len={}):", text.len());
                                println!("      {}", text.chars().take(200).collect::<String>());
                            }
                        } else {
                            println!("    ext → base64 (decoded bytes not UTF-8, len={})", dlen);
                        }
                    }
                    Err(_) => {
                        println!("    ext → probably base64 (but decode failed) preview: {}", s.chars().take(80).collect::<String>());
                    }
                }
            } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s) {
                println!("    ext → JSON string:");
                for line in serde_json::to_string_pretty(&parsed).unwrap().lines() {
                    println!("      {}", line);
                }
            } else {
                println!("    ext → plain string (len={}): {}", s.len(), s.chars().take(200).collect::<String>());
            }
        }
        serde_json::Value::Object(m) => {
            println!("    ext → JSON object ({} keys):", m.len());
            for line in serde_json::to_string_pretty(ext).unwrap().lines() {
                println!("      {}", line);
            }
        }
        serde_json::Value::Array(a) => {
            println!("    ext → JSON array ({} items):", a.len());
            for line in serde_json::to_string_pretty(ext).unwrap().lines() {
                println!("      {}", line);
            }
        }
        other => {
            println!("    ext → {}: {}", serde_json::to_string(other).unwrap().chars().take(100).collect::<String>(), other);
        }
    }
}

#[tokio::test]
async fn fetch_and_print_fantaiying_sites() {
    let client = reqwest::Client::new();
    let url = "http://www.饭太硬.cc/tv";
    let resp = client.get(url).send().await.unwrap();
    let bytes = resp.bytes().await.unwrap();
    println!("Fetched {} bytes from {}", bytes.len(), url);
    let decoded = SourceDecoder::decode(&bytes).unwrap();
    let config: SourceConfig = serde_json::from_str(&decoded).unwrap();

    let type3: Vec<&rivu_core::models::Site> = config.sites.iter().filter(|s| s.site_type == 3).collect();
    println!("\n=== Type-3 sites (spider): {} total ===\n", type3.len());

    for (i, site) in type3.iter().enumerate() {
        println!("[{}/{}] key=\"{}\" name=\"{}\" api=\"{}\"", i, type3.len(), site.key, site.name, site.api);
        println!("    jar: {:?}", site.jar);
        if let Some(ref ext) = site.ext {
            analyze_ext(ext);
        } else {
            println!("    ext → None");
        }
        println!();
    }

    println!("Total sites in config: {} (type=3: {})", config.sites.len(), type3.len());
}
