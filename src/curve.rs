#![cfg(any(
    feature = "fruits",
    all(feature = "osu", not(feature = "no_sliders_no_leniency"))
))]

use crate::{
    math_util,
    parse::{PathType, Pos2},
};

const BEZIER_TOLERANCE: f32 = 0.25;
const CATMULL_DETAIL: f32 = 50.0;

pub(crate) enum Points {
    Single(Pos2),
    Multi(Vec<Pos2>),
}

impl Points {
    #[inline]
    fn point_at_distance(&self, dist: f32) -> Pos2 {
        match self {
            Points::Multi(points) => math_util::point_at_distance(points, dist),
            Points::Single(point) => *point,
        }
    }
}

pub(crate) enum Curve<'p> {
    Bezier(Points),
    Catmull(Points),
    Linear(&'p [Pos2]),
    Perfect {
        origin: Pos2,
        center: Pos2,
        radius: f32,
    },
}

impl<'p> Curve<'p> {
    #[inline]
    pub(crate) fn new(points: &'p [Pos2], kind: PathType) -> Self {
        match kind {
            PathType::Bezier => Self::bezier(points),
            PathType::Catmull => Self::catmull(points),
            PathType::Linear => Self::Linear(points),
            PathType::PerfectCurve => Self::perfect(points),
        }
    }

    fn bezier(points: &[Pos2]) -> Self {
        if points.len() == 1 {
            return Self::Bezier(Points::Single(points[0]));
        }

        let mut start = 0;
        let mut result = Vec::new();

        for (end, (curr, next)) in (1..).zip(points.iter().zip(points.iter().skip(1))) {
            if end - start > 1 && curr == next {
                Self::_bezier(&mut result, &points[start..end]);
                start = end;
            }
        }

        Self::_bezier(&mut result, &points[start..]);

        Self::Bezier(Points::Multi(result))
    }

    fn _bezier(result: &mut Vec<Pos2>, points: &[Pos2]) {
        let step = (BEZIER_TOLERANCE / points.len() as f32).max(0.01);
        let mut i = 0.0;
        let n = points.len() as i32 - 1;

        while i < 1.0 + step {
            let point = (0..).zip(points).fold(Pos2::zero(), |point, (p, curr)| {
                point + *curr * math_util::cpn(p, n) * (1.0 - i).powi(n - p) * i.powi(p)
            });

            result.push(point);
            i += step;
        }
    }

    fn catmull(points: &[Pos2]) -> Self {
        let len = points.len();

        if len == 1 {
            return Self::Catmull(Points::Single(points[0]));
        }

        let mut result = Vec::with_capacity((len as f32 * CATMULL_DETAIL * 2.0) as usize);

        // Handle first iteration distinctly because of v1
        let v1 = points[0];
        let v2 = points[0];
        let v3 = points.get(1).copied().unwrap_or(v2);
        let v4 = points.get(2).copied().unwrap_or_else(|| v3 * 2.0 - v2);

        Self::catmull_points(&mut result, v1, v2, v3, v4);

        // Remaining iterations
        for (i, (&v1, &v2)) in (2..).zip(points.iter().zip(points.iter().skip(1))) {
            let v3 = points.get(i).copied().unwrap_or_else(|| v2 * 2.0 - v1);
            let v4 = points.get(i + 1).copied().unwrap_or_else(|| v3 * 2.0 - v2);

            Self::catmull_points(&mut result, v1, v2, v3, v4);
        }

        Self::Catmull(Points::Multi(result))
    }

    #[inline]
    fn catmull_points(result: &mut Vec<Pos2>, v1: Pos2, v2: Pos2, v3: Pos2, v4: Pos2) {
        let mut c = 0.0;

        let x1 = 2.0 * v1.x;
        let x2 = -v1.x + v3.x;
        let x3 = 2.0 * v1.x - 5.0 * v2.x + 4.0 * v3.x - v4.x;
        let x4 = -v1.x + 3.0 * (v2.x - v3.x) + v4.x;

        let y1 = 2.0 * v1.y;
        let y2 = -v1.y + v3.y;
        let y3 = 2.0 * v1.y - 5.0 * v2.y + 4.0 * v3.y - v4.y;
        let y4 = -v1.y + 3.0 * (v2.y - v3.y) + v4.y;

        loop {
            let t1 = c / CATMULL_DETAIL;
            let t2 = t1 * t1;
            let t3 = t2 * t1;

            result.push(Pos2 {
                x: 0.5 * (x1 + x2 * t1 + x3 * t2 + x4 * t3),
                y: 0.5 * (y1 + y2 * t1 + y3 * t2 + y4 * t3),
            });

            let t1 = (c + 1.0) / CATMULL_DETAIL;
            let t2 = t1 * t1;
            let t3 = t2 * t1;

            result.push(Pos2 {
                x: 0.5 * (x1 + x2 * t1 + x3 * t2 + x4 * t3),
                y: 0.5 * (y1 + y2 * t1 + y3 * t2 + y4 * t3),
            });

            c += 1.0;

            if c >= CATMULL_DETAIL {
                return;
            }
        }
    }

    fn perfect(points: &[Pos2]) -> Self {
        let (a, b, c) = (points[0], points[1], points[2]);
        let (center, mut radius) = math_util::get_circum_circle(a, b, c);
        radius *= ((!math_util::is_left(a, b, c)) as i8 * 2 - 1) as f32;

        Self::Perfect {
            origin: a,
            center,
            radius,
        }
    }

    pub(crate) fn point_at_distance(&self, dist: f32) -> Pos2 {
        match self {
            Self::Bezier(points) => points.point_at_distance(dist),
            Self::Catmull(points) => points.point_at_distance(dist),
            Self::Linear(points) => math_util::point_at_distance(points, dist),
            Self::Perfect {
                origin,
                center,
                radius,
            } => math_util::rotate(*center, *origin, dist / *radius),
        }
    }
}
