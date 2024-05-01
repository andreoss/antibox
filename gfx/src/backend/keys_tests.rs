    use super::*;

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

    #[test]
    fn function_keysyms_produce_no_char() {
        for ks in [0xFF52u32, 0xFF54, 0xFF51, 0xFF53, 0xFF0D, 0xFF1B, 0xFFBE].iter().copied() {
            assert_eq!(keysym_to_char(ks), None);
        }
        assert_eq!(keysym_to_char(0xE4), Some('\u{e4}'));
        assert_eq!(keysym_to_char(0x0100_0416), Some('\u{416}'));
    }

    #[test]
    fn normalize_reports_unshifted_sym_and_shifted_char() {
        let m = mapping(&[(38, 'a' as u32, 'A' as u32)]);
        let k = normalize(38, 0x01, &m);
        assert_eq!(k.keysym, 'a' as u32);
        assert_eq!(k.ch, Some('A'));
        assert!(k.shift);
    }

