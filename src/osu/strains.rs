use crate::{
    Beatmap, Difficulty, any::difficulty::{skills::StrainSkill, skills_new::strain_skill::NewStrainSkill}, model::mode::ConvertError, osu::convert::prepare_map,
};

use super::difficulty::{DifficultyValues, skills::OsuSkills};

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

pub fn strains(difficulty: &Difficulty, map: &Beatmap) -> Result<OsuStrains, ConvertError> {
    let map = prepare_map(difficulty, map)?;

    let DifficultyValues {
        osu_objects: _,
        skills:
            OsuSkills {
                aim,
                aim_no_sliders,
                speed,
                flashlight,
            },
        attrs: _,
    } = DifficultyValues::calculate(difficulty, &map);

    Ok(OsuStrains {
        aim: aim.into_current_strain_peaks(),
        aim_no_sliders: aim_no_sliders.into_current_strain_peaks(),
        speed: speed.into_current_strain_peaks(),
        flashlight: flashlight.into_current_strain_peaks(),
    })
}
