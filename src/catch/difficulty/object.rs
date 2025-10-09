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
    pub player_pos: f32,
    pub last_player_pos: f32,
    pub dist_moved: f32,
    pub exact_dist_moved: f32,
    pub strain_time: f64,
    pub last_object: LastObject,
}

impl CatchDifficultyObject {
    pub const NORMALIZED_HALF_CATCHER_WIDTH: f32 = 41.0;

    const ABSOLUTE_PLAYER_POSITIONING_ERROR: f32 = 16.0;

    pub fn new(
        hit_object: &PalpableObject,
        last_object: &PalpableObject,
        clock_rate: f64,
        scaling_factor: f32,
        last_player_pos: Option<f32>,
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
            player_pos: last_player_pos,
        };

        let mut this = Self {
            idx,
            start_time,
            delta_time,
            normalized_pos,
            last_normalized_pos,
            player_pos: 0.0,
            last_player_pos: 0.0,
            dist_moved: 0.0,
            exact_dist_moved: 0.0,
            strain_time,
            last_object,
        };

        this.set_movement_state();

        this
    }

    fn set_movement_state(&mut self) {
        self.last_player_pos = self
            .last_object
            .player_pos
            .unwrap_or(self.last_normalized_pos);

        let term = Self::NORMALIZED_HALF_CATCHER_WIDTH - Self::ABSOLUTE_PLAYER_POSITIONING_ERROR;

        self.player_pos = self
            .last_player_pos
            .clamp(self.normalized_pos - term, self.normalized_pos + term);

        self.dist_moved = self.player_pos - self.last_player_pos;

        // * For the exact position we consider that the catcher is in the correct position for both objects
        self.exact_dist_moved = self.normalized_pos - self.last_player_pos;

        // * After a hyperdash we ARE in the correct position. Always!
        if self.last_object.hyper_dash {
            self.player_pos = self.normalized_pos;
        }
    }
}

pub struct LastObject {
    pub hyper_dash: bool,
    pub dist_to_hyper_dash: f32,
    pub player_pos: Option<f32>,
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
