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


    #[test]
    fn cyrillic_keysyms_map_to_unicode() {
        assert_eq!(keysym_to_char(0x06C1), Some('а'));
        assert_eq!(keysym_to_char(0x06E1), Some('А'));
        assert_eq!(keysym_to_char(0x06D1), Some('я'));
        assert_eq!(keysym_to_char(0x06F1), Some('Я'));
        assert_eq!(keysym_to_char(0x06A3), Some('ё'));
        assert_eq!(keysym_to_char(0x06B3), Some('Ё'));
        assert_eq!(keysym_to_char(0x06A6), Some('і'));
        assert_eq!(keysym_to_char(0x06AD), Some('ґ'));
        assert_eq!(keysym_to_char(0x06B0), Some('№'));
        assert_eq!(keysym_to_char(0x06A0), None);
    }

    #[test]
    fn cyrillic_block_is_dense_and_in_range() {
        let mut mapped = 0;
        for ks in 0x06A1u32..=0x06FF {
            let Some(c) = keysym_to_char(ks) else { continue };
            mapped += 1;
            let u = c as u32;
            assert!(
                (0x0400..=0x04FF).contains(&u) || u == 0x2116,
                "keysym {ks:#06x} mapped outside cyrillic: {u:#06x}"
            );
        }
        assert_eq!(mapped, 95);
    }

    #[test]
    fn legacy_keysyms_beyond_cyrillic_also_map() {
        assert_eq!(keysym_to_char(0x07E1), Some('α'));
        assert_eq!(keysym_to_char(0x07C1), Some('Α'));
        assert_eq!(keysym_to_char(0x01E8), Some('č'));
        assert_eq!(keysym_to_char(0x0CE0), Some('א'));
    }

    fn mapping4(kc: u8, cols: [u32; 4]) -> KeyboardMapping {
        let kpc = 4usize;
        let mut keysyms = vec![0u32; 256 * kpc];
        let base = (kc as usize - 8) * kpc;
        keysyms[base..base + 4].copy_from_slice(&cols);
        KeyboardMapping {
            keysyms_per_keycode: kpc as u8,
            keysyms,
        }
    }

    #[test]
    fn second_group_yields_its_own_letters() {
        let m = mapping4(38, ['a' as u32, 'A' as u32, 0x06C6, 0x06E6]);
        let latin = normalize(38, 0x00, &m);
        assert_eq!(latin.ch, Some('a'));
        let cyr = normalize(38, 1 << 13, &m);
        assert_eq!(cyr.ch, Some('ф'));
        let cyr_shift = normalize(38, (1 << 13) | 0x01, &m);
        assert_eq!(cyr_shift.ch, Some('Ф'));
    }

    #[test]
    fn bindings_keep_using_the_first_group() {
        let m = mapping4(38, ['a' as u32, 'A' as u32, 0x06C6, 0x06E6]);
        assert_eq!(normalize(38, 1 << 13, &m).keysym, 'a' as u32);
    }

    #[test]
    fn missing_group_column_falls_back() {
        let m = mapping(&[(38, 'a' as u32, 'A' as u32)]);
        assert_eq!(normalize(38, 1 << 13, &m).ch, Some('a'));
    }
