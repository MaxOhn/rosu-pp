use super::math_util;

use parse::Pos2;

const SLIDER_QUALITY: f32 = 50.0;

pub(crate) enum Points {
    Single(Pos2),
    Multi(Vec<Pos2>),
}

pub(crate) enum Curve {
    Linear {
        a: Pos2,
        b: Pos2,
    },
    Bezier(Points),
    Catmull(Points),
    Perfect {
        origin: Pos2,
        cx: f32,
        cy: f32,
        radius: f32,
    },
}

impl Curve {
    #[inline]
    pub(crate) fn linear(a: Pos2, b: Pos2) -> Self {
        Self::Linear { a, b }
    }

    pub(crate) fn bezier(points: &[Pos2]) -> Self {
        if points.len() == 1 {
            return Self::Bezier(Points::Single(points[0]));
        }

        let mut start = 0;
        let mut end = 0;
        let mut result = Vec::with_capacity(4);

        for i in 0..points.len() - 1 {
            if end - start > 1 && points[i] == points[end - 1] {
                Self::_bezier(&mut result, &points[start..end]);
                start = end;
            }

            end += 1;
        }

        Self::_bezier(&mut result, &points[start..end + 1]);

        Self::Bezier(Points::Multi(result))
    }

    fn _bezier(result: &mut Vec<Pos2>, points: &[Pos2]) {
        let step = 0.25 / SLIDER_QUALITY / points.len() as f32;
        let mut i = 0.0;
        let n = points.len() as i32 - 1;

        while i < 1.0 + step {
            let point = (0..=n).fold(Pos2 { x: 0.0, y: 0.0 }, |point, p| {
                let factor = math_util::cpn(p, n) * (1.0 - i).powi(n - p) * i.powi(p);

                point + points[p as usize] * factor
            });

            result.push(point);
            i += step;
        }
    }

    pub(crate) fn catmull(points: &[Pos2]) -> Self {
        if points.len() == 1 {
            return Self::Catmull(Points::Single(points[0]));
        }

        let order = points.len();
        let step = 2.5 / SLIDER_QUALITY;
        let target = step + 1.0;

        let mut resulting_points = Vec::with_capacity(4);

        for x in 0..order - 1 {
            let mut t = 0.0;

            while t < target {
                let v1 = if x >= 1 { points[x - 1] } else { points[x] };
                let v2 = points[x];

                let v3 = if x + 1 < order {
                    points[x + 1]
                } else {
                    v2.add_scaled(v2.add_scaled(v1, -1.0), 1.0)
                };

                let v4 = if x + 2 < order {
                    points[x + 2]
                } else {
                    v3.add_scaled(v3.add_scaled(v2, -1.0), 1.0)
                };

                let point = Self::catmull_point(v1, v2, v3, v4, t);
                resulting_points.push(point);
                t += step;
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

    pub(crate) fn perfect(points: &[Pos2]) -> Self {
        let (cx, cy, mut radius) = math_util::get_circum_circle(&points);
        radius *= ((!math_util::is_left(&points)) as i8 * 2 - 1) as f32;

        Self::Perfect {
            origin: points[0],
            cx,
            cy,
            radius,
        }
    }

    pub(crate) fn point_at_distance(&self, len: f32) -> Pos2 {
        let points = match self {
            Self::Bezier(points) => points,
            Self::Catmull(points) => points,
            Self::Linear { a, b } => return math_util::point_on_line(*a, *b, len),
            Self::Perfect {
                origin,
                cx,
                cy,
                radius,
            } => return math_util::rotate(*cx, *cy, *origin, len / *radius),
        };

        match points {
            Points::Single(point) => *point,
            Points::Multi(points) => math_util::point_at_distance(points, len),
        }
    }
}
