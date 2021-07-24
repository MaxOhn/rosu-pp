#[cfg(any(
    feature = "fruits",
    all(feature = "osu", not(feature = "no_sliders_no_leniency"))
))]
pub(crate) use fruits_osu::*;

#[cfg(any(
    feature = "fruits",
    all(feature = "osu", not(feature = "no_sliders_no_leniency"))
))]
mod fruits_osu {
    use crate::parse::Pos2;

    #[inline]
    pub(crate) fn cpn(mut p: i32, n: i32) -> f32 {
        if p < 0 || p > n {
            return 0.0;
        }

        p = p.min(n - p);
        let diff = n - p;
        let mut out = 1.0;

        for i in 1..=p {
            out *= (diff + i) as f32 / i as f32;
        }

        out
    }

    pub(crate) fn point_at_distance(points: &[Pos2], dist: f32) -> Pos2 {
        if points.len() < 2 {
            return Pos2::zero();
        } else if dist.abs() <= f32::EPSILON {
            return points[0];
        }

        let mut curr_dist = 0.0;

        // If points.len() < 2 it wont be reassigned and would cause division by zero.
        // Before that division happens though, unwrapping the last two elements
        // would already have panicked so this is fine to keep at zero.
        let mut new_dist = 0.0;

        for (&curr, &next) in points.iter().zip(points.iter().skip(1)) {
            new_dist = (curr - next).length();
            curr_dist += new_dist;

            if dist <= curr_dist {
                let remaining_dist = dist - (curr_dist - new_dist);

                return if remaining_dist.abs() <= f32::EPSILON {
                    curr
                } else {
                    curr + (next - curr) * (remaining_dist / new_dist)
                };
            }
        }

        let remaining_dist = dist - (curr_dist - new_dist);
        let pre_last = points[points.len() - 2];
        let last = points[points.len() - 1];

        pre_last + (last - pre_last) * (remaining_dist / new_dist)
    }

    pub(crate) fn get_circum_circle(p0: Pos2, p1: Pos2, p2: Pos2) -> (Pos2, f32) {
        let a = 2.0 * (p0.x * (p1.y - p2.y) - p0.y * (p1.x - p2.x) + p1.x * p2.y - p2.x * p1.y);

        let q0 = p0.length_squared();
        let q1 = p1.length_squared();
        let q2 = p2.length_squared();

        let cx = (q0 * (p1.y - p2.y) + q1 * (p2.y - p0.y) + q2 * (p0.y - p1.y)) / a;
        let cy = (q0 * (p2.x - p1.x) + q1 * (p0.x - p2.x) + q2 * (p1.x - p0.x)) / a;

        let r = (cx - p0.x).hypot(cy - p0.y);

        (Pos2 { x: cx, y: cy }, r)
    }

    #[inline]
    pub(crate) fn is_left(p0: Pos2, p1: Pos2, p2: Pos2) -> bool {
        ((p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x)) < 0.0
    }

    #[inline]
    pub(crate) fn is_linear(p0: Pos2, p1: Pos2, p2: Pos2) -> bool {
        ((p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x)).abs() <= f32::EPSILON
    }

    #[inline]
    pub(crate) fn valid_linear(points: &[Pos2]) -> bool {
        for (curr, next) in points.iter().skip(1).zip(points.iter().skip(2)).step_by(2) {
            if curr != next {
                return false;
            }
        }

        true
    }

    #[inline]
    pub(crate) fn rotate(center: Pos2, origin: Pos2, theta: f32) -> Pos2 {
        let (sin, cos) = theta.sin_cos();
        let diff = origin - center;

        let offset = Pos2 {
            x: cos * diff.x - sin * diff.y,
            y: sin * diff.x + cos * diff.y,
        };

        center + offset
    }
}

#[cfg(feature = "osu")]
#[inline]
pub(crate) fn lerp(start: f32, end: f32, percent: f32) -> f32 {
    start + (end - start) * percent
}
