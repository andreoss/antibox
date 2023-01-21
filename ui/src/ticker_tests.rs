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
