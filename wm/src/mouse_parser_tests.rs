    use super::*;

    #[test]
    fn test_mouse_button_mapping() {
        assert_eq!(mouse_button_from_keysym(0x010014), Some(1));
        assert_eq!(mouse_button_from_keysym(0x010015), Some(2));
        assert_eq!(mouse_button_from_keysym(0x010016), Some(3));
        assert_eq!(mouse_button_from_keysym(0x010017), Some(4));
        assert_eq!(mouse_button_from_keysym(0x010018), Some(5));
        assert_eq!(mouse_button_from_keysym(0xFFBE), None);
    }

    #[test]
    fn test_modifier_conversion() {
        assert_eq!(modifiers_to_button_mask(0x04), 0x04);
        assert_eq!(modifiers_to_button_mask(0x08), 0x08);
        assert_eq!(modifiers_to_button_mask(0x01), 0x01);
        assert_eq!(modifiers_to_button_mask(0x40), 0x40);
        assert_eq!(modifiers_to_button_mask(0x04 | 0x08), 0x04 | 0x08);
    }
