use crate::{
    any::difficulty::object::{HasStartTime, IDifficultyObject},
    catch::object::palpable::PalpableObject,
};

pub struct CatchDifficultyObject {
    pub idx: usize,
    pub start_time: f64,
    pub delta_time: f64,
    pub normalized_pos: f32,
    pub last_normalized_pos: f32,
    pub strain_time: f64,
    pub last_object: LastObject,
}

impl CatchDifficultyObject {
    pub const NORMALIZED_HITOBJECT_RADIUS: f32 = 41.0;

    pub fn new(
        hit_object: &PalpableObject,
        last_object: &PalpableObject,
        clock_rate: f64,
        scaling_factor: f32,
        idx: usize,
    ) -> Self {
        let normalized_pos = hit_object.effective_x() * scaling_factor;
        let last_normalized_pos = last_object.effective_x() * scaling_factor;

        let start_time = hit_object.start_time / clock_rate;
        let delta_time = (hit_object.start_time - last_object.start_time) / clock_rate;
        let strain_time = delta_time.max(40.0);

        let last_object = LastObject {
            hyper_dash: last_object.hyper_dash,
            dist_to_hyper_dash: last_object.dist_to_hyper_dash,
        };

        Self {
            idx,
            start_time,
            delta_time,
            normalized_pos,
            last_normalized_pos,
            strain_time,
            last_object,
        }
    }
}

pub struct LastObject {
    pub hyper_dash: bool,
    pub dist_to_hyper_dash: f32,
}

impl IDifficultyObject for CatchDifficultyObject {
    type DifficultyObjects = [Self];

    fn idx(&self) -> usize {
        self.idx
    }
}

impl HasStartTime for CatchDifficultyObject {
    fn start_time(&self) -> f64 {
        self.start_time
    }
}
