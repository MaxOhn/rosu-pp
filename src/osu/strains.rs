use crate::Difficulty;

use super::{
    convert::OsuBeatmap,
    difficulty::{skills::OsuSkills, DifficultyValues},
};

/// The result of calculating the strains on a osu! map.
///
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug, PartialEq)]
pub struct OsuStrains {
    /// Strain peaks of the aim skill.
    pub aim: Vec<f64>,
    /// Strain peaks of the aim skill without sliders.
    pub aim_no_sliders: Vec<f64>,
    /// Strain peaks of the speed skill.
    pub speed: Vec<f64>,
    /// Strain peaks of the flashlight skill.
    pub flashlight: Vec<f64>,
}

impl OsuStrains {
    /// Time between two strains in ms.
    pub const SECTION_LEN: f64 = 400.0;
}

pub fn strains(difficulty: &Difficulty, converted: &OsuBeatmap<'_>) -> OsuStrains {
    let DifficultyValues {
        skills:
            OsuSkills {
                aim,
                aim_no_sliders,
                speed,
                flashlight,
            },
        attrs: _,
    } = DifficultyValues::calculate(difficulty, converted);

    OsuStrains {
        aim: aim.get_curr_strain_peaks().into_vec(),
        aim_no_sliders: aim_no_sliders.get_curr_strain_peaks().into_vec(),
        speed: speed.get_curr_strain_peaks().into_vec(),
        flashlight: flashlight.get_curr_strain_peaks().into_vec(),
    }
}
