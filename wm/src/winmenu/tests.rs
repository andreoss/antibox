    use super::*;
    
    

    fn tc() -> crate::render::ThemeColors {
        crate::render::ThemeColors::default()
    }

    #[test]
    fn test_new() {
        let m = WindowActionMenu::new();
        assert!(!m.visible);
        assert!(m.items.is_empty());
    }

    #[test]
    fn test_for_focused_client() {
        let m = WindowActionMenu::for_focused_client(4, &tc());
        assert!(!m.visible);
        assert!(!m.items.is_empty());
        assert_eq!(m.items[0].label, "_Restore");
        assert_eq!(crate::render::mnemonic_key(&m.items[0].label), Some('R'));
        assert!(m.items[4].submenu.is_some());
        let tile = m
            .items
            .iter()
            .find(|it| it.label == "_Tile")
            .expect("top-level Tile entry");
        assert_eq!(tile.submenu.as_ref().map(Vec::len), Some(8));
        let max = m.items[4].submenu.as_ref().expect("Maximize submenu");
        assert!(max.iter().all(|it| it.submenu.is_none()));
        let ws = m
            .items
            .iter()
            .find(|it| it.label == "_Workspace")
            .and_then(|it| it.submenu.as_ref())
            .expect("Workspace submenu");
        assert_eq!(ws.len(), 4);
    }
