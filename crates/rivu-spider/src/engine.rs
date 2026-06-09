pub struct SpiderEngine;

impl SpiderEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn fetch(&self, _url: &str) {
        // TODO: implement TVBox API fetching
    }
}

impl Default for SpiderEngine {
    fn default() -> Self {
        Self::new()
    }
}
