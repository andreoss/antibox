    use super::*;

    fn event(name: &str) -> Vec<u8> {
        let mut padded = name.as_bytes().to_vec();
        padded.push(0);
        while padded.len() % 16 != 0 {
            padded.push(0);
        }
        let mut buf = Vec::new();
        buf.extend_from_slice(&1i32.to_ne_bytes());
        buf.extend_from_slice(&IN_CLOSE_WRITE.to_ne_bytes());
        buf.extend_from_slice(&0u32.to_ne_bytes());
        buf.extend_from_slice(&(padded.len() as u32).to_ne_bytes());
        buf.extend_from_slice(&padded);
        buf
    }

    #[test]
    fn parses_single_event_name() {
        let buf = event("config.ini");
        assert_eq!(event_names(&buf), vec!["config.ini".to_string()]);
    }

    #[test]
    fn parses_multiple_events() {
        let mut buf = event("other.txt");
        buf.extend_from_slice(&event("config.ini"));
        assert_eq!(
            event_names(&buf),
            vec!["other.txt".to_string(), "config.ini".to_string()]
        );
    }

    #[test]
    fn ignores_nameless_and_truncated() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1i32.to_ne_bytes());
        buf.extend_from_slice(&IN_MODIFY.to_ne_bytes());
        buf.extend_from_slice(&0u32.to_ne_bytes());
        buf.extend_from_slice(&0u32.to_ne_bytes());
        assert!(event_names(&buf).is_empty());
        assert!(event_names(&buf[..7]).is_empty());
    }

    #[test]
    fn take_changed_is_edge_triggered() {
        CHANGED.store(true, Ordering::Relaxed);
        assert!(take_changed());
        assert!(!take_changed());
    }
