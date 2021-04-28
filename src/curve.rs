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

pub(crate) enum Curve<'p> {
    Linear(&'p [Pos2]),
    Bezier(Points),
    Catmull(Points),
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
                let factor = math_util::cpn(p, n) * (1.0 - i).powi(n - p) * i.powi(p);

                point + *curr * factor
            });

            result.push(point);
            i += step;
        }
    }

    fn catmull(points: &[Pos2]) -> Self {
        if points.len() == 1 {
            return Self::Catmull(Points::Single(points[0]));
        }

        let order = points.len();

        let mut resulting_points =
            Vec::with_capacity(((order - 1) as f32 * CATMULL_DETAIL * 2.0) as usize);

        for i in 0..order - 1 {
            let v1 = points[i.saturating_sub(1)];
            let v2 = points[i];

            let v3 = if i < order - 1 {
                points[i + 1]
            } else {
                v2 * 2.0 - v1
            };

            let v4 = if i < order - 2 {
                points[i + 2]
            } else {
                v3 * 2.0 - v2
            };

            let mut c = 0.0;

            while c < CATMULL_DETAIL {
                resulting_points.push(Self::catmull_point(v1, v2, v3, v4, c / CATMULL_DETAIL));
                resulting_points.push(Self::catmull_point(
                    v1,
                    v2,
                    v3,
                    v4,
                    (c + 1.0) / CATMULL_DETAIL,
                ));

                c += 1.0;
            }
        }

        Self::Catmull(Points::Multi(resulting_points))
    }

    #[inline]
    fn catmull_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, len: f32) -> Pos2 {
        Pos2 {
            x: math_util::catmull(p0.x, p1.x, p2.x, p3.x, len),
            y: math_util::catmull(p0.y, p1.y, p2.y, p3.y, len),
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
        let points = match self {
            Self::Bezier(points) => points,
            Self::Catmull(points) => points,
            Self::Linear(points) => return math_util::point_on_lines(points, dist),
            Self::Perfect {
                origin,
                center,
                radius,
            } => return math_util::rotate(*center, *origin, dist / *radius),
        };

        match points {
            Points::Single(point) => *point,
            Points::Multi(points) => math_util::point_at_distance(points, dist),
        }
    }
}
