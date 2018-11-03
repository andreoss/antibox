    use super::*;
    

    #[test]
    fn test_new() {
        let t = ToolTip::new();
        assert!(!t.visible);
        assert!(t.text.is_empty());
    }

    #[test]
    fn test_set_text() {
        let mut t = ToolTip::new();
        t.set_text("Hello");
        assert_eq!(t.text, "Hello");
    }

