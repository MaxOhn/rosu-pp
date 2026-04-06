pub use self::{
    attributes::{DifficultyAttributes, PerformanceAttributes},
    calc_error::CalculateError,
    difficulty::{Difficulty, gradual::GradualDifficulty, inspect::InspectDifficulty},
    hitresult_generator::HitResultGenerator,
    performance::{
        HitResultPriority, Performance,
        gradual::GradualPerformance,
        inspectable::InspectablePerformance,
        into::{IntoModePerformance, IntoPerformance},
    },
    score_state::ScoreState,
    strains::Strains,
};

/// Hitresult generators that implement [`HitResultGenerator`].
///
/// [`HitResultGenerator`]: hitresult_generator::HitResultGenerator
pub mod hitresult_generator;

mod attributes;
mod calc_error;
pub(crate) mod difficulty;
pub(crate) mod hit_result;
mod performance;
mod score_state;
mod strains;
