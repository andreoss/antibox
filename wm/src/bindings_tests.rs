    use super::*;

    #[test]
    fn test_find_keycode_found() {
        assert_eq!(
            find_keycode(&[0x01, 0x02, 0xFF09, 0x04, 0x05], 1, 10, 0xFF09),
            Some(12)
        );
    }

    #[test]
    fn test_find_keycode_not_found() {
        assert_eq!(find_keycode(&[0x01, 0x02, 0x03], 1, 10, 0xFF09), None);
    }

