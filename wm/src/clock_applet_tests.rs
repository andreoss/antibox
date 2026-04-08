    use super::*;

    #[test]
    fn test_time_format_hmm() {
        let s = current_time_str("%H:%M");
        assert_eq!(s.len(), 5);
        assert_eq!(s.as_bytes()[2], b':');
    }

    #[test]
    fn test_time_format_hmmss() {
        let s = current_time_str("%H:%M:%S");
        assert_eq!(s.len(), 8);
        assert_eq!(s.as_bytes()[2], b':');
        assert_eq!(s.as_bytes()[5], b':');
    }


    #[test]
    fn test_day_and_month_names_come_from_the_locale() {
        let names = current_time_str("%A %B");
        assert_eq!(names.split_whitespace().count(), 2);
        assert!(names.chars().all(|c| !c.is_ascii_digit()));
    }
