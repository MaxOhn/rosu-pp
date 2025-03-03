use crate::{
    taiko::difficulty::object::TaikoDifficultyObject,
    util::{
        interval_grouping::HasInterval,
        sync::{RefCount, Weak},
    },
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
    pub fn new(
        previous: Option<Weak<Self>>,
        hit_objects: Vec<Weak<TaikoDifficultyObject>>,
    ) -> Self {
        // * Calculate the average interval between hitobjects, or null if there are fewer than two
        let hit_object_interval = if hit_objects.len() < 2 {
            None
        } else {
            duration(&hit_objects).map(|duration| duration / (hit_objects.len() - 1) as f64)
        };

        let upgraded_prev = upgraded_previous(previous.as_ref());

        // * Calculate the ratio between this group's interval and the previous group's interval
        let hit_object_interval_ratio = if let Some((prev, curr)) = upgraded_prev
            .as_ref()
            .and_then(|prev| prev.get().hit_object_interval)
            .zip(hit_object_interval)
        {
            curr / prev
        } else {
            1.0
        };

        // * Calculate the interval from the previous group's start time
        let interval = upgraded_prev
            .as_ref()
            .and_then(|prev| prev.get().start_time())
            .zip(start_time(&hit_objects))
            .map_or(f64::INFINITY, |(prev, curr)| curr - prev);

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
