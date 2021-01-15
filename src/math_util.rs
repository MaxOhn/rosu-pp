use crate::Pos2;

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
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t.powi(2)
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3 * t.powi(3)))
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
pub(crate) fn angle_from_points(p0: Pos2, p1: Pos2) -> f32 {
    (p1.y - p0.y).atan2(p1.x - p0.x)
}

#[inline]
pub(crate) fn distance_from_points(arr: &[Pos2]) -> f32 {
    arr.iter()
        .skip(1)
        .zip(arr.iter())
        .map(|(curr, prev)| curr.distance(prev))
        .sum()
}

#[inline]
pub(crate) fn cart_from_pol(r: f32, t: f32) -> Pos2 {
    Pos2 {
        x: r * t.cos(),
        y: r * t.sin(),
    }
}

pub(crate) fn point_at_distance(array: &[Pos2], distance: f32) -> Pos2 {
    if array.len() < 2 {
        return Pos2 { x: 0.0, y: 0.0 };
    } else if distance.abs() < f32::EPSILON {
        return array[0];
    } else if distance_from_points(array) <= distance {
        return array[array.len() - 1];
    }

    let mut i = 0;
    let mut current_distance = 0.0;
    let mut new_distance = 0.0;

    while i < array.len() - 2 {
        new_distance = (array[i] - array[i + 1]).length();
        current_distance += new_distance;

        if distance <= current_distance {
            break;
        }

        i += 1;
    }

    current_distance -= new_distance;

    if (distance - current_distance).abs() <= f32::EPSILON {
        array[i]
    } else {
        let angle = angle_from_points(array[i], array[i + 1]);
        let cart = cart_from_pol(distance - current_distance, angle);

        array[i] + cart * ((array[i].x <= array[i + 1].x) as i8 * 2 - 1) as f32
    }
}

pub(crate) fn get_circum_circle(p: &[Pos2]) -> (f32, f32, f32) {
    let d = 2.0
        * (p[0].x * (p[1].y - p[2].y) + p[1].x * (p[2].y - p[0].y) + p[2].x * (p[0].y - p[1].y));

    let p0 = p[0].x * p[0].x + p[0].y * p[0].y;
    let p1 = p[1].x * p[1].x + p[1].y * p[1].y;
    let p2 = p[2].x * p[2].x + p[2].y * p[2].y;

    let ux = (p0 * (p[1].y - p[2].y) + p1 * (p[2].y - p[0].y) + p2 * (p[0].y - p[1].y)) / d;
    let uy = (p0 * (p[2].x - p[1].x) + p1 * (p[0].x - p[2].x) + p2 * (p[1].x - p[0].x)) / d;

    let px = ux - p[0].x;
    let py = uy - p[0].y;
    let r = (px * px + py * py).sqrt();

    (ux, uy, r)
}

#[inline]
pub(crate) fn is_left(p: &[Pos2]) -> bool {
    ((p[1].x - p[0].x) * (p[2].y - p[0].y) - (p[1].y - p[0].y) * (p[2].x - p[0].x)) < 0.0
}

#[inline]
pub(crate) fn rotate(cx: f32, cy: f32, p: Pos2, radians: f32) -> Pos2 {
    let cos = radians.cos();
    let sin = radians.sin();

    Pos2 {
        x: (cos * (p.x - cx)) - (sin * (p.y - cy)) + cx,
        y: (sin * (p.x - cx)) + (cos * (p.y - cy)) + cy,
    }
}
