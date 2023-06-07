use std::fmt;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(std::io::Error),
    Unsupported(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn message<T: Into<String>>(what: T) -> Error {
        Error::Message(what.into())
    }

    pub fn unsupported(what: &'static str) -> Error {
        Error::Unsupported(what)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(s) => f.write_str(s),
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::Unsupported(s) => write!(f, "unsupported: {}", s),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::Message(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Message(s)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::Message(e.to_string())
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Error {
        Error::Message(e.to_string())
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Error {
        Error::Message(e.to_string())
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
