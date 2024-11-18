use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill, StrainDecaySkill},
    },
    catch::difficulty::object::CatchDifficultyObject,
    util::strains_vec::StrainsVec,
};

const ABSOLUTE_PLAYER_POSITIONING_ERROR: f32 = 16.0;
const NORMALIZED_HITOBJECT_RADIUS: f32 = 41.0;
const DIRECTION_CHANGE_BONUS: f64 = 21.0;

const SKILL_MULTIPLIER: f64 = 1.0;
const STRAIN_DECAY_BASE: f64 = 0.2;

const DECAY_WEIGHT: f64 = 0.94;

const SECTION_LEN: f64 = 750.0;

pub struct Movement {
    inner: StrainDecaySkill,
    last_player_pos: Option<f32>,
    last_dist_moved: f32,
    last_strain_time: f64,
    clock_rate: f64,
}

impl Movement {
    pub fn new(clock_rate: f64) -> Self {
        Self {
            inner: StrainDecaySkill::default(),
            last_player_pos: None,
            last_dist_moved: 0.0,
            last_strain_time: 0.0,
            clock_rate,
        }
    }

    const fn curr_strain(&self) -> f64 {
        self.inner.curr_strain
    }

    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.inner.curr_strain
    }

    fn strain_value_at(&mut self, curr: &CatchDifficultyObject) -> f64 {
        *self.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.curr_strain_mut() += self.strain_value_of(curr) * SKILL_MULTIPLIER;

        self.curr_strain()
    }

    fn strain_value_of(&mut self, curr: &CatchDifficultyObject) -> f64 {
        let last_player_pos = self.last_player_pos.unwrap_or(curr.last_normalized_pos);

        let term = NORMALIZED_HITOBJECT_RADIUS - ABSOLUTE_PLAYER_POSITIONING_ERROR;
        let mut player_pos =
            last_player_pos.clamp(curr.normalized_pos - term, curr.normalized_pos + term);

        let dist_moved = player_pos - last_player_pos;

        let weighted_strain_time = curr.strain_time + 13.0 + (3.0 / self.clock_rate);

        let mut dist_addition = f64::from(dist_moved.abs()).powf(1.3) / 510.0;
        let sqrt_strain = weighted_strain_time.sqrt();

        let mut edge_dash_bonus: f64 = 0.0;

        if dist_moved.abs() > 0.1 {
            if self.last_dist_moved.abs() > 0.1
                && dist_moved.signum() != self.last_dist_moved.signum()
            {
                let bonus_factor = f64::from(dist_moved.abs().min(50.0) / 50.0);
                let anti_flow_factor =
                    f64::from(self.last_dist_moved.abs().min(70.0) / 70.0).max(0.38);

                dist_addition += DIRECTION_CHANGE_BONUS / (self.last_strain_time + 16.0).sqrt()
                    * bonus_factor
                    * anti_flow_factor
                    * (1.0 - (weighted_strain_time / 1000.0).powf(3.0)).max(0.0);
            }

            dist_addition += 12.5
                * f64::from(dist_moved.abs().min(NORMALIZED_HITOBJECT_RADIUS * 2.0))
                / f64::from(NORMALIZED_HITOBJECT_RADIUS * 6.0)
                / sqrt_strain;
        }

        if curr.last_object.dist_to_hyper_dash <= 20.0 {
            if curr.last_object.hyper_dash {
                player_pos = curr.normalized_pos;
            } else {
                edge_dash_bonus += 5.7;
            }

            dist_addition *= 1.0
                + edge_dash_bonus
                    * f64::from((20.0 - curr.last_object.dist_to_hyper_dash) / 20.0)
                    * ((curr.strain_time * self.clock_rate).min(265.0) / 265.0).powf(1.5);
        }

        self.last_player_pos = Some(player_pos);
        self.last_dist_moved = dist_moved;
        self.last_strain_time = curr.strain_time;

        dist_addition / weighted_strain_time
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn difficulty_value(self) -> f64 {
        Self::static_difficulty_value(self.inner)
    }

    /// Use [`difficulty_value`] instead whenever possible because
    /// [`as_difficulty_value`] clones internally.
    pub fn as_difficulty_value(&self) -> f64 {
        Self::static_difficulty_value(self.inner.clone())
    }

    fn static_difficulty_value(skill: StrainDecaySkill) -> f64 {
        skill.difficulty_value(DECAY_WEIGHT)
    }
}

impl ISkill for Movement {
    type DifficultyObjects<'a> = [CatchDifficultyObject];
}

impl<'a> Skill<'a, Movement> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &CatchDifficultyObject) -> f64 {
        let prev_start_time = curr
            .previous(0, self.diff_objects)
            .map_or(0.0, |prev| prev.start_time);

        self.inner.curr_strain() * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    const fn curr_section_peak(&self) -> f64 {
        self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_peak
    }

    const fn curr_section_end(&self) -> f64 {
        self.inner.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_end
    }

    pub fn process(&mut self, curr: &CatchDifficultyObject) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / SECTION_LEN).ceil() * SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += SECTION_LEN;
        }

        let strain_value_at = self.inner.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }
}
