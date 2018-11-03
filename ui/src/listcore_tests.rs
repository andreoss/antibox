    use super::*;

    fn list(rows: usize, row_h: i32, view_h: i32) -> ListCore {
        let mut c = ListCore::new();
        c.set_counts(rows, row_h, view_h);
        c
    }

    #[test]
    fn ensure_visible_scrolls_down_to_the_row_bottom() {
        let mut c = list(100, 10, 25);
        c.ensure_visible(4);
        assert_eq!(c.scroll, 25);
        c.ensure_visible(4);
        assert_eq!(c.scroll, 25);
    }

    #[test]
    fn ensure_visible_scrolls_up_to_the_row_top() {
        let mut c = list(100, 10, 25);
        c.scroll = 77;
        c.ensure_visible(3);
        assert_eq!(c.scroll, 30);
        c.ensure_visible(3);
        assert_eq!(c.scroll, 30);
    }

