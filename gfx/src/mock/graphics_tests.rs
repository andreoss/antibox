    use super::*;

    fn sample() -> Vec<MockCommand> {
        vec![
            MockCommand::SetForeground(0xff0000),
            MockCommand::FillRect(0, 0, 10, 10),
            MockCommand::SetForeground(0x00ff00),
            MockCommand::FillRect(5, 5, 2, 2),
            MockCommand::DrawText(1, 1, "hi".to_string()),
        ]
    }

    #[test]
    fn ordered_matches_subsequence() {
        assert_ordered(
            &sample(),
            &[
                MockCommand::SetForeground(0xff0000),
                MockCommand::FillRect(5, 5, 2, 2),
            ],
        );
    }

    #[test]
    #[should_panic]
    fn ordered_rejects_out_of_order() {
        assert_ordered(
            &sample(),
            &[
                MockCommand::FillRect(5, 5, 2, 2),
                MockCommand::SetForeground(0xff0000),
            ],
        );
    }

    #[test]
    #[should_panic]
    fn ordered_rejects_missing_command() {
        assert_ordered(&sample(), &[MockCommand::FillRect(9, 9, 1, 1)]);
    }

    #[test]
    fn painted_covers_region() {
        assert_painted(&sample(), &Rect::new(0, 0, 10, 10));
    }

    #[test]
    #[should_panic]
    fn painted_rejects_uncovered_region() {
        assert_painted(&sample(), &Rect::new(0, 0, 11, 10));
    }

    #[test]
    #[should_panic]
    fn assert_colour_at_rejects_wrong_colour() {
        assert_colour_at(&sample(), 6, 6, 0xff0000);
    }

