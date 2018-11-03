    use super::*;

    #[test]
    fn test_resolution_dpi_formula() {
        assert_eq!(resolution_dpi(1600, 900), 96);
        assert_eq!(resolution_dpi(1280, 720), 96);
        assert_eq!(resolution_dpi(1920, 1080), 96);
        assert_eq!(resolution_dpi(2560, 1440), 96);
        assert_eq!(resolution_dpi(3840, 2160), 134);
    }

    #[test]
    fn test_scaled_rounds() {
        set_dpi(96);
        assert_eq!(scaled(19), 19);
        set_dpi(192);
        assert_eq!(scaled(19), 38);
        set_dpi(144);
        assert_eq!(scaled(19), 29);
        set_dpi(96);
    }

