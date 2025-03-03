pub trait StrainSkill: Sized {
    type DifficultyObject<'a>;
    type DifficultyObjects<'a>: ?Sized;

    const DECAY_WEIGHT: f64 = 0.9;
    const SECTION_LENGTH: i32 = 400;

    fn process<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    );

    fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64;

    fn save_current_peak(&mut self);

    fn start_new_section_from<'a>(
        &mut self,
        time: f64,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    );

    fn into_current_strain_peaks(self) -> Vec<f64>;

    fn get_current_strain_peaks(mut strain_peaks: Vec<f64>, current_section_peak: f64) -> Vec<f64> {
        strain_peaks.push(current_section_peak);

        strain_peaks
    }

    fn difficulty_value(current_strain_peaks: Vec<f64>) -> f64;

    fn into_difficulty_value(self) -> f64;

    fn cloned_difficulty_value(&self) -> f64;
}

pub trait StrainDecaySkill: StrainSkill {
    fn calculate_initial_strain<'a>(
        &self,
        time: f64,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;

    fn strain_value_at<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;

    fn strain_decay(ms: f64) -> f64;
}

pub fn difficulty_value(current_strain_peaks: Vec<f64>, decay_weight: f64) -> f64 {
    let mut difficulty = 0.0;
    let mut weight = 1.0;

    // * Sections with 0 strain are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
    // * These sections will not contribute to the difficulty.
    let mut peaks = current_strain_peaks;
    peaks.retain(|&p| p > 0.0);
    peaks.sort_unstable_by(|a, b| b.total_cmp(a));

    // * Difficulty is the weighted sum of the highest strains from every section.
    // * We're sorting from highest to lowest strain.
    for strain in peaks {
        difficulty += strain * weight;
        weight *= decay_weight;
    }

    difficulty
}

// ------- OLD STUFF; TODO: remove ---------

pub fn strain_decay(ms: f64, strain_decay_base: f64) -> f64 {
    strain_decay_base.powf(ms / 1000.0)
}
