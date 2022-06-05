    use super::*;

    const BS: u32 = 22;
    const DEL: u32 = 107;
    const LEFT: u32 = 113;
    const RIGHT: u32 = 114;
    const HOME: u32 = 110;
    const END: u32 = 115;

    fn line(text: &str, cursor: usize) -> InputLine {
        let mut edit = EditCore::new();
        edit.set_text(text);
        edit.set_cursor(cursor);
        InputLine {
            conn: Arc::new(antibox_gfx::mock::MockDisplay::new(800, 600, 24))
                as Arc<dyn RenderBackend>,
            window: Box::new(antibox_gfx::mock::MockWindow::new(1)),
            edit,
            focused: false,
            frameless: false,
            dragging: false,
            x: 0,
            y: 0,
            w: 200,
            h: 24,
        }
    }

    fn mapping(entries: &[(u8, u32, u32)]) -> KeyboardMapping {
        let kpc = 2usize;
        let mut keysyms = vec![0u32; 256 * kpc];
        for &(kc, lo, hi) in entries {
            let base = (kc as usize - 8) * kpc;
            keysyms[base] = lo;
            keysyms[base + 1] = hi;
        }
        KeyboardMapping {
            keysyms_per_keycode: kpc as u8,
            keysyms,
        }
    }

    fn empty_mapping() -> KeyboardMapping {
        mapping(&[
            (BS as u8, 0xFF08, 0xFF08),
            (DEL as u8, 0xFFFF, 0xFFFF),
            (LEFT as u8, 0xFF51, 0xFF51),
            (RIGHT as u8, 0xFF53, 0xFF53),
            (HOME as u8, 0xFF50, 0xFF50),
            (END as u8, 0xFF57, 0xFF57),
        ])
    }

    #[test]
    fn test_set_get_text() {
        let mut il = line("", 0);
        il.set_text("hello");
        assert_eq!(il.get_text(), "hello");
        assert_eq!(il.cursor_pos(), 5);
    }

    #[test]
    fn test_backspace_deletes_prev_char() {
        let mut il = line("hello", 5);
        il.handle_key(BS, 0, &empty_mapping());
        assert_eq!(il.get_text(), "hell");
        assert_eq!(il.cursor_pos(), 4);
    }

