    use super::*;
    use std::thread;

    #[test]
    fn test_monotime() {
        let start = Monotime::now();
        thread::sleep(Duration::from_millis(10));
        let added = start.checked_add(Duration::from_secs(1));
        assert!(added.is_some());
    }
