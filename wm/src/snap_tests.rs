    use super::*;

    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        w: 1200,
        h: 800,
    };

    #[test]
    fn zones_cover_halves_and_quadrants() {
        let left = SnapZone {
            h: Some(Horz::Left),
            v: None,
        };
        assert_eq!(zone_rect(left, AREA), Some(Rect::new(0, 0, 600, 800)));
        let tr = SnapZone {
            h: Some(Horz::Right),
            v: Some(Vert::Top),
        };
        assert_eq!(zone_rect(tr, AREA), Some(Rect::new(600, 0, 600, 400)));
        assert_eq!(zone_rect(SnapZone::default(), AREA), None);
    }

    #[test]
    fn compose_toggles_and_refines() {
        let left = compose(None, SnapDir::Left).unwrap();
        assert_eq!(left.h, Some(Horz::Left));
        let tl = compose(Some(left), SnapDir::Up).unwrap();
        assert_eq!(
            tl,
            SnapZone {
                h: Some(Horz::Left),
                v: Some(Vert::Top)
            }
        );
        let back = compose(Some(tl), SnapDir::Up).unwrap();
        assert_eq!(back, left);
        assert_eq!(compose(Some(left), SnapDir::Left), None);
        let right = compose(Some(left), SnapDir::Right).unwrap();
        assert_eq!(right.h, Some(Horz::Right));
    }

    #[test]
    fn zone_at_detects_edges_and_corners() {
        let mon = Rect::new(0, 0, 1280, 800);
        let l = zone_at(Point::new(0, 400), mon).unwrap();
        assert_eq!(l.h, Some(Horz::Left));
        assert_eq!(l.v, None);
        let t = zone_at(Point::new(640, 0), mon).unwrap();
        assert_eq!(t.h, None);
        assert_eq!(t.v, Some(Vert::Top));
        let tl = zone_at(Point::new(0, 0), mon).unwrap();
        assert_eq!(tl.h, Some(Horz::Left));
        assert_eq!(tl.v, Some(Vert::Top));
        assert!(zone_at(Point::new(640, 400), mon).is_none());
    }

    #[test]
    fn zone_for_window_detects_overhang() {
        let mon = Rect::new(0, 0, 1280, 800);
        assert_eq!(zone_for_window(Rect::new(0, 300, 200, 200), mon).unwrap().h, Some(Horz::Left));
        assert_eq!(zone_for_window(Rect::new(1080, 300, 200, 200), mon).unwrap().h, Some(Horz::Right));
        assert!(zone_for_window(Rect::new(500, 300, 200, 200), mon).is_none());
    }

