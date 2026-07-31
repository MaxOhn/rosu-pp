use crate::util::{difficulty::logistic, float_ext::FloatExt};

pub fn count_top_weighted_sliders(slider_strains: &[f64], consistent_top_strain: f64) -> f64 {
    if FloatExt::eq(consistent_top_strain, 0.0) {
        return 0.0;
    }

    slider_strains
        .iter()
        .map(|s| logistic(*s / consistent_top_strain, 0.88, 10.0, Some(1.1)))
        .sum()
}
