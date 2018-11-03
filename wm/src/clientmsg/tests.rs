    use super::*;
    
    use antibox_core::backend::AtomManager;
    
    use antibox_core::mock::MockDisplay;
    

    fn setup_atoms() -> AtomManager {
        let display = MockDisplay::new(1280, 720, 24);
        let mut atoms = AtomManager::new();
        atoms.intern_all(&display).unwrap();
        atoms
    }

    #[test]
    fn test_client_message_xdnd_enter_accepts_version_5() {
        let atoms = setup_atoms();
        let mut wm = WindowManager::<MockDisplay>::new_test();
        wm.atoms = atoms;
        let xe = wm.atoms.get("XdndEnter").unwrap();
        let mut data = [0u32; 5];
        data[0] = 42;
        data[1] = 5 << 24;
        client_message(&mut wm, 100, xe, data);
        assert_eq!(wm.xdnd_source, Some(42));
    }

    #[test]
    fn test_client_message_xdnd_enter_rejects_version_over_5() {
        let atoms = setup_atoms();
        let mut wm = WindowManager::<MockDisplay>::new_test();
        wm.atoms = atoms;
        let xe = wm.atoms.get("XdndEnter").unwrap();
        let mut data = [0u32; 5];
        data[0] = 42;
        data[1] = 6 << 24;
        client_message(&mut wm, 100, xe, data);
        assert_eq!(wm.xdnd_source, None);
    }

    
    
    

