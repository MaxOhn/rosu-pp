use crate::util::{difficulty::logistic, float_ext::FloatExt, strains_vec::StrainsVec};

pub trait OsuStrainSkill {
    const REDUCED_SECTION_COUNT: usize = 10;
    const REDUCED_STRAIN_BASELINE: f64 = 0.75;

    fn difficulty_to_performance(difficulty: f64) -> f64 {
        difficulty_to_performance(difficulty)
    }
}

pub fn difficulty_value(
    current_strain_peaks: StrainsVec,
    reduced_section_count: usize,
    reduced_strain_baseline: f64,
    decay_weight: f64,
) -> f64 {
    let mut difficulty = 0.0;
    let mut weight = 1.0;

    let mut peaks = current_strain_peaks;

    // Note that we remove all initial zeros here.
    let peaks_iter = peaks.sorted_non_zero_iter_mut().take(reduced_section_count);

    for (i, strain) in peaks_iter.enumerate() {
        // Note that unless `reduced_strain_baseline == 0.0`, `strain` can
        // never be `0.0`.
        let clamped = f64::from((i as f32 / reduced_section_count as f32).clamp(0.0, 1.0));
        let scale = f64::log10(lerp(1.0, 10.0, clamped));
        *strain *= lerp(reduced_strain_baseline, 1.0, scale);
    }

    peaks.sort_desc();

    // Sanity assert; will most definitely never panic
    debug_assert!(reduced_strain_baseline != 0.0);

    // SAFETY: As noted, zeros were removed from all initial strains and no
    // strain was mutated to a zero afterwards.
    let peaks = unsafe { peaks.transmute_into_vec() };

    // Using `Vec<f64>` is much faster for iteration than `StrainsVec`

    for strain in peaks {
        difficulty += strain * weight;
        weight *= decay_weight;
    }

    difficulty
}

pub fn count_top_weighted_sliders(slider_strains: &[f64], difficulty_value: f64) -> f64 {
    if slider_strains.is_empty() {
        return 0.0;
    }

    // * What would the top strain be if all strain values were identical
    let consistent_top_strain = difficulty_value / 10.0;

    if FloatExt::eq(consistent_top_strain, 0.0) {
        return 0.0;
    }

    slider_strains
        .iter()
        .map(|s| logistic(*s / consistent_top_strain, 0.88, 10.0, Some(1.1)))
        .sum()
}

pub fn difficulty_to_performance(difficulty: f64) -> f64 {
    f64::powf(5.0 * f64::max(1.0, difficulty / 0.0675) - 4.0, 3.0) / 100_000.0
}

const fn lerp(start: f64, end: f64, amount: f64) -> f64 {
    start + (end - start) * amount
}
