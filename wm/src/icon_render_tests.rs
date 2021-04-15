    use super::*;

    fn icon(w: u32, h: u32, fill: u32) -> IconData {
        IconData {
            width: w,
            height: h,
            pixels: vec![fill; (w * h) as usize],
        }
    }

    #[test]
    fn test_from_property_parses_sizes() {
        let mut bytes = Vec::new();
        for word in [2u32, 2, 1, 2, 3, 4] {
            bytes.extend_from_slice(&word.to_ne_bytes());
        }
        let icons = IconData::from_property(&bytes);
        assert_eq!(icons.len(), 1);
        assert_eq!((icons[0].width, icons[0].height), (2, 2));
        assert_eq!(icons[0].pixels, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_from_property_rejects_truncated() {
        let mut bytes = Vec::new();
        for word in [8u32, 8, 1, 2] {
            bytes.extend_from_slice(&word.to_ne_bytes());
        }
        assert!(IconData::from_property(&bytes).is_empty());
    }

    #[test]
    fn test_select_icon_prefers_smallest_ge_target() {
        let icons = [icon(48, 48, 0), icon(16, 16, 0), icon(32, 32, 0)];
        let got = select_icon(&icons, 20).unwrap();
        assert_eq!(got.width, 32);
    }

    #[test]
    fn test_select_icon_falls_back_to_largest() {
        let icons = [icon(8, 8, 0), icon(12, 12, 0)];
        let got = select_icon(&icons, 20).unwrap();
        assert_eq!(got.width, 12);
        assert!(select_icon(&[], 16).is_none());
    }

    #[test]
    fn test_icon_to_pixmap_size_and_alpha() {
        let ic = icon(4, 4, 0xFF11_2233);
        let pm = icon_to_pixmap(&ic, 16, 16, 0x000000);
        assert_eq!((pm.width, pm.height), (16, 16));
        assert_eq!(&pm.data[..4], &[0x11, 0x22, 0x33, 255]);
        let clear = icon(4, 4, 0x0000_0000);
        let pm = icon_to_pixmap(&clear, 8, 8, 0xAABBCC);
        assert_eq!(&pm.data[..4], &[0xAA, 0xBB, 0xCC, 255]);
    }

    #[test]
    fn test_default_icon_opaque_and_sized() {
        let pm = default_icon(16, 0xD4D0C8);
        assert_eq!((pm.width, pm.height), (16, 16));
        assert!(pm.data.chunks_exact(4).all(|px| px[3] == 255));
    }

    #[test]
    fn test_resolve_client_icon_falls_back() {
        let pm = resolve_client_icon(&[], 16, 0xD4D0C8);
        assert_eq!((pm.width, pm.height), (16, 16));
        let pm = resolve_client_icon(&[icon(32, 32, 0xFF00FF00)], 16, 0);
        assert_eq!(&pm.data[..4], &[0, 0xFF, 0, 255]);
    }
