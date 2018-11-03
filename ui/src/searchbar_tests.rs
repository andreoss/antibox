    use super::*;

    const RET: u32 = 36;
    const ESC: u32 = 9;

    fn mapping(entries: &[(u8, u32)]) -> KeyboardMapping {
        let kpc = 2usize;
        let mut keysyms = vec![0u32; 256 * kpc];
        for &(kc, ks) in entries {
            keysyms[(kc as usize - 8) * kpc] = ks;
        }
        KeyboardMapping {
            keysyms_per_keycode: kpc as u8,
            keysyms,
        }
    }

    fn keys() -> KeyboardMapping {
        mapping(&[(RET as u8, 0xFF0D), (ESC as u8, 0xFF1B), (38, 'a' as u32)])
    }

    fn bar() -> SearchBar {
        let conn: Arc<dyn RenderBackend> =
            Arc::new(antibox_gfx::mock::MockDisplay::new(800, 600, 24));
        SearchBar::new(&conn, 1, 0, 0, 200, 24).unwrap()
    }

    #[test]
    fn typing_reports_changed() {
        let mut sb = bar();
        assert_eq!(sb.handle_key(38, 0, &keys()), SearchEvent::Changed);
        assert_eq!(sb.text(), "a");
    }

    #[test]
    fn return_submits() {
        let mut sb = bar();
        sb.set_text("abc");
        assert_eq!(sb.handle_key(RET, 0, &keys()), SearchEvent::Submitted);
        assert_eq!(sb.text(), "abc");
    }

