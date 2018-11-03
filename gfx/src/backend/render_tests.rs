    use crate::backend::*;

    #[test]
    fn test_event_mask_bits() {
        let mask = EventMask::KEY_PRESS | EventMask::BUTTON_PRESS;
        assert!(mask.contains(EventMask::KEY_PRESS));
        assert!(mask.contains(EventMask::BUTTON_PRESS));
        assert!(!mask.contains(EventMask::KEY_RELEASE));
    }

    #[test]
    fn test_event_keybut_mask() {
        let mask = KeyButMask::MOD1 | KeyButMask::CONTROL;
        assert!(mask.intersects(KeyButMask::MOD1));
        assert!(mask.intersects(KeyButMask::CONTROL));
        assert!(!mask.intersects(KeyButMask::SHIFT));
    }

