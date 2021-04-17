    use super::*;
    use crate::applet::Applet;
    use antibox_core::mock::MockDisplay;
    use std::sync::Arc;

    fn make_conn() -> Arc<dyn DisplayBackend> {
        Arc::new(MockDisplay::new(1280, 720, 24)) as Arc<dyn DisplayBackend>
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_read_net_counters_excludes_loopback() {
        set_net_device("*");
        if let Some(rows) = read_net_counters() {
            assert!(rows.iter().all(|(name, _)| !name.starts_with("lo")));
        }
    }

    #[test]
    fn test_device_selected_empty_matches_none() {
        assert!(!device_selected("", "eth0"));
        assert!(!device_selected("   ", "wlan0"));
    }

    #[test]
    fn test_device_selected_star_matches_all() {
        assert!(device_selected("*", "eth0"));
        assert!(device_selected("*", "wlan0"));
    }

    #[test]
    fn test_device_selected_exact() {
        assert!(device_selected("eth0", "eth0"));
        assert!(!device_selected("eth0", "eth1"));
        assert!(!device_selected("eth0", "eth00"));
    }

    #[test]
    fn test_device_selected_prefix() {
        assert!(device_selected("en*", "enp3s0"));
        assert!(device_selected("en*", "en0"));
        assert!(!device_selected("en*", "wlan0"));
        assert!(device_selected("*", "anything"));
    }

    #[test]
    fn test_device_selected_multiple_tokens() {
        assert!(device_selected("eth0 wlan*", "eth0"));
        assert!(device_selected("eth0,wlan*", "wlan1"));
        assert!(!device_selected("eth0 wlan*", "tun0"));
    }

    #[test]
    fn test_fmt_rate_bytes() {
        assert_eq!(fmt_rate(0.0), "0B");
        assert_eq!(fmt_rate(512.0), "512B");
        assert_eq!(fmt_rate(1023.0), "1023B");
    }

    #[test]
    fn test_fmt_rate_kilobytes() {
        assert_eq!(fmt_rate(1024.0), "1K");
        assert_eq!(fmt_rate(1536.0), "2K");
        assert_eq!(fmt_rate(1024.0 * 1023.0), "1023K");
    }

    #[test]
    fn test_fmt_rate_megabytes() {
        assert_eq!(fmt_rate(1024.0 * 1024.0), "1.0M");
        assert_eq!(fmt_rate(5.5 * 1024.0 * 1024.0), "5.5M");
        assert_eq!(fmt_rate(1024.0 * 1024.0 * 1023.0), "1023.0M");
    }

    #[test]
    fn test_fmt_rate_gigabytes() {
        assert_eq!(fmt_rate(1024.0 * 1024.0 * 1024.0), "1.0G");
        assert_eq!(fmt_rate(2.5 * 1024.0 * 1024.0 * 1024.0), "2.5G");
    }

    #[test]
    fn test_new_applet_default_state() {
        let conn = make_conn();
        let applet = NetStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        assert_ne!(applet.window.id(), 0);
        assert!(applet.prev.is_none());
        assert!(applet.rates.is_empty());
        assert_eq!(applet.rx_total, 0.0);
        assert_eq!(applet.tx_total, 0.0);
        assert_eq!(applet.samples.len(), 0);
    }

    #[test]
    fn test_preferred_sizes() {
        let conn = make_conn();
        let applet = NetStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        assert_eq!(
            applet.preferred_width(),
            antibox_core::scale::scaled(40) as u32
        );
        assert_eq!(applet.preferred_height(), crate::status_graph::pref_h());
    }

    #[test]
    fn test_handle_click_returns_none() {
        let conn = make_conn();
        let mut applet = NetStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        assert!(applet.handle_click(0, 0, 1).is_none());
    }

    #[test]
    fn test_paint_does_not_crash() {
        let conn = make_conn();
        let applet = NetStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        let g = conn.create_graphics(applet.window.id()).unwrap();
        applet.paint(&*g);
    }

    #[test]
    fn test_as_any_downcast() {
        let conn = make_conn();
        let applet = NetStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        let any: &dyn std::any::Any = applet.as_any();
        assert!(any.downcast_ref::<NetStatusApplet>().is_some());
    }

    #[test]
    fn test_update_no_crash() {
        let conn = make_conn();
        let mut applet = NetStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        applet.update();
        assert!(applet.rates.is_empty());
    }

    #[test]
    fn test_tooltip_empty() {
        let conn = make_conn();
        let applet = NetStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        let tip = applet.tooltip();
        assert!(tip.contains("\u{2193}"));
        assert!(tip.contains("\u{2191}"));
        assert!(tip.contains("0B"));
    }

    #[test]
    fn test_read_net_counters_skips_lo() {
        if let Some(interfaces) = read_net_counters() {
            for (name, _cnt) in &interfaces {
                assert!(!name.is_empty());
                assert_ne!(name, &"lo");
            }
        }
    }
