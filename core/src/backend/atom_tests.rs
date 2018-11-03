    use super::*;

    #[test]
    fn test_atom_manager_new() {
        let mgr = AtomManager::new();
        assert!(mgr.get("WM_PROTOCOLS").is_none());
    }

    #[test]
    fn test_atom_manager_cache() {
        let mut mgr = AtomManager::new();
        mgr.atoms.insert("TEST".to_string(), 42);
        assert_eq!(mgr.get("TEST"), Some(42));
    }

