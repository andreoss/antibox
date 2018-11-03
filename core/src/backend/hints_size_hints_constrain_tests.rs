    use super::size_hints_flags::*;
    use super::SizeHints;

    fn h() -> SizeHints {
        SizeHints::default()
    }

    #[test]
    fn test_constrain_min_max() {
        let mut s = h();
        s.flags = P_MIN_SIZE | P_MAX_SIZE;
        s.min_width = 100;
        s.min_height = 80;
        s.max_width = 400;
        s.max_height = 300;
        assert_eq!(s.constrain(50, 50), (100, 80));
        assert_eq!(s.constrain(999, 999), (400, 300));
        assert_eq!(s.constrain(200, 200), (200, 200));
    }

    #[test]
    fn test_constrain_max_zero_is_unlimited() {
        let mut s = h();
        s.flags = P_MAX_SIZE;
        s.max_width = 0;
        s.max_height = 0;
        assert_eq!(s.constrain(5000, 5000), (5000, 5000));
    }

