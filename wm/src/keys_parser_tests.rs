    use super::*;

    #[test]
    fn test_keysym_f1() {
        assert_eq!(keysym_from_name("F1"), Some(0xFFBE));
    }

    #[test]
    fn test_keysym_escape() {
        assert_eq!(keysym_from_name("Escape"), Some(0xFF1B));
        assert_eq!(keysym_from_name("Esc"), Some(0xFF1B));
    }

