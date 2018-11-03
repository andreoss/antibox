    use super::*;
    

    #[test]
    fn test_parse_simple() {
        let xpm = r#"/* XPM */
static char *test[] = {
"2 2 2 1",
" 	c #FF0000",
".	c #00FF00",
" .",
". "
};"#;
        let img = parse_xpm_string(xpm);
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.data.len(), 2 * 2 * 4);
    }

    #[test]
    fn test_comment_between_sections() {
        let xpm = r#"/* XPM */
static char *test[] = {
/* width height ncolours chars_per_pixel */
"2 2 2 1",
/* colours */
" 	c #FF0000",
".	c #00FF00",
/* pixels */
" .",
". "
};"#;
        let img = parse_xpm_string(xpm).expect("comment lines must not break parsing");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.data.len(), 2 * 2 * 4);
    }

