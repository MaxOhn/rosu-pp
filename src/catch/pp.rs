use super::{CatchDifficultyAttributes, CatchPerformanceAttributes, CatchScoreState, CatchStars};
use crate::{
    util::{MapOrElse, MapRef},
    Beatmap, DifficultyAttributes, Mods, OsuPP, PerformanceAttributes,
};
use std::cmp::Ordering;

/// Performance calculator on osu!catch maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{CatchPP, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let pp_result = CatchPP::new(&map)
///     .mods(8 + 64) // HDDT
///     .combo(1234)
///     .accuracy(98.5)
///     .misses(1)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = CatchPP::new(&map)
///     .attributes(pp_result) // reusing previous results for performance
///     .mods(8 + 64) // has to be the same to reuse attributes
///     .accuracy(99.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp(), next_result.stars());
/// ```
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
#[must_use]
pub struct CatchPP<'map> {
    pub(crate) map_or_attrs: MapOrElse<MapRef<'map>, CatchDifficultyAttributes>,
    pub(crate) mods: u32,
    pub(crate) acc: Option<f64>,
    pub(crate) combo: Option<usize>,

    pub(crate) n_fruits: Option<usize>,
    pub(crate) n_droplets: Option<usize>,
    pub(crate) n_tiny_droplets: Option<usize>,
    pub(crate) n_tiny_droplet_misses: Option<usize>,
    pub(crate) n_misses: Option<usize>,
    pub(crate) passed_objects: Option<usize>,
    pub(crate) clock_rate: Option<f64>,
}

