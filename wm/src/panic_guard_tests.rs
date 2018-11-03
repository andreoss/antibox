    use super::*;

    #[test]
    fn test_record_writes_breadcrumb() {
        let dir = std::env::temp_dir().join("antibox-panic-guard-test");
        let path = dir.join("last-panic.log");
        record("KeyPress keycode=27", "panicked at foo.rs:1", &path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("event: KeyPress keycode=27"));
        assert!(text.contains("panicked at foo.rs:1"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_hook_captures_payload() {
        install_hook();
        let _ = std::panic::catch_unwind(|| panic!("guard-test-payload"));
        let last = take_last().expect("panic captured");
        assert!(last.contains("guard-test-payload"));
        assert!(take_last().is_none());
    }
