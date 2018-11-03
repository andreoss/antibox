    use super::*;

    #[test]
    fn layout_roundtrip() {
        assert_eq!(title_buttons_left(), "sp");
        assert_eq!(title_buttons_right(), "xmir");
        assert_eq!(title_justify(), 0);
        set_title_layout("s", "xmir", 250);
        assert_eq!(title_justify(), 100);
        set_title_layout("s", "xmir", -5);
        assert_eq!(title_justify(), antibox_ui::theme::title_justify_default());
        set_taskbar_align("Center");
        assert!(match taskbar_align() {
            antibox_ui::widget::LabelAlign::Center => true,
            _ => false,
        });
        set_taskbar_align("right");
        assert!(match taskbar_align() {
            antibox_ui::widget::LabelAlign::Right => true,
            _ => false,
        });
        set_taskbar_align("left");
        assert!(match taskbar_align() {
            antibox_ui::widget::LabelAlign::Left => true,
            _ => false,
        });
        for empty in ["", "   ", "nonsense"].iter().cloned() {
            set_taskbar_align(empty);
            assert_eq!(TASKBAR_ALIGN.load(Ordering::Relaxed), TASKBAR_ALIGN_THEME);
            assert!(match taskbar_align() {
                    antibox_ui::widget::LabelAlign::Left => true,
                    _ => false,
                });
            assert!(!taskbar_fill());
        }
        set_title_layout(DEFAULT_TITLE_LEFT, DEFAULT_TITLE_RIGHT, 0);
        set_taskbar_align("");
    }

    #[test]
    fn parse_layout_reads_tokens_aliases_and_separators() {
        assert_eq!(parse_layout(""), None);
        assert_eq!(parse_layout("   "), None);
        assert_eq!(parse_layout("nonsense ???"), None);
        assert_eq!(
            parse_layout("pager | tasks  clock"),
            Some(vec![Widget::Workspaces, Widget::Windows, Widget::Clock,])
        );
        assert_eq!(parse_layout("WINDOWLIST SysTray"), Some(vec![Widget::Windows, Widget::Tray]));
    }

