use core::f64;

use crate::{
    GameMods,
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills_new::{
            strain_decay_base,
            variable_length_strain_skill::{StrainPeak, VariableLengthStrainSkill},
        },
    },
    osu::difficulty::{
        evaluators::{AgilityEvaluator, FlowAimEvaluator, SnapAimEvaluator},
        object::OsuDifficultyObject,
        skills::strain::count_top_weighted_sliders,
    },
    util::{
        difficulty::{lerp, logistic, logistic_exp, norm},
        float_ext::FloatExt,
        traits::{IEnumerable, IOrderedEnumerable},
    },
};

define_new_skill! {
    pub struct Aim: VariableLengthStrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64 = 0.0,
        slider_strains: Vec<f64> = Vec::with_capacity(64),
        mods: GameMods,
        include_sliders: bool,
        overall_difficulty: f64,
        obj_radius: f64,
    }
}

impl Aim {
    const SKILL_MULTIPLIER_SNAP: f64 = 70.9;
    const SKILL_MULTIPLIER_AGILITY: f64 = 2.35;
    const SKILL_MULTIPLIER_FLOW: f64 = 242.0;

    const SKILL_MULTIPLIER_TOTAL: f64 = 1.12;
    const COMBINED_SNAP_NORM_EXPONENT: f64 = 1.2;

    fn strain_decay(ms: f64) -> f64 {
        strain_decay_base(ms, 0.2)
    }

    fn calculate_initial_strain<'a>(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'a>,
        objects: &[OsuDifficultyObject<'a>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        self.current_strain * Self::strain_decay(time - prev_start_time)
    }

    fn strain_value_at<'a>(
        &mut self,
        curr: &OsuDifficultyObject<'a>,
        objects: &[OsuDifficultyObject<'a>],
    ) -> f64 {
        if self.mods.ap() {
            return 0.0;
        }

        let decay = Self::strain_decay(curr.adjusted_delta_time);

        self.current_strain *= decay;
        self.current_strain += self.calculate_adjusted_difficulty(curr, objects) * (1.0 - decay);

        if curr.base.is_slider() {
            self.slider_strains.push(self.current_strain);
        }

        self.current_strain
    }

    fn calculate_adjusted_difficulty<'a>(
        &self,
        curr: &OsuDifficultyObject<'a>,
        objects: &[OsuDifficultyObject<'a>],
    ) -> f64 {
        let snap_difficulty =
            SnapAimEvaluator::evaluate_diff_of(curr, objects, self.include_sliders)
                * Self::SKILL_MULTIPLIER_SNAP;
        let agility_difficulty =
            AgilityEvaluator::evaluate_diff_of(curr, objects) * Self::SKILL_MULTIPLIER_AGILITY;
        let flow_difficulty = FlowAimEvaluator::evaluate_diff_of(
            curr,
            objects,
            self.include_sliders,
            self.obj_radius,
        ) * Self::SKILL_MULTIPLIER_FLOW;

        let mut total_difficulty =
            self.calculate_total_value(snap_difficulty, agility_difficulty, flow_difficulty);

        if let Some(attraction_strength) = self.mods.attraction_strength() {
            total_difficulty *= 1.0 - attraction_strength;
        }

        total_difficulty *= 0.985 + self.overall_difficulty.max(0.0).powf(2.0) / 4000.0;

        total_difficulty
    }

    fn calculate_total_value(
        &self,
        snap_difficulty: f64,
        agility_difficulty: f64,
        flow_difficulty: f64,
    ) -> f64 {
        let mut snap_difficulty_new = snap_difficulty;
        let mut flow_difficulty_new = flow_difficulty;

        // * We compare flow to combined snap and agility because snap by itself doesn't have enough difficulty to be above flow on streams
        // * Agility on the other hand is supposed to measure the rate of cursor velocity changes while snapping
        // * So snapping every circle on a stream requires an enormous amount of agility at which point it's easier to flow
        let mut combined_snap_difficulty = norm(
            Self::COMBINED_SNAP_NORM_EXPONENT,
            [snap_difficulty_new, agility_difficulty],
        );

        let p_snap =
            Self::calculate_snap_flow_probability(flow_difficulty / combined_snap_difficulty);
        let p_flow = 1.0 - p_snap;

        if self.mods.td() {
            // * we don't adjust agility here since agility represents TD difficulty in a decent enough way
            snap_difficulty_new = snap_difficulty_new.powf(0.89);
            combined_snap_difficulty = norm(
                Self::COMBINED_SNAP_NORM_EXPONENT,
                [snap_difficulty_new, agility_difficulty],
            );
        }

        if self.mods.rx() {
            combined_snap_difficulty *= 0.75;
            flow_difficulty_new *= 0.6;
        }

        let total_difficulty = combined_snap_difficulty * p_snap + flow_difficulty_new * p_flow;

        total_difficulty * Self::SKILL_MULTIPLIER_TOTAL
    }

