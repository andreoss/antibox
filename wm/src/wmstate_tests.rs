    use super::*;

    #[test]
    fn test_window_state_default() {
        let s = WindowState::default();
        assert!(!s.maximized);
        assert!(!s.minimized);
        assert!(!s.shaded);
        assert!(!s.fullscreen);
        assert!(!s.urgent);
        assert!(!s.above);
        assert!(!s.below);
        assert!(!s.sticky);
        assert!(!s.skip_taskbar);
        assert!(!s.skip_pager);
    }

    #[test]
    fn test_resize_edge_variants() {
        assert_ne!(ResizeEdge::None, ResizeEdge::Left);
        assert_ne!(ResizeEdge::TopLeft, ResizeEdge::TopRight);
        assert_eq!(ResizeEdge::Bottom, ResizeEdge::Bottom);
    }

