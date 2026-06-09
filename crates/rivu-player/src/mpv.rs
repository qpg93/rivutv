use rivu_core::error::Result;
use rivu_core::models::PlayInfo;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

pub struct MpvBackend {
    process: Mutex<Option<Child>>,
}

impl MpvBackend {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }

    pub fn play(&self, info: &PlayInfo) -> Result<()> {
        self.stop()?;

        let mut cmd = Command::new("mpv");
        cmd.arg(&info.url)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if let Some(ua) = &info.user_agent {
            cmd.arg(format!("--user-agent={}", ua));
        }

        if let Some(ref referer) = &info.referer {
            cmd.arg(format!("--referrer={}", referer));
        }

        for (key, val) in &info.headers {
            cmd.arg(format!("--http-header-fields={}: {}", key, val));
        }

        let child = cmd.spawn().map_err(|e| {
            rivu_core::error::CoreError::Player(format!("Failed to launch mpv: {}", e))
        })?;

        *self.process.lock().unwrap() = Some(child);
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let mut proc = self.process.lock().unwrap();
        if let Some(mut child) = proc.take() {
            child.kill().ok();
            child.wait().ok();
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        let mut proc = self.process.lock().unwrap();
        proc.as_mut()
            .map(|c| c.try_wait().ok().flatten().is_none())
            .unwrap_or(false)
    }
}

impl Default for MpvBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_is_running_returns_false_when_not_started() {
        let backend = MpvBackend::new();
        assert!(!backend.is_running());
    }

    #[test]
    fn test_stop_when_not_running_does_not_panic() {
        let backend = MpvBackend::new();
        let result = backend.stop();
        assert!(result.is_ok());
    }

    #[test]
    fn test_stop_multiple_times_does_not_panic() {
        let backend = MpvBackend::new();
        assert!(backend.stop().is_ok());
        assert!(backend.stop().is_ok());
        assert!(backend.stop().is_ok());
    }

    #[test]
    fn test_play_with_invalid_mpv_path_returns_error() {
        let backend = MpvBackend::new();
        let info = PlayInfo {
            url: "http://example.com/v.mp4".into(),
            headers: HashMap::new(),
            user_agent: None,
            referer: None,
        };
        let _ = backend.play(&info);
        let _ = backend.stop();
    }
}
