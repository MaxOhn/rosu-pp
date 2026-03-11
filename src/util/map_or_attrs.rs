use std::{
    borrow::Cow,
    fmt::{Debug, Formatter, Result as FmtResult},
};

use crate::{
    Beatmap, Difficulty,
    model::mode::{ConvertError, IGameMode},
};

pub enum MapOrAttrs<'map, M: IGameMode> {
    Map(Cow<'map, Beatmap>),
    Attrs(M::DifficultyAttributes),
}

impl<M: IGameMode> MapOrAttrs<'_, M> {
    pub fn insert_attrs(&mut self, difficulty: &Difficulty) -> Result<(), ConvertError> {
        match self {
            Self::Map(map) => {
                let attrs = difficulty.calculate_for_mode::<M>(map)?;
                *self = Self::Attrs(attrs);
            }
            Self::Attrs(_) => {}
        }

        Ok(())
    }

    /// Get a reference to the attributes.
    ///
    /// # Safety
    /// Caller must ensure that this [`MapOrAttrs`] contains attributes.
    pub const unsafe fn get_attrs(&self) -> &M::DifficultyAttributes {
        // Returning an immutable reference while requiring a mutable reference
        // as argument, unfortunately, makes it impossible to pass another
        // mutable reference later on. Instead we split it up into two
        // functions: first `insert_attrs` and then `get_attrs`.
        match self {
            Self::Attrs(attrs) => attrs,
            // SAFETY: Up to the caller to uphold
            Self::Map(_) => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

impl<M> Clone for MapOrAttrs<'_, M>
where
    M: IGameMode,
    M::DifficultyAttributes: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Map(map) => Self::Map(map.clone()),
            Self::Attrs(attrs) => Self::Attrs(attrs.clone()),
        }
    }
}

impl<M> Debug for MapOrAttrs<'_, M>
where
    M: IGameMode,
    M::DifficultyAttributes: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        /// Provides a more reasonable Debug implementation for [`Beatmap`] by
        /// not printing list items in full, but rather their lengths.
        struct SlimBeatmap<'a>(&'a Beatmap);

        impl Debug for SlimBeatmap<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                let Beatmap {
                    version,
                    is_convert,
                    stack_leniency,
                    mode,
                    ar,
                    cs,
                    hp,
                    od,
                    slider_multiplier,
                    slider_tick_rate,
                    breaks,
                    timing_points,
                    difficulty_points,
                    effect_points,
                    hit_objects,
                    hit_sounds,
                } = self.0;

                f.debug_struct("Beatmap")
                    .field("version", version)
                    .field("is_convert", is_convert)
                    .field("stack_leniency", stack_leniency)
                    .field("mode", mode)
                    .field("ar", ar)
                    .field("cs", cs)
                    .field("hp", hp)
                    .field("od", od)
                    .field("slider_multiplier", slider_multiplier)
                    .field("slider_tick_rate", slider_tick_rate)
                    .field("breaks", &breaks.len())
                    .field("timing_points", &timing_points.len())
                    .field("difficulty_points", &difficulty_points.len())
                    .field("effect_points", &effect_points.len())
                    .field("hit_objects", &hit_objects.len())
                    .field("hit_sounds", &hit_sounds.len())
                    .finish()
            }
        }

        match self {
            Self::Map(map) => f.debug_tuple("Map").field(&SlimBeatmap(map)).finish(),
            Self::Attrs(attrs) => f.debug_tuple("Attrs").field(attrs).finish(),
        }
    }
}

impl<M> PartialEq for MapOrAttrs<'_, M>
where
    M: IGameMode,
    M::DifficultyAttributes: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Map(a), Self::Map(b)) => a == b,
            (Self::Attrs(a), Self::Attrs(b)) => a == b,
            _ => false,
        }
    }
}

impl<'map, M: IGameMode> From<&'map Beatmap> for MapOrAttrs<'map, M> {
    fn from(map: &'map Beatmap) -> Self {
        Self::Map(Cow::Borrowed(map))
    }
}

impl<M: IGameMode> From<Beatmap> for MapOrAttrs<'_, M> {
    fn from(map: Beatmap) -> Self {
        Self::Map(Cow::Owned(map))
    }
}

macro_rules! from_attrs {
    (
        $(
            $module:ident {
                $mode:ident, $diff:ident, $perf:ident
            }
        ,)*
    ) => {
        $(
            impl From<crate::$module::$diff> for MapOrAttrs<'_, crate::$module::$mode> {
                fn from(attrs: crate::$module::$diff) -> Self {
                    Self::Attrs(attrs)
                }
            }

            impl From<crate::$module::$perf> for MapOrAttrs<'_, crate::$module::$mode> {
                fn from(attrs: crate::$module::$perf) -> Self {
                    Self::Attrs(attrs.difficulty)
                }
            }
        )*
    };
}

from_attrs!(
    osu {
        Osu,
        OsuDifficultyAttributes,
        OsuPerformanceAttributes
    },
    taiko {
        Taiko,
        TaikoDifficultyAttributes,
        TaikoPerformanceAttributes
    },
    catch {
        Catch,
        CatchDifficultyAttributes,
        CatchPerformanceAttributes
    },
    mania {
        Mania,
        ManiaDifficultyAttributes,
        ManiaPerformanceAttributes
    },
);
