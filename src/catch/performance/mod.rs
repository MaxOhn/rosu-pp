use std::cmp::{self, Ordering};

use crate::{
    any::{Difficulty, IntoModePerformance, IntoPerformance},
    model::mods::GameMods,
    osu::OsuPerformance,
    util::map_or_attrs::MapOrAttrs,
    Performance,
};

use super::{
    attributes::{CatchDifficultyAttributes, CatchPerformanceAttributes},
    score_state::CatchScoreState,
    Catch,
};

pub mod gradual;

/// Performance calculator on osu!catch maps.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CatchPerformance<'map> {
    map_or_attrs: MapOrAttrs<'map, Catch>,
    difficulty: Difficulty,
    acc: Option<f64>,
    combo: Option<u32>,
    fruits: Option<u32>,
    droplets: Option<u32>,
    tiny_droplets: Option<u32>,
    tiny_droplet_misses: Option<u32>,
    misses: Option<u32>,
}

impl<'map> CatchPerformance<'map> {
    /// Create a new performance calculator for osu!catch maps.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`CatchDifficultyAttributes`]
    ///   or [`CatchPerformanceAttributes`])
    /// - a beatmap ([`CatchBeatmap<'map>`])
    ///
    /// If a map is given, difficulty attributes will need to be calculated
    /// internally which is a costly operation. Hence, passing attributes
    /// should be prefered.
    ///
    /// However, when passing previously calculated attributes, make sure they
    /// have been calculated for the same map and [`Difficulty`] settings.
    /// Otherwise, the final attributes will be incorrect.
    ///
    /// [`CatchBeatmap<'map>`]: crate::catch::CatchBeatmap
    pub fn new(map_or_attrs: impl IntoModePerformance<'map, Catch>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Try to create a new performance calculator for osu!catch maps.
    ///
    /// Returns `None` if `map_or_attrs` does not belong to osu!catch e.g.
    /// a [`Converted`], [`DifficultyAttributes`], or [`PerformanceAttributes`]
    /// of a different mode.
    ///
    /// See [`CatchPerformance::new`] for more information.
    ///
    /// [`Converted`]: crate::model::beatmap::Converted
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`PerformanceAttributes`]: crate::any::PerformanceAttributes
    pub fn try_new(map_or_attrs: impl IntoPerformance<'map>) -> Option<Self> {
        if let Performance::Catch(calc) = map_or_attrs.into_performance() {
            Some(calc)
        } else {
            None
        }
    }

    /// Specify mods.
    ///
    /// Accepted types are
    /// - `u32`
    /// - [`rosu_mods::GameModsLegacy`]
    /// - [`rosu_mods::GameMods`]
    /// - [`rosu_mods::GameModsIntermode`]
    /// - [`&rosu_mods::GameModsIntermode`](rosu_mods::GameModsIntermode)
    ///
    /// See <https://github.com/ppy/osu-api/wiki#mods>
    pub fn mods(mut self, mods: impl Into<GameMods>) -> Self {
        self.difficulty = self.difficulty.mods(mods);

        self
    }

    /// Specify the max combo of the play.
    pub const fn combo(mut self, combo: u32) -> Self {
        self.combo = Some(combo);

        self
    }

    /// Specify the amount of fruits of a play i.e. n300.
    pub const fn fruits(mut self, n_fruits: u32) -> Self {
        self.fruits = Some(n_fruits);

        self
    }

    /// Specify the amount of droplets of a play i.e. n100.
    pub const fn droplets(mut self, n_droplets: u32) -> Self {
        self.droplets = Some(n_droplets);

        self
    }

    /// Specify the amount of tiny droplets of a play i.e. n50.
    pub const fn tiny_droplets(mut self, n_tiny_droplets: u32) -> Self {
        self.tiny_droplets = Some(n_tiny_droplets);

        self
    }

