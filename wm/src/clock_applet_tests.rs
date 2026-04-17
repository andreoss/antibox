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

    #[test]
    fn test_a_long_clock_splits_onto_two_lines() {
        let lines = split_two_lines("Saturday 19 September 21:39", |s| s.chars().count() <= 14);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines.join(" "), "Saturday 19 September 21:39");
        assert!(lines[0].chars().count() <= 16, "{lines:?}");
    }

    #[test]
    fn test_a_short_clock_stays_on_one_line() {
        assert_eq!(split_two_lines("21:39", |_| true), vec!["21:39"]);
    }

    #[test]
    fn test_a_clock_without_spaces_splits_in_the_middle() {
        let lines = split_two_lines("2139442139", |s| s.chars().count() <= 5);
        assert_eq!(lines, vec!["21394", "42139"]);
    }
