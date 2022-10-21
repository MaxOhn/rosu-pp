use std::cmp::Ordering;

use super::{PathControlPoint, Pos2};

/// "Intermediate" hitobject created through parsing.
/// Each mode will handle them differently.
#[derive(Clone, Debug, PartialEq)]
pub struct HitObject {
    /// The position of the object.
    pub pos: Pos2,
    /// The start time of the object.
    pub start_time: f64,
    /// The type of the object.
    pub kind: HitObjectKind,
}

impl HitObject {
    /// The end time of the object.
    #[inline]
    pub fn end_time(&self) -> f64 {
        match &self.kind {
            HitObjectKind::Circle => self.start_time,
            // incorrect, only called in mania which has no sliders though
            HitObjectKind::Slider { .. } => self.start_time,
            HitObjectKind::Spinner { end_time } => *end_time,
            HitObjectKind::Hold { end_time, .. } => *end_time,
        }
    }

    /// If the object is a circle.
    #[inline]
    pub fn is_circle(&self) -> bool {
        matches!(self.kind, HitObjectKind::Circle)
    }

    /// If the object is a slider.
    #[inline]
    pub fn is_slider(&self) -> bool {
        matches!(self.kind, HitObjectKind::Slider { .. })
    }

    /// If the object is a spinner.
    #[inline]
    pub fn is_spinner(&self) -> bool {
        matches!(self.kind, HitObjectKind::Spinner { .. })
    }
}

impl PartialOrd for HitObject {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_time.partial_cmp(&other.start_time)
    }
}

/// Further data related to specific object types.
#[derive(Clone, Debug, PartialEq)]
pub enum HitObjectKind {
    /// A circle object.
    Circle,
    /// A full slider object.
    Slider {
        /// Total length of the slider in pixels.
        pixel_len: Option<f64>,
        /// The amount of repeat points of the slider.
        repeats: usize,
        /// The control points of the slider.
        control_points: Vec<PathControlPoint>,
        /// Sample sounds for the slider head, end, and repeat points.
        /// Required for converts.
        edge_sounds: Vec<u8>,
    },
    /// A spinner object.
    Spinner {
        /// The end time of the spinner.
        end_time: f64,
    },
    /// A hold note object for osu!mania.
    Hold {
        /// The end time of the hold object.
        end_time: f64,
    },
}
