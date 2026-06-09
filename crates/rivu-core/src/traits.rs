use crate::models::{MediaItem, PlaybackInfo};

pub trait Spider: Send + Sync {
    fn name(&self) -> &str;
    fn fetch_list(&self, url: &str) -> Vec<MediaItem>;
    fn resolve(&self, url: &str) -> PlaybackInfo;
}

pub trait Player: Send + Sync {
    fn name(&self) -> &str;
    fn play(&self, info: &PlaybackInfo);
    fn stop(&self);
}
