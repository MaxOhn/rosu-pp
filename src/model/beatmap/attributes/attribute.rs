/// A beatmap attribute.
///
/// It contains either:
/// - the default value (5.0)
/// - a value taken from a [`Beatmap`] or mutated default value
/// - a user-given value that may be adjusted
/// - a user-given *fixed* value that will stay as-is
///
/// [`Beatmap`]: crate::Beatmap
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum BeatmapAttribute {
    /// No value has been set.
    ///
    /// Will be treated as the default value (5.0).
    #[default]
    None,
    /// Variable value taken from a [`Beatmap`] or mutated default value that
    /// may be overriden and adjusted based on mods and clock rate.
    ///
    /// [`Beatmap`]: crate::Beatmap
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
    /// The default value for a `BeatmapAttribute`.
    pub const DEFAULT: f32 = 5.0;

    /// Overwrites `self` with `other` if `other` is not `None`.
    #[must_use]
    pub const fn overwrite(self, other: Self) -> Self {
        if let Self::None = other { self } else { other }
    }

    /// Mutates the `Value` and `Given` variants.
    pub fn try_mutate(&mut self, f: impl Fn(&mut f32)) {
        if let Self::None = self {
            *self = Self::Value(Self::DEFAULT);
        }

        if let Self::Value(value) | Self::Given(value) = self {
            f(value);
        }
    }

    /// Sets the `Value` variant only.
    pub const fn try_set(&mut self, value: f32) {
        match self {
            Self::None => *self = Self::Value(value),
            Self::Value(old) => *old = value,
            _ => {}
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
            Self::None => f(Self::DEFAULT),
            Self::Value(value) | Self::Given(value) => f(value),
            Self::Fixed(fixed) => default(fixed),
        }
    }

    pub const fn get_raw(self) -> f32 {
        match self {
            Self::None => Self::DEFAULT,
            Self::Value(value) | Self::Given(value) | Self::Fixed(value) => value,
        }
    }
}
