use crate::{
    AnyPP, AnyStars, Beatmap, CatchPP, CatchStars, GameMode, GradualDifficultyAttributes,
    GradualPerformanceAttributes, ManiaPP, ManiaStars, OsuPP, OsuStars, PerformanceAttributes,
    Strains, TaikoPP, TaikoStars,
};

/// Provides some additional methods on [`Beatmap`].
pub trait BeatmapExt {
    /// Calculate the stars and other attributes of a beatmap which are required for pp calculation.
    fn stars(&self) -> AnyStars<'_>;

    /// Calculate the max pp of a beatmap.
    ///
    /// If you seek more fine-tuning you can use the [`pp`](BeatmapExt::pp) method.
    fn max_pp(&self, mods: u32) -> PerformanceAttributes;

    /// Returns a builder for performance calculation.
    ///
    /// Convenient method that matches on the map's mode to choose the appropriate calculator.
    fn pp(&self) -> AnyPP<'_>;

    /// Calculate the strains of a map.
    /// This essentially performs the same calculation as [`BeatmapExt::stars`] but
    /// instead of evaluating the final strains, they are just returned as is.
    ///
    /// Suitable to plot the difficulty of a map over time.
    fn strains(&self, mods: u32) -> Strains;

    /// Return an iterator that gives you the [`DifficultyAttributes`] after each hit object.
    ///
    /// Suitable to efficiently get the map's star rating after multiple different locations.
    fn gradual_difficulty(&self, mods: u32) -> GradualDifficultyAttributes<'_>;

    /// Return a struct that gives you the [`PerformanceAttributes`] after every (few) hit object(s).
    ///
    /// Suitable to efficiently get a score's performance after multiple different locations,
    /// i.e. live update a score's pp.
    fn gradual_performance(&self, mods: u32) -> GradualPerformanceAttributes<'_>;
}

impl BeatmapExt for Beatmap {
    #[inline]
    fn stars(&self) -> AnyStars<'_> {
        match self.mode {
            GameMode::Osu => AnyStars::Osu(OsuStars::new(self)),
            GameMode::Taiko => AnyStars::Taiko(TaikoStars::new(self)),
            GameMode::Catch => AnyStars::Catch(CatchStars::new(self)),
            GameMode::Mania => AnyStars::Mania(ManiaStars::new(self)),
        }
    }

    #[inline]
    fn max_pp(&self, mods: u32) -> PerformanceAttributes {
        match self.mode {
            GameMode::Osu => PerformanceAttributes::Osu(OsuPP::new(self).mods(mods).calculate()),
            GameMode::Taiko => {
                PerformanceAttributes::Taiko(TaikoPP::new(self).mods(mods).calculate())
            }
            GameMode::Catch => {
                PerformanceAttributes::Catch(CatchPP::new(self).mods(mods).calculate())
            }
            GameMode::Mania => {
                PerformanceAttributes::Mania(ManiaPP::new(self).mods(mods).calculate())
            }
        }
    }

    #[inline]
    fn pp(&self) -> AnyPP<'_> {
        AnyPP::new(self)
    }

    #[inline]
    fn strains(&self, mods: u32) -> Strains {
        match self.mode {
            GameMode::Osu => Strains::Osu(OsuStars::new(self).mods(mods).strains()),
            GameMode::Taiko => Strains::Taiko(TaikoStars::new(self).mods(mods).strains()),
            GameMode::Catch => Strains::Catch(CatchStars::new(self).mods(mods).strains()),
            GameMode::Mania => Strains::Mania(ManiaStars::new(self).mods(mods).strains()),
        }
    }

    #[inline]
    fn gradual_difficulty(&self, mods: u32) -> GradualDifficultyAttributes<'_> {
        GradualDifficultyAttributes::new(self, mods)
    }

    #[inline]
    fn gradual_performance(&self, mods: u32) -> GradualPerformanceAttributes<'_> {
        GradualPerformanceAttributes::new(self, mods)
    }
}
