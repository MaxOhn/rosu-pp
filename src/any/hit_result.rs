use rosu_map::section::general::GameMode;

#[derive(Copy, Clone)]
#[allow(unused, reason = "keeping in-sync with lazer")]
pub enum HitResult {
    None,
    Miss,
    Meh,
    Ok,
    Good,
    Great,
    Perfect,
    SmallTickMiss,
    SmallTickHit,
    LargeTickMiss,
    LargeTickHit,
    SmallBonus,
    LargeBonus,
    IgnoreMiss,
    IgnoreHit,
    ComboBreak,
    SliderTailHit,
    LegacyComboIncrease,
}

impl HitResult {
    pub const fn base_score(self, mode: GameMode) -> i32 {
        match mode {
            GameMode::Osu => {
                match self {
                    Self::SmallTickHit => 10,
                    Self::LargeTickHit => 30,
                    Self::SliderTailHit => 150,
                    Self::Meh => 50,
                    Self::Ok => 100,
                    Self::Good => 200,
                    // * Perfect doesn't actually give more score / accuracy directly
                    Self::Great | Self::Perfect => 300,
                    Self::SmallBonus => 10,
                    Self::LargeBonus => 50,
                    _ => 0,
                }
            }
            GameMode::Taiko => unimplemented!(),
            GameMode::Catch => unimplemented!(),
            GameMode::Mania => unimplemented!(),
        }
    }
}
