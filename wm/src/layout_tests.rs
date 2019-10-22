    use super::*;

    #[test]
    fn parse_accepts_aliases_and_rejects_unknown() {
        assert_eq!(Layout::parse("floating"), Some(Layout::Floating));
        assert_eq!(Layout::parse(" Float "), Some(Layout::Floating));
        assert_eq!(Layout::parse("tall"), Some(Layout::Tall));
        assert_eq!(Layout::parse("TILED"), Some(Layout::Tall));
        assert_eq!(Layout::parse(""), Some(Layout::Floating));
        assert_eq!(Layout::parse("mosaic"), None);
    }

    #[test]
    fn parse_list_pads_and_truncates() {
        let l = Layout::parse_list("tall, bogus", 4);
        assert_eq!(
            l,
            vec![
                Layout::Tall,
                Layout::Floating,
                Layout::Floating,
                Layout::Floating
            ]
        );
        let l = Layout::parse_list("tall,tall,tall", 2);
        assert_eq!(l, vec![Layout::Tall, Layout::Tall]);
    }

    #[test]
    fn next_cycles_through_all() {
        assert_eq!(Layout::Floating.next(), Layout::Tall);
        assert_eq!(Layout::Tall.next(), Layout::Wide);
        assert_eq!(Layout::Wide.next(), Layout::Floating);
    }

    #[test]
    fn wide_is_tall_rotated() {
        let r = Rect::new(0, 0, 800, 600);
        let wide = tile_wide(50, r, 1, 3);
        assert_eq!(wide.len(), 3);
        assert_eq!(wide[0], Rect::new(0, 0, 800, 300));
        assert_eq!(wide[1].y, 300);
        assert_eq!(wide[1].h, 300);
        assert_eq!(wide[1].x, 0);
        assert_eq!(wide[2].x, wide[1].x + wide[1].w);
        assert_eq!(wide[2].x + wide[2].w, 800);
    }

    #[test]
    fn wide_parses_and_titles() {
        assert_eq!(Layout::parse("wide"), Some(Layout::Wide));
        assert_eq!(Layout::parse("mirror"), Some(Layout::Wide));
        assert!(Layout::Wide.is_tiled());
        assert_eq!(Layout::Wide.name(), "wide");
    }

    #[test]
    fn split_vertically_covers_exactly() {
        let r = Rect::new(0, 0, 100, 101);
        let rows = split_vertically(3, r);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].y, 0);
        assert_eq!(rows[1].y, rows[0].y + rows[0].h);
        assert_eq!(rows[2].y, rows[1].y + rows[1].h);
        assert_eq!(rows[2].y + rows[2].h, 101);
    }

    #[test]
    fn split_vertically_single_is_full() {
        let r = Rect::new(5, 6, 40, 30);
        assert_eq!(split_vertically(1, r), vec![r]);
    }

    #[test]
    fn tile_one_window_fills_area() {
        let r = Rect::new(0, 0, 800, 600);
        let t = tile(50, r, 1, 1);
        assert_eq!(t, vec![r], "a lone window ignores the master split");
    }

    #[test]
    fn tile_master_and_stack_columns() {
        let r = Rect::new(0, 0, 800, 600);
        let t = tile(50, r, 1, 3);
        assert_eq!(t.len(), 3);
        assert_eq!(t[0], Rect::new(0, 0, 400, 600));
        assert_eq!(t[1].x, 400);
        assert_eq!(t[1].w, 400);
        assert_eq!(t[1].y, 0);
        assert_eq!(t[2].y, t[1].y + t[1].h);
        assert_eq!(t[2].y + t[2].h, 600);
    }

    #[test]
    fn tile_all_master_when_few() {
        let r = Rect::new(0, 0, 800, 600);
        let t = tile(50, r, 2, 2);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].w, 800);
        assert_eq!(t[1].w, 800);
    }
