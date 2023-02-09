use std::{error::Error as StdError, fmt, io::Error as IoError};

/// `Result<_, ParseError>`
pub type ParseResult<T> = Result<T, ParseError>;

/// Anything that could go wrong while parsing a [`Beatmap`](crate::Beatmap).
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum ParseError {
    /// Some IO operation failed.
    IoError(IoError),
    /// The initial data of an `.osu` file was incorrect.
    IncorrectFileHeader,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(_) => f.write_str("IO error"),
            Self::IncorrectFileHeader => {
                write!(f, "expected `osu file format v` at file begin")
            }
        }
    }
}

impl StdError for ParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::IoError(inner) => Some(inner),
            Self::IncorrectFileHeader => None,
        }
    }
}

impl From<IoError> for ParseError {
    fn from(other: IoError) -> Self {
        Self::IoError(other)
    }
}
