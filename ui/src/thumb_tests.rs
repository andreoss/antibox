    use super::*;

    #[test]
    fn len_is_page_fraction_of_track() {
        let t = Thumb::new(0, 25, 0, 100);
        assert_eq!(t.len(200), 50);
    }

    #[test]
    fn len_respects_minimum() {
        let t = Thumb::new(0, 1, 0, 1_000_000);
        assert!(t.len(200) >= min_len().min(200));
        assert!(t.len(200) <= 200);
    }

