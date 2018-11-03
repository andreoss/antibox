    use super::*;
    use antibox_gfx::mock::{MockCommand, MockGraphics};

    fn foregrounds(g: &MockGraphics) -> Vec<u32> {
        g.commands()
            .iter()
            .filter_map(|c| match c {
                MockCommand::SetForeground(p) => Some(*p),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn combo_button_sits_inside_field_and_draws_arrow() {
        let g = MockGraphics::new(200);
        let field_x = 10i16;
        let field_w = 100u16;
        let bx = combo_button(&g, field_x, 0, field_w, 20, false);

        assert!(bx > field_x && bx < field_x + field_w as i16);

        assert!(foregrounds(&g).contains(&theme::text()));
    }

    #[test]
    fn down_arrow_emits_colour() {
        let g = MockGraphics::new(50);
        down_arrow(&g, 10, 10, theme::disabled());
        assert!(foregrounds(&g).contains(&theme::disabled()));
    }

