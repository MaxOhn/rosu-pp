use std::{borrow::Cow, cmp::Ordering};

use rosu_map::section::{
    general::GameMode,
    hit_objects::{BorrowedCurve, Curve, CurveBuffers},
};

pub use rosu_map::{
    section::hit_objects::{PathControlPoint, PathType, SplineType, hit_samples::HitSoundType},
    util::Pos,
};

use crate::model::mods::Reflection;

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

    /// Creates the [`Curve`] of a [`Slider`].
    ///
    /// Applies the [`Reflection`] onto control points before creating the
    /// curve.
    pub(crate) fn curve(
        &self,
        mode: GameMode,
        reflection: Reflection,
        bufs: &mut CurveBuffers,
    ) -> Curve {
        fn reflect<F: Fn(Pos) -> Pos>(points: &mut Cow<'_, [PathControlPoint]>, f: F) {
            points
                .to_mut()
                .iter_mut()
                .for_each(|point| point.pos = f(point.pos));
        }

        let mut points = Cow::Borrowed(self.control_points.as_ref());

        match reflection {
            Reflection::None => {}
            Reflection::Vertical => reflect(&mut points, |pos| Pos::new(pos.x, -pos.y)),
            Reflection::Horizontal => reflect(&mut points, |pos| Pos::new(-pos.x, pos.y)),
            Reflection::Both => reflect(&mut points, |pos| Pos::new(-pos.x, -pos.y)),
        }

        Curve::new(mode, points.as_ref(), self.expected_dist, bufs)
    }

    /// Creates the [`BorrowedCurve`] of a [`Slider`].
    pub(crate) fn borrowed_curve<'bufs>(
        &self,
        mode: GameMode,
        bufs: &'bufs mut CurveBuffers,
    ) -> BorrowedCurve<'bufs> {
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
