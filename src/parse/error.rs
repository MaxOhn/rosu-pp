use super::OSU_FILE_HEADER;

use std::{
    error::Error as StdError,
    fmt,
    io::Error as IOError,
    num::{ParseFloatError, ParseIntError},
};

/// `Result<_, ParseError>`
pub type ParseResult<T> = Result<T, ParseError>;

/// Anything that could go wrong while parsing a [`Beatmap`](crate::Beatmap).
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum ParseError {
    /// Some IO operation failed.
    IOError(IOError),
    /// The initial data of an `.osu` file was incorrect.
    IncorrectFileHeader,
    /// Line in `.osu` was unexpectedly not of the form `key:value`.
    BadLine,
    /// Line in `.osu` that contains a slider was not in the proper format.
    InvalidCurvePoints,
    /// Expected a decimal number, got something else.
    InvalidDecimalNumber,
    /// Expected an integer, got something else.
    InvalidInteger,
    /// Failed to parse game mode.
    InvalidMode,
    /// Expected an additional field.
    MissingField(&'static str),
    /// Reject maps with too many repeat points.
    TooManyRepeats,
    /// Failed to recognized specified type for hitobjects.
    UnknownHitObjectKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(_) => f.write_str("IO error"),
            Self::IncorrectFileHeader => {
                write!(f, "expected `{}` at file begin", OSU_FILE_HEADER)
            }
            Self::BadLine => f.write_str("line not in `Key:Value` pattern"),
            Self::InvalidCurvePoints => f.write_str("invalid curve point"),
            Self::InvalidInteger => f.write_str("invalid integer"),
            Self::InvalidDecimalNumber => f.write_str("invalid float number"),
            Self::InvalidMode => f.write_str("invalid mode"),
            Self::MissingField(field) => write!(f, "missing field `{}`", field),
            Self::TooManyRepeats => f.write_str("repeat count is way too high"),
            Self::UnknownHitObjectKind => f.write_str("unsupported hitobject kind"),
        }
    }
}

impl StdError for ParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::IOError(inner) => Some(inner),
            Self::IncorrectFileHeader => None,
            Self::BadLine => None,
            Self::InvalidCurvePoints => None,
            Self::InvalidInteger => None,
            Self::InvalidDecimalNumber => None,
            Self::InvalidMode => None,
            Self::MissingField(_) => None,
            Self::TooManyRepeats => None,
            Self::UnknownHitObjectKind => None,
        }
    }
}

impl From<IOError> for ParseError {
    fn from(other: IOError) -> Self {
        Self::IOError(other)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidInteger
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(_: ParseFloatError) -> Self {
        Self::InvalidDecimalNumber
    }
}
