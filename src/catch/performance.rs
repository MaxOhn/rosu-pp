use std::cmp::{self, Ordering};

use crate::{
    any::ModeAttributeProvider,
    any::ModeDifficulty,
    osu::OsuPerformance,
    util::{map_or_attrs::MapOrAttrs, mods::Mods},
};

use super::{
    attributes::{CatchDifficultyAttributes, CatchPerformanceAttributes},
    convert::CatchBeatmap,
    score_state::CatchScoreState,
    Catch,
};

/// Performance calculator on osu!catch maps.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CatchPerformance<'map> {
    pub(crate) map_or_attrs: MapOrAttrs<'map, Catch>,
    pub(crate) mods: u32,
    pub(crate) acc: Option<f64>,
    pub(crate) combo: Option<u32>,

    pub(crate) n_fruits: Option<u32>,
    pub(crate) n_droplets: Option<u32>,
    pub(crate) n_tiny_droplets: Option<u32>,
    pub(crate) n_tiny_droplet_misses: Option<u32>,
    pub(crate) n_misses: Option<u32>,
    pub(crate) passed_objects: Option<u32>,
    pub(crate) clock_rate: Option<f64>,
}

impl<'map> CatchPerformance<'map> {
    /// Create a new performance calculator for osu!catch maps.
    pub fn new(map: CatchBeatmap<'map>) -> Self {
        map.into()
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    pub fn attributes(mut self, attributes: impl ModeAttributeProvider<Catch>) -> Self {
        if let Some(attrs) = attributes.attributes() {
            self.map_or_attrs = MapOrAttrs::Attrs(attrs);
        }

        self
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    pub const fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    /// Specify the max combo of the play.
    pub const fn combo(mut self, combo: u32) -> Self {
        self.combo = Some(combo);

        self
    }

    /// Specify the amount of fruits of a play i.e. n300.
    pub const fn fruits(mut self, n_fruits: u32) -> Self {
        self.n_fruits = Some(n_fruits);

        self
    }

    /// Specify the amount of droplets of a play i.e. n100.
    pub const fn droplets(mut self, n_droplets: u32) -> Self {
        self.n_droplets = Some(n_droplets);

        self
    }

    /// Specify the amount of tiny droplets of a play i.e. n50.
    pub const fn tiny_droplets(mut self, n_tiny_droplets: u32) -> Self {
        self.n_tiny_droplets = Some(n_tiny_droplets);

        self
    }

    /// Specify the amount of tiny droplet misses of a play i.e. `n_katu`.
    pub const fn tiny_droplet_misses(mut self, n_tiny_droplet_misses: u32) -> Self {
        self.n_tiny_droplet_misses = Some(n_tiny_droplet_misses);

        self
    }

    /// Specify the amount of fruit / droplet misses of the play.
    pub const fn misses(mut self, n_misses: u32) -> Self {
        self.n_misses = Some(n_misses);

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    #[cfg_attr(
        feature = "gradual",
        doc = "If you want to calculate the performance after every few objects, instead of
        using [`CatchPP`] multiple times with different `passed_objects`, you should use
        [`CatchGradualPerformanceAttributes`](crate::catch::CatchGradualPerformance)."
    )]
    pub const fn passed_objects(mut self, passed_objects: u32) -> Self {
        self.passed_objects = Some(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    pub const fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Provide parameters through an [`CatchScoreState`].
    #[allow(clippy::needless_pass_by_value)]
    pub const fn state(mut self, state: CatchScoreState) -> Self {
        let CatchScoreState {
            max_combo,
            n_fruits,
            n_droplets,
            n_tiny_droplets,
            n_tiny_droplet_misses,
            n_misses,
        } = state;

        self.combo = Some(max_combo);
        self.n_fruits = Some(n_fruits);
        self.n_droplets = Some(n_droplets);
        self.n_tiny_droplets = Some(n_tiny_droplets);
        self.n_tiny_droplet_misses = Some(n_tiny_droplet_misses);
        self.n_misses = Some(n_misses);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc / 100.0);

        self
    }

    /// Create the [`CatchScoreState`] that will be used for performance calculation.
    #[allow(clippy::too_many_lines)]
    pub fn generate_state(&mut self) -> CatchScoreState {
        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => {
                let attrs = self.generate_attributes(map);

                self.map_or_attrs.attrs_or_insert(attrs)
            }
            MapOrAttrs::Attrs(ref attrs) => attrs,
        };

        let n_misses = self
            .n_misses
            .map_or(0, |n| n.min(attrs.n_fruits + attrs.n_droplets));

        let max_combo = self.combo.unwrap_or_else(|| attrs.max_combo() - n_misses);

        let mut best_state = CatchScoreState {
            max_combo,
            n_misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;

        let (n_fruits, n_droplets) = match (self.n_fruits, self.n_droplets) {
            (Some(mut n_fruits), Some(mut n_droplets)) => {
                let n_remaining = (attrs.n_fruits + attrs.n_droplets)
                    .saturating_sub(n_fruits + n_droplets + n_misses);

                let new_droplets = n_remaining.min(attrs.n_droplets.saturating_sub(n_droplets));
                n_droplets += new_droplets;
                n_fruits += n_remaining - new_droplets;

                n_fruits = n_fruits
                    .min((attrs.n_fruits + attrs.n_droplets).saturating_sub(n_droplets + n_misses));
                n_droplets =
                    n_droplets.min(attrs.n_fruits + attrs.n_droplets - n_fruits - n_misses);

                (n_fruits, n_droplets)
            }
            (Some(mut n_fruits), None) => {
                let n_droplets = attrs.n_droplets.saturating_sub(
                    n_misses.saturating_sub(attrs.n_fruits.saturating_sub(n_fruits)),
                );

                n_fruits = attrs.n_fruits + attrs.n_droplets - n_misses - n_droplets;

                (n_fruits, n_droplets)
            }
            (None, Some(mut n_droplets)) => {
                let n_fruits = attrs.n_fruits.saturating_sub(
                    n_misses.saturating_sub(attrs.n_droplets.saturating_sub(n_droplets)),
                );

                n_droplets = attrs.n_fruits + attrs.n_droplets - n_misses - n_fruits;

                (n_fruits, n_droplets)
            }
            (None, None) => {
                let n_droplets = attrs.n_droplets.saturating_sub(n_misses);
                let n_fruits =
                    attrs.n_fruits - (n_misses - (attrs.n_droplets.saturating_sub(n_droplets)));

                (n_fruits, n_droplets)
            }
        };

        best_state.n_fruits = n_fruits;
        best_state.n_droplets = n_droplets;

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
                    n_misses,
                );
                let curr_dist = (acc - curr_acc).abs();

                if curr_dist < best_dist {
                    best_dist = curr_dist;
                    best_state.n_tiny_droplets = n_tiny_droplets;
                    best_state.n_tiny_droplet_misses = n_tiny_droplet_misses;
                }
            }
        };

