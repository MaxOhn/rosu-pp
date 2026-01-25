use std::cmp;

use crate::{
    Difficulty,
    any::{HitResultPriority, InspectablePerformance},
    osu::{Osu, OsuDifficultyAttributes, OsuScoreOrigin},
};

#[derive(Clone, Debug)]
pub struct InspectOsuPerformance<'a> {
    pub attrs: &'a OsuDifficultyAttributes,
    pub difficulty: &'a Difficulty,
    pub acc: Option<f64>,
    pub combo: Option<u32>,
    pub large_tick_hits: Option<u32>,
    pub small_tick_hits: Option<u32>,
    pub slider_end_hits: Option<u32>,
    pub n300: Option<u32>,
    pub n100: Option<u32>,
    pub n50: Option<u32>,
    pub misses: Option<u32>,
    pub hitresult_priority: HitResultPriority,
}

impl InspectOsuPerformance<'_> {
    pub fn total_hits(&self) -> u32 {
        cmp::min(
            self.difficulty.get_passed_objects() as u32,
            self.attrs.n_objects(),
        )
    }

    pub fn misses(&self) -> u32 {
        self.misses.map_or(0, |n| cmp::min(n, self.total_hits()))
    }

    pub fn lazer(&self) -> bool {
        self.difficulty.get_lazer()
    }

    pub fn using_classic_slider_acc(&self) -> bool {
        self.difficulty.get_mods().no_slider_head_acc(self.lazer())
    }

    pub fn origin(&self) -> OsuScoreOrigin {
        match (self.lazer(), self.using_classic_slider_acc()) {
            (false, _) => OsuScoreOrigin::Stable,
            (true, false) => OsuScoreOrigin::WithSliderAcc {
                max_large_ticks: self.attrs.n_large_ticks,
                max_slider_ends: self.attrs.n_sliders,
            },
            (true, true) => OsuScoreOrigin::WithoutSliderAcc {
                max_large_ticks: self.attrs.n_sliders + self.attrs.n_large_ticks,
                max_small_ticks: self.attrs.n_sliders,
            },
        }
    }
}

impl InspectablePerformance for Osu {
    type InspectPerformance<'a> = InspectOsuPerformance<'a>;

    fn inspect_performance<'a>(
        perf: &'a Self::Performance<'_>,
        attrs: &'a Self::DifficultyAttributes,
    ) -> Self::InspectPerformance<'a> {
        InspectOsuPerformance {
            attrs,
            difficulty: &perf.difficulty,
            acc: perf.acc,
            combo: perf.combo,
            large_tick_hits: perf.large_tick_hits,
            small_tick_hits: perf.small_tick_hits,
            slider_end_hits: perf.slider_end_hits,
            n300: perf.n300,
            n100: perf.n100,
            n50: perf.n50,
            misses: perf.misses,
            hitresult_priority: perf.hitresult_priority,
        }
    }
}
