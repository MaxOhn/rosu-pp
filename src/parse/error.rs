use super::OSU_FILE_HEADER;

#[cfg(not(all(
    feature = "osu",
    feature = "taiko",
    feature = "fruits",
    feature = "mania"
)))]
use super::GameMode;

use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum ParseError {
    IOError(IOError),
    IncorrectFileHeader,
    BadLine,
    InvalidCurvePoints,
    InvalidInteger,
    InvalidFloatingPoint,
    InvalidMode,
    InvalidPathType,
    InvalidTimingSignature,
    MissingField(&'static str),
    TooManyRepeats,
    UnknownHitObjectKind,

    #[cfg(not(all(
        feature = "osu",
        feature = "taiko",
        feature = "fruits",
        feature = "mania"
    )))]
    UnincludedMode(GameMode),
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
            Self::InvalidFloatingPoint => f.write_str("invalid float number"),
            Self::InvalidMode => f.write_str("invalid mode"),
            Self::InvalidPathType => f.write_str("invalid path type"),
            Self::InvalidTimingSignature => f.write_str("invalid timing signature"),
            Self::MissingField(field) => write!(f, "missing field `{}`", field),
            Self::TooManyRepeats => f.write_str("repeat count is way too high"),
            Self::UnknownHitObjectKind => f.write_str("unsupported hitobject kind"),

            #[cfg(not(all(
                feature = "osu",
                feature = "taiko",
                feature = "fruits",
                feature = "mania"
            )))]
            Self::UnincludedMode(mode) => write!(
                f,
                "cannot process {:?} map; its mode's feature has not been included",
                mode
            ),
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
            Self::InvalidFloatingPoint => None,
            Self::InvalidMode => None,
            Self::InvalidPathType => None,
            Self::InvalidTimingSignature => None,
            Self::MissingField(_) => None,
            Self::TooManyRepeats => None,
            Self::UnknownHitObjectKind => None,

            #[cfg(not(all(
                feature = "osu",
                feature = "taiko",
                feature = "fruits",
                feature = "mania"
            )))]
            Self::UnincludedMode(_) => None,
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
        Self::InvalidFloatingPoint
    }
}
