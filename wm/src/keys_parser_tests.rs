    use super::*;

    #[test]
    fn test_keysym_f1() {
        assert_eq!(keysym_from_name("F1"), Some(0xFFBE));
    }

    #[test]
    fn test_keysym_escape() {
        assert_eq!(keysym_from_name("Escape"), Some(0xFF1B));
        assert_eq!(keysym_from_name("Esc"), Some(0xFF1B));
    }


    #[test]
    fn test_keysym_single_char() {
        assert_eq!(keysym_from_name("d"), Some(0x64));
        assert_eq!(keysym_from_name("D"), Some(0x64));
        assert_eq!(keysym_from_name("1"), Some(0x31));
    }

    #[test]
    fn test_keysym_case_insensitive_names() {
        assert_eq!(keysym_from_name("escape"), Some(0xFF1B));
        assert_eq!(keysym_from_name("TAB"), Some(0xFF09));
    }

    #[test]
    fn test_modifiers_case_insensitive() {
        assert_eq!(parse_modifiers("super+Left"), (0x40, "Left"));
        assert_eq!(parse_modifiers("SHIFT+ctrl+alt+Right"), (0x01 | 0x04 | 0x08, "Right"));
    }

    #[test]
    fn test_parse_action_exec() {
        assert_eq!(
            parse_action("Exec xterm"),
            Some(Action::Misc(MiscOp::Command("xterm".to_string())))
        );
        assert_eq!(parse_action("Exec "), None);
    }

    #[test]
    fn test_parse_key_binding_full() {
        let e = parse_key_binding("Ctrl+Alt+1", "Workspace1").unwrap();
        assert_eq!(e.keysym, 0x31);
        assert_eq!(e.modifiers, 0x04 | 0x08);
        assert_eq!(e.action, Action::Workspace(WorkspaceOp::Workspace(0)));
    }

    #[test]
    fn test_parse_tap_binding_wants_a_repeated_modifier() {
        let e = parse_tap_binding("Super Super", "RootMenu").unwrap();
        assert_eq!(e.keysyms, vec![0xFFEB, 0xFFEC]);
        assert_eq!(e.action, Action::Menu(MenuOp::RootMenu));
        assert_eq!(
            parse_tap_binding("super+super", "Omni").unwrap().action,
            Action::Menu(MenuOp::Omni)
        );
        assert_eq!(
            parse_tap_binding("Alt Alt", "ShowDesktop").unwrap().keysyms,
            vec![0xFFE9, 0xFFEA]
        );
    }

    #[test]
    fn test_parse_tap_binding_rejects_other_combos() {
        assert!(parse_tap_binding("Super", "RootMenu").is_none());
        assert!(parse_tap_binding("Super Alt", "RootMenu").is_none());
        assert!(parse_tap_binding("Super Super Super", "RootMenu").is_none());
        assert!(parse_tap_binding("Ctrl+Alt+D", "ShowDesktop").is_none());
        assert!(parse_tap_binding("F1 F1", "RootMenu").is_none());
        assert!(parse_tap_binding("Super Super", "").is_none());
        assert!(parse_tap_binding("Super Super", "Nonsense").is_none());
    }

    #[test]
    fn test_tap_and_key_bindings_do_not_overlap() {
        assert!(parse_key_binding("Super Super", "RootMenu").is_none());
        assert!(parse_tap_binding("Super+D", "ShowDesktop").is_none());
        let keys = vec![
            ("Super Super".to_string(), "RootMenu".to_string()),
            ("Alt+F4".to_string(), "Close".to_string()),
        ];
        assert_eq!(entries_from(&keys).len(), 1);
        assert_eq!(taps_from(&keys).len(), 1);
    }

    #[test]
    fn test_expose_action_name() {
        assert_eq!(parse_action("Expose"), Some(Action::Menu(MenuOp::Expose)));
        assert_eq!(
            parse_key_binding("Super+Tab", "Expose").unwrap().action,
            Action::Menu(MenuOp::Expose)
        );
    }

    #[test]
    fn test_start_menu_names_the_root_menu() {
        assert_eq!(
            parse_action("StartMenu"),
            Some(Action::Menu(MenuOp::RootMenu))
        );
        assert_eq!(parse_action("StartMenu"), parse_action("RootMenu"));
        assert_eq!(
            parse_key_binding("Alt+Esc", "StartMenu").unwrap().keysym,
            0xFF1B
        );
    }

    #[test]
    fn test_win_is_a_super_alias() {
        assert_eq!(
            parse_tap_binding("Win Win", "Omni").unwrap(),
            parse_tap_binding("Super Super", "Omni").unwrap()
        );
    }
