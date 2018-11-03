    use super::*;

    #[test]
    fn test_system_config_dirs_cover_both_prefixes() {
        let dirs = system_config_dirs();
        assert!(dirs.contains(&PathBuf::from("/etc/antibox")));
        assert!(dirs.contains(&PathBuf::from("/usr/local/etc/antibox")));
        assert!(dirs.contains(&PathBuf::from("/usr/share/antibox")));
        assert!(dirs.contains(&PathBuf::from("/usr/local/share/antibox")));
    }