        #[allow(clippy::single_match_else)]
        match (self.n_tiny_droplets, self.n_tiny_droplet_misses) {
            (Some(n_tiny_droplets), Some(n_tiny_droplet_misses)) => match self.acc {
                Some(acc) => {
                    match (n_tiny_droplets + n_tiny_droplet_misses).cmp(&attrs.n_tiny_droplets) {
                        Ordering::Equal => {
                            best_state.n_tiny_droplets = n_tiny_droplets;
                            best_state.n_tiny_droplet_misses = n_tiny_droplet_misses;
                        }
                        Ordering::Less | Ordering::Greater => find_best_tiny_droplets(acc),
                    }
                }
                None => {
                    let n_remaining = attrs
                        .n_tiny_droplets
                        .saturating_sub(n_tiny_droplets + n_tiny_droplet_misses);

                    best_state.n_tiny_droplets = n_tiny_droplets + n_remaining;
                    best_state.n_tiny_droplet_misses = n_tiny_droplet_misses;
                }
            },
            (Some(n_tiny_droplets), None) => {
                best_state.n_tiny_droplets = attrs.n_tiny_droplets.min(n_tiny_droplets);
                best_state.n_tiny_droplet_misses =
                    attrs.n_tiny_droplets.saturating_sub(n_tiny_droplets);
            }
            (None, Some(n_tiny_droplet_misses)) => {
                best_state.n_tiny_droplets =
                    attrs.n_tiny_droplets.saturating_sub(n_tiny_droplet_misses);
                best_state.n_tiny_droplet_misses = attrs.n_tiny_droplets.min(n_tiny_droplet_misses);
            }
            (None, None) => match self.acc {
                Some(acc) => find_best_tiny_droplets(acc),
                None => best_state.n_tiny_droplets = attrs.n_tiny_droplets,
            },
        }

        best_state
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> CatchPerformanceAttributes {
        let state = self.generate_state();

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => self.generate_attributes(map),
            MapOrAttrs::Attrs(attrs) => attrs,
        };

        let inner = CatchPerformanceInner {
            attrs,
            mods: self.mods,
            state,
        };

