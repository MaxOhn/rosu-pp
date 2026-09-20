use rosu_map::section::general::GameMode;

#[derive(Copy, Clone)]
pub enum HitResult {
    SmallBonus,
    LargeBonus,
    Other,
}

impl HitResult {
    pub const fn base_score(self, _mode: GameMode) -> i32 {
        match self {
            Self::SmallBonus => 10,
            Self::LargeBonus => 50,
            Self::Other => 0,
        }
    }
}
