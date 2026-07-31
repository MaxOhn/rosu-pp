use crate::{
    any::difficulty::skills_new::skill::Skill,
    util::traits::{IEnumerable, IOrderedEnumerable},
};

pub trait NewStrainSkill: Skill {
    const DECAY_WEIGHT: f64 = 0.9;
    const SECTION_LENGTH: i32 = 400;

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

    fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64;

    fn save_current_peak(&mut self);

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

    fn into_current_strain_peaks(self) -> Vec<f64>;

    fn get_current_strain_peaks(mut strain_peaks: Vec<f64>, current_section_peak: f64) -> Vec<f64> {
        strain_peaks.push(current_section_peak);

        strain_peaks
    }

    fn difficulty_value(current_strain_peaks: Vec<f64>) -> f64;

    fn into_difficulty_value(self) -> f64;

    fn cloned_difficulty_value(&self) -> f64;
}

pub fn strain_skill_difficulty_value(current_strain_peaks: Vec<f64>, decay_weight: f64) -> f64 {
    let mut difficulty = 0.0;
    let mut weight = 1.0;

    // * Sections with 0 strain are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
    // * These sections will not contribute to the difficulty.
    let peaks = current_strain_peaks.cs_where(|&p| p > 0.0);

    // * Difficulty is the weighted sum of the highest strains from every section.
    // * We're sorting from highest to lowest strain.
    for strain in peaks.cs_order_descending() {
        difficulty += strain * weight;
        weight *= decay_weight;
    }

    difficulty
}
