    use super::*;

    #[test]
    fn test_toward_submenu_diagonal_travel_is_safe() {
        let sub = Point::new(125, 320);
        assert!(toward_submenu(
            Point::new(60, 300),
            Point::new(80, 340),
            sub,
            200,
            120
        ));
        assert!(toward_submenu(
            Point::new(80, 340),
            Point::new(110, 360),
            sub,
            200,
            120
        ));
    }

    #[test]
    fn test_toward_submenu_vertical_or_away_switches() {
        let sub = Point::new(125, 320);
        assert!(!toward_submenu(Point::new(60, 300), Point::new(60, 360), sub, 200, 120));
        assert!(!toward_submenu(Point::new(60, 300), Point::new(40, 340), sub, 200, 120));
        assert!(!toward_submenu(Point::new(60, 300), Point::new(80, 800), sub, 200, 120));
    }
