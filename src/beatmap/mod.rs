use crate::parse::HitObject;

pub use self::{
    attributes::BeatmapAttributes,
    control_points::{ControlPoint, ControlPointIter, DifficultyPoint, TimingPoint},
    mode::GameMode,
};

mod attributes;
mod control_points;
mod mode;

/// The main beatmap struct containing all data relevant
/// for difficulty and performance calculation
#[derive(Clone, Default, Debug)]
pub struct Beatmap {
    /// The game mode.
    pub mode: GameMode,
    /// The version of the .osu file.
    pub version: u8,

    /// The amount of circles.
    pub n_circles: u32,
    /// The amount of sliders.
    pub n_sliders: u32,
    /// The amount of spinners.
    pub n_spinners: u32,

    /// The approach rate.
    pub ar: f32,
    /// The overall difficulty.
    pub od: f32,
    /// The circle size.
    pub cs: f32,
    /// The health drain rate.
    pub hp: f32,
    /// Base slider velocity in pixels per beat
    pub slider_mult: f64,
    /// Amount of slider ticks per beat.
    pub tick_rate: f64,
    /// All hitobjects of the beatmap.
    pub hit_objects: Vec<HitObject>,
    /// Store the sounds for all objects in their own Vec to minimize the struct size.
    /// Hitsounds are only used in osu!taiko in which they represent color.
    pub sounds: Vec<u8>,

    /// Timing points that indicate a new timing section.
    pub timing_points: Vec<TimingPoint>,

    /// Timing point for the current timing section.
    pub difficulty_points: Vec<DifficultyPoint>,

    /// The stack leniency that is used to calculate
    /// the stack offset for stacked positions.
    pub stack_leniency: f32,
}

impl Beatmap {
    /// Extract a beatmap's attributes into their own type.
    #[inline]
    pub fn attributes(&self) -> BeatmapAttributes {
        BeatmapAttributes::new(self.ar, self.od, self.cs, self.hp)
    }

    /// The beats per minute of the map.
    #[inline]
    pub fn bpm(&self) -> f64 {
        match self.timing_points.first() {
            Some(point) => point.beat_len.recip() * 1000.0 * 60.0,
            None => 0.0,
        }
    }

    /// Create an iterator over the map's timing- and difficulty points sorted by timestamp.
    #[inline]
    pub fn control_points(&self) -> ControlPointIter<'_> {
        ControlPointIter::new(self)
    }
}
