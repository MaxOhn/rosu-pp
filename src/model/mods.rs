use std::fmt::{Debug, Formatter, Result as FmtResult};

use rosu_mods::{
    generated_mods::{
        DifficultyAdjustCatch, DifficultyAdjustMania, DifficultyAdjustOsu, DifficultyAdjustTaiko,
    },
    GameMod, GameModIntermode, GameMods as GameModsLazer, GameModsIntermode, GameModsLegacy,
};

/// Collection of game mods.
///
/// This type can be created through its `From<T>` implementations where `T`
/// can be
/// - `u32`
/// - [`rosu_mods::GameModsLegacy`]
/// - [`rosu_mods::GameMods`]
/// - [`rosu_mods::GameModsIntermode`]
/// - [`&rosu_mods::GameModsIntermode`](rosu_mods::GameModsIntermode)
///
/// # Example
///
/// ```
/// use akatsuki_pp::GameMods;
/// use rosu_mods::{GameModsIntermode, GameModsLegacy, GameMods as GameModsLazer};
///
/// let int = GameMods::from(64 + 8);
/// let legacy = GameMods::from(GameModsLegacy::Hidden | GameModsLegacy::Easy);
/// let lazer = GameMods::from(GameModsLazer::new());
/// let intermode = GameMods::from(GameModsIntermode::new());
/// ```
#[derive(Clone, PartialEq)]
pub struct GameMods {
    inner: GameModsInner,
}

impl Debug for GameMods {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.inner {
            GameModsInner::Lazer(ref mods) => Debug::fmt(mods, f),
            GameModsInner::Intermode(ref mods) => Debug::fmt(mods, f),
            GameModsInner::Legacy(ref mods) => Debug::fmt(mods, f),
        }
    }
}

/// Inner type of [`GameMods`] so that remote types contained in variants don't
/// need to be re-exported.
#[derive(Clone, PartialEq)]
enum GameModsInner {
    Lazer(GameModsLazer),
    Intermode(GameModsIntermode),
    Legacy(GameModsLegacy),
}

impl GameMods {
    pub(crate) const DEFAULT: Self = Self {
        inner: GameModsInner::Legacy(GameModsLegacy::NoMod),
    };

    /// Returns the mods' clock rate.
    ///
    /// In case of variable clock rates like for `WindUp`, this will return
    /// `1.0`.
    pub(crate) fn clock_rate(&self) -> f64 {
        match self.inner {
            GameModsInner::Lazer(ref mods) => mods.clock_rate().unwrap_or(1.0),
            GameModsInner::Intermode(ref mods) => mods.legacy_clock_rate(),
            GameModsInner::Legacy(mods) => mods.clock_rate(),
        }
    }

    pub(crate) fn od_ar_hp_multiplier(&self) -> f64 {
        if self.hr() {
            1.4
        } else if self.ez() {
            0.5
        } else {
            1.0
        }
    }

    /// Check whether the mods enable `hardrock_offsets`.
    pub(crate) fn hardrock_offsets(&self) -> bool {
        fn custom_hardrock_offsets(mods: &GameMods) -> Option<bool> {
            match mods.inner {
                GameModsInner::Lazer(ref mods) => mods.iter().find_map(|gamemod| match gamemod {
                    GameMod::DifficultyAdjustCatch(DifficultyAdjustCatch {
                        hard_rock_offsets,
                        ..
                    }) => *hard_rock_offsets,
                    _ => None,
                }),
                GameModsInner::Intermode(_) | GameModsInner::Legacy(_) => None,
            }
        }

        custom_hardrock_offsets(self).unwrap_or_else(|| self.hr())
    }

    pub(crate) fn no_slider_head_acc(&self, lazer: bool) -> bool {
        match self.inner {
            GameModsInner::Lazer(ref mods) => mods
                .iter()
                .find_map(|m| match m {
                    GameMod::ClassicOsu(cl) => Some(cl.no_slider_head_accuracy.unwrap_or(true)),
                    _ => None,
                })
                .unwrap_or(!lazer),
            GameModsInner::Intermode(ref mods) => {
                mods.contains(GameModIntermode::Classic) || !lazer
            }
            GameModsInner::Legacy(_) => !lazer,
        }
    }

