    use super::*;

    #[test]
    fn test_action_names() {
        assert_eq!(action_name(&Action::Window(WindowOp::Close)), "Close");
        assert_eq!(action_name(&Action::Window(WindowOp::Kill)), "Kill");
        assert_eq!(
            action_name(&Action::Window(WindowOp::Fullscreen)),
            "Fullscreen"
        );
        assert_eq!(action_name(&Action::Tile(TileOp::Cascade)), "Cascade");
        assert_eq!(
            action_name(&Action::Menu(MenuOp::WindowPickerList)),
            "Window List"
        );
    }

    #[test]
    fn test_workspace_action() {
        let a = Action::Workspace(WorkspaceOp::Workspace(3));
        if let Action::Workspace(WorkspaceOp::Workspace(n)) = a {
            assert_eq!(n, 3);
        } else {
            panic!("Expected Workspace action");
        }
    }

