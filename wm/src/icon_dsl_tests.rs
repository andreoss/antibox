use super::*;
use antibox_core::mock::{MockCommand, MockGraphics};

#[test]
fn test_draw_rect_scales_into_bounding_box() {
    let g = MockGraphics::new(1);
    let icon = Icon(&[Shape::Rect(0.25, 0.25, 0.5, 0.5)]);
    icon.draw(&g, 10, 20, 40, 20, 0xFF0000);
    assert_eq!(
        g.commands(),
        vec![
            MockCommand::SetForeground(0xFF0000),
            MockCommand::FillRect(20, 25, 20, 10),
        ]
    );
}

#[test]
fn test_draw_poly_scales_points() {
    let g = MockGraphics::new(1);
    const PTS: [(f32, f32); 3] = [(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)];
    let icon = Icon(&[Shape::Poly(&PTS)]);
    icon.draw(&g, 0, 0, 10, 10, 0x00FF00);
    assert_eq!(
        g.commands(),
        vec![
            MockCommand::SetForeground(0x00FF00),
            MockCommand::FillPolygon(vec![(0, 0), (10, 0), (5, 10)]),
        ]
    );
}

#[test]
fn test_draw_line_emits_a_thick_polygon() {
    let g = MockGraphics::new(1);
    let icon = Icon(&[Shape::Line(0.0, 0.5, 1.0, 0.5, 0.2)]);
    icon.draw(&g, 0, 0, 10, 10, 0x123456);
    assert!(g
        .commands()
        .iter()
        .any(|c| matches!(c, MockCommand::FillPolygon(pts) if pts.len() == 4)));
}

#[test]
fn test_draw_shaded_cycles_colours_per_shape() {
    let g = MockGraphics::new(1);
    let icon = Icon(&[
        Shape::Rect(0.0, 0.0, 0.5, 0.5),
        Shape::Rect(0.5, 0.5, 0.5, 0.5),
    ]);
    icon.draw_shaded(&g, 0, 0, 10, 10, &[0x111111, 0x222222]);
    let fg: Vec<u32> = g
        .commands()
        .iter()
        .filter_map(|c| match c {
            MockCommand::SetForeground(p) => Some(*p),
            _ => None,
        })
        .collect();
    assert_eq!(fg, vec![0x111111, 0x222222]);
}

#[test]
fn test_stroke_rect_draws_an_outline_not_a_fill() {
    let g = MockGraphics::new(1);
    let icon = Icon(&[Shape::Rect(0.0, 0.0, 1.0, 1.0)]);
    icon.stroke(&g, 5, 5, 10, 10, 0x000000);
    assert_eq!(
        g.commands(),
        vec![
            MockCommand::SetForeground(0x000000),
            MockCommand::DrawRect(5, 5, 10, 10),
        ]
    );
}

#[test]
fn test_stroke_poly_draws_closed_loop_of_lines() {
    let g = MockGraphics::new(1);
    const PTS: [(f32, f32); 3] = [(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)];
    let icon = Icon(&[Shape::Poly(&PTS)]);
    icon.stroke(&g, 0, 0, 10, 10, 0x000000);
    let commands = g.commands();
    let lines: Vec<&MockCommand> = commands
        .iter()
        .filter(|c| matches!(c, MockCommand::DrawLine(..)))
        .collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.contains(&&MockCommand::DrawLine(5, 10, 0, 0)));
}

#[test]
fn test_draw_outlined_fills_then_strokes() {
    let g = MockGraphics::new(1);
    let icon = Icon(&[Shape::Rect(0.0, 0.0, 1.0, 1.0)]);
    icon.draw_outlined(&g, 5, 5, 10, 10, 0xFFFFFF, 0x000000);
    assert_eq!(
        g.commands(),
        vec![
            MockCommand::SetForeground(0xFFFFFF),
            MockCommand::FillRect(5, 5, 10, 10),
            MockCommand::SetForeground(0x000000),
            MockCommand::DrawRect(5, 5, 10, 10),
        ]
    );
}

#[test]
fn test_power_icon_draws_without_crash() {
    let g = MockGraphics::new(1);
    POWER_ICON.draw_outlined(&g, 0, 0, 16, 16, 0xFFFFFF, 0x000000);
    assert!(!g.commands().is_empty());
}

#[test]
fn test_mic_icon_draws_without_crash() {
    let g = MockGraphics::new(1);
    MIC_ICON.draw_outlined(&g, 0, 0, 16, 16, 0xFFFFFF, 0x000000);
    assert!(!g.commands().is_empty());
}

#[test]
fn test_power_volume_and_mic_share_the_same_outlined_style() {
    for icon in [&POWER_ICON, &MIC_ICON] {
        let g = MockGraphics::new(1);
        icon.draw_outlined(&g, 0, 0, 16, 16, 0xFFFFFF, 0x000000);
        let cmds = g.commands();
        let last_fill = cmds
            .iter()
            .rposition(|c| matches!(c, MockCommand::FillRect(..) | MockCommand::FillPolygon(..)));
        let first_stroke = cmds
            .iter()
            .position(|c| matches!(c, MockCommand::DrawRect(..) | MockCommand::DrawLine(..)));
        assert!(last_fill.unwrap() < first_stroke.unwrap());
    }
}
