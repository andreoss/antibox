    use super::*;

    fn core(text: &str, cursor: usize) -> EditCore {
        let mut c = EditCore::new();
        c.set_text(text);
        c.set_cursor(cursor);
        c
    }

    #[test]
    fn secret_display_masks_every_char() {
        let mut c = core("héllo", 5);
        c.secret = true;
        assert_eq!(c.display_text(), "*****");
        assert_eq!(c.text(), "héllo");
    }

    #[test]
    fn plain_display_borrows_logical_text() {
        let c = core("héllo", 0);
        assert!(matches!(c.display_text(), Cow::Borrowed("héllo")));
    }

