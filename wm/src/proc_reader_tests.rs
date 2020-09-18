    use super::*;

    #[test]
    fn test_load_average_non_negative() {
        let avgs = load_average().unwrap();
        assert!(avgs.iter().all(|a| *a >= 0.0));
    }

    #[test]
    fn test_read_proc_meminfo() {
        if let Some((total, free)) = read_proc_meminfo() {
            assert!(total > 0);
            assert!(free <= total);
        }
    }
