pub struct MediaSource {
    pub name: String,
    pub url: String,
    pub api: String,
}

pub struct MediaItem {
    pub title: String,
    pub url: String,
    pub category: String,
    pub source: String,
}

pub struct PlaybackInfo {
    pub url: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
}
