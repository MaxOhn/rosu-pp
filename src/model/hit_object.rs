use std::cmp::Ordering;

use rosu_map::section::{
    general::GameMode,
    hit_objects::{BorrowedCurve, CurveBuffers},
};

pub use rosu_map::{
    section::hit_objects::{PathControlPoint, PathType, SplineType, hit_samples::HitSoundType},
    util::Pos,
};

/// All hitobject related data required for difficulty and performance
/// calculation except for the [`HitSoundType`].
#[derive(Clone, Debug, PartialEq)]
pub struct HitObject {
    pub pos: Pos,
    pub start_time: f64,
    pub kind: HitObjectKind,
}

impl HitObject {
    /// Whether the hitobject is a circle.
    pub const fn is_circle(&self) -> bool {
        matches!(&self.kind, HitObjectKind::Circle)
    }

    /// Whether the hitobject is a slider.
    pub const fn is_slider(&self) -> bool {
        matches!(&self.kind, HitObjectKind::Slider(_))
    }

    /// Whether the hitobject is a spinner.
    pub const fn is_spinner(&self) -> bool {
        matches!(&self.kind, HitObjectKind::Spinner(_))
    }

    /// Whether the hitobject is a hold note.
    pub const fn is_hold_note(&self) -> bool {
        matches!(&self.kind, HitObjectKind::Hold(_))
    }

    /// The end time of the object.
    ///
    /// Note that this will not return the correct value for sliders.
    pub(crate) fn end_time(&self) -> f64 {
        match &self.kind {
            HitObjectKind::Circle | HitObjectKind::Slider { .. } => self.start_time,
            HitObjectKind::Spinner(Spinner { duration })
            | HitObjectKind::Hold(HoldNote { duration }) => self.start_time + *duration,
        }
    }
}

impl PartialOrd for HitObject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_time.partial_cmp(&other.start_time)
    }
}

/// Additional data for a [`HitObject`].
///
/// Note that each mode handles hit objects differently.
#[derive(Clone, Debug, PartialEq)]
pub enum HitObjectKind {
    Circle,
    Slider(Slider),
    Spinner(Spinner),
    Hold(HoldNote),
}

/// A slider.
#[derive(Clone, Debug, PartialEq)]
pub struct Slider {
    pub expected_dist: Option<f64>,
    pub repeats: usize,
    pub control_points: Box<[PathControlPoint]>,
    pub node_sounds: Box<[HitSoundType]>,
}

impl Slider {
    /// The amount of spans of the slider.
    pub const fn span_count(&self) -> usize {
        self.repeats + 1
    }

    pub(crate) fn curve<'a>(
        &self,
        mode: GameMode,
        bufs: &'a mut CurveBuffers,
    ) -> BorrowedCurve<'a> {
        BorrowedCurve::new(mode, &self.control_points, self.expected_dist, bufs)
    }
}

/// A spinner.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spinner {
    pub duration: f64,
}

/// A hold note.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HoldNote {
    pub duration: f64,
}
