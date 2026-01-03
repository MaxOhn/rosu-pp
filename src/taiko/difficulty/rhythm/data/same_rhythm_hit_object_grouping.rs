use crate::{
    taiko::difficulty::{
        object::TaikoDifficultyObject,
        utils::{
            delta_time_normalizer::DeltaTimeNormalizer, has_interval::HasInterval,
            interval_grouping::IntervalGroupingUtils,
        },
    },
    util::sync::{RefCount, Weak},
};

#[derive(Debug)]
pub struct SameRhythmHitObjectGrouping {
    pub hit_objects: Vec<Weak<TaikoDifficultyObject>>,
    /// Use [`Self::upgraded_previous`] to access
    previous: Option<Weak<SameRhythmHitObjectGrouping>>,
    pub hit_object_interval: Option<f64>,
    pub hit_object_interval_ratio: f64,
    pub interval: f64,
}

impl SameRhythmHitObjectGrouping {
    const SNAP_TOLERANCE: f64 = IntervalGroupingUtils::MARGIN_OF_ERROR;

    pub fn new(
        previous: Option<Weak<Self>>,
        hit_objects: Vec<Weak<TaikoDifficultyObject>>,
    ) -> Self {
        let upgraded_prev = upgraded_previous(previous.as_ref());

        // * Cluster and normalise each hitobjects delta-time.
        let normalized_hit_objects =
            DeltaTimeNormalizer::normalize(&hit_objects, Self::SNAP_TOLERANCE);

        let normalized_hit_object_delta_time_count = hit_objects.len().saturating_sub(1);

        // * Secondary check to ensure there isn't any 'noise' or outliers by taking the modal delta time.
        let modal_delta = hit_objects
            .get(1)
            .and_then(|h| normalized_hit_objects.get(h))
            .copied()
            .map(f64::round)
            .unwrap_or(0.0);

        // * Calculate the average interval between hitobjects.
        let hit_object_interval = if normalized_hit_object_delta_time_count > 0 {
            if let Some(previous_delta) = upgraded_prev
                .as_ref()
                .and_then(|rc| rc.get().hit_object_interval)
                && f64::abs(modal_delta - previous_delta) <= Self::SNAP_TOLERANCE
            {
                Some(previous_delta)
            } else {
                Some(modal_delta)
            }
        } else {
            None
        };

        // * Calculate the ratio between this group's interval and the previous group's interval
        let hit_object_interval_ratio = if let Some((previous_interval, current_interval)) =
            upgraded_prev
                .as_ref()
                .and_then(|rc| rc.get().hit_object_interval)
                .zip(hit_object_interval)
        {
            current_interval / previous_interval
        } else {
            1.0
        };

        // * Calculate the interval from the previous group's start time
        let interval = if let Some((prev, curr)) = upgraded_prev
            .as_ref()
            .and_then(|prev| prev.get().start_time())
            .zip(start_time(&hit_objects))
        {
            if f64::abs(curr - prev) <= Self::SNAP_TOLERANCE {
                0.0
            } else {
                curr - prev
            }
        } else {
            f64::INFINITY
        };

        Self {
            hit_objects,
            previous,
            hit_object_interval,
            hit_object_interval_ratio,
            interval,
        }
    }

    pub fn upgraded_previous(&self) -> Option<RefCount<Self>> {
        upgraded_previous(self.previous.as_ref())
    }

    pub fn first_hit_object(&self) -> Option<RefCount<TaikoDifficultyObject>> {
        first_hit_object(&self.hit_objects)
    }

    pub fn start_time(&self) -> Option<f64> {
        start_time(&self.hit_objects)
    }

    pub fn duration(&self) -> Option<f64> {
        duration(&self.hit_objects)
    }

    pub fn upgraded_hit_objects(
        &self,
    ) -> impl Iterator<Item = RefCount<TaikoDifficultyObject>> + use<'_> {
        self.hit_objects.iter().filter_map(Weak::upgrade)
    }
}

fn upgraded_previous(
    previous: Option<&Weak<SameRhythmHitObjectGrouping>>,
) -> Option<RefCount<SameRhythmHitObjectGrouping>> {
    previous.and_then(Weak::upgrade)
}

fn first_hit_object(
    hit_objects: &[Weak<TaikoDifficultyObject>],
) -> Option<RefCount<TaikoDifficultyObject>> {
    hit_objects.first().and_then(Weak::upgrade)
}

fn start_time(hit_objects: &[Weak<TaikoDifficultyObject>]) -> Option<f64> {
    first_hit_object(hit_objects).map(|h| h.get().start_time)
}

fn duration(hit_objects: &[Weak<TaikoDifficultyObject>]) -> Option<f64> {
    hit_objects
        .last()
        .and_then(Weak::upgrade)
        .zip(start_time(hit_objects))
        .map(|(last, start)| last.get().start_time - start)
}

impl HasInterval for SameRhythmHitObjectGrouping {
    fn interval(&self) -> f64 {
        self.interval
    }
}