    /// Specify the amount of tiny droplet misses of a play i.e. `n_katu`.
    pub const fn tiny_droplet_misses(mut self, n_tiny_droplet_misses: u32) -> Self {
        self.tiny_droplet_misses = Some(n_tiny_droplet_misses);

        self
    }

    /// Specify the amount of fruit / droplet misses of the play.
    pub const fn misses(mut self, n_misses: u32) -> Self {
        self.misses = Some(n_misses);

        self
    }

    /// Use the specified settings of the given [`Difficulty`].
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects,
    /// instead of using [`CatchPerformance`] multiple times with different
    /// `passed_objects`, you should use [`CatchGradualPerformance`].
    ///
    /// [`CatchGradualPerformance`]: crate::catch::CatchGradualPerformance
    pub fn passed_objects(mut self, passed_objects: u32) -> Self {
        self.difficulty = self.difficulty.passed_objects(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | 0.01    | 100     |
    pub fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.difficulty = self.difficulty.clock_rate(clock_rate);

        self
    }

    /// Override a beatmap's set AR.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn ar(mut self, ar: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.ar(ar, with_mods);

        self
    }

    /// Override a beatmap's set CS.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn cs(mut self, cs: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.cs(cs, with_mods);

        self
    }

    /// Override a beatmap's set HP.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn hp(mut self, hp: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.hp(hp, with_mods);

        self
    }

    /// Override a beatmap's set OD.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn od(mut self, od: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.od(od, with_mods);

        self
    }

    /// Adjust patterns as if the HR mod is enabled.
    pub fn hardrock_offsets(mut self, hardrock_offsets: bool) -> Self {
        self.difficulty = self.difficulty.hardrock_offsets(hardrock_offsets);

        self
    }

    /// Provide parameters through an [`CatchScoreState`].
    #[allow(clippy::needless_pass_by_value)]
    pub const fn state(mut self, state: CatchScoreState) -> Self {
        let CatchScoreState {
            max_combo,
            fruits: n_fruits,
            droplets: n_droplets,
            tiny_droplets: n_tiny_droplets,
            tiny_droplet_misses: n_tiny_droplet_misses,
            misses,
        } = state;

        self.combo = Some(max_combo);
        self.fruits = Some(n_fruits);
        self.droplets = Some(n_droplets);
        self.tiny_droplets = Some(n_tiny_droplets);
        self.tiny_droplet_misses = Some(n_tiny_droplet_misses);
        self.misses = Some(misses);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc.clamp(0.0, 100.0) / 100.0);

        self
    }

    /// Create the [`CatchScoreState`] that will be used for performance calculation.
    #[allow(clippy::too_many_lines)]
    pub fn generate_state(&mut self) -> CatchScoreState {
        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => {
                let attrs = self.difficulty.with_mode().calculate(map);

                self.map_or_attrs.insert_attrs(attrs)
            }
            MapOrAttrs::Attrs(ref attrs) => attrs,
        };

        let misses = self
            .misses
            .map_or(0, |n| cmp::min(n, attrs.n_fruits + attrs.n_droplets));

        let max_combo = self.combo.unwrap_or_else(|| attrs.max_combo() - misses);

        let mut best_state = CatchScoreState {
            max_combo,
            misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;

        let (n_fruits, n_droplets) = match (self.fruits, self.droplets) {
            (Some(mut n_fruits), Some(mut n_droplets)) => {
                let n_remaining = (attrs.n_fruits + attrs.n_droplets)
                    .saturating_sub(n_fruits + n_droplets + misses);

                let new_droplets =
                    cmp::min(n_remaining, attrs.n_droplets.saturating_sub(n_droplets));
                n_droplets += new_droplets;
                n_fruits += n_remaining - new_droplets;

                n_fruits = cmp::min(
                    n_fruits,
                    (attrs.n_fruits + attrs.n_droplets).saturating_sub(n_droplets + misses),
                );
                n_droplets = cmp::min(
                    n_droplets,
                    attrs.n_fruits + attrs.n_droplets - n_fruits - misses,
                );

                (n_fruits, n_droplets)
            }
            (Some(mut n_fruits), None) => {
                let n_droplets = attrs
                    .n_droplets
                    .saturating_sub(misses.saturating_sub(attrs.n_fruits.saturating_sub(n_fruits)));

                n_fruits = attrs.n_fruits + attrs.n_droplets - misses - n_droplets;

                (n_fruits, n_droplets)
            }
            (None, Some(mut n_droplets)) => {
                let n_fruits = attrs.n_fruits.saturating_sub(
                    misses.saturating_sub(attrs.n_droplets.saturating_sub(n_droplets)),
                );

                n_droplets = attrs.n_fruits + attrs.n_droplets - misses - n_fruits;

                (n_fruits, n_droplets)
            }
            (None, None) => {
                let n_droplets = attrs.n_droplets.saturating_sub(misses);
                let n_fruits =
                    attrs.n_fruits - (misses - (attrs.n_droplets.saturating_sub(n_droplets)));

                (n_fruits, n_droplets)
            }
        };

        best_state.fruits = n_fruits;
        best_state.droplets = n_droplets;

        let mut find_best_tiny_droplets = |acc: f64| {
            let raw_tiny_droplets = acc
                * f64::from(attrs.n_fruits + attrs.n_droplets + attrs.n_tiny_droplets)
                - f64::from(n_fruits + n_droplets);
            let min_tiny_droplets =
                cmp::min(attrs.n_tiny_droplets, raw_tiny_droplets.floor() as u32);
            let max_tiny_droplets =
                cmp::min(attrs.n_tiny_droplets, raw_tiny_droplets.ceil() as u32);

            for n_tiny_droplets in min_tiny_droplets..=max_tiny_droplets {
                let n_tiny_droplet_misses = attrs.n_tiny_droplets - n_tiny_droplets;

                let curr_acc = accuracy(
                    n_fruits,
                    n_droplets,
                    n_tiny_droplets,
                    n_tiny_droplet_misses,
                    misses,
                );
                let curr_dist = (acc - curr_acc).abs();

                if curr_dist < best_dist {
                    best_dist = curr_dist;
                    best_state.tiny_droplets = n_tiny_droplets;
                    best_state.tiny_droplet_misses = n_tiny_droplet_misses;
                }
            }
        };

        #[allow(clippy::single_match_else)]
        match (self.tiny_droplets, self.tiny_droplet_misses) {
            (Some(n_tiny_droplets), Some(n_tiny_droplet_misses)) => match self.acc {
                Some(acc) => {
                    match (n_tiny_droplets + n_tiny_droplet_misses).cmp(&attrs.n_tiny_droplets) {
                        Ordering::Equal => {
                            best_state.tiny_droplets = n_tiny_droplets;
                            best_state.tiny_droplet_misses = n_tiny_droplet_misses;
                        }
                        Ordering::Less | Ordering::Greater => find_best_tiny_droplets(acc),
                    }
                }
                None => {
                    let n_remaining = attrs
                        .n_tiny_droplets
                        .saturating_sub(n_tiny_droplets + n_tiny_droplet_misses);

                    best_state.tiny_droplets = n_tiny_droplets + n_remaining;
                    best_state.tiny_droplet_misses = n_tiny_droplet_misses;
                }
            },
            (Some(n_tiny_droplets), None) => {
                best_state.tiny_droplets = cmp::min(attrs.n_tiny_droplets, n_tiny_droplets);
                best_state.tiny_droplet_misses =
                    attrs.n_tiny_droplets.saturating_sub(n_tiny_droplets);
            }
            (None, Some(n_tiny_droplet_misses)) => {
                best_state.tiny_droplets =
                    attrs.n_tiny_droplets.saturating_sub(n_tiny_droplet_misses);
                best_state.tiny_droplet_misses =
                    cmp::min(attrs.n_tiny_droplets, n_tiny_droplet_misses);
            }
            (None, None) => match self.acc {
                Some(acc) => find_best_tiny_droplets(acc),
                None => best_state.tiny_droplets = attrs.n_tiny_droplets,
            },
        }

        self.combo = Some(best_state.max_combo);
        self.fruits = Some(best_state.fruits);
        self.droplets = Some(best_state.droplets);
        self.tiny_droplets = Some(best_state.tiny_droplets);
        self.tiny_droplet_misses = Some(best_state.tiny_droplet_misses);
        self.misses = Some(best_state.misses);

        best_state
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> CatchPerformanceAttributes {
        let state = self.generate_state();

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => self.difficulty.with_mode().calculate(map),
            MapOrAttrs::Attrs(attrs) => attrs,
        };

        let inner = CatchPerformanceInner {
            attrs,
            mods: self.difficulty.get_mods(),
            state,
        };

        inner.calculate()
    }

    pub(crate) const fn from_map_or_attrs(map_or_attrs: MapOrAttrs<'map, Catch>) -> Self {
        Self {
            map_or_attrs,
            difficulty: Difficulty::new(),
            acc: None,
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: None,
        }
    }
}

