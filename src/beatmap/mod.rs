use std::{borrow::Cow, cmp::Ordering, collections::HashMap};

use crate::{parse::HitObject, util::SortedVec};

pub use self::{
    attributes::{BeatmapAttributes, BeatmapAttributesBuilder, BeatmapHitWindows},
    breaks::Break,
    control_points::{DifficultyPoint, EffectPoint, TimingPoint},
    ext::*,
    mode::GameMode,
};

mod attributes;
mod breaks;
mod control_points;
mod converts;
mod ext;
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
    pub timing_points: SortedVec<TimingPoint>,

    /// Timing point for the current timing section.
    pub difficulty_points: SortedVec<DifficultyPoint>,

    /// Control points for effect sections.
    pub effect_points: SortedVec<EffectPoint>,

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
        // This is incorrect if the last object is a slider since there
        // is no reasonable way to get the slider end time at this point.
        let last_time = self
            .hit_objects
            .last()
            .map(HitObject::end_time)
            .or_else(|| self.timing_points.last().map(|t| t.time))
            .unwrap_or(0.0);

        /// Maps beat_len to a cumulative duration
        #[derive(Debug)]
        struct BeatLenDuration {
            last_time: f64,
            map: HashMap<u64, f64>,
        }

        impl BeatLenDuration {
            fn new(last_time: f64) -> Self {
                Self {
                    last_time,
                    map: HashMap::default(),
                }
            }

            fn add(&mut self, beat_len: f64, curr_time: f64, next_time: f64) {
                let beat_len = (1000.0 * beat_len).round() / 1000.0;
                let entry = self.map.entry(beat_len.to_bits()).or_default();

                if curr_time <= self.last_time {
                    *entry += next_time - curr_time;
                }
            }
        }

        let mut bpm_points = BeatLenDuration::new(last_time);

        // * osu-stable forced the first control point to start at 0.
        // * This is reproduced here to maintain compatibility around
        // * osu!mania scroll speed and song select display.
        match &self.timing_points[..] {
            [curr] => bpm_points.add(curr.beat_len, 0.0, last_time),
            [curr, next, ..] => bpm_points.add(curr.beat_len, 0.0, next.time),
            [] => {}
        }

        self.timing_points
            .iter()
            .skip(1)
            .zip(self.timing_points.iter().skip(2).map(|t| t.time))
            .for_each(|(curr, next_time)| bpm_points.add(curr.beat_len, curr.time, next_time));

        if let [.., _, curr] = &self.timing_points[..] {
            bpm_points.add(curr.beat_len, curr.time, last_time);
        }

        let most_common_beat_len = bpm_points
            .map
            .into_iter()
            // * Get the most common one, or 0 as a suitable default
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map_or(0.0, |(beat_len, _)| f64::from_bits(beat_len));

        60_000.0 / most_common_beat_len
    }

    /// Sum up the duration of all breaks (in milliseconds).
    #[inline]
    pub fn total_break_time(&self) -> f64 {
        self.breaks.iter().map(Break::duration).sum()
    }

    /// Return the [`TimingPoint`] for the given timestamp.
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

    /// Return the [`EffectPoint`] for the given timestamp.
    ///
    /// If `time` is before the first effect point, `None` is returned.
    #[inline]
    pub fn effect_point_at(&self, time: f64) -> Option<EffectPoint> {
        self.effect_points
            .binary_search_by(|probe| probe.time.partial_cmp(&time).unwrap_or(Ordering::Less))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map(|i| self.effect_points[i])
    }

    /// Convert a [`Beatmap`] of some mode into a different mode.
    ///
    /// # Note
    /// - Since hitsounds are irrelevant for difficulty and performance calculations
    /// in osu!mania, the resulting map of a conversion to mania will not contain hitsounds.
    /// - To avoid having to clone the map for osu!catch conversions, the field `Beatmap::mode`
    /// will not be adjusted in a osu!catch-converted map.
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
            effect_points: self.effect_points.clone(),
            stack_leniency: self.stack_leniency,
            breaks: self.breaks.clone(),
        }
    }
}
