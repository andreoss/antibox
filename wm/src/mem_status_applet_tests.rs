    use super::*;
    use crate::applet::Applet;
    use antibox_core::mock::MockDisplay;
    use std::sync::Arc;

    fn make_conn() -> Arc<dyn DisplayBackend> {
        Arc::new(MockDisplay::new(1280, 720, 24)) as Arc<dyn DisplayBackend>
    }

    #[test]
    fn test_parse_meminfo_line_matches() {
        let result = parse_meminfo_line("MemTotal:       16384000 kB", "MemTotal:");
        assert_eq!(result, Some(16384000 * 1024));
    }

    #[test]
    fn test_parse_meminfo_line_no_match() {
        let result = parse_meminfo_line("MemFree: 8000000 kB", "MemTotal:");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_meminfo_line_empty_value() {
        let result = parse_meminfo_line("MemTotal:", "MemTotal:");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_meminfo_line_non_numeric() {
        let result = parse_meminfo_line("MemTotal: abc kB", "MemTotal:");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_meminfo_line_case_sensitive() {
        let result = parse_meminfo_line("memtotal: 100 kB", "MemTotal:");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_meminfo_line_buffers_and_cached() {
        assert_eq!(
            parse_meminfo_line("Buffers:       123456 kB", "Buffers:"),
            Some(123456 * 1024)
        );
        assert_eq!(
            parse_meminfo_line("Cached: 789012 kB", "Cached:"),
            Some(789012 * 1024)
        );
    }

    #[test]
    fn test_new_applet_creates_window() {
        let conn = make_conn();
        let applet = MemStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        assert_ne!(applet.window.id(), 0);
    }

    #[test]
    fn test_preferred_sizes() {
        let conn = make_conn();
        let applet = MemStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        assert_eq!(
            applet.preferred_width(),
            antibox_core::scale::scaled(40) as u32
        );
        assert_eq!(applet.preferred_height(), crate::status_graph::pref_h());
    }

    #[test]
    fn test_update_sets_used_pct() {
        let conn = make_conn();
        let mut applet = MemStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        applet.update();
        if crate::proc_reader::read_proc_meminfo().is_some() {
            assert!(applet.used_pct > 0.0);
        }
        assert!(applet.used_pct <= 1.0);
    }

    #[test]
    fn test_paint_does_not_crash() {
        let conn = make_conn();
        let applet = MemStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        let g = conn.create_graphics(applet.window.id()).unwrap();
        applet.paint(&*g);
    }

    #[test]
    fn test_as_any_downcast() {
        let conn = make_conn();
        let applet = MemStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();
        let any: &dyn std::any::Any = applet.as_any();
        assert!(any.downcast_ref::<MemStatusApplet>().is_some());
    }

    #[test]
    fn test_paint_graph_auto_ranges_tiny_changes() {
        use antibox_core::mock::{MockCommand, MockGraphics};
        let conn = make_conn();
        let mut applet = MemStatusApplet::new(&conn, conn.root().read_id(), 40).unwrap();

        applet.samples = crate::status_graph::Samples::new();
        let base = 8_000_000u64;
        for k in 0..5u64 {
            applet.samples.push(MemSample {
                vals: [base + k, 50, 50, 1_000_000],
            });
        }
        let g = MockGraphics::new(64);
        applet.paint_graph(&g);
        let heights: Vec<u16> = g
            .commands()
            .iter()
            .filter_map(|c| match c {
                MockCommand::FillRect(_, _, _, h) if *h > 0 => Some(*h),
                _ => None,
            })
            .collect();
        let max = heights.iter().cloned().max().unwrap_or(0);
        let min = heights.iter().cloned().min().unwrap_or(0);

        assert!(
            max >= 2 * min.max(1),
            "tiny changes are amplified by auto-ranging (max={}, min={})",
            max,
            min
        );
    }
