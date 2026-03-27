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


    #[test]
    fn word_motion_lands_on_char_boundaries_in_cyrillic() {
        let text = "привет мир";
        let c = core(text, text.len());
        let p = c.prev_word();
        assert!(text.is_char_boundary(p), "prev_word gave {p}");
        assert_eq!(&text[p..], "мир");

        let c0 = core(text, 0);
        let n = c0.next_word();
        assert!(text.is_char_boundary(n), "next_word gave {n}");
        assert_eq!(&text[..n], "привет");
    }

    #[test]
    fn delete_prev_word_keeps_cyrillic_intact() {
        let mut c = core("привет мир", "привет мир".len());
        c.delete_prev_word();
        assert_eq!(c.text(), "привет ");
    }
