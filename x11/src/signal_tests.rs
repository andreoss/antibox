    use super::*;

    #[test]
    fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        assert!(handler.is_ok());
    }

    #[test]
    fn test_signal_handler_fd() {
        let handler = SignalHandler::new().unwrap();
        assert!(handler.read_fd() >= 0);
        assert!(!handler.was_signalled());
    }
