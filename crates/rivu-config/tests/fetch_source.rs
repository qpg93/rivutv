use rivu_core::decoder::SourceDecoder;
use rivu_core::models::SourceConfig;

fn trunc(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max])
    } else {
        s.to_string()
    }
}

fn ext_val(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "null".into(),
        serde_json::Value::String(s) if s.is_empty() => "empty".into(),
        serde_json::Value::String(s) => trunc(s, 100),
        other => trunc(&other.to_string(), 100),
    }
}

#[tokio::test]
async fn fetch_and_print_fantaiying_sites() {
    let client = reqwest::Client::new();
    let url = "http://www.饭太硬.cc/tv";
    let resp = client.get(url).send().await.unwrap();
    let bytes = resp.bytes().await.unwrap();
    let decoded = SourceDecoder::decode(&bytes).unwrap();
    let config: SourceConfig = serde_json::from_str(&decoded).unwrap();

    for (i, site) in config.sites.iter().enumerate() {
        let api_field = trunc(&site.api, 100);
        let jar_field = site.jar.as_deref().map(|j| trunc(j, 100)).unwrap_or_default();
        let ext_field = site.ext.as_ref().map(ext_val).unwrap_or_default();
        println!(
            r#"{idx}: type={type} name="{name}" api="{api}" jar="{jar}" ext_preview="{ext}""#,
            idx = i,
            type = site.site_type,
            name = site.name,
            api = api_field,
            jar = jar_field,
            ext = ext_field,
        );
    }
    println!("Total sites: {}", config.sites.len());
}
