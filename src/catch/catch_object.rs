use crate::parse::Pos2;

use super::fruit_or_juice::FruitParams;

const PLAYFIELD_WIDTH: f32 = 512.0;
const BASE_SPEED: f64 = 1.0;

#[derive(Clone, Debug)]
pub struct CatchObject {
    pub(crate) pos: f32,
    pub(crate) time: f64,

    pub(crate) hyper_dash: bool,
    pub(crate) hyper_dist: f32,
}

impl CatchObject {
    #[inline]
    pub(crate) fn new((pos, time): (Pos2, f64)) -> Self {
        Self {
            pos: pos.x,
            time,
            hyper_dash: false,
            hyper_dist: 0.0,
        }
    }

    pub(crate) fn with_hr(mut self, params: &mut FruitParams<'_>) -> Self {
        let mut offset_pos = self.pos;
        let time_diff = self.time - params.last_time;

        if let Some(last_pos_ref) = params.last_pos.filter(|_| time_diff <= 1000.0) {
            let pos_diff = offset_pos - last_pos_ref;

            if pos_diff.abs() > f32::EPSILON {
                if pos_diff.abs() < (time_diff as f32 / 3.0).floor() {
                    if pos_diff > 0.0 {
                        if offset_pos + pos_diff < PLAYFIELD_WIDTH {
                            offset_pos += pos_diff;
                        }
                    } else if offset_pos + pos_diff > 0.0 {
                        offset_pos += pos_diff;
                    }
                }

                params.last_pos.replace(offset_pos);
                params.last_time = self.time;
            }

            self.pos = offset_pos;
        } else {
            params.last_pos.replace(offset_pos);
            params.last_time = self.time;
        }

        self
    }

    pub(crate) fn init_hyper_dash(
        &mut self,
        half_catcher_width: f64,
        next: &CatchObject,
        last_direction: &mut i8,
        last_excess: &mut f64,
    ) {
        let next_x = next.pos;
        let curr_x = self.pos;

        let this_direction = (next_x > curr_x) as i8 * 2 - 1;
        let time_to_next = next.time - self.time - 1000.0 / 60.0 / 4.0;

        let sub = if *last_direction == this_direction {
            *last_excess
        } else {
            half_catcher_width
        };

        let dist_to_next = (next_x - curr_x).abs() as f64 - sub;
        let hyper_dist = (time_to_next * BASE_SPEED - dist_to_next) as f32;

        if hyper_dist < 0.0 {
            self.hyper_dash = true;
            *last_excess = half_catcher_width;
        } else {
            self.hyper_dist = hyper_dist;
            *last_excess = (hyper_dist as f64).max(0.0).min(half_catcher_width);
        }

        *last_direction = this_direction;
    }
}
