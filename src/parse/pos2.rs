use std::fmt;
use std::ops;

/// Simple (x, y) coordinate / vector
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Pos2 {
    /// Position on the x-axis.
    pub x: f32,
    /// Position on the y-axis.
    pub y: f32,
}

impl Pos2 {
    /// Return the null vector.
    #[inline]
    pub fn zero() -> Self {
        Self::default()
    }

    /// Return a position with both coordinates on the given value.
    #[inline]
    pub fn new(value: f32) -> Self {
        Self { x: value, y: value }
    }

    /// Return the position's length squared.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.dot(*self)
    }

    /// Return the position's length.
    #[inline]
    pub fn length(&self) -> f32 {
        ((self.x * self.x + self.y * self.y) as f64).sqrt() as f32
    }

    /// Return the dot product.
    #[inline]
    pub fn dot(&self, other: Self) -> f32 {
        (self.x * other.x) + (self.y * other.y)
    }

    /// Return the distance to another position.
    #[inline]
    pub fn distance(&self, other: Self) -> f32 {
        (*self - other).length()
    }

    /// Normalize the coordinates with respect to the vector's length.
    #[inline]
    pub fn normalize(mut self) -> Pos2 {
        let scale = self.length().recip();
        self.x *= scale;
        self.y *= scale;

        self
    }
}

impl ops::Add<Pos2> for Pos2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub<Pos2> for Pos2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Mul<f32> for Pos2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Div<f32> for Pos2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl ops::AddAssign for Pos2 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl fmt::Display for Pos2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Pos2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
