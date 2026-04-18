    use super::*;
    use antibox_core::mock::MockDisplay;

    use std::sync::Arc;
    fn make_theme() -> ThemeColors {
        ThemeColors {
            workspace_active_bg: 0x5080D0,
            workspace_active_fg: 0xFFFFFF,
            workspace_normal_bg: 0x404040,
            workspace_normal_fg: 0xFFFFFF,
            ..ThemeColors::default()
        }
    }
    #[test]
    fn test_new() {
        let d = MockDisplay::new(1280, 720, 24);
        let conn = Arc::new(d) as Arc<dyn DisplayBackend>;
        let names = vec!["1".into(), "2".into(), "3".into()];
        let pane = WorkspacesPane::new(&conn, conn.root().read_id(), &names, make_theme()).unwrap();
        assert_eq!(pane.buttons.len(), 3);
        assert!(pane.buttons[0].active);
        assert!(!pane.buttons[1].active);
    }
    #[test]
    fn test_set_names_rebuilds_and_preserves_active() {
        let d = MockDisplay::new(1280, 720, 24);
        let conn = Arc::new(d) as Arc<dyn DisplayBackend>;
        let names = vec!["W1".into(), "W2".into(), "W3".into(), "W4".into()];
        let mut pane =
            WorkspacesPane::new(&conn, conn.root().read_id(), &names, make_theme()).unwrap();
        pane.set_active(2);
        pane.set_names(&["Main".into(), "Dev".into(), "Chat".into()]);
        assert_eq!(pane.buttons.len(), 3);
        assert_eq!(pane.buttons[0].label, "Main");
        assert_eq!(pane.buttons[2].label, "Chat");
        assert!(pane.buttons[2].active);
        assert!(!pane.buttons[0].active);
    }

    #[test]
    fn test_mini_icon_rect_gates_on_size() {
        assert!(mini_icon_rect(0, 0, 4, 4).is_none());
        let big = mini_icon_rect(10, 5, 40, 40);
        let (ix, iy, side) = big.expect("large mini fits an icon");
        assert!(side >= 4);
        assert!(ix >= 10);
        assert!(iy > 5 + mini_title_height(40) as i16 - 1);
        assert!(ix as i32 + side as i32 <= 50);
        assert!(iy as i32 + side as i32 <= 45);
    }

    fn pane_with(names: &[&str]) -> WorkspacesPane {
        let conn = Arc::new(MockDisplay::new(1280, 800, 24)) as Arc<dyn DisplayBackend>;
        let owned: Vec<String> = names.iter().map(|s| (*s).to_string()).collect();
        WorkspacesPane::new(&conn, conn.root().read_id(), &owned, make_theme()).unwrap()
    }

    #[test]
    fn test_columns_fit_the_wrap_width_and_balance() {
        assert_eq!(grid_columns(None, 26, 3, 9), 9);
        assert_eq!(grid_columns(Some(148), 26, 3, 9), 5);
        assert_eq!(grid_columns(Some(148), 26, 3, 3), 3);
        assert_eq!(grid_columns(Some(10), 26, 3, 9), 1);
        assert_eq!(grid_columns(Some(90), 26, 3, 4), 2, "four desktops split evenly");
        assert_eq!(grid_columns(Some(148), 26, 3, 7), 4, "seven desktops become 4+3");
    }

    #[test]
    fn test_wrapping_puts_the_desktops_on_several_rows() {
        let mut pane = pane_with(&["1", "2", "3", "4", "5", "6", "7", "8", "9"]);
        assert_eq!(pane.rows(), 1);
        let single = pane.total_width;
        pane.set_wrap_width(Some(148));
        assert!(pane.rows() > 1, "nine desktops must wrap in 148px");
        assert!(pane.total_width < single);
        assert!(pane.total_width <= 148, "{}", pane.total_width);
        let cols = pane.buttons.iter().filter(|b| b.rect.1 == pane.buttons[0].rect.1).count();
        assert!(cols * pane.rows() as usize >= 9);
        let first_row_y = pane.buttons[0].rect.1;
        assert!(pane.buttons[cols].rect.1 > first_row_y, "the next row sits lower");
        assert_eq!(pane.buttons[0].rect.0, 0, "a full row starts at the left");
        let last = pane.buttons.last().expect("cells exist");
        let right = pane.total_width as i16 - (last.rect.0 + last.rect.2 as i16);
        assert_eq!(
            pane.buttons[cols].rect.0, right,
            "a short row is centred, not left-aligned"
        );
        assert!(pane.buttons[cols].rect.0 > 0);
    }

    #[test]
    fn test_a_wrapped_pager_is_taller_than_one_row() {
        let mut pane = pane_with(&["1", "2", "3", "4", "5", "6", "7", "8", "9"]);
        let flat = pane.preferred_height();
        pane.set_wrap_width(Some(148));
        assert!(pane.preferred_height() > flat);
        assert_eq!(pane.pane_h, pane.preferred_height() as u16);
    }

    #[test]
    fn test_a_click_finds_the_cell_on_the_second_row() {
        let mut pane = pane_with(&["1", "2", "3", "4", "5", "6", "7", "8", "9"]);
        pane.set_wrap_width(Some(148));
        let cols = pane.buttons.iter().filter(|b| b.rect.1 == pane.buttons[0].rect.1).count();
        let target = &pane.buttons[cols + 1];
        let (x, y) = (
            i32::from(target.rect.0) + i32::from(target.rect.2) / 2,
            i32::from(target.rect.1) + i32::from(target.rect.3) / 2,
        );
        assert_eq!(cell_at(&pane.buttons, x, y, pane.rows()), Some(cols + 1));
        let top = &pane.buttons[1];
        let (tx, ty) = (
            i32::from(top.rect.0) + i32::from(top.rect.2) / 2,
            i32::from(top.rect.1) + i32::from(top.rect.3) / 2,
        );
        assert_eq!(cell_at(&pane.buttons, tx, ty, pane.rows()), Some(1));
    }
