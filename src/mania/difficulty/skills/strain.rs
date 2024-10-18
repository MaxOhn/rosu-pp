use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill, StrainDecaySkill},
    },
    mania::difficulty::object::ManiaDifficultyObject,
    util::strains_vec::StrainsVec,
};

const INDIVIDUAL_DECAY_BASE: f64 = 0.125;
const OVERALL_DECAY_BASE: f64 = 0.3;
const RELEASE_THRESHOLD: f64 = 30.0;

const SKILL_MULTIPLIER: f64 = 1.0;
const STRAIN_DECAY_BASE: f64 = 1.0;

#[allow(clippy::struct_field_names)]
pub struct Strain {
    start_times: Box<[f64]>,
    end_times: Box<[f64]>,
    individual_strains: Box<[f64]>,

    individual_strain: f64,
    overall_strain: f64,

    inner: StrainDecaySkill,
}

impl Strain {
    pub fn new(total_columns: usize) -> Self {
        Self {
            start_times: vec![0.0; total_columns].into_boxed_slice(),
            end_times: vec![0.0; total_columns].into_boxed_slice(),
            individual_strains: vec![0.0; total_columns].into_boxed_slice(),
            individual_strain: 0.0,
            overall_strain: 1.0,
            inner: StrainDecaySkill::default(),
        }
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
        skill.difficulty_value(StrainDecaySkill::DECAY_WEIGHT)
    }

    const fn curr_strain(&self) -> f64 {
        self.inner.curr_strain
    }

    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.inner.curr_strain
    }

    fn strain_value_at(&mut self, curr: &ManiaDifficultyObject) -> f64 {
        *self.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.curr_strain_mut() += self.strain_value_of(curr) * SKILL_MULTIPLIER;

        self.curr_strain()
    }

    fn strain_value_of(&mut self, curr: &ManiaDifficultyObject) -> f64 {
        let mania_curr = curr;
        let start_time = mania_curr.start_time;
        let end_time = mania_curr.end_time;
        let column = mania_curr.base_column;
        let mut is_overlapping = false;

        // * Lowest value we can assume with the current information
        let mut closest_end_time = (end_time - start_time).abs();
        // * Factor to all additional strains in case something else is held
        let mut hold_factor = 1.0;
        // * Addition to the current note in case it's a hold and has to be released awkwardly
        let mut hold_addition = 0.0;

        for i in 0..self.end_times.len() {
            // * The current note is overlapped if a previous note or end is overlapping the current note body
            is_overlapping |= self.end_times[i] > start_time + 1.0
                && end_time > self.end_times[i] + 1.0
                && start_time > self.start_times[i] + 1.0;

            // * We give a slight bonus to everything if something is held meanwhile
            if self.end_times[i] > end_time + 1.0 && start_time > self.start_times[i] + 1.0 {
                hold_factor = 1.25;
            }

            closest_end_time = (end_time - self.end_times[i]).abs().min(closest_end_time);
        }

        // * The hold addition is given if there was an overlap, however it is only valid if there are no other note with a similar ending.
        // * Releasing multiple notes is just as easy as releasing 1. Nerfs the hold addition by half if the closest release is release_threshold away.
        // * holdAddition
        // *     ^
        // * 1.0 + - - - - - -+-----------
        // *     |           /
        // * 0.5 + - - - - -/   Sigmoid Curve
        // *     |         /|
        // * 0.0 +--------+-+---------------> Release Difference / ms
        // *         release_threshold
        if is_overlapping {
            hold_addition = (1.0 + (0.27 * (RELEASE_THRESHOLD - closest_end_time)).exp()).recip();
        }

        // * Decay and increase individualStrains in own column
        self.individual_strains[column] = apply_decay(
            self.individual_strains[column],
            start_time - self.start_times[column],
            INDIVIDUAL_DECAY_BASE,
        );
        self.individual_strains[column] += 2.0 * hold_factor;

        // * For notes at the same time (in a chord), the individualStrain should be the hardest individualStrain out of those columns
        self.individual_strain = if mania_curr.delta_time <= 1.0 {
            self.individual_strain.max(self.individual_strains[column])
        } else {
            self.individual_strains[column]
        };

        // * Decay and increase overallStrain
        self.overall_strain = apply_decay(self.overall_strain, curr.delta_time, OVERALL_DECAY_BASE);
        self.overall_strain += (1.0 + hold_addition) * hold_factor;

        // * Update startTimes and endTimes arrays
        self.start_times[column] = start_time;
        self.end_times[column] = end_time;

        // * By subtracting CurrentStrain, this skill effectively only considers the maximum strain of any one hitobject within each strain section.
        self.individual_strain + self.overall_strain - self.curr_strain()
    }
}

impl ISkill for Strain {
    type DifficultyObjects<'a> = [ManiaDifficultyObject];
}

impl Skill<'_, Strain> {
    fn calculate_initial_strain(&mut self, offset: f64, curr: &ManiaDifficultyObject) -> f64 {
        let prev_start_time = curr
            .previous(0, self.diff_objects)
            .map_or(0.0, |prev| prev.start_time);

        let time = offset - prev_start_time;

        let individual = apply_decay(self.inner.individual_strain, time, INDIVIDUAL_DECAY_BASE);
        let overall = apply_decay(self.inner.overall_strain, time, OVERALL_DECAY_BASE);

        individual + overall
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

    pub fn process(&mut self, curr: &ManiaDifficultyObject) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / StrainDecaySkill::SECTION_LEN).ceil()
                * StrainDecaySkill::SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += StrainDecaySkill::SECTION_LEN;
        }

        let strain_value_at = self.inner.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }
}

fn apply_decay(value: f64, delta_time: f64, decay_base: f64) -> f64 {
    value * decay_base.powf(delta_time / 1000.0)
}
