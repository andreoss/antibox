    use super::parse_names;
    use super::super::bindings::{xcb_charinfo_t, xcb_query_font_reply_t};
    use std::mem::size_of;

    #[test]
    fn query_font_reply_matches_wire_layout() {
        assert_eq!(size_of::<xcb_charinfo_t>(), 12);
        assert_eq!(size_of::<xcb_query_font_reply_t>(), 60);
    }

    #[test]
    fn parses_length_prefixed_names() {
        let data = [
            11, b'-', b'm', b'i', b's', b'c', b'-', b'f', b'i', b'x', b'e', b'd',
            16, b'-', b'a', b'd', b'o', b'b', b'e', b'-', b'h', b'e', b'l', b'v', b'e', b't', b'i', b'c', b'a',
        ];
        let names = parse_names(&data, 2);
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "-misc-fixed");
        assert_eq!(names[1], "-adobe-helvetica");
    }


    const fn font(min_char: u16, max_char: u16, min_byte1: u8, max_byte1: u8, widths: Vec<i16>) -> super::XcbFont {
        super::XcbFont {
            id: 1,
            ascent: 10,
            descent: 2,
            min_char,
            max_char,
            min_byte1,
            max_byte1,
            widths,
            default_width: 7,
        }
    }

    #[test]
    fn linear_font_width() {
        let f = font(32, 126, 0, 0, vec![5; 95]);
        assert_eq!(f.text_width("AB"), 10);
        assert_eq!(f.text_width("\u{044F}"), 7);
    }

    #[test]
    fn two_byte_font_width() {
        let mut widths = vec![6; 2 * 256];
        widths[256 + 0x4F] = 9;
        let f = font(0, 255, 3, 4, widths);
        assert_eq!(f.text_width("\u{044F}"), 9);
        assert_eq!(f.text_width("\u{0301}"), 6);
        assert_eq!(f.text_width("A"), 7);
    }
