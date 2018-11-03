    use super::*;

    fn list_with(n: usize) -> WinListMenu {
        let mut m = WinListMenu::new();
        m.w = scaled(280) as u16;
        m.h = scaled(340) as u16;
        m.items = (0..n)
            .map(|i| WinListItem {
                title: format!("win{}", i),
                client_id: i as u32 + 1,
                workspace: 0,
            })
            .collect();
        m.rows = std::iter::once(Row::Header(0))
            .chain((0..n).map(Row::Win))
            .collect();
        m
    }

    #[test]
    fn test_new() {
        let m = WinListMenu::new();
        assert!(!m.visible);
        assert!(m.window.is_none());
        assert!(!m.owns_window(5));
    }

    #[test]
    fn window_at_survives_rows_referencing_missing_items() {
        let mut m = list_with(3);
        m.visible = true;
        m.items.truncate(1);
        for vr in 0..(m.rows.len() as i16 + 2) {
            let py = TOP_PAD + vr * row_h() + 2;
            let _ = m.window_at(Point::new(10, py as i32));
        }
    }

