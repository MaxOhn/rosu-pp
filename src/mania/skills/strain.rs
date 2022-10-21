use crate::mania::difficulty_object::ManiaDifficultyObject;

use super::{previous, Skill, StrainDecaySkill, StrainSkill};

#[derive(Clone, Debug)]
pub(crate) struct Strain {
    start_times: Vec<f64>,
    end_times: Vec<f64>,
    individual_strains: Vec<f64>,

    individual_strain: f64,
    overall_strain: f64,

    curr_strain: f64,
    curr_section_peak: f64,
    curr_section_end: f64,

    pub(crate) strain_peaks: Vec<f64>,
}

impl Strain {
    const INDIVIDUAL_DECAY_BASE: f64 = 0.125;
    const OVERALL_DECAY_BASE: f64 = 0.3;
    const RELEASE_THRESHOLD: f64 = 24.0;

    pub(crate) fn new(total_columns: usize) -> Self {
        Self {
            start_times: vec![0.0; total_columns],
            end_times: vec![0.0; total_columns],
            individual_strains: vec![0.0; total_columns],
            individual_strain: 0.0,
            overall_strain: 1.0,
            curr_strain: 0.0,
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            strain_peaks: Vec::new(),
        }
    }

    fn apply_decay(value: f64, delta_time: f64, decay_base: f64) -> f64 {
        value * decay_base.powf(delta_time / 1000.0)
    }
}

impl Skill for Strain {
    #[inline]
    fn process(&mut self, curr: &ManiaDifficultyObject, diff_objects: &[ManiaDifficultyObject]) {
        <Self as StrainSkill>::process(self, curr, diff_objects)
    }

    #[inline]
    fn difficulty_value(self) -> f64 {
        <Self as StrainSkill>::difficulty_value(self)
    }
}

impl StrainSkill for Strain {
    const DECAY_WEIGHT: f64 = 0.9;

    #[inline]
    fn curr_section_end(&self) -> f64 {
        self.curr_section_end
    }

    #[inline]
    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.curr_section_end
    }

    #[inline]
    fn curr_section_peak(&self) -> f64 {
        self.curr_section_peak
    }

    #[inline]
    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.curr_section_peak
    }

    #[inline]
    fn strain_peaks_mut(&mut self) -> &mut Vec<f64> {
        &mut self.strain_peaks
    }

    #[inline]
    fn strain_value_at(&mut self, curr: &ManiaDifficultyObject) -> f64 {
        <Self as StrainDecaySkill>::strain_value_at(self, curr)
    }

    #[inline]
    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &ManiaDifficultyObject,
        diff_objects: &[ManiaDifficultyObject],
    ) -> f64 {
        <Self as StrainDecaySkill>::calculate_initial_strain(self, time, curr, diff_objects)
    }
}

impl StrainDecaySkill for Strain {
    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 1.0;

    #[inline]
    fn curr_strain(&self) -> f64 {
        self.curr_strain
    }

    #[inline]
    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.curr_strain
    }

    fn strain_value_of(&mut self, curr: &ManiaDifficultyObject) -> f64 {
        let mania_curr = curr;
        let start_time = mania_curr.start_time;
        let end_time = mania_curr.end_time;
        let col = mania_curr.base_column;
        let mut is_overlapping = false;

        // * Lowest value we can assume with the current information
        let mut closest_end_time = (end_time - start_time).abs();
        // * Factor to all additional strains in case something else is held
        let mut hold_factor = 1.0;
        // * Addition to the current note in case it's a hold and has to be released awkwardly
        let mut hold_addition = 0.0;

        for i in 0..self.end_times.len() {
            // * The current note is overlapped if a previous note or end is overlapping the current note body
            is_overlapping |=
                self.end_times[i] > start_time + 1.0 && end_time > self.end_times[i] + 1.0;

            // * We give a slight bonus to everything if something is held meanwhile
            if self.end_times[i] > end_time + 1.0 {
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
            hold_addition =
                (1.0 + (0.5 * (Self::RELEASE_THRESHOLD - closest_end_time)).exp()).recip();
        }

        // * Decay and increase individualStrains in own column
        self.individual_strains[col] = Self::apply_decay(
            self.individual_strains[col],
            start_time - self.start_times[col],
            Self::INDIVIDUAL_DECAY_BASE,
        );
        self.individual_strains[col] += 2.0 * hold_factor;

        // * For notes at the same time (in a chord), the individualStrain should be the hardest individualStrain out of those columns
        self.individual_strain = if mania_curr.delta_time <= 1.0 {
            self.individual_strain.max(self.individual_strains[col])
        } else {
            self.individual_strains[col]
        };

        // * Decay and increase overallStrain
        self.overall_strain = Self::apply_decay(
            self.overall_strain,
            curr.delta_time,
            Self::OVERALL_DECAY_BASE,
        );
        self.overall_strain += (1.0 + hold_addition) * hold_factor;

        // * Update startTimes and endTimes arrays
        self.start_times[col] = start_time;
        self.end_times[col] = end_time;

        // * By subtracting CurrentStrain, this skill effectively only considers the maximum strain of any one hitobject within each strain section.
        self.individual_strain + self.overall_strain - self.curr_strain
    }

    fn calculate_initial_strain(
        &self,
        offset: f64,
        curr: &ManiaDifficultyObject,
        diff_objects: &[ManiaDifficultyObject],
    ) -> f64 {
        let prev_start = previous(diff_objects, curr.idx, 0).map_or(0.0, |h| h.start_time);

        let individual_decay = Self::apply_decay(
            self.individual_strain,
            offset - prev_start,
            Self::INDIVIDUAL_DECAY_BASE,
        );

        let overall_decay = Self::apply_decay(
            self.overall_strain,
            offset - prev_start,
            Self::OVERALL_DECAY_BASE,
        );

        individual_decay + overall_decay
    }
}
