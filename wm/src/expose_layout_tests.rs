    use super::*;

    const fn overlaps(a: &Cell, b: &Cell) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    fn assert_disjoint(cells: &[Cell]) {
        for (i, a) in cells.iter().enumerate() {
            for b in cells.iter().skip(i + 1) {
                assert!(!overlaps(a, b), "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn test_empty_layout() {
        let (cells, w, h) = arrange(&[], 20);
        assert!(cells.is_empty());
        assert_eq!((w, h), (0, 0));
        assert!(layout(&[], (1280, 720), 20, 40, false).is_empty());
    }

    #[test]
    fn test_single_window_keeps_its_shape() {
        let (cells, w, h) = arrange(&[(800, 600)], 20);
        assert_eq!(cells.len(), 1);
        assert_eq!((w, h), (800, 600));
        assert_eq!(cells[0], Cell { x: 0, y: 0, w: 800, h: 600 });
    }

    #[test]
    fn test_windows_do_not_overlap() {
        let sizes = [
            (800, 600),
            (640, 480),
            (1024, 768),
            (320, 240),
            (500, 500),
            (900, 300),
            (200, 700),
        ];
        let (cells, _, _) = arrange(&sizes, 20);
        assert_eq!(cells.len(), sizes.len());
        assert_disjoint(&cells);
    }

    #[test]
    fn test_short_windows_stack_in_one_column() {
        let (cells, w, _) = arrange(&[(400, 200), (400, 200), (400, 800)], 20);
        assert_eq!(cells[0].x, cells[1].x);
        assert!(cells[1].y >= cells[0].y + cells[0].h + 20);
        assert!(cells[2].x >= cells[0].x + 400);
        assert_eq!(w, 820);
    }

    #[test]
    fn test_layout_fits_the_screen_and_preserves_aspect() {
        let sizes = [(1600, 1200), (1600, 1200), (1600, 1200)];
        let cells = layout(&sizes, (1280, 720), 20, 40, false);
        assert_disjoint(&cells);
        for c in &cells {
            assert!(c.x >= 0 && c.y >= 0);
            assert!(c.x + c.w <= 1280, "{c:?}");
            assert!(c.y + c.h <= 720, "{c:?}");
            let ratio = c.w as f64 / c.h as f64;
            assert!((ratio - 4.0 / 3.0).abs() < 0.05, "{ratio}");
        }
    }

    #[test]
    fn test_small_windows_are_not_upscaled() {
        let cells = layout(&[(100, 80)], (1280, 720), 20, 40, false);
        assert_eq!(cells[0].w, 100);
        assert_eq!(cells[0].h, 80);
        assert_eq!(cells[0].centre(), (640, 360));
    }

    #[test]
    fn test_upscale_fills_the_screen_when_allowed() {
        let cells = layout(&[(100, 80)], (1280, 720), 20, 40, true);
        assert_eq!(cells[0], Cell { x: 240, y: 40, w: 800, h: 640 });
        assert_eq!(cells[0].centre(), (640, 360));
    }

    #[test]
    fn test_scale_to_centres_the_block() {
        let mut cells = vec![Cell { x: 0, y: 0, w: 200, h: 100 }];
        let scale = scale_to(&mut cells, (200, 100), (1000, 500), 0, false);
        assert!((scale - 1.0).abs() < f64::EPSILON);
        assert_eq!(cells[0], Cell { x: 400, y: 200, w: 200, h: 100 });
    }

    #[test]
    fn test_nearest_moves_in_the_asked_direction() {
        let cells = vec![
            Cell { x: 0, y: 0, w: 100, h: 100 },
            Cell { x: 200, y: 0, w: 100, h: 100 },
            Cell { x: 0, y: 200, w: 100, h: 100 },
            Cell { x: 200, y: 200, w: 100, h: 100 },
        ];
        assert_eq!(nearest(&cells, 0, 1, 0), 1);
        assert_eq!(nearest(&cells, 0, 0, 1), 2);
        assert_eq!(nearest(&cells, 3, -1, 0), 2);
        assert_eq!(nearest(&cells, 3, 0, -1), 1);
        assert_eq!(nearest(&cells, 1, 1, 0), 1);
        assert_eq!(nearest(&[], 0, 1, 0), 0);
    }

    #[test]
    fn test_cell_at_hits_and_misses() {
        let cells = vec![
            Cell { x: 0, y: 0, w: 100, h: 100 },
            Cell { x: 200, y: 0, w: 100, h: 100 },
        ];
        assert_eq!(cell_at(&cells, 50, 50), Some(0));
        assert_eq!(cell_at(&cells, 250, 99), Some(1));
        assert_eq!(cell_at(&cells, 150, 50), None);
        assert_eq!(cell_at(&cells, 100, 100), None);
    }
