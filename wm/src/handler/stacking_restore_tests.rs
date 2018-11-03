    use super::apply_saved_stacking;
    
    
    
    

    #[test]
    fn saved_stacking_reorders_listed_windows() {
        let children = [10u32, 20, 30, 40];
        let saved = [30u32, 10];
        assert_eq!(apply_saved_stacking(&children, &saved), vec![30, 20, 10, 40]);
    }

    #[test]
    fn unlisted_windows_keep_current_order() {
        let children = [1u32, 2, 3];
        assert_eq!(apply_saved_stacking(&children, &[]), vec![1, 2, 3]);
        assert_eq!(apply_saved_stacking(&children, &[9, 8]), vec![1, 2, 3]);
        assert_eq!(apply_saved_stacking(&children, &[2]), vec![1, 2, 3]);
    }

