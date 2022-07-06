/// The mode of a beatmap.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameMode {
    /// osu!standard
    Osu = 0,
    /// osu!taiko
    Taiko = 1,
    /// osu!catch
    Catch = 2,
    /// osu!mania
    Mania = 3,
}

impl Default for GameMode {
    #[inline]
    fn default() -> Self {
        Self::Osu
    }
}
