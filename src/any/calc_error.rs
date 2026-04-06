use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

use crate::model::{beatmap::TooSuspicious, mode::ConvertError};

/// Error type when beatmap conversion or its suspicion-check fails.
#[derive(Copy, Clone, Debug)]
pub enum CalculateError {
    /// Conversion failed.
    Convert(ConvertError),
    /// Suspicion check failed.
    Suspicion(TooSuspicious),
}

impl Error for CalculateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CalculateError::Convert(err) => Some(err),
            CalculateError::Suspicion(err) => Some(err),
        }
    }
}

impl Display for CalculateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Failed to calculate attributes")?;

        match self {
            CalculateError::Convert(_) => f.write_str(" (Convert)")?,
            CalculateError::Suspicion(_) => f.write_str(" (Suspicion)")?,
        }

        Ok(())
    }
}

impl From<ConvertError> for CalculateError {
    fn from(value: ConvertError) -> Self {
        CalculateError::Convert(value)
    }
}

impl From<TooSuspicious> for CalculateError {
    fn from(value: TooSuspicious) -> Self {
        CalculateError::Suspicion(value)
    }
}
