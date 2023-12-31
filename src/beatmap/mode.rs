/// The mode of a beatmap.
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub enum GameMode {
    /// osu!standard
    #[default]
    Osu = 0,
    /// osu!taiko
    Taiko = 1,
    /// osu!catch
    Catch = 2,
    /// osu!mania
    Mania = 3,
}

impl From<u8> for GameMode {
    #[inline]
    fn from(mode: u8) -> Self {
        // `0` will happen most commonly so it should be the first branch
        #[allow(clippy::match_same_arms)]
        match mode {
            0 => Self::Osu,
            1 => Self::Taiko,
            2 => Self::Catch,
            3 => Self::Mania,
            _ => Self::Osu,
        }
    }
}
