use crate::{Beatmap, Mods};

pub struct PpResult {
    pub pp: f32,
    pub stars: f32,
}

pub trait PpProvider {
    fn pp(&self) -> PpCalculator;
}

impl PpProvider for Beatmap {
    #[inline]
    fn pp(&self) -> PpCalculator {
        PpCalculator::new(self)
    }
}

pub struct PpCalculator<'m> {
    map: &'m Beatmap,
    stars: Option<f32>,
    mods: u32,
    max_combo: usize,
    combo: Option<usize>,
    acc: f32,
    n_misses: usize,
    passed_objects: Option<usize>,
}

impl<'m> PpCalculator<'m> {
    #[inline]
    pub fn new(map: &'m Beatmap) -> Self {
        let max_combo = map.hit_objects.iter().filter(|h| h.is_circle()).count();

        Self {
            map,
            stars: None,
            mods: 0,
            max_combo,
            combo: None,
            acc: 1.0,
            n_misses: 0,
            passed_objects: None,
        }
    }

    #[inline]
    pub fn stars(mut self, stars: f32) -> Self {
        self.stars.replace(stars);

        self
    }

    #[inline]
    pub fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    #[inline]
    pub fn combo(mut self, combo: usize) -> Self {
        self.combo.replace(combo);

        self
    }

    #[inline]
    pub fn misses(mut self, n_misses: usize) -> Self {
        self.n_misses = n_misses;

        self
    }

    /// Set the accuracy between 0.0 and 100.0;
    #[inline]
    pub fn accuracy(mut self, acc: f32) -> Self {
        self.acc = acc / 100.0;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    #[inline]
    pub fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects.replace(passed_objects);

        self
    }

    pub fn calculate(self) -> PpResult {
        let stars = self
            .stars
            .unwrap_or_else(|| super::stars(self.map, self.mods, self.passed_objects));

        let mut multiplier = 1.1;

        if self.mods.nf() {
            multiplier *= 0.9;
        }

        if self.mods.hd() {
            multiplier *= 1.1;
        }

        let strain_value = self.compute_strain_value(stars);
        let acc_value = self.compute_accuracy_value();

        let pp = (strain_value.powf(1.1) + acc_value.powf(1.1)).powf(1.0 / 1.1) * multiplier;

        PpResult { stars, pp }
    }

    fn compute_strain_value(&self, stars: f32) -> f32 {
        let exp_base = 5.0 * (stars / 0.0075).max(1.0) - 4.0;
        let mut strain = exp_base * exp_base / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 1.0 + 0.1 * (self.max_combo as f32 / 1500.0).min(1.0);
        strain *= len_bonus;

        // Penalize misses exponentially
        strain *= 0.985_f32.powi(self.n_misses as i32);

        // HD bonus
        if self.mods.hd() {
            strain *= 1.025;
        }

        // FL bonus
        if self.mods.fl() {
            strain *= 1.05 * len_bonus;
        }

        // Scale with accuracy
        strain * self.acc
    }

    #[inline]
    fn compute_accuracy_value(&self) -> f32 {
        let mut od = self.map.od;

        if self.mods.hr() {
            od *= 1.4;
        } else if self.mods.ez() {
            od *= 0.5;
        }

        let hit_window = difficulty_range_od(od) / self.mods.speed();

        (150.0 / hit_window).powf(1.1)
            * self.acc.powi(15)
            * 22.0
            * (self.max_combo as f32 / 1500.0).powf(0.3).min(1.15)
    }
}

const HITWINDOW_MIN: f32 = 50.0;
const HITWINDOW_AVG: f32 = 35.0;
const HITWINDOW_MAX: f32 = 20.0;

#[inline]
pub(crate) fn difficulty_range_od(ar: f32) -> f32 {
    crate::difficulty_range(ar, HITWINDOW_MAX, HITWINDOW_AVG, HITWINDOW_MIN)
}
