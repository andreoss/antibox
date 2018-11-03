    use super::*;

    #[test]
    fn test_parse_hex() {
        let c: RgbColour = "#FF0000".parse().unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_parse_rgb() {
        let c: RgbColour = "rgb:FF/00/00".parse().unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

