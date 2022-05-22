use std::fmt::{Display, Formatter};

/// An error that occurred during deserialization.
#[derive(Debug)]
pub enum Error {
    /// An error occurred while parsing the regular expression
    BadRegex(regex::Error),

    /// The string doesn't match the pattern
    NoMatch(),

    /// A value couldn't be parsed into the required type
    BadValue {
        /// The name of the group
        name: String,

        /// The value that couldn't be converted to the target value
        value: String,
    },

    /// Some other deserialization/serde related error
    Custom(String),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match *self {
            BadRegex(ref err) => err.fmt(f),
            NoMatch() => write!(f, "String doesn't match pattern"),
            BadValue {
                ref name,
                ref value,
            } => {
                write!(f, "Unable to convert value for group {}: {}", name, value)
            }
            Custom(ref err) => write!(f, "{}", err),
        }
    }
}

// Do not use this alias in public parts of the crate because
// it would hide the direct link to the actual error type in rustdoc.
pub(crate) type Result<T> = std::result::Result<T, Error>;
