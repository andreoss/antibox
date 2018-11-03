    use super::*;

    #[test]
    fn test_ucs_to_keysym_latin() {
        assert_eq!(ucs_to_keysym(0x0101), 0x03e0);
    }

    #[test]
    fn test_ucs_to_keysym_cyrillic() {
        assert_eq!(ucs_to_keysym(0x0430), 0x06c1);
    }

