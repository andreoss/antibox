    use super::mwm_decor;
    
    use super::mwm_hints_flags::DECORATIONS;
    use super::MwmHints;

    fn mwm(flags: u32, decorations: u32) -> MwmHints {
        MwmHints {
            flags,
            functions: 0,
            decorations,
            input_mode: 0,
        }
    }

    #[test]
    fn test_undecorated_borderless_request() {
        assert!(mwm(DECORATIONS, 0).undecorated());
    }

    #[test]
    fn test_decorated_when_all_or_title_or_border() {
        assert!(!mwm(DECORATIONS, mwm_decor::ALL).undecorated());
        assert!(!mwm(DECORATIONS, mwm_decor::TITLE).undecorated());
        assert!(!mwm(DECORATIONS, mwm_decor::BORDER).undecorated());
    }

