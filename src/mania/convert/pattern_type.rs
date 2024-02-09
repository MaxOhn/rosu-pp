use std::{
    fmt,
    ops::{BitAndAssign, BitOr, BitOrAssign, Not},
};

#[derive(Copy, Clone, Default)]
pub struct PatternType(u16);

#[rustfmt::skip]
impl PatternType {
    pub const FORCE_STACK: Self      = Self(1 << 0);
    pub const FORCE_NOT_STACK: Self  = Self(1 << 1);
    pub const KEEP_SINGLE: Self      = Self(1 << 2);
    pub const LOW_PROBABILITY: Self  = Self(1 << 3);
    // pub const ALTERNATE: Self        = Self(1 << 4);
    // pub const FORCE_SIG_SLIDER: Self = Self(1 << 5);
    // pub const FORCE_NOT_SLIDER: Self = Self(1 << 6);
    pub const GATHERED: Self         = Self(1 << 7);
    pub const MIRROR: Self           = Self(1 << 8);
    pub const REVERSE: Self          = Self(1 << 9);
    pub const CYCLE: Self            = Self(1 << 10);
    pub const STAIR: Self            = Self(1 << 11);
    pub const REVERSE_STAIR: Self    = Self(1 << 12);
}

impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut written = false;

        macro_rules! write_pattern {
            ($self:ident, $f:ident, $written:ident: $($pat:ident,)*) => {
                $(
                    if $self.contains(Self::$pat) {
                        if $written {
                            $f.write_str(", ")?;
                        } else {
                            $written = true;
                        }

                        $f.write_str(stringify!($pat))?;
                    }
                )*
            }
        }

        write_pattern! {
            self, f, written:
            FORCE_STACK,
            FORCE_NOT_STACK,
            KEEP_SINGLE,
            LOW_PROBABILITY,
            GATHERED,
            MIRROR,
            REVERSE,
            CYCLE,
            STAIR,
            REVERSE_STAIR,
        }

        if !written {
            f.write_str("NONE")?;
        }

        Ok(())
    }
}

impl PatternType {
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for PatternType {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PatternType {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for PatternType {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for PatternType {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
