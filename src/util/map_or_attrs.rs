use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::model::{beatmap::Converted, mode::IGameMode};

pub enum MapOrAttrs<'map, M: IGameMode> {
    Map(Converted<'map, M>),
    Attrs(M::DifficultyAttributes),
}

impl<'map, M: IGameMode> MapOrAttrs<'map, M> {
    /// Return a mutable reference to the attributes.
    ///
    /// If `self` is of variant `Map`, store `attrs` in `self`, and return a
    /// mutable reference to it.
    pub fn insert_attrs(&mut self, attrs: M::DifficultyAttributes) -> &mut M::DifficultyAttributes {
        // TODO: dont match, just overwrite
        match self {
            MapOrAttrs::Map(_) => {
                *self = Self::Attrs(attrs);

                let Self::Attrs(ref mut attrs) = self else {
                    unreachable!()
                };

                attrs
            }
            MapOrAttrs::Attrs(ref mut attrs) => attrs,
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
            Self::Map(converted) => Self::Map(converted.clone()),
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
        match self {
            Self::Map(converted) => f.debug_tuple("Map").field(converted).finish(),
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

impl<'map, M: IGameMode> From<Converted<'map, M>> for MapOrAttrs<'map, M> {
    fn from(converted: Converted<'map, M>) -> Self {
        Self::Map(converted)
    }
}

impl<'map, M: IGameMode> From<&'map Converted<'_, M>> for MapOrAttrs<'map, M> {
    fn from(converted: &'map Converted<'_, M>) -> Self {
        Self::Map(converted.as_owned())
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
