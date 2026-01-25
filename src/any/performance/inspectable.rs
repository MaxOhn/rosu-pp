use crate::model::mode::IGameMode;

pub trait InspectablePerformance: IGameMode {
    type InspectPerformance<'a>;

    fn inspect_performance<'a>(
        perf: &'a Self::Performance<'_>,
        attrs: &'a Self::DifficultyAttributes,
    ) -> Self::InspectPerformance<'a>;
}
