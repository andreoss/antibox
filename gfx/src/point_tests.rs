    use super::*;

    #[test]
    fn test_point_new() {
        let p = Point::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }
