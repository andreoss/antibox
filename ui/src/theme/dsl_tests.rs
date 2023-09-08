    use super::*;
    use crate::theme::NT_THEME;

    fn nt() -> ThemeDef {
        crate::theme::load::from_toml(NT_THEME).unwrap()
    }

    #[test]
    fn coord_literals_and_variables() {
        assert_eq!(parse_coord("0").unwrap().eval(100, 50, 2), 0);
        assert_eq!(parse_coord("w").unwrap().eval(100, 50, 2), 100);
        assert_eq!(parse_coord("h").unwrap().eval(100, 50, 2), 50);
        assert_eq!(parse_coord("s").unwrap().eval(100, 50, 2), 2);
    }

    #[test]
    fn coord_arithmetic_and_precedence() {
        assert_eq!(parse_coord("w-2*s").unwrap().eval(100, 50, 2), 96);
        assert_eq!(parse_coord("h-1*s").unwrap().eval(100, 50, 2), 48);
        assert_eq!(parse_coord("2*s+3").unwrap().eval(100, 50, 2), 7);
        assert_eq!(parse_coord("(w-4)*2").unwrap().eval(100, 50, 2), 192);
    }

    #[test]
    fn coord_rejects_garbage() {
        assert!(parse_coord("").is_none());
        assert!(parse_coord("w+").is_none());
        assert!(parse_coord("q*2").is_none());
        assert!(parse_coord("w-2*s z").is_none());
    }

    #[test]
    fn colour_ref_forms() {
        assert_eq!(
            parse_colour_ref("face").unwrap(),
            ColourRef::Named("face".to_string())
        );
        assert!(matches!(
            parse_colour_ref("face*0.5").unwrap(),
            ColourRef::Scale(_, _)
        ));
        assert!(matches!(
            parse_colour_ref("face~shadow*0.25").unwrap(),
            ColourRef::Blend(_, _, _)
        ));
    }

    #[test]
    fn colour_ref_scales_and_blends() {
        let def = nt();
        let half = resolve_colour(&def, &parse_colour_ref("face*0.5").unwrap());
        assert_eq!(half, Some(0x606060));
        let one = resolve_colour(&def, &parse_colour_ref("face*1.0").unwrap());
        assert_eq!(one, Some(0xC0C0C0));
        let blend = resolve_colour(&def, &parse_colour_ref("face~shadow*1.0").unwrap());
        assert_eq!(blend, Some(0x808080));
    }

    #[test]
    fn nt_theme_carries_the_builtin_constants() {
        let def = nt();
        assert_eq!(def.name, "nt");
        assert_eq!(def.colour("face"), Some(0xC0C0C0));
        assert_eq!(def.colour("title_active"), Some(0x000080));
        assert_eq!(def.colour("tooltip_bg"), Some(0xFFFFE1));
        assert_eq!(def.metric("pad"), Some(4));
        assert_eq!(def.metric("title_height"), Some(18));
        assert_eq!(def.metric("sunken_depth"), Some(2));
        assert_eq!(def.string("title_buttons"), Some("xmi"));
        assert_eq!(def.string("title_layout_right"), Some("xmir"));
    }

    #[test]
    fn nt_theme_defines_elements() {
        let def = nt();
        for name in &["panel", "field", "well", "button", "button_pressed", "title_active", "title_inactive", "menu_sel", "tooltip", "progress"] {
            assert!(
                def.element(name).map_or(false, |o| !o.is_empty()),
                "element {} must have ops",
                name
            );
        }
        assert!(def.element("nope").is_none());
    }

    #[test]
    fn draw_element_paints_known_elements() {
        let g = antibox_gfx::mock::MockGraphics::new(1);
        assert!(crate::theme::draw_element(&g, "panel", 0, 0, 20, 20));
        assert!(crate::theme::draw_element(&g, "button", 0, 0, 20, 20));
        assert!(!crate::theme::draw_element(&g, "nope", 0, 0, 20, 20));
    }
