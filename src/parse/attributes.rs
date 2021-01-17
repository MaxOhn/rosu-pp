use crate::Mods;

/// Summary struct for a [`Beatmap`](crate::Beatmap)'s attributes.
#[derive(Clone, Debug)]
pub struct BeatmapAttributes {
    pub ar: f32,
    pub od: f32,
    pub cs: f32,
    pub hp: f32,
    pub clock_rate: f32,
}

impl BeatmapAttributes {
    const AR0_MS: f32 = 1800.0;
    const AR5_MS: f32 = 1200.0;
    const AR10_MS: f32 = 450.0;
    const AR_MS_STEP_1: f32 = (Self::AR0_MS - Self::AR5_MS) / 5.0;
    const AR_MS_STEP_2: f32 = (Self::AR5_MS - Self::AR10_MS) / 5.0;

    #[inline]
    pub(crate) fn new(ar: f32, od: f32, cs: f32, hp: f32) -> Self {
        Self {
            ar,
            od,
            cs,
            hp,
            clock_rate: 1.0,
        }
    }

    /// Adjusts attributes w.r.t. mods.
    /// AR is further adjusted by the its hitwindow.
    /// OD is __not__ adjusted by its hitwindow.
    pub fn mods(self, mods: impl Mods) -> Self {
        if !mods.change_map() {
            return self;
        }

        let clock_rate = mods.speed();
        let multiplier = mods.od_ar_hp_multiplier();

        // AR
        let mut ar = self.ar * multiplier;
        let mut ar_ms = if ar <= 5.0 {
            Self::AR0_MS - Self::AR_MS_STEP_1 * ar
        } else {
            Self::AR5_MS - Self::AR_MS_STEP_2 * (ar - 5.0)
        };

        ar_ms = ar_ms.max(Self::AR10_MS).min(Self::AR0_MS);
        ar_ms /= clock_rate;

        ar = if ar_ms > Self::AR5_MS {
            (Self::AR0_MS - ar_ms) / Self::AR_MS_STEP_1
        } else {
            5.0 + (Self::AR5_MS - ar_ms) / Self::AR_MS_STEP_2
        };

        // OD
        let od = (self.od * multiplier).min(10.0);

        // CS
        let mut cs = self.cs;
        if mods.hr() {
            cs *= 1.3;
        } else if mods.ez() {
            cs *= 0.5;
        }
        cs = cs.min(10.0);

        // HP
        let hp = (self.hp * multiplier).min(10.0);

        Self {
            ar,
            od,
            cs,
            hp,
            clock_rate,
        }
    }
}
