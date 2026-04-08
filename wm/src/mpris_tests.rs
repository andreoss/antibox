    use super::*;

    fn variant(v: Value) -> Value {
        Value::Variant(Box::new(v))
    }

    fn entry(key: &str, v: Value) -> Value {
        Value::Entry(Box::new(Value::str(key)), Box::new(variant(v)))
    }

    fn metadata() -> Value {
        Value::Array(vec![
            entry("xesam:title", Value::str("Fratres")),
            entry(
                "xesam:artist",
                Value::Array(vec![Value::str("Arvo Pärt"), Value::str("Gidon Kremer")]),
            ),
            entry("xesam:album", Value::str("Tabula Rasa")),
            entry("mpris:length", Value::I64(672_000_000)),
        ])
    }

    fn playing_props() -> Value {
        Value::Array(vec![
            entry("PlaybackStatus", Value::str("Playing")),
            entry("Metadata", metadata()),
            entry("CanGoNext", Value::Bool(true)),
            entry("CanGoPrevious", Value::Bool(true)),
            entry("CanControl", Value::Bool(true)),
        ])
    }

    fn player(bus: &str, status: Playback) -> Player {
        let mut p = Player::new(bus);
        p.status = status;
        p
    }

    #[test]
    fn test_playback_parse() {
        assert_eq!(Playback::parse("Playing"), Playback::Playing);
        assert_eq!(Playback::parse("paused"), Playback::Paused);
        assert_eq!(Playback::parse("Stopped"), Playback::Stopped);
        assert_eq!(Playback::parse(""), Playback::Stopped);
        assert_eq!(Playback::Paused.text(), "Paused");
    }

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(0), "0:00");
        assert_eq!(format_time(65_000_000), "1:05");
        assert_eq!(format_time(3_672_000_000), "1:01:12");
        assert_eq!(format_time(-5), "0:00");
    }

    #[test]
    fn test_parse_track() {
        let t = parse_track(&metadata());
        assert_eq!(t.title, "Fratres");
        assert_eq!(t.artist, "Arvo Pärt, Gidon Kremer");
        assert_eq!(t.album, "Tabula Rasa");
        assert_eq!(t.length, 672_000_000);
    }

    #[test]
    fn test_parse_track_falls_back_to_album_artist() {
        let meta = Value::Array(vec![
            entry("xesam:title", Value::str("Nocturne")),
            entry("xesam:albumArtist", Value::Array(vec![Value::str("Chopin")])),
        ]);
        let t = parse_track(&meta);
        assert_eq!(t.artist, "Chopin");
        assert_eq!(t.length, 0);
        assert!(t.album.is_empty());
    }

    #[test]
    fn test_parse_track_plain_string_artist() {
        let meta = Value::Array(vec![entry("xesam:artist", Value::str("Satie"))]);
        assert_eq!(parse_track(&meta).artist, "Satie");
    }

    #[test]
    fn test_player_name_from_bus() {
        let p = Player::new("org.mpris.MediaPlayer2.vlc");
        assert_eq!(p.short_name(), "vlc");
        assert_eq!(p.name(), "Vlc");
        let mut named = p;
        named.identity = "VLC media player".to_string();
        assert_eq!(named.name(), "VLC media player");
    }

    #[test]
    fn test_player_name_strips_instance_suffix() {
        let p = Player::new("org.mpris.MediaPlayer2.firefox.instance_1_7");
        assert_eq!(p.name(), "Firefox");
    }

    #[test]
    fn test_player_label() {
        let mut p = Player::new("org.mpris.MediaPlayer2.mpv");
        assert_eq!(p.label(), "Mpv");
        p.track.title = "Fratres".to_string();
        assert_eq!(p.label(), "Fratres");
        p.track.artist = "Arvo Pärt".to_string();
        assert_eq!(p.label(), "Arvo Pärt \u{2014} Fratres");
        p.track.title = String::new();
        assert_eq!(p.label(), "Arvo Pärt");
    }

    #[test]
    fn test_player_tooltip() {
        let mut p = Player::new("org.mpris.MediaPlayer2.mpv");
        p.apply_all(&playing_props());
        let tip = p.tooltip();
        let lines: Vec<&str> = tip.lines().collect();
        assert_eq!(lines[0], "Mpv");
        assert_eq!(lines[1], "Fratres");
        assert_eq!(lines[2], "Arvo Pärt, Gidon Kremer");
        assert_eq!(lines[3], "Tabula Rasa");
        assert_eq!(lines[4], "Playing  11:12");
    }

    #[test]
    fn test_apply_all_reports_change() {
        let mut p = Player::new("org.mpris.MediaPlayer2.mpv");
        assert!(p.apply_all(&playing_props()));
        assert_eq!(p.status, Playback::Playing);
        assert!(p.can_next && p.can_previous && p.can_control);
        assert_eq!(p.track.title, "Fratres");
        assert!(!p.apply_all(&playing_props()));
    }

    #[test]
    fn test_apply_identity_and_unknown_keys() {
        let mut p = Player::new("org.mpris.MediaPlayer2.mpv");
        assert!(p.apply("Identity", &variant(Value::str("mpv Media Player"))));
        assert_eq!(p.identity, "mpv Media Player");
        assert!(!p.apply("DesktopEntry", &variant(Value::str("mpv"))));
    }

    #[test]
    fn test_pick_prefers_a_playing_player() {
        let list = vec![
            player("org.mpris.MediaPlayer2.a", Playback::Paused),
            player("org.mpris.MediaPlayer2.b", Playback::Playing),
        ];
        assert_eq!(pick(&list, ""), "org.mpris.MediaPlayer2.b");
        assert_eq!(
            pick(&list, "org.mpris.MediaPlayer2.a"),
            "org.mpris.MediaPlayer2.b"
        );
    }

    #[test]
    fn test_pick_keeps_current_while_it_plays() {
        let list = vec![
            player("org.mpris.MediaPlayer2.a", Playback::Playing),
            player("org.mpris.MediaPlayer2.b", Playback::Playing),
        ];
        assert_eq!(
            pick(&list, "org.mpris.MediaPlayer2.b"),
            "org.mpris.MediaPlayer2.b"
        );
    }

    #[test]
    fn test_pick_keeps_idle_current_and_falls_back() {
        let list = vec![
            player("org.mpris.MediaPlayer2.a", Playback::Paused),
            player("org.mpris.MediaPlayer2.b", Playback::Stopped),
        ];
        assert_eq!(
            pick(&list, "org.mpris.MediaPlayer2.b"),
            "org.mpris.MediaPlayer2.b"
        );
        assert_eq!(pick(&list, "org.mpris.MediaPlayer2.gone"), "org.mpris.MediaPlayer2.a");
        assert_eq!(pick(&[], "org.mpris.MediaPlayer2.a"), "");
    }

    #[test]
    fn test_next_bus_cycles() {
        let list = vec![
            player("org.mpris.MediaPlayer2.a", Playback::Paused),
            player("org.mpris.MediaPlayer2.b", Playback::Paused),
        ];
        assert_eq!(
            next_bus(&list, "org.mpris.MediaPlayer2.a"),
            "org.mpris.MediaPlayer2.b"
        );
        assert_eq!(
            next_bus(&list, "org.mpris.MediaPlayer2.b"),
            "org.mpris.MediaPlayer2.a"
        );
        assert_eq!(next_bus(&list, ""), "org.mpris.MediaPlayer2.a");
        assert_eq!(next_bus(&[], ""), "");
    }

    #[test]
    fn test_seek_target_clamps_to_the_track() {
        assert_eq!(seek_target(60_000_000, 300_000_000, 5_000_000), 65_000_000);
        assert_eq!(seek_target(2_000_000, 300_000_000, -5_000_000), 0);
        assert_eq!(seek_target(299_000_000, 300_000_000, 5_000_000), 300_000_000);
        assert_eq!(seek_target(10_000_000, 0, 5_000_000), 15_000_000);
        assert_eq!(seek_target(i64::MAX, 0, 5_000_000), i64::MAX);
    }

    #[test]
    fn test_apply_can_seek() {
        let mut p = Player::new("org.mpris.MediaPlayer2.mpv");
        assert!(!p.can_seek);
        assert!(p.apply("CanSeek", &variant(Value::Bool(true))));
        assert!(p.can_seek);
    }

    #[test]
    fn test_command_targets() {
        assert_eq!(Command::PlayPause.member(), "PlayPause");
        assert_eq!(Command::PlayPause.interface(), PLAYER_IFACE);
        assert_eq!(Command::Next.interface(), PLAYER_IFACE);
        assert_eq!(Command::Raise.member(), "Raise");
        assert_eq!(Command::Raise.interface(), ROOT_IFACE);
    }
