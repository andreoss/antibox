    use super::*;
    use antibox_core::mock::{MockCommand, MockGraphics};

    #[test]
    fn nt_theme_supplies_chrome_colours() {
        let tc = ThemeColors::default();
        assert_eq!(tc.active_title_top, 0x00_00_80);
        assert_eq!(tc.task_bar_colour, 0xC0_C0_C0);
        assert_eq!(tc.active_text, 0xFF_FF_FF);

        let g = MockGraphics::new(1);
        draw_title_bar(
            &g,
            false,
            true,
            false,
            TitleBarDims { fw_w: 300, bw: 1 },
            &tc,
            true,
        )
        .unwrap();
        assert!(g
            .commands()
            .iter()
            .any(|c| matches!(c, MockCommand::FillRect(_, _, _, _))));

        let fills = |g: &MockGraphics| -> Vec<u32> {
            g.commands()
                .iter()
                .filter_map(|c| match c {
                    MockCommand::SetForeground(p) => Some(*p),
                    _ => None,
                })
                .collect()
        };

        let focused = MockGraphics::new(1);
        let (fg, _) = antibox_ui::theme::title_button(
            &focused,
            Rect::px(0, 0, 20, 18),
            tc.button_bg,
            tc.button_fg,
            true,
            false,
        );
        assert!(fills(&focused).contains(&tc.button_bg));
        assert_eq!(fg, tc.button_fg);

        let unfocused = MockGraphics::new(1);
        let (fg, _) = antibox_ui::theme::title_button(
            &unfocused,
            Rect::px(0, 0, 20, 18),
            tc.button_bg,
            tc.button_fg,
            false,
            false,
        );
        assert!(fills(&unfocused).contains(&tc.button_bg));
        assert_eq!(fg, tc.button_fg);

        let pressed = MockGraphics::new(1);
        let (fg, _) = antibox_ui::theme::title_button(
            &pressed,
            Rect::px(0, 0, 20, 18),
            tc.button_bg,
            tc.button_fg,
            true,
            true,
        );
        assert!(fills(&pressed).contains(&tc.button_bg));
        assert_ne!(fg, tc.button_bg);

        assert_eq!(tc.task_bar_colour, 0xC0_C0_C0);
    }

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(parse_mnemonic("Re_boot"), ("Reboot".to_string(), Some(2)));
        assert_eq!(parse_mnemonic("_Cancel"), ("Cancel".to_string(), Some(0)));
        assert_eq!(
            parse_mnemonic("Reload ke_ys"),
            ("Reload keys".to_string(), Some(9))
        );
        assert_eq!(parse_mnemonic("No marker"), ("No marker".to_string(), None));

        let (t, h) = parse_mnemonic("Shut_down");
        assert_eq!(t.chars().nth(h.unwrap()), Some('d'));
    }

