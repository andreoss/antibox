    use super::*;

    #[test]
    fn test_parse_workspace_names_icewm_comma_quoted() {
        let got = parse_workspace_names("\" I \", \" II \", \" III \", \" IV \"", 4);
        assert_eq!(got, vec!["I", "II", "III", "IV"]);
    }

    #[test]
    fn test_parse_workspace_names_colon() {
        let got = parse_workspace_names("Main:Dev:Chat", 3);
        assert_eq!(got, vec!["Main", "Dev", "Chat"]);
    }

