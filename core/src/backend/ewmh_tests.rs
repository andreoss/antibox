    fn ne(v: u32) -> [u8; 4] {
        [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8]
    }
    use super::*;

    #[test]
    fn test_strut_from_bytes() {
        let mut data = Vec::new();
        data.extend_from_slice(&ne(10u32));
        data.extend_from_slice(&ne(20u32));
        data.extend_from_slice(&ne(30u32));
        data.extend_from_slice(&ne(40u32));

        let strut = Strut::from_bytes(&data).unwrap();
        assert_eq!(strut.left, 10);
        assert_eq!(strut.right, 20);
        assert_eq!(strut.top, 30);
        assert_eq!(strut.bottom, 40);
    }

    #[test]
    fn test_strut_short_data() {
        assert!(Strut::from_bytes(&[0u8; 8]).is_none());
    }

