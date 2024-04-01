use std::borrow::Cow;

use rosu_map::section::general::GameMode;

use crate::{
    any::{DifficultyAttributes, PerformanceAttributes},
    model::mode::IGameMode,
    Beatmap, Converted, Performance,
};

/// Turning a type into the generic [`IGameMode`]'s performance calculator.
pub trait IntoModePerformance<'map, M: IGameMode> {
    fn into_performance(self) -> M::Performance<'map>;
}

/// Turning a type into a performance calculator of any mode.
pub trait IntoPerformance<'a> {
    fn into_performance(self) -> Performance<'a>;
}

macro_rules! impl_from_mode {
    (
        $(
            $module:ident {
                $mode:ident, $diff:ident, $perf:ident
            }
        ,)*
    ) => {
        $(
            macro_rules! mode {
                () => { crate::$module::$mode };
            }

            impl<'map> IntoModePerformance<'map, mode!()> for Converted<'map, mode!()> {
                fn into_performance(self) -> <mode!() as IGameMode>::Performance<'map> {
                    <mode!() as IGameMode>::Performance::from_map_or_attrs(self.into())
                }
            }

            impl<'map> IntoModePerformance<'map, mode!()> for &'map Converted<'_, mode!()> {
                fn into_performance(self) -> <mode!() as IGameMode>::Performance<'map> {
                    <mode!() as IGameMode>::Performance::from_map_or_attrs(self.as_owned().into())
                }
            }

            impl<'map> IntoModePerformance<'map, mode!()> for crate::$module::$diff {
                fn into_performance(self) -> <mode!() as IGameMode>::Performance<'map> {
                    <mode!() as IGameMode>::Performance::from_map_or_attrs(self.into())
                }
            }

            impl<'map> IntoModePerformance<'map, mode!()> for crate::$module::$perf {
                fn into_performance(self) -> <mode!() as IGameMode>::Performance<'map> {
                    <mode!() as IGameMode>::Performance::from_map_or_attrs(self.difficulty.into())
                }
            }

            impl<'map> IntoPerformance<'map> for Converted<'map, mode!()> {
                fn into_performance(self) -> Performance<'map> {
                    Performance::$mode(
                        <Self as IntoModePerformance<'map, mode!()>>::into_performance(self)
                    )
                }
            }

            impl<'map> IntoPerformance<'map> for &'map Converted<'_, mode!()> {
                fn into_performance(self) -> Performance<'map> {
                    Performance::$mode(
                        <Self as IntoModePerformance<'map, mode!()>>::into_performance(self)
                    )
                }
            }

            impl<'a> IntoPerformance<'a> for crate::$module::$diff {
                fn into_performance(self) -> Performance<'a> {
                    Performance::$mode(
                        <Self as IntoModePerformance<'a, mode!()>>::into_performance(self)
                    )
                }
            }

            impl<'a> IntoPerformance<'a> for crate::$module::$perf {
                fn into_performance(self) -> Performance<'a> {
                    Performance::$mode(
                        <Self as IntoModePerformance<'a, mode!()>>::into_performance(self)
                    )
                }
            }
        )*
    };
}

impl_from_mode!(
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

impl<'a> IntoPerformance<'a> for Beatmap {
    fn into_performance(self) -> Performance<'a> {
        map_to_performance(self.mode, Cow::Owned(self))
    }
}

impl<'map> IntoPerformance<'map> for &'map Beatmap {
    fn into_performance(self) -> Performance<'map> {
        map_to_performance(self.mode, Cow::Borrowed(self))
    }
}

fn map_to_performance(mode: GameMode, map: Cow<'_, Beatmap>) -> Performance<'_> {
    match mode {
        GameMode::Osu => Performance::Osu(Converted::new(map).into()),
        GameMode::Taiko => Performance::Taiko(Converted::new(map).into()),
        GameMode::Catch => Performance::Catch(Converted::new(map).into()),
        GameMode::Mania => Performance::Mania(Converted::new(map).into()),
    }
}

impl<'a> IntoPerformance<'a> for DifficultyAttributes {
    fn into_performance(self) -> Performance<'a> {
        match self {
            Self::Osu(attrs) => Performance::Osu(attrs.into()),
            Self::Taiko(attrs) => Performance::Taiko(attrs.into()),
            Self::Catch(attrs) => Performance::Catch(attrs.into()),
            Self::Mania(attrs) => Performance::Mania(attrs.into()),
        }
    }
}

impl<'a> IntoPerformance<'a> for PerformanceAttributes {
    fn into_performance(self) -> Performance<'a> {
        match self {
            Self::Osu(attrs) => Performance::Osu(attrs.difficulty.into()),
            Self::Taiko(attrs) => Performance::Taiko(attrs.difficulty.into()),
            Self::Catch(attrs) => Performance::Catch(attrs.difficulty.into()),
            Self::Mania(attrs) => Performance::Mania(attrs.difficulty.into()),
        }
    }
}