        inner.calculate()
    }

    fn generate_attributes(&self, map: &CatchBeatmap<'_>) -> CatchDifficultyAttributes {
        let mut calculator = ModeDifficulty::new();

        if let Some(passed_objects) = self.passed_objects {
            calculator = calculator.passed_objects(passed_objects);
        }

        if let Some(clock_rate) = self.clock_rate {
            calculator = calculator.clock_rate(clock_rate);
        }

        calculator.mods(self.mods).calculate(map)
    }

    /// Try to create [`CatchPerformance`] through a [`ModeAttributeProvider`].
    ///
    /// If you already calculated the attributes for the current map-mod
    /// combination, the [`CatchBeatmap`] is no longer necessary to calculate
    /// performance attributes so this method can be used instead of
    /// [`CatchPerformance::new`].
    ///
    /// Returns `None` only if the [`ModeAttributeProvider`] did not contain
    /// attributes for catch e.g. if it's [`DifficultyAttributes::Taiko`].
    ///
    /// [`DifficultyAttributes::Taiko`]: crate::any::DifficultyAttributes::Taiko
    pub fn try_from_attributes(attributes: impl ModeAttributeProvider<Catch>) -> Option<Self> {
        attributes.attributes().map(Self::from)
    }

    /// Create [`CatchPerformance`] through a [`ModeAttributeProvider`].
    ///
    /// If you already calculated the attributes for the current map-mod
    /// combination, the [`CatchBeatmap`] is no longer necessary to calculate
    /// performance attributes so this method can be used instead of
    /// [`CatchPerformance::new`].
    ///
    /// # Panics
    ///
    /// Panics if the [`ModeAttributeProvider`] did not contain attributes for
    /// catch e.g. if it's [`DifficultyAttributes::Taiko`].
    ///
    /// [`DifficultyAttributes::Taiko`]: crate::any::DifficultyAttributes::Taiko
    pub fn unchecked_from_attributes(attributes: impl ModeAttributeProvider<Catch>) -> Self {
        Self::try_from_attributes(attributes).expect("invalid catch attributes")
    }
}

impl<'map> TryFrom<OsuPerformance<'map>> for CatchPerformance<'map> {
    type Error = OsuPerformance<'map>;

    /// Try to create [`CatchPerformance`] through [`OsuPerformance`].
    ///
    /// Returns `None` if [`OsuPerformance`] already replaced its internal
    /// beatmap with [`OsuDifficultyAttributes`], i.e. if
    /// [`OsuPerformance::attributes`] or [`OsuPerformance::generate_state`]
    /// was called.
    ///
    /// [`OsuDifficultyAttributes`]: crate::osu::OsuDifficultyAttributes
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
            mods,
            acc,
            combo,
            n300,
            n100,
            n50,
            n_misses,
            passed_objects,
            clock_rate,
            hitresult_priority: _,
        } = osu;

        Ok(Self {
            map_or_attrs: MapOrAttrs::Map(map),
            mods,
            acc,
            combo,
            n_fruits: n300,
            n_droplets: n100,
            n_tiny_droplets: n50,
            n_tiny_droplet_misses: None,
            n_misses,
            passed_objects,
            clock_rate,
        })
    }
}

impl<'map> From<CatchBeatmap<'map>> for CatchPerformance<'map> {
    fn from(map: CatchBeatmap<'map>) -> Self {
        Self {
            map_or_attrs: MapOrAttrs::Map(map),
            mods: 0,
            acc: None,
            combo: None,

            n_fruits: None,
            n_droplets: None,
            n_tiny_droplets: None,
            n_tiny_droplet_misses: None,
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
        }
    }
}

impl From<CatchDifficultyAttributes> for CatchPerformance<'_> {
    fn from(attrs: CatchDifficultyAttributes) -> Self {
        Self {
            map_or_attrs: MapOrAttrs::Attrs(attrs),
            mods: 0,
            acc: None,
            combo: None,

            n_fruits: None,
            n_droplets: None,
            n_tiny_droplets: None,
            n_tiny_droplet_misses: None,
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
        }
    }
}

impl From<CatchPerformanceAttributes> for CatchPerformance<'_> {
    fn from(attrs: CatchPerformanceAttributes) -> Self {
        attrs.difficulty.into()
    }
}

struct CatchPerformanceInner {
    attrs: CatchDifficultyAttributes,
    mods: u32,
    state: CatchScoreState,
}

impl CatchPerformanceInner {
    fn calculate(self) -> CatchPerformanceAttributes {
        let attributes = &self.attrs;
        let stars = attributes.stars;
        let max_combo = attributes.max_combo();

        // Relying heavily on aim
        let mut pp = (5.0 * (stars / 0.0049).max(1.0) - 4.0).powi(2) / 100_000.0;

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
        pp *= 0.97_f64.powi(self.state.n_misses as i32);

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
            pp *= 0.9;
        }

        CatchPerformanceAttributes {
            difficulty: self.attrs,
            pp,
        }
    }

    const fn combo_hits(&self) -> u32 {
        self.state.n_fruits + self.state.n_droplets + self.state.n_misses
    }
}

fn accuracy(
    n_fruits: u32,
    n_droplets: u32,
    n_tiny_droplets: u32,
    n_tiny_droplet_misses: u32,
    n_misses: u32,
) -> f64 {
    let numerator = n_fruits + n_droplets + n_tiny_droplets;
    let denominator = numerator + n_tiny_droplet_misses + n_misses;

    f64::from(numerator) / f64::from(denominator)
}
