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

