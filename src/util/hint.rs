#[inline]
#[cold]
const fn cold() {}

/// Hints at the compiler that the condition is likely `true`.
#[inline]
pub const fn likely(b: bool) -> bool {
    if !b {
        cold();
    }

    b
}

/// Hints at the compiler that the condition is likely `false`.
#[inline]
pub const fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }

    b
}
