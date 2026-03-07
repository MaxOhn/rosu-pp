use std::marker::PhantomData;

use crate::any::InspectablePerformance;

/// Provides flexible hitresult generation based on mode and whether it should
/// focus on performance, accuracy, or other factors.
pub trait HitResultGenerator<M: InspectablePerformance> {
    fn generate_hitresults(inspect: M::InspectPerformance<'_>) -> M::HitResults;
}

/// [`HitResultGenerator`] whose result is generated as fast as possible.
///
/// This generator prioritizes performance over accuracy.
pub struct Fast;

/// [`HitResultGenerator`] whose result is the closest to the target accuracy.
///
/// Although the result is not guaranteed to be unique, i.e. there may be other
/// results with the same accuracy, [`Closest`] guarantees that there are no
/// other results that are *closer* to the target accuracy.
pub struct Closest;

/// [`HitResultGenerator`] that strives for a middleground between performance
/// and accuracy through a statistical approach.
///
/// Currently not implemented.
pub struct Statistical;

/// [`HitResultGenerator`] that ignores accuracy and generates solely based on
/// [`HitResultPriority`].
///
/// [`HitResultPriority`]: crate::any::HitResultPriority
pub struct IgnoreAccuracy;

/// [`HitResultGenerator`] consisting of a dedicated generator for each mode.
pub struct Composable<Osu, Taiko, Catch, Mania>(PhantomData<(Osu, Taiko, Catch, Mania)>);

macro_rules! impl_composable_generator {
    ( $module:ident :: $mode:ident ) => {
        impl<Osu, Taiko, Catch, Mania> HitResultGenerator<crate::$module::$mode>
            for Composable<Osu, Taiko, Catch, Mania>
        where
            $mode: HitResultGenerator<crate::$module::$mode>,
        {
            fn generate_hitresults(
                inspect: <crate::$module::$mode as crate::any::InspectablePerformance>::InspectPerformance<'_>,
            ) -> <crate::$module::$mode as crate::model::mode::IGameMode>::HitResults {
                $mode::generate_hitresults(inspect)
            }
        }
    };
}

impl_composable_generator!(osu::Osu);
impl_composable_generator!(taiko::Taiko);
impl_composable_generator!(catch::Catch);
impl_composable_generator!(mania::Mania);
