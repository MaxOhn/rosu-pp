use std::mem;

use crate::catch::performance::CatchPerformance;

/// The result of a difficulty calculation on an osu!catch map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CatchDifficultyAttributes {
    /// The final star rating
    pub stars: f64,
    /// Time preempt (AR time window).
    pub preempt: f64,
    /// The amount of fruits.
    pub n_fruits: u32,
    /// The amount of droplets.
    pub n_droplets: u32,
    /// The amount of tiny droplets.
    pub n_tiny_droplets: u32,
    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub is_convert: bool,
}

impl CatchDifficultyAttributes {
    /// Return the maximum combo.
    pub const fn max_combo(&self) -> u32 {
        self.n_fruits + self.n_droplets
    }

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.is_convert
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> CatchPerformance<'a> {
        self.into()
    }

    pub(crate) const fn set_object_count(&mut self, count: &ObjectCount) {
        self.n_fruits = count.fruits;
        self.n_droplets = count.droplets;
        self.n_tiny_droplets = count.tiny_droplets;
    }

    pub(crate) const fn add_object_count(&mut self, count: GradualObjectCount) {
        if count.fruit {
            self.n_fruits += 1;
        } else {
            self.n_droplets += 1;
        }

        self.n_tiny_droplets += count.tiny_droplets;
    }
}

/// The result of a performance calculation on an osu!catch map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CatchPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: CatchDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
}

impl CatchPerformanceAttributes {
    /// Return the star value.
    pub const fn stars(&self) -> f64 {
        self.difficulty.stars
    }

    /// Return the performance point value.
    pub const fn pp(&self) -> f64 {
        self.pp
    }

    /// Return the maximum combo of the map.
    pub const fn max_combo(&self) -> u32 {
        self.difficulty.max_combo()
    }

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.difficulty.is_convert
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> CatchPerformance<'a> {
        self.difficulty.into()
    }
}

impl From<CatchPerformanceAttributes> for CatchDifficultyAttributes {
    fn from(attributes: CatchPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}

#[derive(Clone, Default)]
pub struct ObjectCount {
    fruits: u32,
    droplets: u32,
    tiny_droplets: u32,
}

#[derive(Copy, Clone, Default)]
pub struct GradualObjectCount {
    fruit: bool,
    tiny_droplets: u32,
}

pub enum ObjectCountBuilder {
    Regular {
        count: ObjectCount,
        take: usize,
    },
    Gradual {
        count: GradualObjectCount,
        all: Vec<GradualObjectCount>,
    },
}

impl ObjectCountBuilder {
    pub fn new_regular(take: usize) -> Self {
        Self::Regular {
            count: ObjectCount::default(),
            take,
        }
    }

    pub fn new_gradual() -> Self {
        Self::Gradual {
            count: GradualObjectCount::default(),
            all: Vec::with_capacity(512),
        }
    }

    pub fn into_regular(self) -> ObjectCount {
        if let Self::Regular { count, .. } = self {
            count
        } else {
            unreachable!()
        }
    }

    pub fn into_gradual(self) -> Vec<GradualObjectCount> {
        if let Self::Gradual { all, .. } = self {
            all
        } else {
            unreachable!()
        }
    }

    pub fn record_fruit(&mut self) {
        match self {
            Self::Regular { count, take } => {
                if *take > 0 {
                    *take -= 1;
                    count.fruits += 1;
                }
            }
            Self::Gradual { count, all } => {
                count.fruit = true;
                all.push(mem::take(count));
            }
        }
    }

    pub fn record_droplet(&mut self) {
        match self {
            Self::Regular { count, take } => {
                if *take > 0 {
                    *take -= 1;
                    count.droplets += 1;
                }
            }
            Self::Gradual { count, all } => all.push(mem::take(count)),
        }
    }

    pub const fn record_tiny_droplets(&mut self, n: u32) {
        match self {
            Self::Regular { count, take } => {
                if *take > 0 {
                    count.tiny_droplets += n;
                }
            }
            Self::Gradual { count, .. } => count.tiny_droplets += n,
        }
    }
}
