use std::marker::PhantomData;

/// TODO: docs
pub trait HitResultGenerator<P: HitResultParams> {
    fn generate_hitresults(params: &P) -> P::HitResults;
}

/// TODO: docs
pub trait HitResultParams {
    type HitResults;
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

/// TODO: docs
pub struct Statistical;

/// TODO: docs
pub struct Composable<Osu, Taiko, Catch, Mania>(PhantomData<(Osu, Taiko, Catch, Mania)>);

macro_rules! impl_composable_generator {
    ( $mode:ident: $params:path ) => {
        impl<Osu, Taiko, Catch, Mania> HitResultGenerator<$params>
            for Composable<Osu, Taiko, Catch, Mania>
        where
            $mode: HitResultGenerator<$params>,
        {
            fn generate_hitresults(params: &$params) -> <$params as HitResultParams>::HitResults {
                $mode::generate_hitresults(params)
            }
        }
    };
}

impl_composable_generator!(Osu: crate::osu::OsuHitResultParams);
impl_composable_generator!(Taiko: crate::taiko::TaikoHitResultParams);
// TODO: uncomment
// impl_composable_generator!(Catch: crate::catch::CatchHitResultParams);
impl_composable_generator!(Mania: crate::mania::ManiaHitResultParams);
