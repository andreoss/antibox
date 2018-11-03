    use super::*;

    fn combo() -> ComboBox {
        let mut c = ComboBox::new(
            ["File Cabinet", "C:\\", "CHICAGO"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        c.set_rect(0, 0, 160, 20);
        c
    }

    #[test]
    fn selects_first_item_by_default() {
        assert_eq!(combo().selected_text(), "File Cabinet");
    }

    #[test]
    fn dropdown_drops_below_by_default_and_above_when_upward() {
        let mut c = combo();
        let (_, dy_down, _, dh) = c.dropdown_rect();
        assert_eq!(dy_down, c.y + c.h as i16);
        c.open_upward = true;
        let (_, dy_up, _, _) = c.dropdown_rect();
        assert_eq!(dy_up, c.y - dh as i16);
    }

