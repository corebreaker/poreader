use std::{
    fmt::{Debug, Display, Formatter, Result},
    io::Error as IoError,
};

/// Error in reading (and, in future, writing) a catalogue.
pub enum Error {
    /// An I/O error from file operation.
    ///
    /// The first parameter is line number if applicable, the second is the system error.
    Io(usize, IoError),

    /// A parse error.
    ///
    /// Parameters are line number, the unexpected token (empty string if no token) and the expected tokens.
    /// Unset unexpected token means the formula is not smart enough to remember what it stopped on.
    /// Empty array of expected items means the formula is not smart enough to remember what it
    /// could have accepted instead.
    Parse(usize, String, String),

    /// An expected error.
    ///
    /// Something abnormal happened
    Unexpected(usize, String),

    /// A plural error
    ///
    /// Error detected while the parse of plural form header
    PluralForms(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &Error::Io(0, ref err) => Display::fmt(err, f),
            &Error::Io(line, ref err) => write!(f, "{} at line {}", err, line),
            &Error::Unexpected(line, ref msg) => {
                if line > 0 {
                    write!(f, "Unexpected error at line {}: {}", line, msg)
                } else {
                    write!(f, "Unexpected error: {}", msg)
                }
            }
            Error::PluralForms(msg) => write!(f, "Error in plurals forms: {}", msg),
            Error::Parse(line, got, exp) => {
                write!(f, "Parse error at line {}", line)?;

                if !exp.is_empty() {
                    write!(f, " expected ‘{}’", exp)?;
                }

                if !got.is_empty() {
                    write!(f, ", got ‘{}’", got)?;
                }

                Ok(())
            }
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &Error::Io(0, ref err) => Debug::fmt(err, f),
            &Error::Io(line, ref err) => write!(f, "{:?} at line {}", err, line),
            &Error::Unexpected(line, ref msg) => {
                if line > 0 {
                    write!(f, "Unexpected error at line {}: {}", line, msg)
                } else {
                    write!(f, "Unexpected error: {}", msg)
                }
            }
            Error::PluralForms(msg) => write!(f, "Error in plurals forms: {}", msg),
            &Error::Parse(line, ref got, ref exp) => {
                write!(f, "Parse error at line {}", line)?;

                if !exp.is_empty() {
                    write!(f, " expected ‘{}’", exp)?;
                }

                if !got.is_empty() {
                    write!(f, ", got ‘{}’", got)?;
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            &Error::Io(_, ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, error)
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error as StdErr, io::ErrorKind};

    fn make_error() -> Error {
        Error::PluralForms(String::from("message"))
    }

    impl PartialEq for Error {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Error::PluralForms(l), Error::PluralForms(r)) => r == l,
                (Error::Unexpected(ll, lm), Error::Unexpected(rl, rm)) => (ll == rl) && (lm == rm),
                (Error::Parse(ll, lu, le), Error::Parse(rl, ru, re)) => (ll == rl) && (lu == ru) && (le == re),
                (Error::Io(ll, le), Error::Io(rl, re)) => {
                    (ll == rl)
                        && (le.kind() == re.kind())
                        && (le.raw_os_error() == re.raw_os_error())
                        && (le.get_ref().map(|v| v.to_string()) == re.get_ref().map(|v| v.to_string()))
                }
                _ => false,
            }
        }
    }

    impl Eq for Error {}

    #[test]
    fn test_format() {
        let err = make_error();

        assert_eq!(format!("{:?}", err), format!("{}", err));
    }

    #[test]
    fn test_eq() {
        assert_eq!(
            Error::Io(123, IoError::from(make_error())),
            Error::Io(123, IoError::from(Error::PluralForms(String::from("message")))),
        );
    }

    #[test]
    fn test_func_source() {
        let err = Error::Io(10, std::io::Error::new(ErrorKind::Other, make_error()));
        let other = Error::Unexpected(15, String::from("weird"));

        assert!(other.source().is_none(), "Other error should have no source");
        assert_eq!(
            format!("{}", err.source().unwrap_or(&other)),
            format!("{}", make_error())
        );
    }

    #[test]
    fn test_trait_from() {
        let err = std::io::Error::from(make_error());

        assert_eq!(err.kind(), ErrorKind::Other);
        assert_eq!(format!("{}", err), format!("{}", make_error()));
    }

    #[test]
    fn test_trait_display() {
        assert_eq!(
            format!("{}", Error::Io(0, std::io::Error::from(make_error()))),
            String::from("Error in plurals forms: message"),
        );

        assert_eq!(
            format!("{}", Error::Io(10, std::io::Error::from(make_error()))),
            String::from("Error in plurals forms: message at line 10"),
        );

        assert_eq!(
            format!("{}", Error::Unexpected(0, String::from("message"))),
            String::from("Unexpected error: message"),
        );

        assert_eq!(
            format!("{}", Error::Unexpected(10, String::from("message"))),
            String::from("Unexpected error at line 10: message"),
        );

        assert_eq!(
            format!("{}", Error::Parse(10, String::from("token1"), String::from("token2"))),
            String::from("Parse error at line 10 expected ‘token2’, got ‘token1’"),
        );

        assert_eq!(
            format!("{}", Error::PluralForms(String::from("message"))),
            format!("Error in plurals forms: message"),
        );
    }

    #[test]
    fn test_trait_debug() {
        assert_eq!(
            format!("{:?}", Error::Io(0, std::io::Error::from(make_error()))),
            String::from("Custom { kind: Other, error: Error in plurals forms: message }"),
        );

        assert_eq!(
            format!("{:?}", Error::Io(10, std::io::Error::from(make_error()))),
            String::from("Custom { kind: Other, error: Error in plurals forms: message } at line 10"),
        );

        assert_eq!(
            format!("{:?}", Error::Unexpected(0, String::from("message"))),
            String::from("Unexpected error: message"),
        );

        assert_eq!(
            format!("{:?}", Error::Unexpected(10, String::from("message"))),
            String::from("Unexpected error at line 10: message"),
        );

        assert_eq!(
            format!("{:?}", Error::Parse(10, String::from("token1"), String::from("token2"))),
            String::from("Parse error at line 10 expected ‘token2’, got ‘token1’"),
        );

        assert_eq!(
            format!("{:?}", Error::PluralForms(String::from("message"))),
            format!("Error in plurals forms: message"),
        );
    }
}
// no-coverage:stop
