    use super::*;
    #[test]
    fn test_parse_geometry_full() {
        let gf = parse_geometry("100x200+10+20");
        assert!(gf.0.contains(GeoFlags::W));
        assert!(gf.0.contains(GeoFlags::H));
        assert!(gf.0.contains(GeoFlags::X));
        assert!(gf.0.contains(GeoFlags::Y));
        assert_eq!((gf.3, gf.4, gf.1, gf.2), (100, 200, 10, 20));
    }
    #[test]
    fn test_parse_geometry_position() {
        let gf = parse_geometry("+50+100");
        assert!(!gf.0.contains(GeoFlags::W));
        assert!(!gf.0.contains(GeoFlags::H));
        assert!(gf.0.contains(GeoFlags::X));
        assert!(gf.0.contains(GeoFlags::Y));
        assert_eq!((gf.1, gf.2), (50, 100));
    }