impl<'map> TryFrom<OsuPerformance<'map>> for CatchPerformance<'map> {
    type Error = OsuPerformance<'map>;

    /// Try to create [`CatchPerformance`] through [`OsuPerformance`].
    ///
    /// Returns `None` if [`OsuPerformance`] does not contain a beatmap, i.e.
    /// if it was constructed through attributes or
    /// [`OsuPerformance::generate_state`] was called.
    fn try_from(mut osu: OsuPerformance<'map>) -> Result<Self, Self::Error> {
        let MapOrAttrs::Map(converted) = osu.map_or_attrs else {
            return Err(osu);
        };

        let map = match converted.try_convert() {
            Ok(map) => map,
            Err(map) => {
                osu.map_or_attrs = MapOrAttrs::Map(map);

                return Err(osu);
            }
        };

        let OsuPerformance {
            map_or_attrs: _,
            difficulty,
            acc,
            combo,
            large_tick_hits: _,
            slider_end_hits: _,
            n300,
            n100,
            n50,
            misses,
            hitresult_priority: _,
        } = osu;

        Ok(Self {
            map_or_attrs: MapOrAttrs::Map(map),
            difficulty,
            acc,
            combo,
            fruits: n300,
            droplets: n100,
            tiny_droplets: n50,
            tiny_droplet_misses: None,
            misses,
        })
    }
}

