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
        assert!(spk_w(h) >= 4, "h={}", h);
        assert!(bar_zone_w(h) >= 5, "h={}", h);
        assert!(bar_w(h) >= 2, "h={}", h);
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
fn test_mic_tooltip_shows_muted_and_readers() {
    let v = MicView::new(Some(AudioState {
        sink_muted: false,
        volume: 42,
        source_muted: true,
        readers: vec!["cap".to_string()],
    }));
    assert!(v.tooltip().contains("Mic: muted"));
    assert!(v.tooltip().contains("cap"));
}

#[test]
fn test_mic_tooltip_shows_readers() {
    let v = MicView::new(Some(AudioState {
        sink_muted: false,
        volume: 50,
        source_muted: false,
        readers: vec!["Firefox".to_string()],
    }));
    assert!(v.tooltip().contains("Firefox"));
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

fn rects_with_colour(commands: &[antibox_core::mock::MockCommand], colour: u32) -> usize {
    use antibox_core::mock::MockCommand;
    let mut fg = 0u32;
    let mut n = 0;
    for c in commands {
        match c {
            MockCommand::SetForeground(p) => fg = *p,
            MockCommand::FillRect(..) if fg == colour => n += 1,
            _ => {}
        }
    }
    n
}

fn lit_bars(volume: i32) -> usize {
    let g = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(volume, false, false))).draw(&g, 0, 18);
    rects_with_colour(&g.commands(), COLOR_ACTIVE)
}

#[test]
fn test_lit_bars_grow_with_volume() {
    assert!(lit_bars(10) < lit_bars(50));
    assert!(lit_bars(50) < lit_bars(90));
}

#[test]
fn test_full_volume_lights_three_bars() {
    assert_eq!(lit_bars(100), 3);
}

#[test]
fn test_zero_volume_lights_no_bars_but_keeps_them_visible() {
    assert_eq!(lit_bars(0), 0);
    for h in [18u16, 28] {
        let g = antibox_core::mock::MockGraphics::new(1);
        AudioView::new(Some(state(0, false, false))).draw(&g, 0, h);
        let cmds = g.commands();
        let unlit = rects_with_colour(&cmds, theme::shadow())
            + rects_with_colour(&cmds, theme::graph_bg());
        assert!(unlit >= 3, "h={}: unlit bars missing ({})", h, unlit);
    }
}

#[test]
fn test_muted_draws_no_bars_but_draws_shape() {
    let muted = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(90, true, false))).draw(&muted, 0, 18);
    assert_eq!(rects_with_colour(&muted.commands(), COLOR_ACTIVE), 0);
    assert_eq!(rects_with_colour(&muted.commands(), theme::graph_bg()), 0);
    assert!(
        polys_with_colour(&muted.commands(), COLOR_MUTED) >= 2,
        "muted icon draws the red mute cross"
    );
    assert!(
        polys_with_colour(&muted.commands(), theme::text()) >= 1,
        "speaker body stays in the text colour when muted"
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
fn test_source_mute_alone_does_not_change_volume_icon() {
    let plain = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(50, false, false))).draw(&plain, 0, 18);
    let with_mic = antibox_core::mock::MockGraphics::new(1);
    AudioView::new(Some(state(50, false, true))).draw(&with_mic, 0, 18);
    assert_eq!(
        with_mic.commands().len(),
        plain.commands().len(),
        "mic state must not leak into the volume icon"
    );
}

#[test]
fn test_mic_view_draws_only_when_recording() {
    let idle = antibox_core::mock::MockGraphics::new(1);
    MicView::new(Some(state(50, false, true))).draw(&idle, 0, 18);
    assert!(idle.commands().is_empty(), "no recording, no mic icon");
    assert!(!MicView::new(Some(state(50, false, true))).present());
    let rec = recording_state(false);
    assert!(MicView::new(Some(rec.clone())).present());
    let g = antibox_core::mock::MockGraphics::new(1);
    MicView::new(Some(rec)).draw(&g, 0, 18);
    assert!(!g.commands().is_empty());
}

fn recording_state(source_muted: bool) -> AudioState {
    AudioState {
        sink_muted: false,
        volume: 50,
        source_muted,
        readers: vec!["cap".to_string()],
    }
}

#[test]
fn test_muted_mic_view_draws_red_slash() {
    let g = antibox_core::mock::MockGraphics::new(1);
    MicView::new(Some(recording_state(true))).draw(&g, 0, 28);
    assert!(
        polys_with_colour(&g.commands(), COLOR_MUTED) >= 1,
        "muted recording mic must carry the red slash"
    );
}

#[test]
fn test_mic_view_stays_within_natural_width() {
    for h in [16u16, 18, 22, 28] {
        let g = antibox_core::mock::MockGraphics::new(1);
        MicView::new(Some(recording_state(false))).draw(&g, 0, h);
        assert!(max_x(&g.commands()) <= MicView::natural_width(h) as i16);
    }
}

#[test]
fn test_recording_surfaces_the_mic_view() {
    assert!(!MicView::new(Some(state(50, false, false))).present());
    assert!(MicView::new(Some(recording_state(false))).present());
    assert!(MicView::new(None).present() == false);
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
