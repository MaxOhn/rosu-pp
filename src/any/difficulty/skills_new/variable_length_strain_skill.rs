use crate::{any::difficulty::skills_new::skill::Skill, util::traits::IEnumerable};

pub trait VariableLengthStrainSkill: Skill {
    const DECAY_WEIGHT: f64 = 0.9;
    const MAX_SECTION_LENGTH: f64 = 400.0;
    const MAX_STORED_LENGTH: f64 = 11.0 / (1.0 - Self::DECAY_WEIGHT);

    fn process_internal<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;

    #[expect(dead_code, reason = "used by process_internal")]
    fn strain_value_at<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;

    fn backfill_peaks<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    );

    fn save_current_peak(&mut self, section_length: f64);

    fn start_new_section_from<'a>(
        &mut self,
        time: f64,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    );

    #[expect(dead_code, reason = "used by start_new_section_from")]
    fn calculate_initial_strain<'a>(
        &self,
        time: f64,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;

    fn into_current_strain_peaks(self) -> Vec<StrainPeak>;

    fn get_current_strain_peaks(
        mut strain_peaks: Vec<StrainPeak>,
        final_peak: Option<StrainPeak>,
        current_section_peak: f64,
        current_section_begin: f64,
        current_section_end: f64,
    ) -> Vec<StrainPeak> {
        if final_peak.is_none() {
            let final_peak = StrainPeak::new(
                current_section_peak,
                current_section_end - current_section_begin,
            );
            strain_peaks.cs_add_in_place(final_peak);
        }

        strain_peaks
    }

    fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64;
}

#[derive(Debug, Clone, Copy)]
pub struct StrainPeak {
    pub value: f64,
    pub section_length: f64,
}

impl StrainPeak {
    pub const fn new(value: f64, section_length: f64) -> Self {
        Self {
            value,
            section_length: section_length.round_ties_even(),
        }
    }
}

impl PartialEq for StrainPeak {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for StrainPeak {}

impl Ord for StrainPeak {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // * Reverse sort, highest is first.
        other.value.total_cmp(&self.value)
    }
}

impl PartialOrd for StrainPeak {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
