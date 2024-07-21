use crate::{catch::CatchStrains, mania::ManiaStrains, osu::OsuStrains, taiko::TaikoStrains};

/// The result of calculating the strains on a map.
///
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug, PartialEq)]
pub enum Strains {
    Osu(OsuStrains),
    Taiko(TaikoStrains),
    Catch(CatchStrains),
    Mania(ManiaStrains),
}

impl Strains {
    /// Time inbetween two strains in ms.
    pub const fn section_len(&self) -> f64 {
        match self {
            Strains::Osu(_) => OsuStrains::SECTION_LEN,
            Strains::Taiko(_) => TaikoStrains::SECTION_LEN,
            Strains::Catch(_) => CatchStrains::SECTION_LEN,
            Strains::Mania(_) => ManiaStrains::SECTION_LEN,
        }
    }
}

macro_rules! from_mode_strains {
    ( $mode:ident: $strains:ident ) => {
        impl From<$strains> for Strains {
            fn from(strains: $strains) -> Self {
                Self::$mode(strains)
            }
        }
    };
}

from_mode_strains!(Osu: OsuStrains);
from_mode_strains!(Taiko: TaikoStrains);
from_mode_strains!(Catch: CatchStrains);
from_mode_strains!(Mania: ManiaStrains);
