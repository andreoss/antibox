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


    #[test]
    fn test_tap_lookup_matches_either_side_key() {
        let mut b = KeyBindings::new();
        b.taps = vec![crate::keys_parser::TapEntry {
            keysyms: vec![0xFFEB, 0xFFEC],
            action: Action::Menu(MenuOp::RootMenu),
        }];
        assert_eq!(b.lookup_tap(0xFFEB), Some(&Action::Menu(MenuOp::RootMenu)));
        assert_eq!(b.lookup_tap(0xFFEC), Some(&Action::Menu(MenuOp::RootMenu)));
        assert_eq!(b.lookup_tap(0xFFE9), None);
        assert_eq!(KeyBindings::new().lookup_tap(0xFFEB), None);
    }
