use rosu_pp::any::hitresult_generator::{Closest, Fast, HitResultGenerator, IgnoreAccuracy};

fn main() {
    divan::main();
}

#[divan::bench_group]
mod mania {
    use rosu_pp::mania::Mania;

    use super::*;

    macro_rules! benches {
        () => {
            benches!(@LEN short);
            benches!(@LEN long);
        };

        ( @LEN $len:ident ) => {
            mod $len {
                use super::*;

                benches!(@ACC $len, low_acc);
                benches!(@ACC $len, high_acc);

                #[divan::bench]
                fn ignore_acc() {
                    <IgnoreAccuracy as HitResultGenerator<Mania>>::generate_hitresults(
                        benches!(@INSPECT $len low_acc)
                    );
                }
            }
        };

        ( @ACC $len:ident, $acc:ident ) => {
            #[divan::bench_group]
            mod $acc {
                use super::*;

                #[divan::bench]
                fn fast() {
                    <Fast as HitResultGenerator<Mania>>::generate_hitresults(
                        benches!(@INSPECT $len $acc)
                    );
                }

                #[divan::bench]
                fn closest() {
                    <Closest as HitResultGenerator<Mania>>::generate_hitresults(
                        benches!(@INSPECT $len $acc)
                    );
                }
            }
        };

        ( @INSPECT short $acc:ident ) => {
            benches!(@INSPECT 200 50 $acc)
        };
        ( @INSPECT long $acc:ident ) => {
            benches!(@INSPECT 3000 500 $acc)
        };
        ( @INSPECT $n_objects:literal $n_hold_notes:literal low_acc ) => {
            benches!(@INSPECT $n_objects $n_hold_notes 0.42)
        };
        ( @INSPECT $n_objects:literal $n_hold_notes:literal high_acc ) => {
            benches!(@INSPECT $n_objects $n_hold_notes 0.98)
        };
        ( @INSPECT $n_objects:literal $n_hold_notes:literal $acc:literal ) => {
            rosu_pp::mania::InspectManiaPerformance {
                attrs: &rosu_pp::mania::ManiaDifficultyAttributes {
                    n_objects: $n_objects,
                    n_hold_notes: $n_hold_notes,
                    ..Default::default()
                },
                difficulty: &rosu_pp::Difficulty::new(),
                n320: Some($n_objects / 2),
                n300: None,
                n200: None,
                n100: None,
                n50: None,
                misses: Some(2),
                acc: Some($acc),
                hitresult_priority: rosu_pp::any::HitResultPriority::BestCase,
            }
        };
    }

    benches!();
}

#[divan::bench_group]
mod osu {
    use rosu_pp::osu::Osu;

    use super::*;

    macro_rules! benches {
        () => {
            benches!(@LEN short);
            benches!(@LEN long);
        };

        ( @LEN $len:ident ) => {
            mod $len {
                use super::*;

                benches!(@ACC $len, low_acc);
                benches!(@ACC $len, high_acc);

                #[divan::bench]
                fn ignore_acc() {
                    <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(
                        benches!(@INSPECT $len low_acc)
                    );
                }
            }
        };

        ( @ACC $len:ident, $acc:ident ) => {
            #[divan::bench_group]
            mod $acc {
                use super::*;

                #[divan::bench]
                fn fast() {
                    <Fast as HitResultGenerator<Osu>>::generate_hitresults(
                        benches!(@INSPECT $len $acc)
                    );
                }

                #[divan::bench]
                fn closest() {
                    <Closest as HitResultGenerator<Osu>>::generate_hitresults(
                        benches!(@INSPECT $len $acc)
                    );
                }
            }
        };

        ( @INSPECT short $acc:ident ) => {
            benches!(@INSPECT 200 50 $acc)
        };
        ( @INSPECT long $acc:ident ) => {
            benches!(@INSPECT 3000 500 $acc)
        };
        ( @INSPECT $n_circles:literal $n_sliders:literal low_acc ) => {
            benches!(@INSPECT $n_circles $n_sliders 0.42)
        };
        ( @INSPECT $n_circles:literal $n_sliders:literal high_acc ) => {
            benches!(@INSPECT $n_circles $n_sliders 0.98)
        };
        ( @INSPECT $n_circles:literal $n_sliders:literal $acc:literal ) => {
            rosu_pp::osu::InspectOsuPerformance {
               attrs: &rosu_pp::osu::OsuDifficultyAttributes {
                   n_circles: $n_circles,
                   n_sliders: $n_sliders,
                   ..Default::default()
               },
               difficulty: &rosu_pp::Difficulty::new(),
               acc: Some($acc),
               combo: None,
               large_tick_hits: None,
               small_tick_hits: None,
               slider_end_hits: None,
               n300: Some($n_circles / 2),
               n100: None,
               n50: None,
               misses: Some(2),
               hitresult_priority: rosu_pp::any::HitResultPriority::BestCase,
           }
        };
    }

    benches!();
}

#[divan::bench_group]
mod taiko {
    use rosu_pp::taiko::Taiko;

    use super::*;

    macro_rules! benches {
        () => {
            benches!(@LEN short);
            benches!(@LEN long);
        };

        ( @LEN $len:ident ) => {
            mod $len {
                use super::*;

                benches!(@ACC $len, low_acc);
                benches!(@ACC $len, high_acc);

                #[divan::bench]
                fn ignore_acc() {
                    <IgnoreAccuracy as HitResultGenerator<Taiko>>::generate_hitresults(
                        benches!(@INSPECT $len low_acc)
                    );
                }
            }
        };

        ( @ACC $len:ident, $acc:ident ) => {
            #[divan::bench_group]
            mod $acc {
                use super::*;

                #[divan::bench]
                fn fast() {
                    <Fast as HitResultGenerator<Taiko>>::generate_hitresults(
                        benches!(@INSPECT $len $acc)
                    );
                }

                #[divan::bench]
                fn closest() {
                    <Closest as HitResultGenerator<Taiko>>::generate_hitresults(
                        benches!(@INSPECT $len $acc)
                    );
                }
            }
        };

        ( @INSPECT short $acc:ident ) => {
            benches!(@INSPECT 200 $acc)
        };
        ( @INSPECT long $acc:ident ) => {
            benches!(@INSPECT 3000 $acc)
        };
        ( @INSPECT $max_combo:literal low_acc ) => {
            benches!(@INSPECT $max_combo 0.42)
        };
        ( @INSPECT $max_combo:literal high_acc ) => {
            benches!(@INSPECT $max_combo 0.98)
        };
        ( @INSPECT $max_combo:literal $acc:literal ) => {
            rosu_pp::taiko::InspectTaikoPerformance {
               attrs: &rosu_pp::taiko::TaikoDifficultyAttributes {
                   max_combo: $max_combo,
                   ..Default::default()
               },
               difficulty: &rosu_pp::Difficulty::new(),
               acc: Some($acc),
               combo: None,
               n300: Some($max_combo / 2),
               n100: None,
               misses: Some(2),
               hitresult_priority: rosu_pp::any::HitResultPriority::BestCase,
           }
        };
    }

    benches!();
}
