    use super::*;
    use antibox_gfx::mock::{MockCommand, MockGraphics};

    #[test]
    fn bevel_emits_light_and_dark_edges() {
        let g = MockGraphics::new(100);
        bevel(&g, 0, 0, 40, 20, false);
        let fgs: Vec<u32> = g
            .commands()
            .iter()
            .filter_map(|c| match c {
                MockCommand::SetForeground(p) => Some(*p),
                _ => None,
            })
            .collect();
        assert!(fgs.contains(&light()));
        assert!(fgs.contains(&dark()));
    }

    #[test]
    fn sunken_inverts_the_outer_ring() {
        let g = MockGraphics::new(100);
        bevel(&g, 0, 0, 40, 20, true);
        let first = g.commands().iter().find_map(|c| match c {
            MockCommand::SetForeground(p) => Some(*p),
            _ => None,
        });
        assert_eq!(first, Some(shadow()));
    }

