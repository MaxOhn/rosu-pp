/// The mode of a beatmap.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum GameMode {
    /// osu!standard
    STD = 0,
    /// osu!taiko
    TKO = 1,
    /// osu!catch
    CTB = 2,
    /// osu!mania
    MNA = 3,
}

impl Default for GameMode {
    #[inline]
    fn default() -> Self {
        Self::STD
    }
}
