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


    #[test]
    fn parse_layout_dedupes_aliases_preserving_first_order() {
        assert_eq!(
            parse_layout("clock pager clock tasks"),
            Some(vec![Widget::Clock, Widget::Workspaces, Widget::Windows])
        );
        assert_eq!(
            parse_layout("volume power audio"),
            Some(vec![Widget::PowerAudio])
        );
    }

    #[test]
    fn set_taskbar_layout_drives_presence_and_omission() {
        set_taskbar_layout("clock pager");
        assert!(taskbar_wants(Widget::Clock), "listed widget is present");
        assert!(taskbar_wants(Widget::Workspaces), "listed widget is present");
        assert!(!taskbar_wants(Widget::Cpu), "unlisted widget is omitted");
        assert!(!taskbar_wants(Widget::Windows), "unlisted widget is omitted");
        assert_eq!(
            taskbar_layout(),
            Some(vec![Widget::Clock, Widget::Workspaces])
        );
        set_taskbar_layout(crate::wmconfig::DEFAULT_TASKBAR_LAYOUT);
    }
