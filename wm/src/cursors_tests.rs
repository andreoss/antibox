    use super::*;

    #[test]
    fn test_resize_edge_cursor_left() {
        assert_eq!(
            resize_edge_cursor(&crate::wmstate::ResizeEdge::Left),
            idx::SIZE_LEFT
        );
    }

    #[test]
    fn test_resize_edge_cursor_right() {
        assert_eq!(
            resize_edge_cursor(&crate::wmstate::ResizeEdge::Right),
            idx::SIZE_RIGHT
        );
    }

