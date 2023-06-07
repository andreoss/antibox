    use super::*;

    #[test]
    fn display_carries_the_message() {
        assert_eq!(Error::Message("boom".to_string()).to_string(), "boom");
        assert_eq!(Error::unsupported("thing").to_string(), "unsupported: thing");
    }

    #[test]
    fn converts_from_string_types() {
        let a: Error = "boom".into();
        let b: Error = String::from("boom").into();
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn io_error_is_the_source() {
        let e = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "nope"));
        assert!(std::error::Error::source(&e).is_some());
        assert!(std::error::Error::source(&Error::Message("x".to_string())).is_none());
    }

    #[test]
    fn converts_into_boxed_trait_object() {
        let e: Error = "boom".into();
        let boxed: Box<dyn std::error::Error> = e.into();
        assert_eq!(boxed.to_string(), "boom");
    }
