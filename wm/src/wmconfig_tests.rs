    use super::*;
    use crate::option::WindowFlags;

    #[test]
    fn test_apply_winoptions_parses_lines() {
        let mut o = WindowOptions::new();
        apply_winoptions(
            &mut o,
            "# a comment\n\nfirefox.Firefox.workspace: 2\nxterm.XTerm.geometry: 100x200+10+20\n",
        );
        assert_eq!(o.len(), 2);
        let (found, i) = o.find("firefox.Firefox");
        assert!(found);
        assert_eq!(o.get(i).unwrap().placement.workspace, Some(2));
        let (found, i) = o.find("xterm.XTerm");
        assert!(found);
        let g = &o.get(i).unwrap().geom;
        assert_eq!((g.gx, g.gy, g.gw, g.gh), (10, 20, 100, 200));
    }

    #[test]
    fn test_apply_winoptions_flags_and_masks() {
        let mut o = WindowOptions::new();
        apply_winoptions(&mut o, "xterm.XTerm.startMaximized: 1\nxterm.XTerm.ignoreTaskBar: 0\n");
        assert_eq!(o.len(), 1);
        let (_, i) = o.find("xterm.XTerm");
        let wo = o.get(i).unwrap();
        assert!(wo.has_option(WindowFlags::MAXIMIZED_BOTH));
        assert!(!wo.has_option(WindowFlags::IGNORE_TASKBAR));
    }

    #[test]
    fn test_apply_winoptions_ignores_malformed() {
        let mut o = WindowOptions::new();
        apply_winoptions(&mut o, "nocolon\nnokey: 1\n: 2\n\n   \n");
        assert!(o.is_empty());
    }

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
    fn test_parse_prefs_comments() {
        let p = parse_prefs("# antibox\n[workspace]\ncount = 6 # six of them\n[font]\nname = \"fixed\"\n");
        assert_eq!(p.workspace.count, 6);
        assert_eq!(p.font.name, "fixed");
    }

    #[test]
    fn test_parse_prefs_ignores_unknown_and_invalid() {
        let p = parse_prefs("[general]\ncount = 9\n[workspace]\ncount = \"zero\"\nname = \"x\"\n[font]\nsize = 12\n");
        assert_eq!(p.workspace.count, 4);
        assert_eq!(p.font, Prefs::default().font);
    }

    #[test]
    fn test_parse_prefs_rejects_malformed_toml() {
        let p = parse_prefs("[workspace\ncount = 6\n");
        assert_eq!(p.workspace.count, 4);
        assert_eq!(p.font, Prefs::default().font);
    }

    #[test]
    fn test_parse_prefs_semicolon_is_not_a_comment() {
        let p = parse_prefs("; note\n[workspace]\ncount = 6\n");
        assert_eq!(p.workspace.count, 4);
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
    fn test_defaults_toml_matches_builtin_defaults() {
        let d = default_prefs();
        assert_eq!(d.font, Prefs::default().font);
        assert_eq!(d.workspace, Prefs::default().workspace);
        assert_eq!(d.keyboard, Prefs::default().keyboard);
        assert_eq!(d.clock, Prefs::default().clock);
        assert_eq!(d.pointer, Prefs::default().pointer);
        assert_eq!(d.graph, Prefs::default().graph);
        assert_eq!(d.winlist, Prefs::default().winlist);
        assert_eq!(d.expose, Prefs::default().expose);
        assert_eq!(d.expose.modes, DEFAULT_EXPOSE_MODES);
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
    fn test_defaults_toml_keys_all_parse() {
        for (combo, action) in default_prefs().keys {
            assert!(
                crate::keys_parser::parse_key_binding(&combo, &action).is_some()
                    || crate::keys_parser::parse_tap_binding(&combo, &action).is_some(),
                "{combo} = {action}"
            );
        }
    }

    #[test]
    fn test_keys_merge_overrides_and_unbinds() {
        let mut p = parse_prefs("[keys]\n\"Alt+F4\" = \"Close\"\n\"Super+D\" = \"ShowDesktop\"\n");
        assert_eq!(p.keys.len(), 2);
        apply_prefs(&mut p, "[keys]\n\"alt+f4\" = \"Kill\"\n\"Super+K\" = \"Kill\"\n\"Super+D\" = \"\"\n");
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
    fn test_parse_prefs_clock_format() {
        assert_eq!(parse_prefs("").clock.format, "%H:%M:%S");
        let p = parse_prefs("[clock]\nformat = \"%a %H:%M\"\n");
        assert_eq!(p.clock.format, "%a %H:%M");
        assert_eq!(parse_prefs("[clock]\nformat = \"\"\n").clock.format, "%H:%M:%S");
    }

    #[test]
    fn test_parse_prefs_graph_colours() {
        let d = parse_prefs("");
        assert_eq!(d.graph.series, "000080,000080,7F7FBF,808080");
        assert_eq!(d.graph.heat, "C82020");
        assert_eq!(parse_colour("C82020"), Some(0xC82020));
        assert_eq!(parse_colour("#00FF00"), Some(0x00FF00));
        assert_eq!(parse_colour("xyz"), None);
        assert_eq!(parse_colour_list("102030, 405060"), vec![0x102030, 0x405060]);
        let p = parse_prefs("[graph]\nseries = \"FF0000,00FF00\"\nheat = \"AA0000\"\n");
        assert_eq!(p.graph.series, "FF0000,00FF00");
        assert_eq!(p.graph.heat, "AA0000");
        let p = parse_prefs("[graph]\nseries = \"bogus\"\nheat = \"nope\"\n");
        assert_eq!(p.graph.series, "000080,000080,7F7FBF,808080");
        assert_eq!(p.graph.heat, "C82020");
    }

    #[test]
    fn test_parse_prefs_taskbar_layout() {
        assert_eq!(parse_prefs("").taskbar.layout, DEFAULT_TASKBAR_LAYOUT);
        let p = parse_prefs("[taskbar]\nlayout = \"clock pager tasks\"\n");
        assert_eq!(p.taskbar.layout, "clock pager tasks");
        let empty = parse_prefs("[taskbar]\nlayout = \"\"\n");
        assert_eq!(empty.taskbar.layout, DEFAULT_TASKBAR_LAYOUT);
    }

    #[test]
    fn test_parse_prefs_pointer() {
        assert!(!parse_prefs("").pointer.warp);
        assert!(parse_prefs("[pointer]\nwarp = true\n").pointer.warp);
        assert!(!parse_prefs("[pointer]\nwarp = \"bogus\"\n").pointer.warp);
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

    #[test]
    fn test_a_legacy_ini_config_is_migrated_to_the_toml_name() {
        let dir = std::env::temp_dir().join("antibox_cfg_migrate_a");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let legacy = dir.join(crate::config_watch::LEGACY_CONFIG_FILE);
        let current = dir.join(crate::config_watch::CONFIG_FILE);
        std::fs::write(&legacy, "[keyboard]\nlayouts = \"us,ru\"\n").unwrap();

        migrate_legacy_config(&dir);

        assert!(!legacy.is_file(), "the old name is gone");
        let text = std::fs::read_to_string(&current).unwrap();
        assert_eq!(parse_prefs(&text).keyboard.layouts, "us,ru");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_migration_keeps_an_existing_toml_config() {
        let dir = std::env::temp_dir().join("antibox_cfg_migrate_b");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let legacy = dir.join(crate::config_watch::LEGACY_CONFIG_FILE);
        let current = dir.join(crate::config_watch::CONFIG_FILE);
        std::fs::write(&legacy, "[keyboard]\nlayouts = \"old\"\n").unwrap();
        std::fs::write(&current, "[keyboard]\nlayouts = \"new\"\n").unwrap();

        migrate_legacy_config(&dir);

        assert!(legacy.is_file(), "the old file is left alone");
        let text = std::fs::read_to_string(&current).unwrap();
        assert_eq!(parse_prefs(&text).keyboard.layouts, "new");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_prefs_expose() {
        assert_eq!(parse_prefs("").expose, ExposePrefs::default());
        let p = parse_prefs(
            "[expose]\nmodes = \"icon filled\"\nmouse1 = \"shade\"\nmouse2 = \"no\"\ndistance = 12\nicon_size = 64\nallow_upscale = true\nall_desktops = true\nshow_minimized = false\nshow_skip_taskbar = true\ninclude_frame = false\n",
        );
        assert_eq!(p.expose.modes, "icon filled");
        assert_eq!(p.expose.mouse1, "shade");
        assert_eq!(p.expose.mouse2, "no");
        assert_eq!(p.expose.distance, 12);
        assert_eq!(p.expose.icon_size, 64);
        assert!(p.expose.allow_upscale);
        assert!(p.expose.all_desktops);
        assert!(!p.expose.show_minimized);
        assert!(p.expose.show_skip_taskbar);
        assert!(!p.expose.include_frame);
    }

    #[test]
    fn test_parse_prefs_expose_bounds() {
        let d = ExposePrefs::default();
        let p = |t: &str| parse_prefs(t).expose;
        assert_eq!(p("[expose]\ndistance = 0\n").distance, d.distance);
        assert_eq!(p("[expose]\ndistance = 401\n").distance, d.distance);
        assert_eq!(p("[expose]\ndistance = \"wide\"\n").distance, d.distance);
        assert_eq!(p("[expose]\nicon_size = 4\n").icon_size, d.icon_size);
        assert_eq!(p("[expose]\nicon_size = 257\n").icon_size, d.icon_size);
        assert_eq!(p("[expose]\nicon_size = 8\n").icon_size, 8);
        assert!(!p("[expose]\nallow_upscale = 1\n").allow_upscale);
        assert_eq!(p("[expose]\nmodes = \"\"\n").modes, d.modes);
        assert_eq!(p("[expose]\nmodes = 3\n").modes, d.modes);
    }

    #[test]
    fn test_parse_prefs_expose_tint() {
        let d = ExposePrefs::default();
        let p = |t: &str| parse_prefs(t).expose;
        let got = p("[expose]\nopacity = 90\ntint = \"#204080\"\ntint_opacity = 33\nhighlight_opacity = 120\nhighlight_tint = \"ffffff\"\nhighlight_tint_opacity = 200\n");
        assert_eq!(got.opacity, 90);
        assert_eq!(got.tint, "#204080");
        assert_eq!(got.tint_opacity, 33);
        assert_eq!(got.highlight_opacity, 120);
        assert_eq!(got.highlight_tint, "ffffff");
        assert_eq!(got.highlight_tint_opacity, 200);
        assert_eq!(p("[expose]\nopacity = 256\n").opacity, d.opacity);
        assert_eq!(p("[expose]\nopacity = -1\n").opacity, d.opacity);
        assert_eq!(p("[expose]\ntint = \"mauve\"\n").tint, d.tint);
        assert_eq!(parse_colour(&d.highlight_tint), Some(0));
    }

    #[test]
    fn test_parse_prefs_expose_tooltip() {
        let d = ExposePrefs::default();
        assert!(d.tooltip && d.tooltip_follows_mouse);
        assert_eq!((d.tooltip_offset_x, d.tooltip_offset_y), (20, 20));
        assert_eq!(d.tooltip_align, "left");
        let p = |t: &str| parse_prefs(t).expose;
        let got = p("[expose]\ntooltip = false\ntooltip_follows_mouse = false\ntooltip_offset_x = -12\ntooltip_offset_y = 4\ntooltip_align = \"mid\"\n");
        assert!(!got.tooltip);
        assert!(!got.tooltip_follows_mouse);
        assert_eq!((got.tooltip_offset_x, got.tooltip_offset_y), (-12, 4));
        assert_eq!(got.tooltip_align, "mid");
        assert_eq!(p("[expose]\ntooltip_align = \"sideways\"\n").tooltip_align, d.tooltip_align);
        assert_eq!(p("[expose]\ntooltip_offset_x = 4000\n").tooltip_offset_x, d.tooltip_offset_x);
    }

    #[test]
    fn test_parse_prefs_expose_warp() {
        let d = ExposePrefs::default();
        assert!(!d.warp_on_show && !d.warp_on_select && !d.warp_on_activate);
        let got = parse_prefs("[expose]\nwarp_on_show = true\nwarp_on_select = true\nwarp_on_activate = true\n").expose;
        assert!(got.warp_on_show && got.warp_on_select && got.warp_on_activate);
    }

    #[test]
    fn test_parse_prefs_expose_monitors() {
        assert!(ExposePrefs::default().all_monitors);
        assert!(!parse_prefs("[expose]\nall_monitors = false\n").expose.all_monitors);
    }

    #[test]
    fn test_parse_prefs_expose_live_thumbnails() {
        assert!(ExposePrefs::default().live_thumbnails);
        assert!(!parse_prefs("[expose]\nlive_thumbnails = false\n").expose.live_thumbnails);
    }

    #[test]
    fn test_parse_prefs_hover_previews() {
        let d = Prefs::default();
        assert!(d.taskbar.preview && d.omni.preview);
        assert!(d.winlist.preview && d.pager.preview);
        assert!(default_prefs().winlist.preview && default_prefs().pager.preview);
        let off = parse_prefs("[winlist]\npreview = false\n[pager]\npreview = false\n");
        assert!(!off.winlist.preview && !off.pager.preview);
        assert_eq!(off.winlist.position, d.winlist.position);
        assert!(default_prefs().taskbar.preview && default_prefs().omni.preview);
        let p = parse_prefs("[taskbar]\npreview = false\n[omni]\npreview = false\n");
        assert!(!p.taskbar.preview && !p.omni.preview);
        assert_eq!(p.taskbar.layout, d.taskbar.layout);
        assert!(parse_prefs("[omni]\npreview = \"yes\"\n").omni.preview);
    }
