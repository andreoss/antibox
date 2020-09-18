    use super::*;
    use crate::applet::Applet;
    use crate::status_graph::MAX_SAMPLES;

    fn make_applet() -> CpuStatusApplet {
        CpuStatusApplet {
            conn: Arc::new(antibox_core::mock::MockDisplay::new(800, 600, 24))
                as Arc<dyn DisplayBackend>,
            window: Box::new(antibox_core::mock::MockWindow::new(42)) as Box<dyn WindowHandle>,
            tooltip: None,
            samples: crate::status_graph::Samples::new(),
            prev_times: None,
            cpu_count: 1,
            w: antibox_core::scale::scaled(40) as u16,
            h: 20,
            pref_w: antibox_core::scale::scaled(40) as u16,
        }
    }

    #[test]
    fn test_new_applet_default_state() {
        let app = make_applet();
        assert_eq!(app.samples.len(), 0);
    }

    #[test]
    fn test_update_no_prev() {
        let mut app = make_applet();
        assert!(!app.update());
        assert_eq!(app.samples.len(), 0);
    }

    #[test]
    fn test_last_percent_no_history() {
        let app = make_applet();
        assert_eq!(app.last_percent(), 0.0);
    }

    #[test]
    fn test_last_percent_with_data() {
        let mut app = make_applet();
        app.samples.push(CpuDelta {
            vals: [100, 20, 30, 800, 10, 5, 3, 2],
        });
        let pct = app.last_percent();
        let total = 100 + 20 + 30 + 800 + 10 + 5 + 3 + 2;
        let busy = total - 800;
        let expected = busy as f32 / total as f32;
        assert!((pct - expected).abs() < 0.001);
    }

    #[test]
    fn test_total_delta() {
        let d = [10, 20, 30, 40, 5, 3, 2, 1];
        assert_eq!(total_delta(&d), 111);
    }

    #[test]
    fn test_compute_deltas_single_cpu() {
        let prev = [[1000, 200, 300, 5000, 100, 50, 25, 10]];
        let cur = [[1100, 220, 330, 5100, 110, 55, 30, 12]];
        let d = compute_deltas(&prev, &cur).unwrap();
        assert_eq!(d[IWM_USER], 100);
        assert_eq!(d[IWM_STEAL], 2);
    }

    #[test]
    fn test_compute_deltas_multi_cpu() {
        let prev = [
            [1000, 200, 300, 5000, 100, 50, 25, 10],
            [800, 100, 200, 4000, 50, 20, 10, 5],
        ];
        let cur = [
            [1100, 220, 330, 5100, 110, 55, 30, 12],
            [900, 120, 230, 4100, 60, 25, 15, 7],
        ];
        let d = compute_deltas(&prev, &cur).unwrap();
        assert_eq!(d[IWM_USER], (1100 - 1000) + (900 - 800));
        assert_eq!(d[IWM_STEAL], (12 - 10) + (7 - 5));
    }

    #[test]
    fn test_compute_deltas_mismatched_len() {
        let prev = [[1, 2, 3, 4, 5, 6, 7, 8]];
        let cur = [[1, 2, 3, 4, 5, 6, 7, 8], [9, 10, 11, 12, 13, 14, 15, 16]];
        assert!(compute_deltas(&prev, &cur).is_none());
    }

    #[test]
    fn test_compute_deltas_empty() {
        let prev: Vec<[u64; IWM_STATES]> = vec![];
        let cur: Vec<[u64; IWM_STATES]> = vec![];
        assert!(compute_deltas(&prev, &cur).is_none());
    }

    #[test]
    fn test_parse_cpu_line() {
        let parts = vec![
            "cpu", "100", "200", "300", "400", "500", "600", "700", "800",
        ];
        let vals = parse_cpu_line(&parts);
        assert_eq!(vals[IWM_USER], 100);
        assert_eq!(vals[IWM_NICE], 200);
        assert_eq!(vals[IWM_SYS], 300);
        assert_eq!(vals[IWM_IDLE], 400);
        assert_eq!(vals[IWM_IOWAIT], 500);
        assert_eq!(vals[IWM_INTR], 600);
        assert_eq!(vals[IWM_SOFTIRQ], 700);
        assert_eq!(vals[IWM_STEAL], 800);
    }

    #[test]
    fn test_parse_cpu_line_short() {
        let parts = vec!["cpu", "100", "200", "300"];
        let vals = parse_cpu_line(&parts);
        assert_eq!(vals[IWM_USER], 100);
        assert_eq!(vals[IWM_IDLE], 0);
    }

    #[test]
    fn test_fmt_freq_ghz() {
        let s = fmt_freq(2_400_000.0);
        assert!(s.contains("2.40GHz") || s.contains("2.4GHz"));
    }

    #[test]
    fn test_fmt_freq_mhz() {
        let s = fmt_freq(800_000.0);
        assert!(s.contains("800") && (s.contains("MHz") || s.contains("KHz")));
    }

    #[test]
    fn test_fmt_mem_units() {
        assert!(!fmt_mem(2 * 1024 * 1024).is_empty());
        assert!(!fmt_mem(512 * 1024).is_empty());
        assert_eq!(fmt_mem(100), "100K");
    }

    #[test]
    fn test_click_returns_none() {
        let mut app = make_applet();
        assert!(app.handle_click(0, 0, 1).is_none());
        assert!(app.handle_click(0, 0, 3).is_none());
    }

    #[test]
    fn test_preferred_width_from_config() {
        let app = make_applet();
        assert_eq!(
            app.preferred_width(),
            antibox_core::scale::scaled(40) as u32
        );
    }

    #[test]
    fn test_tooltip_not_empty() {
        let app = make_applet();
        let tt = app.tooltip();
        assert!(!tt.is_empty());
        assert!(tt.contains("CPU") || tt.contains("RAM"));
    }

    #[test]
    fn test_update_circular_buffer() {
        let mut app = make_applet();
        for i in 0..MAX_SAMPLES + 5 {
            app.samples.push(CpuDelta {
                vals: [(1000 + i as u64); IWM_STATES],
            });
        }
        assert_eq!(app.samples.len(), MAX_SAMPLES);
    }

    #[test]
    fn test_read_cpu_times_live() {
        if let Some(times) = read_cpu_times() {
            assert!(!times.is_empty());
        }
    }

    #[test]
    fn test_read_cpu_freq_no_crash() {
        let _freqs = read_cpu_freq();
    }

    #[test]
    fn test_read_acpi_temp_no_crash() {
        let _temps = read_acpi_temp();
    }
