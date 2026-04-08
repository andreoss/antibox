    use super::*;

    fn metadata() -> Value {
        Value::Array(vec![
            Value::Entry(
                Box::new(Value::str("xesam:title")),
                Box::new(Value::Variant(Box::new(Value::str("Spiegel im Spiegel")))),
            ),
            Value::Entry(
                Box::new(Value::str("xesam:artist")),
                Box::new(Value::Variant(Box::new(Value::Array(vec![
                    Value::str("Arvo Pärt"),
                ])))),
            ),
            Value::Entry(
                Box::new(Value::str("mpris:length")),
                Box::new(Value::Variant(Box::new(Value::I64(543_000_000)))),
            ),
        ])
    }

    #[test]
    fn test_signature_of_values() {
        assert_eq!(signature(&Value::str("x")), "s");
        assert_eq!(signature(&Value::U32(1)), "u");
        assert_eq!(signature(&Value::Array(vec![Value::str("a")])), "as");
        assert_eq!(signature(&metadata()), "a{sv}");
        assert_eq!(
            signature(&Value::Struct(vec![Value::Byte(1), Value::I64(2)])),
            "(yx)"
        );
    }

    #[test]
    fn test_split_signature() {
        assert_eq!(split_signature("sa{sv}as").unwrap(), ["s", "a{sv}", "as"]);
        assert_eq!(split_signature("a(yv)").unwrap(), ["a(yv)"]);
        assert_eq!(split_signature("").unwrap().len(), 0);
        assert_eq!(
            split_signature("(ii)a{s(ii)}b").unwrap(),
            ["(ii)", "a{s(ii)}", "b"]
        );
        assert!(split_signature("a").is_none());
        assert!(split_signature("(is").is_none());
    }

    #[test]
    fn test_alignment_of_types() {
        assert_eq!(alignment("y"), 1);
        assert_eq!(alignment("q"), 2);
        assert_eq!(alignment("s"), 4);
        assert_eq!(alignment("x"), 8);
        assert_eq!(alignment("{sv}"), 8);
        assert_eq!(alignment("v"), 1);
    }

    #[test]
    fn test_round_trip_basic_types() {
        let values = vec![
            Value::Byte(7),
            Value::Bool(true),
            Value::I16(-3),
            Value::U16(9),
            Value::I32(-100_000),
            Value::U32(100_000),
            Value::I64(-5_000_000_000),
            Value::U64(5_000_000_000),
            Value::F64(0.25),
            Value::str("hello"),
            Value::path("/org/mpris/MediaPlayer2"),
        ];
        let sig: String = values.iter().map(signature).collect();
        let mut enc = Encoder::new();
        for v in &values {
            enc.value(v);
        }
        let bytes = enc.take();
        let mut dec = Decoder::new(&bytes, true);
        assert_eq!(dec.values(&sig).unwrap(), values);
    }

    #[test]
    fn test_round_trip_dictionary() {
        let value = metadata();
        let mut enc = Encoder::new();
        enc.value(&value);
        let bytes = enc.take();
        let mut dec = Decoder::new(&bytes, true);
        let back = dec.value("a{sv}").unwrap();
        assert_eq!(back, value);
        assert_eq!(
            back.lookup("xesam:title").and_then(Value::as_str),
            Some("Spiegel im Spiegel")
        );
        assert_eq!(
            back.lookup("xesam:artist").map(Value::strings),
            Some(vec!["Arvo Pärt".to_string()])
        );
        assert_eq!(
            back.lookup("mpris:length").and_then(Value::as_i64),
            Some(543_000_000)
        );
    }

    #[test]
    fn test_round_trip_empty_array() {
        let mut enc = Encoder::new();
        enc.value(&Value::Array(Vec::new()));
        let bytes = enc.take();
        assert_eq!(bytes.len(), 4);
        let mut dec = Decoder::new(&bytes, true);
        assert_eq!(dec.value("as").unwrap(), Value::Array(Vec::new()));
    }

    #[test]
    fn test_message_round_trip() {
        let msg = Message::call(
            "org.mpris.MediaPlayer2.vlc",
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get",
        )
        .arg(Value::str("org.mpris.MediaPlayer2.Player"))
        .arg(Value::str("PlaybackStatus"));
        let bytes = msg.encode(17);
        assert_eq!(bytes[0], b'l');
        assert_eq!(bytes[1], CALL);
        let Decoded::Message(back, used) = Message::decode(&bytes) else {
            panic!("expected a message")
        };
        assert_eq!(used, bytes.len());
        assert_eq!(back.serial, 17);
        assert_eq!(back.destination, "org.mpris.MediaPlayer2.vlc");
        assert_eq!(back.path, "/org/mpris/MediaPlayer2");
        assert_eq!(back.interface, "org.freedesktop.DBus.Properties");
        assert_eq!(back.member, "Get");
        assert_eq!(back.arg_str(0), "org.mpris.MediaPlayer2.Player");
        assert_eq!(back.arg_str(1), "PlaybackStatus");
    }

    #[test]
    fn test_message_body_signature() {
        let msg = Message::call("a", "/b", "c", "d")
            .arg(Value::str("s"))
            .arg(metadata());
        assert_eq!(msg.body_signature(), "sa{sv}");
    }

    #[test]
    fn test_decode_signal_with_properties() {
        let mut msg = Message::call("", "/org/mpris/MediaPlayer2", "org.freedesktop.DBus.Properties", "PropertiesChanged");
        msg.kind = SIGNAL;
        msg.body = vec![
            Value::str("org.mpris.MediaPlayer2.Player"),
            metadata(),
            Value::Array(Vec::new()),
        ];
        let bytes = msg.encode(4);
        let Decoded::Message(back, _) = Message::decode(&bytes) else {
            panic!("expected a message")
        };
        assert_eq!(back.kind, SIGNAL);
        assert_eq!(back.member, "PropertiesChanged");
        assert_eq!(back.arg_str(0), "org.mpris.MediaPlayer2.Player");
        assert_eq!(
            back.body[1].lookup("xesam:title").and_then(Value::as_str),
            Some("Spiegel im Spiegel")
        );
    }

    #[test]
    fn test_decode_two_messages_in_one_buffer() {
        let first = Message::call("a", "/b", "c", "one").encode(1);
        let second = Message::call("a", "/b", "c", "two").encode(2);
        let mut buf = first.clone();
        buf.extend_from_slice(&second);
        let Decoded::Message(m1, used) = Message::decode(&buf) else {
            panic!("expected a message")
        };
        assert_eq!(m1.member, "one");
        assert_eq!(used, first.len());
        let Decoded::Message(m2, used2) = Message::decode(&buf[used..]) else {
            panic!("expected a message")
        };
        assert_eq!(m2.member, "two");
        assert_eq!(used2, second.len());
    }

    #[test]
    fn test_decode_incomplete_and_bad() {
        let bytes = Message::call("a", "/b", "c", "one").encode(1);
        assert_eq!(Message::decode(&bytes[..8]), Decoded::Incomplete);
        assert_eq!(Message::decode(&bytes[..bytes.len() - 1]), Decoded::Incomplete);
        let mut bad = bytes.clone();
        bad[0] = b'X';
        assert_eq!(Message::decode(&bad), Decoded::Bad);
        let mut wrong_version = bytes;
        wrong_version[3] = 9;
        assert_eq!(Message::decode(&wrong_version), Decoded::Bad);
    }

    #[test]
    fn test_decode_big_endian_message() {
        let mut buf: Vec<u8> = vec![b'B', CALL, 0, 1];
        buf.extend_from_slice(&0u32.to_be_bytes());
        buf.extend_from_slice(&5u32.to_be_bytes());
        buf.extend_from_slice(&12u32.to_be_bytes());
        buf.push(3);
        buf.extend_from_slice(&[1, b's', 0]);
        buf.extend_from_slice(&3u32.to_be_bytes());
        buf.extend_from_slice(b"one\0");
        buf.extend_from_slice(&[0, 0, 0, 0]);
        let Decoded::Message(back, used) = Message::decode(&buf) else {
            panic!("expected a message")
        };
        assert_eq!(used, 32);
        assert_eq!(back.serial, 5);
        assert_eq!(back.member, "one");
    }

    #[test]
    fn test_socket_path_from_address() {
        assert_eq!(
            socket_path("unix:path=/run/user/1000/bus").as_deref(),
            Some("/run/user/1000/bus")
        );
        assert_eq!(
            socket_path("unix:path=/tmp/dbus%2dtest,guid=abc").as_deref(),
            Some("/tmp/dbus-test")
        );
        assert_eq!(
            socket_path("tcp:host=localhost;unix:path=/run/bus").as_deref(),
            Some("/run/bus")
        );
        assert_eq!(socket_path("unix:abstract=/tmp/dbus-XYZ"), None);
        assert_eq!(socket_path(""), None);
    }

    #[test]
    fn test_auth_command_is_hex_uid() {
        assert_eq!(auth_command(1000), "AUTH EXTERNAL 31303030\r\n");
        assert_eq!(auth_command(0), "AUTH EXTERNAL 30\r\n");
    }

    #[test]
    fn test_no_reply_flag() {
        let msg = Message::call("a", "/b", "c", "d").no_reply();
        let bytes = msg.encode(1);
        assert_eq!(bytes[2], NO_REPLY);
    }
