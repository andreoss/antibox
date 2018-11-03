    use super::*;
    

    #[test]
    fn test_ycursor_new_with_path() {
        let cursor = Cursor::new(Some("cursors/left_ptr.xpm"), None, None);
        assert_eq!(cursor.path(), Some("cursors/left_ptr.xpm"));
        assert!(cursor.glyph().is_none());
        assert!(cursor.xname().is_none());
    }

    #[test]
    fn test_ycursor_from_path() {
        let cursor = Cursor::from_path("test.xpm");
        assert_eq!(cursor.path(), Some("test.xpm"));
    }

