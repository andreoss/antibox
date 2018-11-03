    use super::*;

    #[test]
    fn ui_font_override_and_reset() {
        set_ui_font("");
        assert_eq!(ui_font(), DEFAULT_UI_FONT);
        assert_eq!(FontSpec::ui(12).family, DEFAULT_UI_FONT);

        set_ui_font("Noto Sans");
        assert_eq!(ui_font(), "Noto Sans");
        assert_eq!(FontSpec::ui(12).family, "Noto Sans");

        set_ui_font("   ");
        assert_eq!(ui_font(), DEFAULT_UI_FONT);
    }

    #[test]
    fn parse_font_desc_splits_family_size_style() {
        assert_eq!(parse_font_desc(""), None);
        assert_eq!(
            parse_font_desc("DejaVu Serif"),
            Some(("DejaVu Serif".to_string(), 0, false, false))
        );
        assert_eq!(
            parse_font_desc("Noto Sans 12 bold"),
            Some(("Noto Sans".to_string(), 12, true, false))
        );
        assert_eq!(
            parse_font_desc("Fira Code 14 italic bold"),
            Some(("Fira Code".to_string(), 14, true, true))
        );
    }

