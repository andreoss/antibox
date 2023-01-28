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


    #[test]
    fn test_blue_to_green_moves_blue_intensity_to_green() {
        assert_eq!(blue_to_green(0x000080), 0x008000);
        assert_eq!(blue_to_green(0x0000FF), 0x00FF00);
        assert_eq!(blue_to_green(0x808080), 0x808080);
        assert_eq!(blue_to_green(0x123456), 0x125634);
    }

    #[test]
    fn test_greenish_makes_a_dark_active_green() {
        let active = greenish(0x000080);
        assert_eq!(active, 0x005300);
        assert!((active & 0xFF00) >> 8 < 0x80, "darker than the plain swap");
    }

    #[test]
    fn test_grey_green_stays_mostly_grey_with_a_touch_of_green() {
        let c = grey_green(0x808080);
        let (r, g, b) = ((c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF);
        assert_eq!(g, 0x80, "keeps the grey's brightness in the green channel");
        assert_eq!(r, b, "balanced on the red/blue axes");
        assert!(g > r, "a touch of green");
        assert!(r >= 0x68, "stays close to grey, only a slight tint: {c:06X}");
        assert!(greenish(0x808080) != c, "greyer than the active treatment");
    }