impl<'map> CatchPP<'map> {
    /// Create a new performance calculator for osu!catch maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map_or_attrs: MapOrElse::from(map),
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

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    #[inline]
    pub fn attributes(mut self, attributes: impl CatchAttributeProvider) -> Self {
        if let Some(attrs) = attributes.attributes() {
            self.map_or_attrs = MapOrElse::Else(attrs);
        }

        self
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub const fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    /// Specify the max combo of the play.
    #[inline]
    pub const fn combo(mut self, combo: usize) -> Self {
        self.combo = Some(combo);

        self
    }

    /// Specify the amount of fruits of a play i.e. n300.
    #[inline]
    pub const fn fruits(mut self, n_fruits: usize) -> Self {
        self.n_fruits = Some(n_fruits);

        self
    }

    /// Specify the amount of droplets of a play i.e. n100.
    #[inline]
    pub const fn droplets(mut self, n_droplets: usize) -> Self {
        self.n_droplets = Some(n_droplets);

        self
    }

    /// Specify the amount of tiny droplets of a play i.e. n50.
    #[inline]
    pub const fn tiny_droplets(mut self, n_tiny_droplets: usize) -> Self {
        self.n_tiny_droplets = Some(n_tiny_droplets);

        self
    }

    /// Specify the amount of tiny droplet misses of a play i.e. `n_katu`.
    #[inline]
    pub const fn tiny_droplet_misses(mut self, n_tiny_droplet_misses: usize) -> Self {
        self.n_tiny_droplet_misses = Some(n_tiny_droplet_misses);

        self
    }

    /// Specify the amount of fruit / droplet misses of the play.
    #[inline]
    pub const fn misses(mut self, n_misses: usize) -> Self {
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
    #[inline]
    pub const fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects = Some(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    #[inline]
    pub const fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Provide parameters through an [`CatchScoreState`].
    #[inline]
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
    #[inline]
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc / 100.0);

        self
    }

    /// Create the [`CatchScoreState`] that will be used for performance calculation.
    #[allow(clippy::too_many_lines)]
    pub fn generate_state(&mut self) -> CatchScoreState {
        let attrs = match self.map_or_attrs {
            MapOrElse::Map(ref map) => {
                let attrs = self.generate_attributes(map.as_ref());

                self.map_or_attrs.else_or_insert(attrs)
            }
            MapOrElse::Else(ref attrs) => attrs,
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
                * (attrs.n_fruits + attrs.n_droplets + attrs.n_tiny_droplets) as f64
                - (n_fruits + n_droplets) as f64;
            let min_tiny_droplets = attrs
                .n_tiny_droplets
                .min(raw_tiny_droplets.floor() as usize);
            let max_tiny_droplets = attrs.n_tiny_droplets.min(raw_tiny_droplets.ceil() as usize);

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
            MapOrElse::Map(ref map) => self.generate_attributes(map.as_ref()),
            MapOrElse::Else(attrs) => attrs,
        };

        let inner = CatchPPInner {
            attrs,
            mods: self.mods,
            state,
        };

        inner.calculate()
    }

    fn generate_attributes(&self, map: &Beatmap) -> CatchDifficultyAttributes {
        let mut calculator = CatchStars::new(map).mods(self.mods);

        if let Some(passed_objects) = self.passed_objects {
            calculator = calculator.passed_objects(passed_objects);
        }

        if let Some(clock_rate) = self.clock_rate {
            calculator = calculator.clock_rate(clock_rate);
        }

        calculator.calculate()
    }

    /// Try to create [`CatchPP`] through [`OsuPP`].
    ///
    /// Returns `None` if [`OsuPP`] already replaced its internal [`Beatmap`]
    /// with [`OsuDifficultyAttributes`], i.e. if [`OsuPP::attributes`]
    /// or [`OsuPP::generate_state`] was called.
    ///
    /// [`OsuDifficultyAttributes`]: crate::osu::OsuDifficultyAttributes
    #[inline]
    pub const fn try_from_osu(osu: OsuPP<'map>) -> Option<Self> {
        let OsuPP {
            map_or_attrs,
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

        let MapOrElse::Map(map) = map_or_attrs else {
            return None;
        };

        Some(Self {
            map_or_attrs: MapOrElse::Map(map),
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

    /// Try to create [`CatchPP`] through a [`CatchAttributeProvider`].
    ///
    /// If you already calculated the attributes for the current map-mod
    /// combination, the [`Beatmap`] is no longer necessary to calculate
    /// performance attributes so this method can be used instead of
    /// [`CatchPP::new`].
    ///
    /// Returns `None` only if the [`CatchAttributeProvider`] did not contain
    /// attributes for catch e.g. if it's [`DifficultyAttributes::Taiko`].
    #[inline]
    pub fn try_from_attributes(attributes: impl CatchAttributeProvider) -> Option<Self> {
        attributes.attributes().map(Self::from)
    }
}

struct CatchPPInner {
    attrs: CatchDifficultyAttributes,
    mods: u32,
    state: CatchScoreState,
}

impl CatchPPInner {
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
        let len_bonus = 0.95
            + 0.3 * (combo_hits as f64 / 2500.0).min(1.0)
            + f64::from(u8::from(combo_hits > 2500)) * (combo_hits as f64 / 2500.0).log10() * 0.475;

        pp *= len_bonus;

        // Penalize misses exponentially
        pp *= 0.97_f64.powi(self.state.n_misses as i32);

        // Combo scaling
        if self.state.max_combo > 0 {
            pp *= (self.state.max_combo as f64 / max_combo as f64)
                .powf(0.8)
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

    const fn combo_hits(&self) -> usize {
        self.state.n_fruits + self.state.n_droplets + self.state.n_misses
    }
}

fn accuracy(
    n_fruits: usize,
    n_droplets: usize,
    n_tiny_droplets: usize,
    n_tiny_droplet_misses: usize,
    n_misses: usize,
) -> f64 {
    let numerator = n_fruits + n_droplets + n_tiny_droplets;
    let denominator = numerator + n_tiny_droplet_misses + n_misses;

    numerator as f64 / denominator as f64
}

impl From<CatchDifficultyAttributes> for CatchPP<'_> {
    fn from(attrs: CatchDifficultyAttributes) -> Self {
        Self {
            map_or_attrs: MapOrElse::Else(attrs),
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

impl From<CatchPerformanceAttributes> for CatchPP<'_> {
    fn from(attrs: CatchPerformanceAttributes) -> Self {
        attrs.difficulty.into()
    }
}

/// Abstract type to provide flexibility when passing difficulty attributes to a performance calculation.
pub trait CatchAttributeProvider {
    /// Provide the actual difficulty attributes.
    fn attributes(self) -> Option<CatchDifficultyAttributes>;
}

impl CatchAttributeProvider for CatchDifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<CatchDifficultyAttributes> {
        Some(self)
    }
}

impl CatchAttributeProvider for CatchPerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<CatchDifficultyAttributes> {
        Some(self.difficulty)
    }
}

impl CatchAttributeProvider for DifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<CatchDifficultyAttributes> {
        if let Self::Catch(attributes) = self {
            Some(attributes)
        } else {
            None
        }
    }
}

impl CatchAttributeProvider for PerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<CatchDifficultyAttributes> {
        if let Self::Catch(attributes) = self {
            Some(attributes.difficulty)
        } else {
            None
        }
    }
}

#[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
#[cfg(test)]
mod test {
    use super::*;
    use crate::Beatmap;
    use proptest::{option, prelude::*};
    use std::sync::OnceLock;

    static DATA: OnceLock<(Beatmap, CatchDifficultyAttributes)> = OnceLock::new();

    const N_FRUITS: usize = 728;
    const N_DROPLETS: usize = 2;
    const N_TINY_DROPLETS: usize = 291;

    fn test_data() -> (&'static Beatmap, CatchDifficultyAttributes) {
        let (map, attrs) = DATA.get_or_init(|| {
            let path = "./maps/2118524.osu";
            let map = Beatmap::from_path(path).unwrap();
            let attrs = CatchStars::new(&map).calculate();

            assert_eq!(
                (N_FRUITS, N_DROPLETS, N_TINY_DROPLETS),
                (attrs.n_fruits, attrs.n_droplets, attrs.n_tiny_droplets)
            );

            (map, attrs)
        });

        (map, attrs.to_owned())
    }

    /// Checks all remaining hitresult combinations w.r.t. the given parameters
    /// and returns the [`OsuScoreState`] that matches `acc` the best.
    ///
    /// Very slow but accurate.
    fn brute_force_best(
        acc: f64,
        n_fruits: Option<usize>,
        n_droplets: Option<usize>,
        n_tiny_droplets: Option<usize>,
        n_tiny_droplet_misses: Option<usize>,
        n_misses: usize,
    ) -> CatchScoreState {
        let n_misses = n_misses.min(N_FRUITS + N_DROPLETS);

        let mut best_state = CatchScoreState {
            max_combo: N_FRUITS + N_DROPLETS - n_misses,
            n_misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;

        let (new_fruits, new_droplets) = match (n_fruits, n_droplets) {
            (Some(mut n_fruits), Some(mut n_droplets)) => {
                let n_remaining =
                    (N_FRUITS + N_DROPLETS).saturating_sub(n_fruits + n_droplets + n_misses);

                let new_droplets = n_remaining.min(N_DROPLETS.saturating_sub(n_droplets));
                n_droplets += new_droplets;
                n_fruits += n_remaining - new_droplets;

                n_fruits =
                    n_fruits.min((N_FRUITS + N_DROPLETS).saturating_sub(n_droplets + n_misses));
                n_droplets = n_droplets.min(N_FRUITS + N_DROPLETS - n_fruits - n_misses);

                (n_fruits, n_droplets)
            }
            (Some(mut n_fruits), None) => {
                let n_droplets = N_DROPLETS
                    .saturating_sub(n_misses.saturating_sub(N_FRUITS.saturating_sub(n_fruits)));
                n_fruits = N_FRUITS + N_DROPLETS - n_misses - n_droplets;

                (n_fruits, n_droplets)
            }
            (None, Some(mut n_droplets)) => {
                let n_fruits = N_FRUITS
                    .saturating_sub(n_misses.saturating_sub(N_DROPLETS.saturating_sub(n_droplets)));
                n_droplets = N_FRUITS + N_DROPLETS - n_misses - n_fruits;

                (n_fruits, n_droplets)
            }
            (None, None) => {
                let n_droplets = N_DROPLETS.saturating_sub(n_misses);
                let n_fruits = N_FRUITS - (n_misses - (N_DROPLETS.saturating_sub(n_droplets)));

                (n_fruits, n_droplets)
            }
        };

        best_state.n_fruits = new_fruits;
        best_state.n_droplets = new_droplets;

        let (min_tiny_droplets, max_tiny_droplets) = match (n_tiny_droplets, n_tiny_droplet_misses)
        {
            (Some(n_tiny_droplets), Some(n_tiny_droplet_misses)) => {
                match (n_tiny_droplets + n_tiny_droplet_misses).cmp(&N_TINY_DROPLETS) {
                    Ordering::Equal => (
                        N_TINY_DROPLETS.min(n_tiny_droplets),
                        N_TINY_DROPLETS.min(n_tiny_droplets),
                    ),
                    Ordering::Less | Ordering::Greater => (0, N_TINY_DROPLETS),
                }
            }
            (Some(n_tiny_droplets), None) => (
                N_TINY_DROPLETS.min(n_tiny_droplets),
                N_TINY_DROPLETS.min(n_tiny_droplets),
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
                n_misses,
            );

            let curr_dist = (acc - curr_acc).abs();

            if curr_dist < best_dist {
                best_dist = curr_dist;
                best_state.n_tiny_droplets = new_tiny_droplets;
                best_state.n_tiny_droplet_misses = new_tiny_droplet_misses;
            }
        }

        best_state
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20_000))]
        #[test]
        fn catch_hitresults(
            acc in 0.0..=1.0,
            n_fruits in option::weighted(0.10, 0_usize..=N_FRUITS + 10),
            n_droplets in option::weighted(0.10, 0_usize..=N_DROPLETS + 10),
            n_tiny_droplets in option::weighted(0.10, 0_usize..=N_TINY_DROPLETS + 10),
            n_tiny_droplet_misses in option::weighted(0.10, 0_usize..=N_TINY_DROPLETS + 10),
            n_misses in option::weighted(0.15, 0_usize..=N_FRUITS + N_DROPLETS + 10),
        ) {
            let (map, attrs) = test_data();

            let mut state = CatchPP::new(map)
                .attributes(attrs)
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

            if let Some(n_misses) = n_misses {
                state = state.misses(n_misses);
            }

            let state = state.generate_state();

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
        let (map, attrs) = test_data();

        let state = CatchPP::new(map)
            .attributes(attrs)
            .fruits(N_FRUITS - 10)
            .droplets(N_DROPLETS - 1)
            .tiny_droplets(N_TINY_DROPLETS - 50)
            .tiny_droplet_misses(20)
            .misses(2)
            .generate_state();

        let expected = CatchScoreState {
            max_combo: N_FRUITS + N_DROPLETS - 2,
            n_fruits: N_FRUITS - 2,
            n_droplets: N_DROPLETS,
            n_tiny_droplets: N_TINY_DROPLETS - 20,
            n_tiny_droplet_misses: 20,
            n_misses: 2,
        };

        assert_eq!(state, expected);
    }
}
