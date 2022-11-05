use super::*;

fn state(volume: i32, sink_muted: bool, source_muted: bool) -> AudioState {
    AudioState {
        sink_muted,
        volume,
        source_muted,
        readers: Vec::new(),
    }
}

#[test]
fn test_present_false_without_state() {
    assert!(!AudioView::new(None).present());
}

#[test]
fn test_present_true_with_state() {
    assert!(AudioView::new(Some(state(50, false, false))).present());
}

#[test]
fn test_glyph_occupies_a_square_slot() {
    for h in [16u16, 18, 22, 28] {
        assert_eq!(AudioView::natural_width(h), h, "h={}", h);
        assert!(box_w(h) >= 1, "h={}", h);
        assert!(cone_w(h) >= 1, "h={}", h);
        assert!(wave_zone_w(h) >= 1, "h={}", h);
    }
}

#[test]
fn test_tooltip_empty_without_state() {
    assert_eq!(AudioView::new(None).tooltip(), "");
}

#[test]
fn test_tooltip_shows_volume() {
    let v = AudioView::new(Some(state(42, false, false)));
    assert!(v.tooltip().contains("42%"));
}

#[test]
fn test_tooltip_shows_muted() {
    let v = AudioView::new(Some(state(42, true, false)));
    assert!(v.tooltip().contains("Muted"));
}

#[test]
fn test_tooltip_shows_mic_muted() {
    let v = AudioView::new(Some(state(42, false, true)));
    assert!(v.tooltip().contains("Mic: muted"));
}

#[test]
fn test_tooltip_shows_readers() {
    let v = AudioView::new(Some(AudioState {
        sink_muted: false,
        volume: 50,
        source_muted: false,
        readers: vec!["Firefox".to_string()],
    }));
    assert!(v.tooltip().contains("Firefox"));
}

fn fill_polygon_count(commands: &[antibox_core::mock::MockCommand]) -> usize {
    use antibox_core::mock::MockCommand;
    commands
        .iter()
        .filter(|c| matches!(c, MockCommand::FillPolygon(_)))
        .count()
}

fn polys_with_colour(commands: &[antibox_core::mock::MockCommand], colour: u32) -> usize {
    use antibox_core::mock::MockCommand;
    let mut fg = 0u32;
    let mut n = 0;
    for c in commands {
        match c {
            MockCommand::SetForeground(p) => fg = *p,
            MockCommand::FillPolygon(_) if fg == colour => n += 1,
            _ => {}
        }
    }
    n
}

fn lit_segments(volume: i32) -> usize {
    let g = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(volume, false, false))).draw(&g, 0, 18);
    polys_with_colour(&g.commands(), COLOR_ACTIVE) / 2
}

#[test]
fn test_lit_wave_segments_grow_with_volume() {
    assert!(lit_segments(10) < lit_segments(50));
    assert!(lit_segments(50) < lit_segments(90));
}

#[test]
fn test_full_volume_lights_three_segments() {
    assert_eq!(lit_segments(100), 3);
}

#[test]
fn test_zero_volume_lights_no_segments_but_keeps_them_visible() {
    assert_eq!(lit_segments(0), 0);
    let g = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(0, false, false))).draw(&g, 0, 18);
    assert_eq!(polys_with_colour(&g.commands(), theme::shadow()), 6);
}

#[test]
fn test_muted_draws_no_waves_but_draws_shape() {
    let muted = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(90, true, false))).draw(&muted, 0, 18);
    let unmuted = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(90, false, false))).draw(&unmuted, 0, 18);
    assert!(
        fill_polygon_count(&muted.commands()) < fill_polygon_count(&unmuted.commands()),
        "muted icon swaps wave segments for a mute mark, so it draws fewer polygons"
    );
    assert!(
        polys_with_colour(&muted.commands(), COLOR_MUTED) >= 3,
        "muted icon draws the red speaker and the mute mark"
    );
}

#[test]
fn test_active_colour_distinct_from_shape_colour() {
    assert_ne!(
        COLOR_ACTIVE,
        theme::text(),
        "wave colour must contrast, or it is invisible against the speaker body"
    );
}

#[test]
fn test_muted_colour_distinct_from_shape_colour() {
    assert_ne!(
        COLOR_MUTED,
        theme::text(),
        "mute mark must contrast against the default speaker colour"
    );
}

#[test]
fn test_draw_all_states_no_crash() {
    let g = antibox_core::mock::MockGraphics::new(1);
    for (vol, muted, src_muted) in [
        (0, false, false),
        (20, false, false),
        (50, false, true),
        (90, true, false),
    ] {
        let v = AudioView::new(Some(state(vol, muted, src_muted)));
        v.draw(&g, 0, 18);
    }
    AudioView::new(None).draw(&g, 0, 18);
}

fn max_x(commands: &[antibox_core::mock::MockCommand]) -> i16 {
    use antibox_core::mock::MockCommand;
    commands
        .iter()
        .flat_map(|c| match c {
            MockCommand::FillRect(x, _, w, _) => vec![x + *w as i16],
            MockCommand::DrawLine(x1, _, x2, _) => vec![*x1, *x2],
            MockCommand::FillPolygon(pts) => pts.iter().map(|(x, _)| *x).collect(),
            _ => vec![],
        })
        .max()
        .unwrap_or(0)
}

#[test]
fn test_drawn_extent_fits_within_natural_width() {
    for h in [16u16, 18, 22, 28] {
        for (vol, muted) in [(90, false), (5, false), (0, true)] {
            let g = antibox_core::mock::MockGraphics::new(1);
            let v = AudioView::new(Some(state(vol, muted, false)));
            v.draw(&g, 0, h);
            let reached = max_x(&g.commands());
            assert!(
                reached <= AudioView::natural_width(h) as i16,
                "h={} vol={} muted={}: drew to x={} but natural_width={}",
                h,
                vol,
                muted,
                reached,
                AudioView::natural_width(h)
            );
        }
    }
}