impl<'map, T: IntoModePerformance<'map, Catch>> From<T> for CatchPerformance<'map> {
    fn from(into: T) -> Self {
        into.into_performance()
    }
}

struct CatchPerformanceInner<'mods> {
    attrs: CatchDifficultyAttributes,
    mods: &'mods GameMods,
    state: CatchScoreState,
}

impl CatchPerformanceInner<'_> {
    fn calculate(self) -> CatchPerformanceAttributes {
        let attributes = &self.attrs;
        let stars = attributes.stars;
        let max_combo = attributes.max_combo();

        // Relying heavily on aim
        let mut pp = (5.0 * (stars / 0.0049).max(1.0) - 4.0).powf(2.0) / 100_000.0;

        let mut combo_hits = self.combo_hits();

        if combo_hits == 0 {
            combo_hits = max_combo;
        }

        // Longer maps are worth more
        let mut len_bonus = 0.95 + 0.3 * (f64::from(combo_hits) / 2500.0).min(1.0);

        if combo_hits > 2500 {
            len_bonus += (f64::from(combo_hits) / 2500.0).log10() * 0.475;
        }

        pp *= len_bonus;

        // Penalize misses exponentially
        pp *= 0.97_f64.powf(f64::from(self.state.misses));

        // Combo scaling
        if self.state.max_combo > 0 {
            pp *= (f64::from(self.state.max_combo).powf(0.8) / f64::from(max_combo).powf(0.8))
                .min(1.0);
        }

        // AR scaling
        let ar = attributes.ar;
        let mut ar_factor = 1.0;
        if ar > 9.0 {
            ar_factor += 0.1 * (ar - 9.0) + f64::from(u8::from(ar > 10.0)) * 0.1 * (ar - 10.0);
        } else if ar < 8.0 {
            ar_factor += 0.025 * (8.0 - ar);
        }
        pp *= ar_factor;

        // HD bonus
        if self.mods.hd() {
            if ar <= 10.0 {
                pp *= 1.05 + 0.075 * (10.0 - ar);
            } else if ar > 10.0 {
                pp *= 1.01 + 0.04 * (11.0 - ar.min(11.0));
            }
        }

        // FL bonus
        if self.mods.fl() {
            pp *= 1.35 * len_bonus;
        }

        // Accuracy scaling
        pp *= self.state.accuracy().powf(5.5);

        // NF penalty
        if self.mods.nf() {
            pp *= (1.0 - 0.02 * f64::from(self.state.misses)).max(0.9);
        }

        CatchPerformanceAttributes {
            difficulty: self.attrs,
            pp,
        }
    }

    const fn combo_hits(&self) -> u32 {
        self.state.fruits + self.state.droplets + self.state.misses
    }
}

