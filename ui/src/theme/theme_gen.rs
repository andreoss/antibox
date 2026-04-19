    use super::*;

    #[test]
    fn nt_glyphs_available() {
        assert!(title_glyph("minimize").is_some());
        assert!(title_glyph("maximize").is_some());
        assert!(title_glyph("restore").is_some());
        assert!(title_glyph("close").is_some());
        assert!(title_glyph("missing").is_none());
    }

    #[test]
    fn test_selection_is_visible_in_every_builtin_theme() {
        for (name, text) in BUILTIN_THEMES {
            let def = load::from_toml(text).unwrap();
            let get = |k: &str, fallback: Colour| def.colour(k).unwrap_or(fallback);
            let face = get("face", 0xC0C0C0);
            let field = get("field", 0xFFFFFF);
            let menu_bg = get("menu_bg", face);
            let menu_sel = get("menu_sel_bg", get("title_active", 0x000080));
            let sel = get("sel_bg", 0x000080);
            assert!(
                def.element("menu_sel").is_some() || distinct(menu_sel, menu_bg) == menu_sel,
                "{name}: menu selection blends into the menu"
            );
            assert_eq!(distinct(sel, field), sel, "{name}: text selection blends into the field");
        }
    }