    pub(crate) fn reflection(&self) -> Reflection {
        match self.inner {
            GameModsInner::Lazer(ref mods) => {
                if mods.contains_intermode(GameModIntermode::HardRock) {
                    return Reflection::Vertical;
                }

                mods.iter()
                    .find_map(|m| match m {
                        GameMod::MirrorOsu(mr) => match mr.reflection.as_deref() {
                            None => Some(Reflection::Horizontal),
                            Some("1") => Some(Reflection::Vertical),
                            Some("2") => Some(Reflection::Both),
                            Some(_) => Some(Reflection::None),
                        },
                        GameMod::MirrorCatch(_) => Some(Reflection::Horizontal),
                        _ => None,
                    })
                    .unwrap_or(Reflection::None)
            }
            GameModsInner::Intermode(ref mods) => {
                if mods.contains(GameModIntermode::HardRock) {
                    Reflection::Vertical
                } else {
                    Reflection::None
                }
            }
            GameModsInner::Legacy(mods) => {
                if mods.contains(GameModsLegacy::HardRock) {
                    Reflection::Vertical
                } else {
                    Reflection::None
                }
            }
        }
    }

    pub(crate) fn mania_keys(&self) -> Option<f32> {
        match self.inner {
            GameModsInner::Lazer(ref mods) => {
                if mods.contains_intermode(GameModIntermode::OneKey) {
                    Some(1.0)
                } else if mods.contains_intermode(GameModIntermode::TwoKeys) {
                    Some(2.0)
                } else if mods.contains_intermode(GameModIntermode::ThreeKeys) {
                    Some(3.0)
                } else if mods.contains_intermode(GameModIntermode::FourKeys) {
                    Some(4.0)
                } else if mods.contains_intermode(GameModIntermode::FiveKeys) {
                    Some(5.0)
                } else if mods.contains_intermode(GameModIntermode::SixKeys) {
                    Some(6.0)
                } else if mods.contains_intermode(GameModIntermode::SevenKeys) {
                    Some(7.0)
                } else if mods.contains_intermode(GameModIntermode::EightKeys) {
                    Some(8.0)
                } else if mods.contains_intermode(GameModIntermode::NineKeys) {
                    Some(9.0)
                } else if mods.contains_intermode(GameModIntermode::TenKeys) {
                    Some(10.0)
                } else {
                    None
                }
            }
            GameModsInner::Intermode(ref mods) => {
                if mods.contains(GameModIntermode::OneKey) {
                    Some(1.0)
                } else if mods.contains(GameModIntermode::TwoKeys) {
                    Some(2.0)
                } else if mods.contains(GameModIntermode::ThreeKeys) {
                    Some(3.0)
                } else if mods.contains(GameModIntermode::FourKeys) {
                    Some(4.0)
                } else if mods.contains(GameModIntermode::FiveKeys) {
                    Some(5.0)
                } else if mods.contains(GameModIntermode::SixKeys) {
                    Some(6.0)
                } else if mods.contains(GameModIntermode::SevenKeys) {
                    Some(7.0)
                } else if mods.contains(GameModIntermode::EightKeys) {
                    Some(8.0)
                } else if mods.contains(GameModIntermode::NineKeys) {
                    Some(9.0)
                } else if mods.contains(GameModIntermode::TenKeys) {
                    Some(10.0)
                } else {
                    None
                }
            }
            GameModsInner::Legacy(ref mods) => {
                if mods.contains(GameModsLegacy::Key1) {
                    Some(1.0)
                } else if mods.contains(GameModsLegacy::Key2) {
                    Some(2.0)
                } else if mods.contains(GameModsLegacy::Key3) {
                    Some(3.0)
                } else if mods.contains(GameModsLegacy::Key4) {
                    Some(4.0)
                } else if mods.contains(GameModsLegacy::Key5) {
                    Some(5.0)
                } else if mods.contains(GameModsLegacy::Key6) {
                    Some(6.0)
                } else if mods.contains(GameModsLegacy::Key7) {
                    Some(7.0)
                } else if mods.contains(GameModsLegacy::Key8) {
                    Some(8.0)
                } else if mods.contains(GameModsLegacy::Key9) {
                    Some(9.0)
                } else {
                    None
                }
            }
        }
    }
}

