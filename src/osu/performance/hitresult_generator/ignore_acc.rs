use std::cmp;

use crate::{
    any::{HitResultGenerator, HitResultPriority, hitresult_generator::IgnoreAccuracy},
    osu::{InspectOsuPerformance, Osu, OsuHitResults},
};

impl HitResultGenerator<Osu> for IgnoreAccuracy {
    #[expect(clippy::too_many_lines, reason = "it is what it is /shrug")]
    fn generate_hitresults(inspect: InspectOsuPerformance<'_>) -> OsuHitResults {
        let lazer = inspect.lazer();
        let using_classic_slider_acc = inspect.using_classic_slider_acc();

        let total_hits = inspect.total_hits();
        let misses = inspect.misses();

        let (slider_end_hits, large_tick_hits, small_tick_hits) =
            match (lazer, using_classic_slider_acc) {
                (false, _) => (0, 0, 0),
                (true, false) => {
                    let slider_end_hits = inspect
                        .slider_end_hits
                        .map_or(inspect.attrs.n_sliders, |n| {
                            cmp::min(n, inspect.attrs.n_sliders)
                        });

                    let large_tick_hits = inspect
                        .large_tick_hits
                        .map_or(inspect.attrs.n_large_ticks, |n| {
                            cmp::min(n, inspect.attrs.n_large_ticks)
                        });

                    (slider_end_hits, large_tick_hits, 0)
                }
                (true, true) => {
                    let small_tick_hits = inspect
                        .small_tick_hits
                        .map_or(inspect.attrs.n_sliders, |n| {
                            cmp::min(n, inspect.attrs.n_sliders)
                        });

                    let large_tick_hits = inspect
                        .large_tick_hits
                        .map_or(inspect.attrs.n_sliders + inspect.attrs.n_large_ticks, |n| {
                            cmp::min(n, inspect.attrs.n_sliders + inspect.attrs.n_large_ticks)
                        });

                    (0, large_tick_hits, small_tick_hits)
                }
            };

        let remain = total_hits - misses;

        let (n300, n100, n50) = match (inspect.n300, inspect.n100, inspect.n50) {
            // Three specified
            (Some(n300), Some(n100), Some(n50)) => match inspect.hitresult_priority {
                HitResultPriority::BestCase => {
                    let n300 = cmp::min(n300, remain);
                    let n100 = cmp::min(n100, remain - n300);
                    let n50 = cmp::min(n50, remain - n300 - n100);

                    (n300, n100, n50)
                }
                HitResultPriority::WorstCase => {
                    let n50 = cmp::min(n50, remain);
                    let n100 = cmp::min(n100, remain - n50);
                    let n300 = cmp::min(n300, remain - n50 - n100);

                    (n300, n100, n50)
                }
                HitResultPriority::Fastest => todo!(),
            },

            // Two specified
            (Some(n300), Some(n100), None) => {
                let (n300, n100) = match inspect.hitresult_priority {
                    HitResultPriority::BestCase => {
                        let n300 = cmp::min(n300, remain);
                        let n100 = cmp::min(n100, remain - n300);

                        (n300, n100)
                    }
                    HitResultPriority::WorstCase => {
                        let n100 = cmp::min(n100, remain);
                        let n300 = cmp::min(n300, remain - n100);

                        (n300, n100)
                    }
                    HitResultPriority::Fastest => todo!(),
                };

                (n300, n100, remain - n300 - n100)
            }
            (Some(n300), None, Some(n50)) => {
                let (n300, n50) = match inspect.hitresult_priority {
                    HitResultPriority::BestCase => {
                        let n300 = cmp::min(n300, remain);
                        let n50 = cmp::min(n50, remain - n300);

                        (n300, n50)
                    }
                    HitResultPriority::WorstCase => {
                        let n50 = cmp::min(n50, remain);
                        let n300 = cmp::min(n300, remain - n50);

                        (n300, n50)
                    }
                    HitResultPriority::Fastest => todo!(),
                };

                (n300, remain - n300 - n50, n50)
            }
            (None, Some(n100), Some(n50)) => {
                let (n100, n50) = match inspect.hitresult_priority {
                    HitResultPriority::BestCase => {
                        let n100 = cmp::min(n100, remain);
                        let n50 = cmp::min(n50, remain - n100);

                        (n100, n50)
                    }
                    HitResultPriority::WorstCase => {
                        let n50 = cmp::min(n50, remain);
                        let n100 = cmp::min(n100, remain - n50);

                        (n100, n50)
                    }
                    HitResultPriority::Fastest => todo!(),
                };

                (remain - n100 - n50, n100, n50)
            }

            // One specified
            (Some(n300), None, None) => {
                let n300 = cmp::min(n300, remain);

                match inspect.hitresult_priority {
                    HitResultPriority::BestCase => (n300, remain - n300, 0),
                    HitResultPriority::WorstCase => (n300, 0, remain - n300),
                    HitResultPriority::Fastest => todo!(),
                }
            }
            (None, Some(n100), None) => {
                let n100 = cmp::min(n100, remain);

                match inspect.hitresult_priority {
                    HitResultPriority::BestCase => (remain - n100, n100, 0),
                    HitResultPriority::WorstCase => (0, n100, remain - n100),
                    HitResultPriority::Fastest => todo!(),
                }
            }
            (None, None, Some(n50)) => {
                let n50 = cmp::min(n50, remain);

                match inspect.hitresult_priority {
                    HitResultPriority::BestCase => (remain - n50, 0, n50),
                    HitResultPriority::WorstCase => (0, remain - n50, n50),
                    HitResultPriority::Fastest => todo!(),
                }
            }

            // None specified
            (None, None, None) => match inspect.hitresult_priority {
                HitResultPriority::BestCase => (remain, 0, 0),
                HitResultPriority::WorstCase => (0, 0, remain),
                HitResultPriority::Fastest => todo!(),
            },
        };

        OsuHitResults {
            large_tick_hits,
            small_tick_hits,
            slider_end_hits,
            n300,
            n100,
            n50,
            misses,
        }
    }
}
