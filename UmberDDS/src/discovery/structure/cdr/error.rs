use std::{fmt, io, str::Utf8Error};

/// Convenient wrapper around `std::Result`.
pub type Result<T> = std::result::Result<T, Error>;

/// The Error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("does not support the serde::Deserializer::deserialize_any method")]
    DeserializeAnyNotSupported,

    #[error("expected 0 or 1, found {0}")]
    InvalidBoolEncoding(u8),

    #[error("expected char of width 1, found {0}")]
    InvalidChar(char),

    #[error("char is not valid UTF-8")]
    InvalidCharEncoding,

    #[error("encapsulation is not valid")]
    InvalidEncapsulation,

    #[error("{0}")]
    InvalidUtf8Encoding(#[source] Utf8Error),

    #[error("each character must have a length of 1, given \"{0}\"")]
    InvalidString(String),

    #[error("sequence is too long")]
    NumberOutOfRange,

    #[error("sequences must have a knowable size ahead of time")]
    SequenceMustHaveLength,

    #[error("the size limit has been reached")]
    SizeLimit,

    #[error("unsupported type")]
    TypeNotSupported,
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Message(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Message(msg.to_string())
    }
}
