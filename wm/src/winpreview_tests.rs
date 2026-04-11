    use super::*;

    #[test]
    fn test_fit_keeps_the_aspect_inside_the_box() {
        assert_eq!(fit((800, 600), (240, 160)), (213, 159));
        assert_eq!(fit((1600, 900), (240, 160)), (240, 135));
        assert_eq!(fit((100, 2000), (240, 160)), (8, 160));
        assert_eq!(fit((0, 0), (240, 160)), (1, 1));
    }

    #[test]
    fn test_fit_never_upscales() {
        assert_eq!(fit((100, 80), (240, 160)), (100, 80));
        assert_eq!(fit((240, 160), (240, 160)), (240, 160));
    }

    #[test]
    fn test_place_sits_above_the_anchor() {
        let bar = Rect::new(300, 760, 120, 24);
        let (x, y) = place(bar, (200, 140), (1280, 800), false);
        assert_eq!(x, 260);
        assert!(y + 140 <= bar.y, "{y}");
    }

    #[test]
    fn test_place_flips_below_when_there_is_no_room_above() {
        let top = Rect::new(20, 0, 120, 24);
        let (_, y) = place(top, (200, 140), (1280, 800), false);
        assert!(y >= top.y + top.h, "{y}");
    }

    #[test]
    fn test_place_beside_flips_at_the_right_edge() {
        let list = Rect::new(900, 100, 300, 400);
        let (x, _) = place(list, (200, 140), (1280, 800), true);
        assert!(x + 200 <= list.x, "{x}");
        let roomy = Rect::new(100, 100, 300, 400);
        let (x, _) = place(roomy, (200, 140), (1280, 800), true);
        assert!(x >= roomy.x + roomy.w, "{x}");
    }

    #[test]
    fn test_place_stays_on_screen() {
        let edge = Rect::new(1240, 780, 40, 20);
        let (x, y) = place(edge, (200, 140), (1280, 800), false);
        assert!(x >= 0 && x + 200 <= 1280, "{x}");
        assert!(y >= 0 && y + 140 <= 800, "{y}");
    }

    #[test]
    fn test_preview_opens_for_a_window_and_closes_again() {
        let conn = antibox_core::mock::MockDisplay::new(1280, 800, 24);
        let mut p = WindowPreview::new();
        p.show(&conn, 1234, None, Rect::new(10, 700, 100, 24), false);
        assert_eq!(p.shown_for(), Some(1234));
        let id = p.window.as_ref().map(|w| w.id()).unwrap_or(0);
        assert!(p.owns_window(id));
        p.show(&conn, 1234, None, Rect::new(10, 700, 100, 24), false);
        assert_eq!(p.window.as_ref().map(|w| w.id()), Some(id));
        p.hide(&conn);
        assert_eq!(p.shown_for(), None);
        assert!(!p.owns_window(id));
    }

    #[test]
    fn test_a_window_without_a_picture_shows_a_placeholder() {
        let conn = antibox_core::mock::MockDisplay::new(1280, 800, 24);
        let mut p = WindowPreview::new();
        let icon = crate::icon_render::default_icon(16, 0);
        p.show(&conn, 0, Some(icon), Rect::new(10, 700, 100, 24), false);
        assert_eq!(p.shown_for(), Some(0));
        assert_eq!(p.parts.len(), 1);
        assert!(p.parts[0].art.is_none());
        assert!(p.parts[0].icon.is_some());
        assert!(!p.complete());
        p.paint(&conn);
    }

    #[test]
    fn test_pump_retries_until_the_picture_arrives() {
        let conn = antibox_core::mock::MockDisplay::new(1280, 800, 24);
        let mut p = WindowPreview::new();
        p.show(&conn, 0, None, Rect::new(10, 700, 100, 24), false);
        assert_eq!(p.shown_for(), Some(0));
        assert!(!p.complete());
        assert_eq!(p.tries, RETRIES);
        for _ in 0..RETRIES {
            p.pump(&conn);
        }
        assert_eq!(p.tries, 0);
        p.pump(&conn);
        assert_eq!(p.tries, 0);

        p.show(&conn, 7, None, Rect::new(10, 700, 100, 24), false);
        assert_eq!(p.shown_for(), Some(7));
        assert!(p.complete());
        let before = p.tries;
        p.pump(&conn);
        assert_eq!(p.tries, before);
    }

    #[test]
    fn test_hide_forgets_the_wanted_window() {
        let conn = antibox_core::mock::MockDisplay::new(1280, 800, 24);
        let mut p = WindowPreview::new();
        p.show(&conn, 0, None, Rect::new(10, 700, 100, 24), false);
        p.hide(&conn);
        assert!(p.wanted.is_none());
        p.pump(&conn);
        assert_eq!(p.shown_for(), None);
    }

    #[test]
    fn test_scale_into_maps_the_screen_to_the_box() {
        let screen = Rect::new(0, 0, 1280, 800);
        let full = scale_into(Rect::new(0, 0, 1280, 800), screen, (128, 80));
        assert_eq!(full, Rect::new(0, 0, 128, 80));
        let half = scale_into(Rect::new(640, 400, 640, 400), screen, (128, 80));
        assert_eq!(half, Rect::new(64, 40, 64, 40));
        let tiny = scale_into(Rect::new(10, 10, 4, 4), screen, (128, 80));
        assert_eq!((tiny.w, tiny.h), (1, 1));
    }

    #[test]
    fn test_scale_into_keeps_windows_inside_the_box() {
        let screen = Rect::new(0, 0, 1280, 800);
        for rect in [
            Rect::new(-200, -100, 600, 400),
            Rect::new(1200, 700, 600, 400),
            Rect::new(1279, 799, 1, 1),
        ] {
            let at = scale_into(rect, screen, (128, 80));
            assert!(at.x >= 0 && at.y >= 0, "{at:?}");
            assert!(at.x + at.w <= 128 && at.y + at.h <= 80, "{at:?}");
            assert!(at.w >= 1 && at.h >= 1, "{at:?}");
        }
    }

    #[test]
    fn test_desk_preview_holds_one_part_per_window() {
        let conn = antibox_core::mock::MockDisplay::new(1280, 800, 24);
        let mut p = WindowPreview::new();
        let screen = Rect::new(0, 0, 1280, 800);
        let wins = vec![
            DeskWin { client: 11, rect: Rect::new(0, 0, 400, 300), icon: None },
            DeskWin { client: 12, rect: Rect::new(600, 200, 400, 300), icon: None },
        ];
        p.show_desk(&conn, 2, wins, screen, Rect::new(10, 780, 60, 20), false);
        assert_eq!(p.parts.len(), 2);
        assert!(p.parts.iter().all(|part| part.art.is_some()));
        assert_eq!(p.shown_for(), None);
        assert!(p.window.is_some());

        p.show_desk(&conn, 3, Vec::new(), screen, Rect::new(10, 780, 60, 20), false);
        assert!(p.parts.is_empty());
        assert!(p.window.is_some());
    }

    #[test]
    fn test_desk_preview_falls_back_when_a_window_has_no_picture() {
        let conn = antibox_core::mock::MockDisplay::new(1280, 800, 24);
        let mut p = WindowPreview::new();
        let screen = Rect::new(0, 0, 1280, 800);
        let icon = crate::icon_render::default_icon(16, 0);
        let wins = vec![
            DeskWin {
                client: 0,
                rect: Rect::new(0, 0, 400, 300),
                icon: Some(icon),
            },
            DeskWin { client: 12, rect: Rect::new(600, 0, 400, 300), icon: None },
        ];
        p.show_desk(&conn, 1, wins, screen, Rect::new(10, 780, 60, 20), false);
        assert_eq!(p.parts.len(), 2);
        assert!(p.parts[0].art.is_none());
        assert!(p.parts[0].icon.is_some());
        assert!(p.parts[1].art.is_some());
        assert!(p.parts[1].icon.is_none());
        p.paint(&conn);
    }