macro_rules! impl_map_attr {
    ( $( $fn:ident: $field:ident [ $( $mode:ident ),* ] [$s:literal] ;)* ) => {
        impl GameMods {
            $(
                #[doc = "Check whether the mods specify a custom "]
                #[doc = $s]
                #[doc = "value."]
                pub(crate) fn $fn(&self) -> Option<f64> {
                    match self.inner {
                        GameModsInner::Lazer(ref mods) => mods.iter().find_map(|gamemod| match gamemod {
                            $( impl_map_attr!( @ $mode $field) => *$field, )*
                            _ => None,
                        }),
                        GameModsInner::Intermode(_) | GameModsInner::Legacy(_) => None,
                    }
                }
            )*
        }
    };

    ( @ Osu $field:ident) => { GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu { $field, .. }) };
    ( @ Taiko $field:ident) => { GameMod::DifficultyAdjustTaiko(DifficultyAdjustTaiko { $field, .. }) };
    ( @ Catch $field:ident) => { GameMod::DifficultyAdjustCatch(DifficultyAdjustCatch { $field, .. }) };
    ( @ Mania $field:ident) => { GameMod::DifficultyAdjustMania(DifficultyAdjustMania { $field, .. }) };
}

impl_map_attr! {
    ar: approach_rate [Osu, Catch] ["ar"];
    cs: circle_size [Osu, Catch] ["cs"];
    hp: drain_rate [Osu, Taiko, Catch, Mania] ["hp"];
    od: overall_difficulty [Osu, Taiko, Catch, Mania] ["od"];
}

macro_rules! impl_has_mod {
    ( $( $fn:ident: $sign:tt $name:ident [ $s:literal ], )* ) => {
        impl GameMods {
            $(
                // workaround for <https://github.com/rust-lang/rust-analyzer/issues/8092>
                #[doc = "Check whether [`GameMods`] contain `"]
                #[doc = $s]
                #[doc = "`."]
                pub(crate) fn $fn(&self) -> bool {
                    match self.inner {
                        GameModsInner::Lazer(ref mods) => {
                            mods.contains_intermode(GameModIntermode::$name)
                        },
                        GameModsInner::Intermode(ref mods) => {
                            mods.contains(GameModIntermode::$name)
                        },
                        GameModsInner::Legacy(_mods) => {
                            impl_has_mod!(LEGACY $sign $name _mods)
                        },
                    }
                }
            )*
        }
    };

    ( LEGACY + $name:ident $mods:ident ) => {
        $mods.contains(GameModsLegacy::$name)
    };

    ( LEGACY - $name:ident $mods:ident ) => {
        false
    };
}

impl_has_mod! {
    nf: + NoFail ["NoFail"],
    ez: + Easy ["Easy"],
    td: + TouchDevice ["TouchDevice"],
    hd: + Hidden ["Hidden"],
    hr: + HardRock ["HardRock"],
    rx: + Relax ["Relax"],
    fl: + Flashlight ["Flashlight"],
    so: + SpunOut ["SpunOut"],
    dt: + DoubleTime ["DoubleTime"],
    nc: + Nightcore ["Nightcore"],
    ht: + HalfTime ["HalfTime"],
    ap: + Autopilot ["Autopilot"],
    bl: - Blinds ["Blinds"],
    tc: - Traceable ["Traceable"],
}

impl Default for GameMods {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<GameModsLazer> for GameMods {
    fn from(mods: GameModsLazer) -> Self {
        Self {
            inner: GameModsInner::Lazer(mods),
        }
    }
}

impl From<GameModsIntermode> for GameMods {
    fn from(mods: GameModsIntermode) -> Self {
        Self {
            inner: GameModsInner::Intermode(mods),
        }
    }
}

impl From<&GameModsIntermode> for GameMods {
    fn from(mods: &GameModsIntermode) -> Self {
        // If only legacy mods are set, use `GameModsLegacy` and thus avoid
        // allocating an owned `GameModsIntermode` instance.
        match mods.checked_bits() {
            Some(bits) => bits.into(),
            None => mods.to_owned().into(),
        }
    }
}

impl From<GameModsLegacy> for GameMods {
    fn from(mods: GameModsLegacy) -> Self {
        Self {
            inner: GameModsInner::Legacy(mods),
        }
    }
}

impl From<u32> for GameMods {
    fn from(bits: u32) -> Self {
        GameModsLegacy::from_bits(bits).into()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reflection {
    None,
    Vertical,
    Horizontal,
    Both,
}
