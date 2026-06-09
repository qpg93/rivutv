pub use crate::mpv::MpvBackend;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpv_backend_can_be_created_via_backends_module() {
        let backend = MpvBackend::new();
        assert!(!backend.is_running());
    }
}
