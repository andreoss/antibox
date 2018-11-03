    use super::*;
    use antibox_gfx::mock::MockGraphics;

    #[test]
    fn measure_counts_explicit_newlines() {
        let g = MockGraphics::new(1);
        let (_, h1, l1) = measure(&g, "one", None);
        let (_, h3, l3) = measure(&g, "one\ntwo\nthree", None);
        assert_eq!(l1.len(), 1);
        assert_eq!(l3.len(), 3);
        assert_eq!(h3, h1 * 3);
    }

    #[test]
    fn measure_width_is_widest_line() {
        let g = MockGraphics::new(1);
        let (w, _, _) = measure(&g, "aa\naaaa\na", None);
        assert_eq!(w as i32, text_w(&g, "aaaa"));
    }

