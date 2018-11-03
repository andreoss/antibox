    use super::*;

    #[test]
    fn parse_accepts_aliases_and_rejects_unknown() {
        assert_eq!(Layout::parse("floating"), Some(Layout::Floating));
        assert_eq!(Layout::parse(" Float "), Some(Layout::Floating));
        assert_eq!(Layout::parse(""), Some(Layout::Floating));
        assert_eq!(Layout::parse("tall"), None);
        assert_eq!(Layout::parse("wide"), None);
        assert_eq!(Layout::parse("mosaic"), None);
    }

    #[test]
    fn parse_list_pads_and_truncates() {
        let l = Layout::parse_list("tall, bogus", 4);
        assert_eq!(
            l,
            vec![
                Layout::Floating,
                Layout::Floating,
                Layout::Floating,
                Layout::Floating
            ]
        );
        let l = Layout::parse_list("floating", 2);
        assert_eq!(l, vec![Layout::Floating, Layout::Floating]);
    }
