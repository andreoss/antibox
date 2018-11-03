    use super::*;

    #[test]
    fn test_new() {
        let p = PreviewWindow::new();
        assert!(!p.visible);
        assert_eq!(p.ws_count, 4);
    }

    #[test]
    fn test_hit_test() {
        let p = PreviewWindow {
            ws_count: 4,
            ..PreviewWindow::new()
        };
        assert_eq!(p.hit_test(Point::new(4, 4)), 0);
        assert_eq!(p.hit_test(Point::new(200, 4)), 1);
        assert_eq!(p.hit_test(Point::new(4, 120)), 2);
        assert_eq!(p.hit_test(Point::new(200, 120)), 3);
    }
