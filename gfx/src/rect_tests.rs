    use super::*;

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0, 0, 100, 100);
        assert!(r.contains(Point::new(50, 50)));
        assert!(!r.contains(Point::new(100, 100)));
        assert!(!r.contains(Point::new(-1, 50)));
    }

    #[test]
    fn test_rect_contains_xy_half_open() {
        let r = Rect::new(10, 20, 30, 40);
        assert!(r.contains_xy(10, 20));
        assert!(r.contains_xy(39, 59));
        assert!(!r.contains_xy(40, 20));
        assert!(!r.contains_xy(10, 60));
        assert!(!r.contains_xy(9, 20));
        assert!(!r.contains_xy(10, 19));
    }