fn accuracy(
    n_fruits: u32,
    n_droplets: u32,
    n_tiny_droplets: u32,
    n_tiny_droplet_misses: u32,
    misses: u32,
) -> f64 {
    let numerator = n_fruits + n_droplets + n_tiny_droplets;
    let denominator = numerator + n_tiny_droplet_misses + misses;

    f64::from(numerator) / f64::from(denominator)
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use proptest::prelude::*;
    use rosu_map::section::general::GameMode;

    use crate::{
        any::{DifficultyAttributes, PerformanceAttributes},
        osu::{Osu, OsuDifficultyAttributes, OsuPerformanceAttributes},
        Beatmap,
    };

    use super::*;

    static ATTRS: OnceLock<CatchDifficultyAttributes> = OnceLock::new();

    const N_FRUITS: u32 = 728;
    const N_DROPLETS: u32 = 2;
    const N_TINY_DROPLETS: u32 = 263;

    fn beatmap() -> Beatmap {
        Beatmap::from_path("./resources/2118524.osu").unwrap()
    }

    fn attrs() -> CatchDifficultyAttributes {
        ATTRS
            .get_or_init(|| {
                let converted = beatmap().unchecked_into_converted::<Catch>();
                let attrs = Difficulty::new().with_mode().calculate(&converted);

                assert_eq!(N_FRUITS, attrs.n_fruits);
                assert_eq!(N_DROPLETS, attrs.n_droplets);
                assert_eq!(N_TINY_DROPLETS, attrs.n_tiny_droplets);

                attrs
            })
            .to_owned()
    }

    /// Checks all remaining hitresult combinations w.r.t. the given parameters
    /// and returns the [`CatchScoreState`] that matches `acc` the best.
    ///
    /// Very slow but accurate.
    fn brute_force_best(
        acc: f64,
        n_fruits: Option<u32>,
        n_droplets: Option<u32>,
        n_tiny_droplets: Option<u32>,
        n_tiny_droplet_misses: Option<u32>,
        misses: u32,
    ) -> CatchScoreState {
        let misses = cmp::min(misses, N_FRUITS + N_DROPLETS);

        let mut best_state = CatchScoreState {
            max_combo: N_FRUITS + N_DROPLETS - misses,
            misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;

        let (new_fruits, new_droplets) = match (n_fruits, n_droplets) {
            (Some(mut n_fruits), Some(mut n_droplets)) => {
                let n_remaining =
                    (N_FRUITS + N_DROPLETS).saturating_sub(n_fruits + n_droplets + misses);

                let new_droplets = cmp::min(n_remaining, N_DROPLETS.saturating_sub(n_droplets));
                n_droplets += new_droplets;
                n_fruits += n_remaining - new_droplets;

                n_fruits = cmp::min(
                    n_fruits,
                    (N_FRUITS + N_DROPLETS).saturating_sub(n_droplets + misses),
                );
                n_droplets = cmp::min(n_droplets, N_FRUITS + N_DROPLETS - n_fruits - misses);

                (n_fruits, n_droplets)
            }
            (Some(mut n_fruits), None) => {
                let n_droplets = N_DROPLETS
                    .saturating_sub(misses.saturating_sub(N_FRUITS.saturating_sub(n_fruits)));
                n_fruits = N_FRUITS + N_DROPLETS - misses - n_droplets;

                (n_fruits, n_droplets)
            }
            (None, Some(mut n_droplets)) => {
                let n_fruits = N_FRUITS
                    .saturating_sub(misses.saturating_sub(N_DROPLETS.saturating_sub(n_droplets)));
                n_droplets = N_FRUITS + N_DROPLETS - misses - n_fruits;

                (n_fruits, n_droplets)
            }
            (None, None) => {
                let n_droplets = N_DROPLETS.saturating_sub(misses);
                let n_fruits = N_FRUITS - (misses - (N_DROPLETS.saturating_sub(n_droplets)));

                (n_fruits, n_droplets)
            }
        };

        best_state.fruits = new_fruits;
        best_state.droplets = new_droplets;

        let (min_tiny_droplets, max_tiny_droplets) = match (n_tiny_droplets, n_tiny_droplet_misses)
        {
            (Some(n_tiny_droplets), Some(n_tiny_droplet_misses)) => {
                match (n_tiny_droplets + n_tiny_droplet_misses).cmp(&N_TINY_DROPLETS) {
                    Ordering::Equal => (
                        cmp::min(N_TINY_DROPLETS, n_tiny_droplets),
                        cmp::min(N_TINY_DROPLETS, n_tiny_droplets),
                    ),
                    Ordering::Less | Ordering::Greater => (0, N_TINY_DROPLETS),
                }
            }
            (Some(n_tiny_droplets), None) => (
                cmp::min(N_TINY_DROPLETS, n_tiny_droplets),
                cmp::min(N_TINY_DROPLETS, n_tiny_droplets),
            ),
            (None, Some(n_tiny_droplet_misses)) => (
                N_TINY_DROPLETS.saturating_sub(n_tiny_droplet_misses),
                N_TINY_DROPLETS.saturating_sub(n_tiny_droplet_misses),
            ),
            (None, None) => (0, N_TINY_DROPLETS),
        };

        for new_tiny_droplets in min_tiny_droplets..=max_tiny_droplets {
            let new_tiny_droplet_misses = N_TINY_DROPLETS - new_tiny_droplets;

            let curr_acc = accuracy(
                new_fruits,
                new_droplets,
                new_tiny_droplets,
                new_tiny_droplet_misses,
                misses,
            );

            let curr_dist = (acc - curr_acc).abs();

            if curr_dist < best_dist {
                best_dist = curr_dist;
                best_state.tiny_droplets = new_tiny_droplets;
                best_state.tiny_droplet_misses = new_tiny_droplet_misses;
            }
        }

        best_state
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn hitresults(
            acc in 0.0..=1.0,
            n_fruits in prop::option::weighted(0.10, 0_u32..=N_FRUITS + 10),
            n_droplets in prop::option::weighted(0.10, 0_u32..=N_DROPLETS + 10),
            n_tiny_droplets in prop::option::weighted(0.10, 0_u32..=N_TINY_DROPLETS + 10),
            n_tiny_droplet_misses in prop::option::weighted(0.10, 0_u32..=N_TINY_DROPLETS + 10),
            n_misses in prop::option::weighted(0.15, 0_u32..=N_FRUITS + N_DROPLETS + 10),
        ) {
            let mut state = CatchPerformance::from(attrs())
                .accuracy(acc * 100.0);

            if let Some(n_fruits) = n_fruits {
                state = state.fruits(n_fruits);
            }

            if let Some(n_droplets) = n_droplets {
                state = state.droplets(n_droplets);
            }

            if let Some(n_tiny_droplets) = n_tiny_droplets {
                state = state.tiny_droplets(n_tiny_droplets);
            }

            if let Some(n_tiny_droplet_misses) = n_tiny_droplet_misses {
                state = state.tiny_droplet_misses(n_tiny_droplet_misses);
            }

            if let Some(misses) = n_misses {
                state = state.misses(misses);
            }

            let first = state.generate_state();
            let state = state.generate_state();
            assert_eq!(first, state);

            let expected = brute_force_best(
                acc,
                n_fruits,
                n_droplets,
                n_tiny_droplets,
                n_tiny_droplet_misses,
                n_misses.unwrap_or(0),
            );

            assert_eq!(state, expected);
        }
    }

    #[test]
    fn fruits_missing_objects() {
        let state = CatchPerformance::from(attrs())
            .fruits(N_FRUITS - 10)
            .droplets(N_DROPLETS - 1)
            .tiny_droplets(N_TINY_DROPLETS - 50)
            .tiny_droplet_misses(20)
            .misses(2)
            .generate_state();

        let expected = CatchScoreState {
            max_combo: N_FRUITS + N_DROPLETS - 2,
            fruits: N_FRUITS - 2,
            droplets: N_DROPLETS,
            tiny_droplets: N_TINY_DROPLETS - 20,
            tiny_droplet_misses: 20,
            misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn create() {
        let mut map = beatmap();
        let converted = map.unchecked_as_converted();

        let _ = CatchPerformance::new(CatchDifficultyAttributes::default());
        let _ = CatchPerformance::new(CatchPerformanceAttributes::default());
        let _ = CatchPerformance::new(&converted);
        let _ = CatchPerformance::new(converted.as_owned());

        let _ = CatchPerformance::try_new(CatchDifficultyAttributes::default()).unwrap();
        let _ = CatchPerformance::try_new(CatchPerformanceAttributes::default()).unwrap();
        let _ = CatchPerformance::try_new(DifficultyAttributes::Catch(
            CatchDifficultyAttributes::default(),
        ))
        .unwrap();
        let _ = CatchPerformance::try_new(PerformanceAttributes::Catch(
            CatchPerformanceAttributes::default(),
        ))
        .unwrap();
        let _ = CatchPerformance::try_new(&converted).unwrap();
        let _ = CatchPerformance::try_new(converted.as_owned()).unwrap();

        let _ = CatchPerformance::from(CatchDifficultyAttributes::default());
        let _ = CatchPerformance::from(CatchPerformanceAttributes::default());
        let _ = CatchPerformance::from(&converted);
        let _ = CatchPerformance::from(converted);

        let _ = CatchDifficultyAttributes::default().performance();
        let _ = CatchPerformanceAttributes::default().performance();

        map.mode = GameMode::Osu;
        let converted = map.unchecked_as_converted::<Osu>();

        assert!(CatchPerformance::try_new(OsuDifficultyAttributes::default()).is_none());
        assert!(CatchPerformance::try_new(OsuPerformanceAttributes::default()).is_none());
        assert!(CatchPerformance::try_new(DifficultyAttributes::Osu(
            OsuDifficultyAttributes::default()
        ))
        .is_none());
        assert!(CatchPerformance::try_new(PerformanceAttributes::Osu(
            OsuPerformanceAttributes::default()
        ))
        .is_none());
        assert!(CatchPerformance::try_new(&converted).is_none());
        assert!(CatchPerformance::try_new(converted).is_none());
    }
}
