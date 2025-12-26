use crate::{
    any::difficulty::object::{HasStartTime, IDifficultyObject},
    mania::difficulty::{
        evaluators::{IndividualStrainEvaluator, OverallStrainEvaluator},
        object::ManiaDifficultyObject,
    },
    util::sync::RefCount,
};

define_skill! {
    #[allow(clippy::struct_field_names)]
    pub struct Strain: StrainDecaySkill => [RefCount<ManiaDifficultyObject>][ManiaDifficultyObject] {
        individual_strains: Box<[f64]>,
        highest_individual_strain: f64 = 0.0,
        overall_strain: f64 = 1.0,
    }

    pub fn new(total_columns: usize) -> Self {
        Self {
            individual_strains: vec![0.0; total_columns].into_boxed_slice(),
            highest_individual_strain: 0.0,
            overall_strain: 1.0,
        }
    }
}

impl Strain {
    const INDIVIDUAL_DECAY_BASE: f64 = 0.125;
    const OVERALL_DECAY_BASE: f64 = 0.3;

    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 1.0;

    fn calculate_initial_strain(
        &self,
        offset: f64,
        curr: &ManiaDifficultyObject,
        objects: &[RefCount<ManiaDifficultyObject>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        apply_decay(
            self.highest_individual_strain,
            offset - prev_start_time,
            Self::INDIVIDUAL_DECAY_BASE,
        ) + apply_decay(
            self.overall_strain,
            offset - prev_start_time,
            Self::OVERALL_DECAY_BASE,
        )
    }

    fn strain_value_of(
        &mut self,
        curr: &ManiaDifficultyObject,
        _: &[RefCount<ManiaDifficultyObject>],
    ) -> f64 {
        let mania_curr = curr;

        self.individual_strains[mania_curr.column] = apply_decay(
            self.individual_strains[mania_curr.column],
            mania_curr.column_strain_time,
            Self::INDIVIDUAL_DECAY_BASE,
        );

        self.individual_strains[mania_curr.column] +=
            IndividualStrainEvaluator::evaluate_diff_of(curr);

        // * Take the hardest individualStrain for notes that happen at the same time (in a chord).
        // * This is to ensure the order in which the notes are processed does not affect the resultant total strain.
        self.highest_individual_strain = if mania_curr.delta_time <= 1.0 {
            self.highest_individual_strain
                .max(self.individual_strains[mania_curr.column])
        } else {
            self.individual_strains[mania_curr.column]
        };

        self.overall_strain = apply_decay(
            self.overall_strain,
            mania_curr.delta_time,
            Self::OVERALL_DECAY_BASE,
        );
        self.overall_strain += OverallStrainEvaluator::evaluate_diff_of(curr);

        // * By subtracting CurrentStrain, this skill effectively only considers the maximum strain of any one hitobject within each strain section.
        self.highest_individual_strain + self.overall_strain
            - self.strain_decay_skill_current_strain
    }
}

fn apply_decay(value: f64, delta_time: f64, decay_base: f64) -> f64 {
    value * f64::powf(decay_base, delta_time / 1000.0)
}
