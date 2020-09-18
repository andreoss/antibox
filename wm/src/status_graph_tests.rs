    use super::*;

    #[test]
    fn plot_geometry_and_columns() {
        let p = plot(28, 20, 100).unwrap();
        assert!(p.count() >= 1);
        assert_eq!(p.top(), INSET);
        assert_eq!(p.bottom(), INSET + p.gh);
        assert!(p.col_x(0) >= INSET);
        assert_eq!(p.sample_idx(0, 0), MAX_SAMPLES - p.count());
    }

    #[test]
    fn samples_ring_tracks_latest_and_window() {
        let mut s: Samples<u32> = Samples::new();
        assert!(s.is_empty());
        for v in 1..=5 {
            s.push(v);
        }
        assert_eq!(s.len(), 5);
        assert_eq!(s.latest(), Some(&5));
        assert_eq!(*s.at(0, 3), 3);
        assert_eq!(*s.at(1, 3), 4);
        assert_eq!(*s.at(2, 3), 5);
    }

    #[test]
    fn samples_ring_wraps_at_capacity() {
        let mut s: Samples<usize> = Samples::new();
        for v in 0..(MAX_SAMPLES + 7) {
            s.push(v);
        }
        assert_eq!(s.len(), MAX_SAMPLES);
        assert_eq!(s.latest(), Some(&(MAX_SAMPLES + 6)));
        assert_eq!(*s.at(MAX_SAMPLES - 1, MAX_SAMPLES), MAX_SAMPLES + 6);
    }

    #[test]
    fn pref_h_nonzero() {
        assert!(pref_h() > 0);
    }

    #[test]
    fn test_wide_graph_is_not_capped_at_forty_columns() {
        let p = plot(120, 20, MAX_SAMPLES).unwrap();
        assert!(p.count() > 40);
    }
