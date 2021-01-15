use super::{stars, DifficultyAttributes};
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

// TODO: Allow partial plays
pub struct PpCalculator<'m> {
    map: &'m Beatmap,
    attributes: Option<DifficultyAttributes>,
    mods: u32,
    combo: Option<usize>,

    n_fruits: Option<usize>,
    n_droplets: Option<usize>,
    n_tiny_droplets: Option<usize>,
    n_tiny_droplet_misses: Option<usize>,
    n_misses: usize,
}

impl<'m> PpCalculator<'m> {
    #[inline]
    pub fn new(map: &'m Beatmap) -> Self {
        Self {
            map,
            attributes: None,
            mods: 0,
            combo: None,

            n_fruits: None,
            n_droplets: None,
            n_tiny_droplets: None,
            n_tiny_droplet_misses: None,
            n_misses: 0,
        }
    }

    #[inline]
    pub fn attributes(mut self, attributes: DifficultyAttributes) -> Self {
        self.attributes.replace(attributes);

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
    pub fn fruits(mut self, n_fruits: usize) -> Self {
        self.n_fruits.replace(n_fruits);

        self
    }

    #[inline]
    pub fn droplets(mut self, n_droplets: usize) -> Self {
        self.n_droplets.replace(n_droplets);

        self
    }

    #[inline]
    pub fn tiny_droplets(mut self, n_tiny_droplets: usize) -> Self {
        self.n_tiny_droplets.replace(n_tiny_droplets);

        self
    }

    #[inline]
    pub fn tiny_droplet_misses(mut self, n_tiny_droplet_misses: usize) -> Self {
        self.n_tiny_droplet_misses.replace(n_tiny_droplet_misses);

        self
    }

    #[inline]
    pub fn misses(mut self, n_misses: usize) -> Self {
        self.n_misses = n_misses;

        self
    }

    /// Generate the hit results with respect to the given accuracy between `0` and `100`.
    ///
    /// Be sure to set `misses` beforehand! Also, if available, set `attributes` beforehand.
    pub fn accuracy(mut self, acc: f32) -> Self {
        if self.attributes.is_none() {
            self.attributes.replace(stars(self.map, self.mods));
        }

        let attributes = self.attributes.as_ref().unwrap();

        let n_droplets = self
            .n_droplets
            .unwrap_or_else(|| attributes.n_droplets.saturating_sub(self.n_misses));

        let n_fruits = self.n_fruits.unwrap_or_else(|| {
            attributes
                .max_combo
                .saturating_sub(self.n_misses.saturating_sub(n_droplets))
        });

        let max_tiny_droplets = 0; // TODO

        let n_tiny_droplets = self.n_tiny_droplets.unwrap_or_else(|| {
            ((acc * (attributes.max_combo + max_tiny_droplets) as f32).round() as usize)
                .saturating_sub(n_fruits)
                .saturating_sub(n_droplets)
        });

        let n_tiny_droplet_misses = max_tiny_droplets - n_tiny_droplets;

        self.n_fruits.replace(n_fruits);
        self.n_droplets.replace(n_droplets);
        self.n_tiny_droplets.replace(n_tiny_droplets);
        self.n_tiny_droplet_misses.replace(n_tiny_droplet_misses);

        self
    }

    pub fn calculate(mut self) -> PpResult {
        let attributes = self
            .attributes
            .take()
            .unwrap_or_else(|| stars(self.map, self.mods));

        let stars = attributes.stars;

        // Relying heavily on aim
        let mut pp = (5.0 * ((stars / 0.0049).max(1.0)) - 4.0).powi(2) / 100_000.0;

        let mut combo_hits = self.combo_hits();
        if combo_hits == 0 {
            combo_hits = attributes.max_combo;
        }

        // Longer maps are worth more
        let len_bonus = 0.95
            + 0.3 * (combo_hits as f32 / 2500.0).min(1.0)
            + (combo_hits > 2500) as u8 as f32 * (combo_hits as f32 / 2500.0).log10() * 0.475;
        pp *= len_bonus;

        // Penalize misses exponentially
        pp *= 0.97_f32.powi(self.n_misses as i32);

        // Combo scaling
        if let Some(combo) = self.combo.filter(|_| attributes.max_combo > 0) {
            pp *= (combo as f32 / attributes.max_combo as f32)
                .powf(0.8)
                .min(1.0);
        }

        // AR scaling
        let ar = self.map.ar;
        let mut ar_factor = 1.0;
        if ar > 9.0 {
            ar_factor += 0.1 * (ar - 9.0) + (ar > 10.0) as u8 as f32 * 0.1 * (ar - 10.0);
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
        pp *= self.acc().powf(5.5);

        // NF penalty
        if self.mods.nf() {
            pp *= 0.9;
        }

        PpResult { pp, stars }
    }

    #[inline]
    fn combo_hits(&self) -> usize {
        self.n_fruits.unwrap_or(0) + self.n_droplets.unwrap_or(0) + self.n_misses
    }

    #[inline]
    fn successful_hits(&self) -> usize {
        self.n_fruits.unwrap_or(0)
            + self.n_droplets.unwrap_or(0)
            + self.n_tiny_droplets.unwrap_or(0)
    }

    #[inline]
    fn total_hits(&self) -> usize {
        self.successful_hits() + self.n_tiny_droplet_misses.unwrap_or(0) + self.n_misses
    }

    #[inline]
    fn acc(&self) -> f32 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            0.0
        } else {
            (self.successful_hits() as f32 / total_hits as f32)
                .max(0.0)
                .min(1.0)
        }
    }
}
