    use super::*;
    use crate::mpris::Track;
    use antibox_core::mock::MockGraphics;

    fn player() -> Player {
        let mut p = Player::new("org.mpris.MediaPlayer2.mpv");
        p.identity = "mpv".to_string();
        p.status = Playback::Playing;
        p.can_control = true;
        p.can_next = true;
        p.can_previous = true;
        p.track = Track {
            title: "Fratres".to_string(),
            artist: "Arvo Pärt".to_string(),
            album: "Tabula Rasa".to_string(),
            length: 672_000_000,
        };
        p
    }

    fn view() -> MprisView {
        MprisView::new(Some(player()), 65_000_000)
    }

    #[test]
    fn test_no_player_shows_nothing() {
        let empty = MprisView::default();
        assert!(!empty.present());
        assert!(empty.slots().is_empty());
        assert_eq!(empty.natural_width(20), 0);
        assert!(empty.timing().is_empty());
        assert!(empty.tooltip().is_empty());
    }

    #[test]
    fn test_slot_order_puts_timing_next_to_play() {
        assert_eq!(
            view().slots(),
            vec![
                MediaSlot::Button(MediaButton::Previous),
                MediaSlot::Button(MediaButton::PlayPause),
                MediaSlot::Timing,
                MediaSlot::Button(MediaButton::Next),
            ]
        );
    }

    #[test]
    fn test_disabled_buttons_are_hidden() {
        let mut p = player();
        p.can_previous = false;
        let v = MprisView::new(Some(p), 0);
        assert_eq!(
            v.slots(),
            vec![
                MediaSlot::Button(MediaButton::PlayPause),
                MediaSlot::Timing,
                MediaSlot::Button(MediaButton::Next),
            ]
        );
    }

    #[test]
    fn test_uncontrollable_player_keeps_only_timing() {
        let mut p = player();
        p.can_control = false;
        let v = MprisView::new(Some(p), 0);
        assert_eq!(v.slots(), vec![MediaSlot::Timing]);
        assert!(v.present());
    }

    #[test]
    fn test_untimed_uncontrollable_player_is_hidden() {
        let mut p = player();
        p.can_control = false;
        p.track.length = 0;
        let v = MprisView::new(Some(p), 0);
        assert!(!v.present());
    }

    #[test]
    fn test_timing_formats() {
        assert_eq!(view().timing(), "1:05 / 11:12");
        let mut p = player();
        p.track.length = 0;
        assert_eq!(MprisView::new(Some(p.clone()), 65_000_000).timing(), "1:05");
        assert_eq!(MprisView::new(Some(p), 0).timing(), "");
        assert_eq!(MprisView::new(Some(player()), 0).timing(), "11:12");
    }

    #[test]
    fn test_button_slots_are_square_and_timing_is_text_wide() {
        let v = view();
        let h = 20u16;
        assert_eq!(v.slot_width(MediaSlot::Button(MediaButton::Next), h), h);
        assert!(v.slot_width(MediaSlot::Timing, h) > h);
        let total: u16 = v.slots().into_iter().map(|s| v.slot_width(s, h)).sum();
        assert_eq!(v.natural_width(h), total);
    }

    #[test]
    fn test_slot_at_walks_the_visible_slots() {
        let v = view();
        let h = 20u16;
        assert_eq!(
            v.slot_at(0, h),
            Some(MediaSlot::Button(MediaButton::Previous))
        );
        assert_eq!(
            v.slot_at(h as i16 + 1, h),
            Some(MediaSlot::Button(MediaButton::PlayPause))
        );
        assert_eq!(v.slot_at(h as i16 * 2 + 1, h), Some(MediaSlot::Timing));
        assert_eq!(
            v.slot_at(v.natural_width(h) as i16 - 1, h),
            Some(MediaSlot::Button(MediaButton::Next))
        );
        assert_eq!(v.slot_at(v.natural_width(h) as i16, h), None);
        assert_eq!(v.slot_at(-1, h), None);
    }

    #[test]
    fn test_tooltips_name_the_track_and_the_buttons() {
        let v = view();
        assert_eq!(
            v.slot_tooltip(MediaSlot::Button(MediaButton::Previous)),
            "Previous track"
        );
        assert_eq!(
            v.slot_tooltip(MediaSlot::Button(MediaButton::Next)),
            "Next track"
        );
        let tip = v.slot_tooltip(MediaSlot::Button(MediaButton::PlayPause));
        assert!(tip.contains("mpv"));
        assert!(tip.contains("Fratres"));
        assert!(tip.contains("Arvo Pärt"));
        assert_eq!(v.slot_tooltip(MediaSlot::Timing), tip);
    }

    #[test]
    fn test_commands_of_buttons() {
        assert_eq!(MediaButton::Previous.command(), Command::Previous);
        assert_eq!(MediaButton::PlayPause.command(), Command::PlayPause);
        assert_eq!(MediaButton::Next.command(), Command::Next);
    }

    #[test]
    fn test_can_seek_follows_the_player() {
        assert!(!view().can_seek());
        let mut p = player();
        p.can_seek = true;
        assert!(MprisView::new(Some(p), 0).can_seek());
        assert!(!MprisView::default().can_seek());
    }

    #[test]
    fn test_playing_flag_drives_the_glyph() {
        let v = view();
        assert!(v.playing());
        let mut paused = player();
        paused.status = Playback::Paused;
        assert!(!MprisView::new(Some(paused), 0).playing());
    }

    #[test]
    fn test_draw_does_not_panic() {
        let g = MockGraphics::new(1);
        view().draw(&g, 0, 20);
        MprisView::default().draw(&g, 0, 20);
    }
