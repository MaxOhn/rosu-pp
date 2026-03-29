use crate::any::ModsDependent;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BeatmapAttribute {
    /// Variable value that may be overriden and adjusted based on mods and
    /// clock rate.
    Value(f32),
    /// Given by the user and may not be overriden by custom mod values.
    ///
    /// Mods and clock rate may *adjust* the value, though.
    ///
    /// # Example
    /// Mods include `DifficultyAdjust` which sets AR to 9.5 but the user
    /// specified AR to be 9.7. In this case, the user's value is should take
    /// precedence.
    Given(f32),
    /// Represents a final value that should not be adjusted based on mods or
    /// clock rate.
    Fixed(f32),
}

impl BeatmapAttribute {
    pub const DEFAULT: Self = Self::Value(5.0);

    pub const fn new(value: ModsDependent) -> Self {
        if value.with_mods {
            Self::Fixed(value.value)
        } else {
            Self::Given(value.value)
        }
    }

    /// Mutates the `Value` and `Given` variants.
    pub fn try_mutate(&mut self, f: impl Fn(&mut f32)) {
        if let Self::Value(value) | Self::Given(value) = self {
            f(value);
        }
    }

    /// Sets the `Value` variant only.
    pub const fn try_set(&mut self, value: f32) {
        if let Self::Value(old) = self {
            *old = value;
        }
    }

    /// Applies `f` onto the `Value` and `Given` variants and `default` onto the
    /// `Fixed` variant.
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce(f32) -> U,
        F: FnOnce(f32) -> U,
    {
        match self {
            Self::Value(value) | Self::Given(value) => f(value),
            Self::Fixed(fixed) => default(fixed),
        }
    }

    pub const fn get_raw(self) -> f32 {
        match self {
            Self::Value(value) | Self::Given(value) | Self::Fixed(value) => value,
        }
    }
}
