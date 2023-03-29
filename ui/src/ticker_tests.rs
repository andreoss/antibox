use super::*;
use antibox_gfx::backend::FontSpec;
use antibox_gfx::mock::MockGraphics;

#[test]
fn test_scroll_walks_pixels_then_chars_and_wraps() {
    let g = MockGraphics::new(200);
    let _ = g.set_font(&FontSpec::ui(10));
    let label = "LongWindowTitle";
    let char_w = g.text_width("L").unwrap() as usize;
    let (t0, s0) = scroll_at(&g, label, 0, 40);
    assert!(t0.starts_with('L'));
    assert_eq!(s0, 0);
    let (t1, s1) = scroll_at(&g, label, 1, 40);
    assert!(t1.starts_with('L'));
    assert_eq!(s1, 1);
    let (t2, s2) = scroll_at(&g, label, char_w, 40);
    assert!(t2.starts_with('o'));
    assert_eq!(s2, 0);
    let total = g.text_width(&format!("{}{}", label, GAP)).unwrap() as usize;
    let (tw, sw) = scroll_at(&g, label, total, 40);
    assert_eq!((tw, sw), (t0.clone(), s0));
    let covered = g.text_width(&t0).unwrap();
    assert!(covered >= 40);
}

#[test]
fn test_fit_latches_and_gates() {
    let g = MockGraphics::new(200);
    let _ = g.set_font(&FontSpec::ui(10));

    set_enabled(true);
    let _ = advance();
    assert!(matches!(
        fit(&g, "ok", 100),
        Fit::Plain(Cow::Borrowed("ok"))
    ));
    assert!(!active());
    assert!(matches!(
        fit(&g, "a very long overflowing label", 20),
        Fit::Scroll { .. }
    ));
    assert!(active());
    assert!(advance().panel);
    assert!(!active());
    assert!(!advance().any());

    let _ = fit_on(Surface::Title, &g, "a very long overflowing label", 20);
    let s = advance();
    assert!(s.title);
    assert!(!s.panel);
    let _ = fit_on(Surface::Panel, &g, "a very long overflowing label", 20);
    let s = advance();
    assert!(s.panel && !s.title);

    set_enabled(false);
    match fit(&g, "a very long overflowing label", 40) {
        Fit::Plain(out) => {
            assert!(out.ends_with('\u{2026}'));
        }
        Fit::Scroll { .. } => panic!("ticker off must not scroll"),
    }
    assert!(!active());
    let _ = advance();
    set_enabled(true);
}

#[test]
fn test_measurement_is_cached_across_frames() {
    let g = MockGraphics::new(200);
    let _ = g.set_font(&FontSpec::ui(10));
    let label = "LongWindowTitle";
    let before = g.text_width_calls();
    let _ = scroll_at(&g, label, 0, 40);
    let measured = g.text_width_calls() - before;
    assert!(measured > 0);
    let before = g.text_width_calls();
    let mut phases = vec![1usize, 2, 3, 5, 8, 13, 21, 34];
    phases.extend((0..40).map(|p| p * 7));
    for phase in phases {
        let a = scroll_at(&g, label, phase, 40);
        let b = scroll_at(&g, label, phase, 40);
        assert_eq!(a, b);
    }
    assert_eq!(g.text_width_calls() - before, 0);
}

#[test]
fn test_cache_tracks_font_change() {
    let g = MockGraphics::new(200);
    let _ = g.set_font(&FontSpec::ui(10));
    let label = "LongWindowTitle";
    let _ = scroll_at(&g, label, 0, 40);
    let _ = g.set_font(&FontSpec::ui(20));
    let before = g.text_width_calls();
    let _ = scroll_at(&g, label, 0, 40);
    assert!(g.text_width_calls() - before > 0);
    let before = g.text_width_calls();
    let _ = scroll_at(&g, label, 3, 40);
    assert_eq!(g.text_width_calls() - before, 0);
}

#[test]
fn test_scroll_output_matches_uncached_measurement() {
    let g = MockGraphics::new(200);
    let _ = g.set_font(&FontSpec::ui(10));
    let label = "Antibox window with a very long title";
    for phase in [0usize, 1, 9, 17, 40, 123, 999] {
        let cached = scroll_at(&g, label, phase, 40);
        let expected = {
            let cycle: Vec<char> = label.chars().chain(GAP.chars()).collect();
            let mut buf = String::new();
            let mut prev = 0u32;
            let widths: Vec<u32> = cycle
                .iter()
                .map(|&c| {
                    buf.push(c);
                    let cum = g.text_width(&buf).unwrap_or(0);
                    let w = cum.saturating_sub(prev);
                    prev = cum;
                    w
                })
                .collect();
            let total: usize = widths.iter().map(|&w| w as usize).sum();
            let mut rem = phase % total;
            let mut first = 0usize;
            for (i, &w) in widths.iter().enumerate() {
                if rem < w as usize {
                    first = i;
                    break;
                }
                rem -= w as usize;
            }
            let shift = rem as u16;
            let want = 40usize + shift as usize;
            let mut out = String::new();
            let mut covered = 0usize;
            let mut i = first;
            while covered < want {
                out.push(cycle[i]);
                covered += widths[i] as usize;
                i = (i + 1) % cycle.len();
            }
            (out, shift)
        };
        assert_eq!(cached, expected);
    }
}

#[test]
fn test_targets_are_recorded_per_surface() {
    let g = MockGraphics::new(7);
    let _ = g.set_font(&FontSpec::ui(10));
    set_enabled(true);
    let _ = advance();
    let _ = fit_on_at(Surface::Panel, &g, "a very long overflowing label", 20, 11);
    let _ = fit_on_at(Surface::Panel, &g, "a very long overflowing label", 20, 11);
    let _ = fit_on_at(Surface::Title, &g, "another long overflowing label", 20, 22);
    let s = advance();
    assert!(s.panel && s.title);
    assert_eq!(s.panel_targets, vec![11]);
    assert_eq!(s.title_targets, vec![22]);
    let s = advance();
    assert!(!s.any());
    assert!(s.panel_targets.is_empty());
    assert!(s.title_targets.is_empty());
    set_enabled(true);
}

#[test]
fn test_rearm_restores_targets() {
    let g = MockGraphics::new(7);
    let _ = g.set_font(&FontSpec::ui(10));
    set_enabled(true);
    let _ = advance();
    let _ = fit_on_at(Surface::Title, &g, "a very long overflowing label", 20, 33);
    let fired = advance();
    assert_eq!(fired.title_targets, vec![33]);
    rearm(fired);
    let again = advance();
    assert_eq!(again.title_targets, vec![33]);
    assert!(advance().title_targets.is_empty());
    set_enabled(true);
}
