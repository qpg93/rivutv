pub struct MpvBackend;

impl MpvBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn play(&self, _url: &str) {
        // TODO: spawn mpv process
    }

    pub fn stop(&self) {
        // TODO: kill mpv process
    }
}
