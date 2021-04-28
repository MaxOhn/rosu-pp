#![cfg(any(
    feature = "fruits",
    all(feature = "osu", not(feature = "no_sliders_no_leniency"))
))]

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

#[inline]
pub(crate) fn catmull(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5 * (2.0 * p1
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
}

#[inline]
pub(crate) fn point_on_line(p1: Pos2, p2: Pos2, len: f32) -> Pos2 {
    let mut dist = p2.distance(p1);
    let remaining_dist = dist - len;
    dist += (dist.abs() <= f32::EPSILON) as u8 as f32;

    p2 + (p1 - p2) * (remaining_dist / dist)
}

#[inline]
pub(crate) fn point_on_lines(points: &[Pos2], len: f32) -> Pos2 {
    let mut dist = 0.0;

    for (curr, &next) in points.iter().zip(points.iter().skip(1)) {
        let curr_dist = curr.distance(next);

        if dist + curr_dist >= len {
            return point_on_line(*curr, next, len - dist);
        }

        dist += curr_dist;
    }

    point_on_line(points[points.len() - 2], points[points.len() - 1], len)
}

pub(crate) fn point_at_distance(points: &[Pos2], dist: f32) -> Pos2 {
    if points.len() < 2 {
        return Pos2::zero();
    } else if dist.abs() <= f32::EPSILON {
        return points[0];
    }

    let mut curr_dist = 0.0;
    let mut new_dist;

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

    points[points.len() - 1]
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
pub(crate) fn rotate(center: Pos2, origin: Pos2, theta: f32) -> Pos2 {
    let (sin, cos) = theta.sin_cos();
    let diff = origin - center;

    let offset = Pos2 {
        x: cos * diff.x - sin * diff.y,
        y: sin * diff.x + cos * diff.y,
    };

    center + offset
}
