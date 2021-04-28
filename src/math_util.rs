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
    let mut out = 1.0;

    for i in 1..=p {
        out *= (n - p + i) as f32 / i as f32;
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
    let mut full_len = ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt();
    let n = full_len - len;

    if full_len.abs() < f32::EPSILON {
        full_len = 1.0;
    }

    (p1 * n + p2 * len) / full_len
}

#[inline]
pub(crate) fn point_on_lines(points: &[Pos2], len: f32) -> Pos2 {
    let mut dist = 0.0;

    for (curr, next) in points.iter().zip(points.iter().skip(1)) {
        let curr_dist = curr.distance(next);

        if dist + curr_dist >= len {
            return point_on_line(*curr, *next, len - dist);
        }

        dist += curr_dist;
    }

    point_on_line(points[points.len() - 2], points[points.len() - 1], len)
}

#[inline]
pub(crate) fn distance_from_points(arr: &[Pos2]) -> f32 {
    arr.iter()
        .skip(1)
        .zip(arr.iter())
        .map(|(curr, prev)| curr.distance(prev))
        .sum()
}

pub(crate) fn point_at_distance(array: &[Pos2], distance: f32) -> Pos2 {
    if array.len() < 2 {
        return Pos2 { x: 0.0, y: 0.0 };
    } else if distance.abs() < f32::EPSILON {
        return array[0];
    } else if distance_from_points(array) <= distance {
        return array[array.len() - 1];
    }

    let mut current_distance = 0.0;
    let mut new_distance;

    for (&curr, &next) in array.iter().zip(array.iter().skip(1)) {
        new_distance = (curr - next).length();
        current_distance += new_distance;

        if distance <= current_distance {
            let remaining_dist = distance - (current_distance - new_distance);

            return if remaining_dist.abs() <= f32::EPSILON {
                curr
            } else {
                curr + (next - curr) * (remaining_dist / new_distance)
            };
        }
    }

    array[array.len() - 1]
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
