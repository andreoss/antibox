    use super::*;

    #[test]
    fn test_parse_workspace_names_icewm_comma_quoted() {
        let got = parse_workspace_names("\" I \", \" II \", \" III \", \" IV \"", 4);
        assert_eq!(got, vec!["I", "II", "III", "IV"]);
    }

    #[test]
    fn test_parse_workspace_names_colon() {
        let got = parse_workspace_names("Main:Dev:Chat", 3);
        assert_eq!(got, vec!["Main", "Dev", "Chat"]);
    }


    #[test]
    fn test_prefs_default() {
        let p = Prefs::default();
        assert_eq!(p.workspace.count, 4);
        assert_eq!(p.font.name, "fixed");
        assert_eq!(p.font.size, 9);
    }

    #[test]
    fn test_parse_prefs_blocks() {
        let p = parse_prefs("[font]\nname = \"6x13\"\nsize = 12\n\n[workspace]\ncount = 6\n");
        assert_eq!(p.font.name, "6x13");
        assert_eq!(p.font.size, 12);
        assert_eq!(p.workspace.count, 6);
    }

    #[test]
    fn test_parse_prefs_unquoted_and_comments() {
        let p = parse_prefs("# antibox\n[workspace]\ncount = 6 # six of them\n; note\n[font]\nname = fixed\n");
        assert_eq!(p.workspace.count, 6);
        assert_eq!(p.font.name, "fixed");
    }

    #[test]
    fn test_parse_prefs_ignores_unknown_and_invalid() {
        let p = parse_prefs("[general]\ncount = 9\n[workspace]\ncount = zero\nname = x\n[font]\nsize = big\n");
        assert_eq!(p.workspace.count, 4);
        assert_eq!(p.font.size, 9);
    }

    #[test]
    fn test_parse_prefs_count_bounds() {
        assert_eq!(parse_prefs("[workspace]\ncount = 0\n").workspace.count, 4);
        assert_eq!(parse_prefs("[workspace]\ncount = 99\n").workspace.count, 4);
        assert_eq!(parse_prefs("[workspace]\ncount = 32\n").workspace.count, 32);
    }

    #[test]
    fn test_parse_prefs_quoted_hash_kept() {
        let p = parse_prefs("[font]\nname = \"a#b\"\n");
        assert_eq!(p.font.name, "a#b");
    }

    #[test]
    fn test_defaults_ini_matches_builtin_defaults() {
        assert_eq!(default_prefs(), Prefs::default());
    }

    #[test]
    fn test_apply_prefs_merges_partial_config() {
        let mut p = parse_prefs("[font]\nname = \"6x13\"\nsize = 12\n[workspace]\ncount = 6\n");
        apply_prefs(&mut p, "[workspace]\ncount = 8\n");
        assert_eq!(p.workspace.count, 8);
        assert_eq!(p.font.name, "6x13");
        assert_eq!(p.font.size, 12);
        apply_prefs(&mut p, "[font]\nsize = 10\n");
        assert_eq!(p.font.size, 10);
        assert_eq!(p.font.name, "6x13");
        assert_eq!(p.workspace.count, 8);
    }
