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
        assert_eq!(
            p.font.name,
            "-misc-fixed-medium-r-semicondensed--13-*-*-*-*-*-iso10646-1"
        );
    }

    #[test]
    fn test_parse_prefs_blocks() {
        let p = parse_prefs("[font]\nname = \"6x13\"\n\n[workspace]\ncount = 6\n");
        assert_eq!(p.font.name, "6x13");
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
        let p = parse_prefs("[general]\ncount = 9\n[workspace]\ncount = zero\nname = x\n[font]\nsize = 12\n");
        assert_eq!(p.workspace.count, 4);
        assert_eq!(p.font, Prefs::default().font);
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
        let d = default_prefs();
        assert_eq!(d.font, Prefs::default().font);
        assert_eq!(d.workspace, Prefs::default().workspace);
        assert_eq!(d.keyboard, Prefs::default().keyboard);
        assert_eq!(d.winlist, Prefs::default().winlist);
        assert!(!d.keys.is_empty());
    }

    #[test]
    fn test_parse_prefs_layouts() {
        let p = parse_prefs("[workspace]\nlayouts = \"tall,floating\"\n[keyboard]\nlayouts = \"us,ru\"\n");
        assert_eq!(p.workspace.layouts, "tall,floating");
        assert_eq!(p.keyboard.layouts, "us,ru");
        assert_eq!(p.workspace.count, 4);
    }

    #[test]
    fn test_defaults_ini_keys_all_parse() {
        for (combo, action) in default_prefs().keys {
            assert!(
                crate::keys_parser::parse_key_binding(&combo, &action).is_some(),
                "{} = {}",
                combo,
                action
            );
        }
    }

    #[test]
    fn test_keys_merge_overrides_and_unbinds() {
        let mut p = parse_prefs("[keys]\nAlt+F4 = \"Close\"\nSuper+D = \"ShowDesktop\"\n");
        assert_eq!(p.keys.len(), 2);
        apply_prefs(&mut p, "[keys]\nalt+f4 = \"Kill\"\nSuper+K = \"Kill\"\nSuper+D = \"\"\n");
        assert_eq!(p.keys.len(), 3);
        assert_eq!(p.keys[0], ("Alt+F4".to_string(), "Kill".to_string()));
        assert_eq!(p.keys[1], ("Super+D".to_string(), String::new()));
        assert_eq!(p.keys[2], ("Super+K".to_string(), "Kill".to_string()));
        assert!(crate::keys_parser::parse_key_binding("Super+D", "").is_none());
    }

    #[test]
    fn test_apply_prefs_merges_partial_config() {
        let mut p = parse_prefs("[font]\nname = \"6x13\"\n[workspace]\ncount = 6\n");
        apply_prefs(&mut p, "[workspace]\ncount = 8\n");
        assert_eq!(p.workspace.count, 8);
        assert_eq!(p.font.name, "6x13");
        apply_prefs(&mut p, "[font]\nname = \"7x14\"\n");
        assert_eq!(p.font.name, "7x14");
        assert_eq!(p.workspace.count, 8);
    }

    #[test]
    fn test_parse_prefs_net() {
        let p = parse_prefs("[net]\nwidth = 60\ndevice = \"en* wlan0\"\n");
        assert_eq!(p.net.width, 60);
        assert_eq!(p.net.device, "en* wlan0");
        assert_eq!(parse_prefs("").net.device, "*");
    }

    #[test]
    fn test_parse_prefs_winlist_position() {
        assert_eq!(parse_prefs("").winlist.position, "centre");
        let p = |t: &str| parse_prefs(t).winlist.position;
        assert_eq!(p("[winlist]\nposition = \"pointer\"\n"), "pointer");
        assert_eq!(p("[winlist]\nposition = \"center\"\n"), "centre");
        assert_eq!(p("[winlist]\nposition = \"mouse\"\n"), "centre");
        assert_eq!(p("[winlist]\nposition = \"bogus\"\n"), "centre");
    }
