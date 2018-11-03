    use super::parse_names;
    use super::super::bindings::{xcb_charinfo_t, xcb_query_font_reply_t};

    #[test]
    fn query_font_reply_matches_wire_layout() {
        assert_eq!(std::mem::size_of::<xcb_charinfo_t>(), 12);
        assert_eq!(std::mem::size_of::<xcb_query_font_reply_t>(), 60);
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

