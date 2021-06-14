use super::DifficultyHitObject;

use std::cmp::Ordering;

pub(crate) struct Strain {
    current_strain: f32,
    current_section_peak: f32,

    individual_strain: f32,
    overall_strain: f32,

    hold_end_times: Vec<f32>,
    individual_strains: Vec<f32>,
    pub(crate) strain_peaks: Vec<f32>,

    prev_time: Option<f32>,
}

const INDIVISUAL_DECAY_BASE: f32 = 0.125;
const OVERALL_DECAY_BASE: f32 = 0.3;
const STRAIN_DECAY_BASE: f32 = 1.0;

const SKILL_MULTIPLIER: f32 = 1.0;
const DECAY_WEIGHT: f32 = 0.9;

impl Strain {
    #[inline]
    pub(crate) fn new(column_count: u8) -> Self {
        Self {
            current_strain: 1.0,
            current_section_peak: 1.0,

            individual_strain: 0.0,
            overall_strain: 1.0,

            hold_end_times: vec![0.0; column_count as usize],
            individual_strains: vec![0.0; column_count as usize],
            strain_peaks: Vec::with_capacity(128),

            prev_time: None,
        }
    }

    #[inline]
    pub(crate) fn save_current_peak(&mut self) {
        self.strain_peaks.push(self.current_section_peak);
    }

    #[inline]
    pub(crate) fn start_new_section_from(&mut self, time: f32) {
        self.current_section_peak = self.peak_strain(time - self.prev_time.unwrap());
    }

    #[inline]
    fn peak_strain(&self, delta_time: f32) -> f32 {
        apply_decay(self.individual_strain, delta_time, INDIVISUAL_DECAY_BASE)
            + apply_decay(self.overall_strain, delta_time, OVERALL_DECAY_BASE)
    }

    #[inline]
    fn strain_decay(&self, ms: f32) -> f32 {
        STRAIN_DECAY_BASE.powf(ms / 1000.0)
    }

    #[inline]
    pub(crate) fn process(&mut self, current: &DifficultyHitObject) {
        self.current_strain *= self.strain_decay(current.delta);
        self.current_strain += self.strain_value_of(&current) * SKILL_MULTIPLIER;
        self.current_section_peak = self.current_strain.max(self.current_section_peak);
        self.prev_time.replace(current.base.start_time);
    }

    fn strain_value_of(&mut self, current: &DifficultyHitObject) -> f32 {
        let end_time = current.base.end_time();

        let mut hold_factor = 1.0;
        let mut hold_addition = 0.0;

        for col in 0..self.hold_end_times.len() {
            let hold_end_time = self.hold_end_times[col];

            if end_time > hold_end_time + 1.0 {
                if hold_end_time > current.base.start_time + 1.0 {
                    hold_addition = 1.0;
                }
            } else if (end_time - hold_end_time).abs() < 1.0 {
                hold_addition = 0.0;
            } else if end_time < hold_end_time - 1.0 {
                hold_factor = 1.25;
            }

            self.individual_strains[col] = apply_decay(
                self.individual_strains[col],
                current.delta,
                INDIVISUAL_DECAY_BASE,
            );
        }

        self.hold_end_times[current.column] = end_time;
        self.individual_strains[current.column] += 2.0 * hold_factor;
        self.individual_strain = self.individual_strains[current.column];

        self.overall_strain = apply_decay(self.overall_strain, current.delta, OVERALL_DECAY_BASE)
            + (1.0 + hold_addition) * hold_factor;

        self.individual_strain + self.overall_strain - self.current_strain
    }

    #[inline]
    pub(crate) fn difficulty_value(&mut self) -> f32 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        self.strain_peaks
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for &strain in self.strain_peaks.iter() {
            difficulty += strain * weight;
            weight *= DECAY_WEIGHT;
        }

        difficulty
    }
}

#[inline]
fn apply_decay(value: f32, delta_time: f32, decay_base: f32) -> f32 {
    value * decay_base.powf(delta_time / 1000.0)
}
