use std::{borrow::Cow, cmp::Ordering};

use crate::parse::HitObject;

pub use self::{
    attributes::{BeatmapAttributes, BeatmapAttributesBuilder, BeatmapHitWindows},
    breaks::Break,
    control_points::{ControlPoint, ControlPointIter, DifficultyPoint, TimingPoint},
    mode::GameMode,
    sorted_vec::SortedVec,
};

mod attributes;
mod breaks;
mod control_points;
mod converts;
mod mode;
mod sorted_vec;

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
    pub timing_points: SortedVec<TimingPoint>,

    /// Timing point for the current timing section.
    pub difficulty_points: SortedVec<DifficultyPoint>,

    /// The stack leniency that is used to calculate
    /// the stack offset for stacked positions.
    pub stack_leniency: f32,

    /// All break points of the beatmap.
    pub breaks: Vec<Break>,
}

impl Beatmap {
    /// Extract a beatmap's attributes into their own type.
    #[inline]
    pub fn attributes(&self) -> BeatmapAttributesBuilder {
        BeatmapAttributesBuilder::new(self)
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

    /// Sum up the duration of all breaks (in milliseconds).
    #[inline]
    pub fn total_break_time(&self) -> f64 {
        self.breaks.iter().map(Break::duration).sum()
    }

    /// Return the [`TimingPoint`] for the given timestamp.
    ///
    /// If `time` is before the first timing point, `None` is returned.
    #[inline]
    pub fn timing_point_at(&self, time: f64) -> TimingPoint {
        let idx_result = self
            .timing_points
            .binary_search_by(|probe| probe.time.partial_cmp(&time).unwrap_or(Ordering::Less));

        match idx_result {
            Ok(idx) => self.timing_points[idx],
            Err(0) => self.timing_points.first().copied().unwrap_or_default(),
            Err(idx) => self.timing_points[idx - 1],
        }
    }

    /// Return the [`DifficultyPoint`] for the given timestamp.
    ///
    /// If `time` is before the first difficulty point, `None` is returned.
    #[inline]
    pub fn difficulty_point_at(&self, time: f64) -> Option<DifficultyPoint> {
        self.difficulty_points
            .binary_search_by(|probe| probe.time.partial_cmp(&time).unwrap_or(Ordering::Less))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map(|i| self.difficulty_points[i])
    }

    /// Convert a [`Beatmap`] of some mode into a different mode.
    ///
    /// # Note
    /// - Since hitsounds are irrelevant for difficulty and performance calculations
    /// in osu!mania, the resulting map of a conversion to mania will not contain hitsounds.
    /// - To avoid having to clone the map for osu!catch conversions, the field `Beatmap::mode`
    /// will not be adjusted in a ctb-converted map.
    #[inline]
    pub fn convert_mode(&self, mode: GameMode) -> Cow<'_, Self> {
        if mode == self.mode {
            return Cow::Borrowed(self);
        }

        match mode {
            GameMode::Osu | GameMode::Catch => Cow::Borrowed(self),
            GameMode::Taiko => Cow::Owned(self.convert_to_taiko()),
            GameMode::Mania => Cow::Owned(self.convert_to_mania()),
        }
    }

    fn clone_without_hit_objects(&self, with_sounds: bool) -> Self {
        Self {
            mode: self.mode,
            version: self.version,
            n_circles: 0,
            n_sliders: 0,
            n_spinners: 0,
            ar: self.ar,
            od: self.od,
            cs: self.cs,
            hp: self.hp,
            slider_mult: self.slider_mult,
            tick_rate: self.tick_rate,
            hit_objects: Vec::with_capacity(self.hit_objects.len()),
            sounds: Vec::with_capacity((with_sounds as usize) * self.sounds.len()),
            timing_points: self.timing_points.clone(),
            difficulty_points: self.difficulty_points.clone(),
            stack_leniency: self.stack_leniency,
            breaks: self.breaks.clone(),
        }
    }
}
