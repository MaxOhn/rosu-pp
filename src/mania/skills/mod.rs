mod strain;
mod traits;

pub(crate) use self::{
    strain::Strain,
    traits::{Skill, StrainDecaySkill, StrainSkill},
};

use super::difficulty_object::ManiaDifficultyObject;

fn previous(
    diff_objects: &[ManiaDifficultyObject],
    curr: usize,
    backwards_idx: usize,
) -> Option<&ManiaDifficultyObject> {
    curr.checked_sub(backwards_idx + 1)
        .and_then(|idx| diff_objects.get(idx))
}
