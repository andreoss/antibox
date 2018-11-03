    use super::*;
    use antibox_core::backend::AtomManager;
    
    use antibox_core::mock::MockDisplay;

    fn setup() -> (MockDisplay, AtomManager) {
        let display = MockDisplay::new(1280, 720, 24);
        let mut atoms = AtomManager::new();
        atoms.intern_all(&display).unwrap();
        (display, atoms)
    }

    #[test]
    fn test_update_client_list() {
        let (display, atoms) = setup();
        let ids = [10u32, 20, 30];
        update_client_list(&display, &atoms, &ids);
    }

    #[test]
    fn test_update_client_list_stacking() {
        let (display, atoms) = setup();
        let ids = [10u32, 20, 30];
        update_client_list_stacking(&display, &atoms, &ids);
    }

