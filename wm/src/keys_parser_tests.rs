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
