    use super::*;

    #[test]
    fn nt_glyphs_available() {
        assert!(title_glyph("minimize").is_some());
        assert!(title_glyph("maximize").is_some());
        assert!(title_glyph("restore").is_some());
        assert!(title_glyph("close").is_some());
        assert!(title_glyph("missing").is_none());
    }