    fn calculate_snap_flow_probability(ratio: f64) -> f64 {
        // * A function that turns the ratio of snap : flow into the probability of snapping/flowing
        // * It has the constraints:
        // * P(snap) + P(flow) = 1 (the object is always either snapped or flowed)
        // * P(snap) = f(snap/flow), P(flow) = f(flow/snap) (ie snap and flow are symmetric and reversible)
        // * Therefore: f(x) + f(1/x) = 1
        // * 0 <= f(x) <= 1 (cannot have negative or greater than 100% probability of snapping or flowing)
        // * This logistic function is a solution, which fits nicely with the general idea of interpolation and provides a tuneable constant
        const K: f64 = 7.27;

        if FloatExt::eq(ratio, 0.0) {
            return 0.0;
        }

        if ratio.is_nan() {
            return 1.0;
        }

        logistic_exp(-K * ratio.log(f64::consts::E), None)
    }

    pub fn get_difficult_sliders(&self) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }

        let max_slider_strain = self.slider_strains.iter().copied().fold(0.0, f64::max);

        if FloatExt::eq(max_slider_strain, 0.0) {
            return 0.0;
        }

        self.slider_strains
            .iter()
            .copied()
            .map(|strain| logistic(strain / max_slider_strain, 0.5, 12.0, None))
            .sum()
    }

    pub fn count_top_weighted_sliders(&self, difficulty_value: f64) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }

        // * What would the top strain be if all strain values were identical
        let consistent_top_strain = difficulty_value / 10.0;

        count_top_weighted_sliders(&self.slider_strains, consistent_top_strain)
    }

    pub fn difficulty_value(
        current_strain_peaks: Vec<StrainPeak>,
        current_section_peak: f64,
        current_section_begin: f64,
        current_section_end: f64,
    ) -> f64 {
        aim_difficulty_value(
            Self::get_reduced_strain_peaks(Self::get_current_strain_peaks(
                current_strain_peaks,
                current_section_peak,
                current_section_begin,
                current_section_end,
            )),
            Self::MAX_SECTION_LENGTH,
            Self::DECAY_WEIGHT,
        )
    }

    pub fn cloned_difficulty_value(&self) -> f64 {
        Self::difficulty_value(
            self.skill_strain_peaks.clone(),
            self.skill_current_section_peak,
            self.skill_current_section_begin,
            self.skill_current_section_end,
        )
    }

    fn get_reduced_strain_peaks(current_strain_peaks: Vec<StrainPeak>) -> Vec<StrainPeak> {
        const REDUCED_SECTION_TIME: f64 = 4000.0;
        const REDUCED_STRAIN_BASELINE: f64 = 0.727;
        const CHUNK_SIZE: f64 = 20.0;

        // * Sections with 0 strain are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
        // * These sections will not contribute to the difficulty.
        let mut strains = current_strain_peaks.cs_where(|p| p.value > 0.0);

        let mut time = 0.0;
        let mut skip_count = 0;

        // * We are reducing the highest strains first to account for extreme difficulty spikes
        // * Strains are split into 20ms chunks to try to mitigate inconsistencies caused by reducing strains
        while strains.len() > skip_count && time < REDUCED_SECTION_TIME {
            let strain = strains[skip_count];

            let mut added_time = 0.0;
            while added_time < strain.section_length {
                let scale = lerp(
                    1.0,
                    10.0,
                    ((time + added_time) / REDUCED_SECTION_TIME).clamp(0.0, 1.0),
                )
                .log10();

                // * intentionally add at end and sort afterwards, should be cheaper.
                strains.push(StrainPeak::new(
                    strain.value * lerp(REDUCED_STRAIN_BASELINE, 1.0, scale),
                    CHUNK_SIZE.min(strain.section_length - added_time),
                ));

                added_time += CHUNK_SIZE;
            }

            time += strain.section_length;
            skip_count += 1;
        }

        strains.split_off(skip_count).cs_order_descending()
    }

    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        4.0 * difficulty.powf(3.0)
    }
}

fn aim_difficulty_value(
    reduced_strain_peaks: Vec<StrainPeak>,
    max_section_length: f64,
    decay_weight: f64,
) -> f64 {
    let mut difficulty = 0.0;
    let mut time = 0.0;

    // * Difficulty is a continuous weighted sum of the sorted strains
    for strain in reduced_strain_peaks {
        /* Weighting function can be thought of as:
                b
                ∫ DecayWeight^x dx
                a
            where a = startTime and b = endTime

            Technically, the function below has been slightly modified from the equation above.
            The real function would be
                double weight = DiffUtils.Pow(DecayWeight, startTime) - DiffUtils.Pow(DecayWeight, endTime);
                ...
                return difficulty / Math.Log(1 / DecayWeight);
            E.g. for a DecayWeight of 0.9, we're multiplying by 10 instead of 9.49122...

            This change makes it so that a map composed solely of MaxSectionLength chunks will have the exact same value when summed in this class and StrainSkill.
            Doing this ensures the relationship between strain values and difficulty values remains the same between the two classes.
        */
        let start_time = time;
        let end_time = time + strain.section_length / max_section_length;

        let weight = decay_weight.powf(start_time) - decay_weight.powf(end_time);

        difficulty += strain.value * weight;
        time = end_time;
    }

    difficulty / (1.0 - decay_weight)
}
