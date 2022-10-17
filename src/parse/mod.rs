mod error;
mod hitobject;
mod hitsound;
mod pos2;
mod reader;
mod sort;

pub use error::{ParseError, ParseResult};
pub use hitobject::{HitObject, HitObjectKind};
pub use hitsound::HitSound;
pub use pos2::Pos2;
pub use slider_parsing::*;

use reader::FileReader;
pub(crate) use sort::legacy_sort;

use std::{cmp::Ordering, ops::Neg};

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
use std::{fs::File, io::Read};

#[cfg(feature = "async_tokio")]
use tokio::{fs::File, io::AsyncRead};

#[cfg(not(feature = "async_std"))]
use std::path::Path;

#[cfg(feature = "async_std")]
use async_std::{fs::File, io::Read as AsyncRead, path::Path};

use crate::beatmap::{Beatmap, Break, DifficultyPoint, EffectPoint, GameMode, TimingPoint};

fn sort_unstable<T: PartialOrd>(slice: &mut [T]) {
    slice.sort_unstable_by(|p1, p2| p1.partial_cmp(p2).unwrap_or(Ordering::Equal));
}

trait OptionExt<T> {
    fn next_field(self, field: &'static str) -> Result<T, ParseError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn next_field(self, field: &'static str) -> Result<T, ParseError> {
        self.ok_or(ParseError::MissingField(field))
    }
}

trait InRange: Sized + Copy + Neg<Output = Self> + PartialOrd {
    const LIMIT: Self;

    fn is_in_range(&self) -> bool {
        (-Self::LIMIT..=Self::LIMIT).contains(self)
    }

    fn is_in_custom_range(&self, limit: Self) -> bool {
        (-limit..=limit).contains(self)
    }
}

impl InRange for i32 {
    const LIMIT: Self = i32::MAX;
}

impl InRange for f32 {
    const LIMIT: Self = i32::MAX as f32;
}

impl InRange for f64 {
    const LIMIT: Self = i32::MAX as f64;
}

const MAX_COORDINATE_VALUE: i32 = 131_072;
const KIAI_FLAG: i32 = 1 << 0;

macro_rules! section {
    ($map:ident, $func:ident, $reader:ident, $section:ident) => {{
        #[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
        if $map.$func(&mut $reader, &mut $section)? {
            break;
        }

        #[cfg(any(feature = "async_std", feature = "async_tokio"))]
        if $map.$func(&mut $reader, &mut $section).await? {
            break;
        }
    }};
}

macro_rules! next_line {
    ($reader:ident) => {{
        #[cfg(any(feature = "async_std", feature = "async_tokio"))]
        {
            $reader.next_line().await
        }

        #[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
        {
            $reader.next_line()
        }
    }};
}

macro_rules! parse_general_body {
    ($self:ident, $reader:ident, $section:ident) => {{
        let mut mode = None;
        let mut empty = true;
        let mut stack_leniency = None;

        while next_line!($reader)? != 0 {
            if let Some(bytes) = $reader.get_section() {
                *$section = Section::from_bytes(bytes);
                empty = false;
                break;
            }

            let (key, value) = $reader.split_colon().ok_or(ParseError::BadLine)?;

            if key == b"Mode" {
                mode = match value {
                    "0" => Some(GameMode::Osu),
                    "1" => Some(GameMode::Taiko),
                    "2" => Some(GameMode::Catch),
                    "3" => Some(GameMode::Mania),
                    _ => return Err(ParseError::InvalidMode),
                };
            }

            if key == b"StackLeniency" {
                if let Some(val) = value.parse().ok().filter(f32::is_in_range) {
                    stack_leniency = Some(val);
                }
            }
        }

        $self.mode = mode.unwrap_or(GameMode::Osu);
        $self.stack_leniency = stack_leniency.unwrap_or(0.7);

        Ok(empty)
    }};
}

macro_rules! parse_difficulty_body {
    ($self:ident, $reader:ident, $section:ident) => {{
        let mut ar = None;
        let mut od = None;
        let mut cs = None;
        let mut hp = None;
        let mut sv = None;
        let mut tick_rate = None;

        let mut empty = true;

        while next_line!($reader)? != 0 {
            if let Some(bytes) = $reader.get_section() {
                *$section = Section::from_bytes(bytes);
                empty = false;
                break;
            }

            let (key, value) = $reader.split_colon().ok_or(ParseError::BadLine)?;

            match key {
                b"ApproachRate" => {
                    if let Some(val) = value.parse().ok().filter(f32::is_in_range) {
                        ar = Some(val);
                    }
                }
                b"OverallDifficulty" => {
                    if let Some(val) = value.parse().ok().filter(f32::is_in_range) {
                        od = Some(val);
                    }
                }
                b"CircleSize" => {
                    if let Some(val) = value.parse().ok().filter(f32::is_in_range) {
                        cs = Some(val);
                    }
                }
                b"HPDrainRate" => {
                    if let Some(val) = value.parse().ok().filter(f32::is_in_range) {
                        hp = Some(val);
                    }
                }
                b"SliderTickRate" => {
                    if let Some(val) = value.parse().ok().filter(f64::is_in_range) {
                        tick_rate = Some(val);
                    }
                }
                b"SliderMultiplier" => {
                    if let Some(val) = value.parse().ok().filter(f64::is_in_range) {
                        sv = Some(val);
                    }
                }
                _ => {}
            }
        }

        const DEFAULT_DIFFICULTY: f32 = 5.0;

        $self.od = od.unwrap_or(DEFAULT_DIFFICULTY);
        $self.cs = cs.unwrap_or(DEFAULT_DIFFICULTY);
        $self.hp = hp.unwrap_or(DEFAULT_DIFFICULTY);
        $self.ar = ar.unwrap_or($self.od);
        $self.slider_mult = sv.unwrap_or(1.0);
        $self.tick_rate = tick_rate.unwrap_or(1.0);

        Ok(empty)
    }};
}

macro_rules! parse_events_body {
    ($self:ident, $reader:ident, $section:ident) => {{
        let mut empty = true;

        while next_line!($reader)? != 0 {
            if let Some(bytes) = $reader.get_section() {
                *$section = Section::from_bytes(bytes);
                empty = false;
                break;
            }

            let line = match $reader.get_line() {
                Ok(line) => line,
                Err(_) => $reader.get_line_ascii()?, // see ranked map id 49374
            };

            let mut split = line.split(',');

            // We're only interested in breaks
            if let Some(b'2') = split.next().and_then(|value| value.bytes().next()) {
                let start_time = split
                    .next()
                    .next_field("break start")?
                    .parse()
                    .ok()
                    .filter(f64::is_in_range);

                let end_time = split
                    .next()
                    .next_field("break end")?
                    .parse()
                    .ok()
                    .filter(f64::is_in_range);

                if let (Some(start_time), Some(end_time)) = (start_time, end_time) {
                    $self.breaks.push(Break {
                        start_time,
                        end_time,
                    });
                }
            }
        }

        Ok(empty)
    }};
}

macro_rules! parse_timingpoints_body {
    ($self:ident, $reader:ident, $section:ident) => {{
        let mut empty = true;

        let mut pending_diff_points_time = 0.0;
        let mut pending_diff_point = None;

        while next_line!($reader)? != 0 {
            if let Some(bytes) = $reader.get_section() {
                *$section = Section::from_bytes(bytes);
                empty = false;
                break;
            }

            let line = $reader.get_line()?;
            let mut split = line.split(',');

            let time: f64 = split
                .next()
                .next_field("timing point time")?
                .trim()
                .parse()?;

            if !time.is_in_range() {
                continue;
            }

            // * beatLength is allowed to be NaN to handle an edge case in which
            // * some beatmaps use NaN slider velocity to disable slider tick
            // * generation (see LegacyDifficultyControlPoint).
            let beat_len: f64 = split.next().next_field("beat len")?.trim().parse()?;

            if !(beat_len.is_in_range() || beat_len.is_nan()) {
                continue;
            }

            let mut timing_change = true;
            let mut kiai = false;

            enum Status {
                Ok,
                Err,
            }

            fn parse_remaining<'s, I>(
                mut split: I,
                timing_change: &mut bool,
                kiai: &mut bool,
            ) -> Status
            where
                I: Iterator<Item = &'s str>,
            {
                match split
                    .next()
                    .filter(|&sig| !sig.starts_with('0'))
                    .map(str::parse::<i32>)
                {
                    Some(Ok(time_sig)) if !time_sig.is_in_range() || time_sig < 1 => {
                        return Status::Err
                    }
                    Some(Ok(_)) => {}
                    None => return Status::Ok,
                    Some(Err(_)) => return Status::Err,
                }

                match split.next().map(str::parse::<i32>) {
                    Some(Ok(sample_set)) if !sample_set.is_in_range() => return Status::Err,
                    Some(Ok(_)) => {}
                    None => return Status::Ok,
                    Some(Err(_)) => return Status::Err,
                }

                match split.next().map(str::parse::<i32>) {
                    Some(Ok(custom_sample)) if !custom_sample.is_in_range() => return Status::Err,
                    Some(Ok(_)) => {}
                    None => return Status::Ok,
                    Some(Err(_)) => return Status::Err,
                }

                match split.next().map(str::parse::<i32>) {
                    Some(Ok(sample_volume)) if !sample_volume.is_in_range() => return Status::Err,
                    Some(Ok(_)) => {}
                    None => return Status::Ok,
                    Some(Err(_)) => return Status::Err,
                }

                if let Some(byte) = split.next().and_then(|value| value.bytes().next()) {
                    *timing_change = byte == b'1';
                } else {
                    return Status::Ok;
                }

                match split.next().map(str::parse::<i32>) {
                    Some(Ok(effect_flags)) if !effect_flags.is_in_range() => return Status::Err,
                    Some(Ok(effect_flags)) => *kiai = (effect_flags & KIAI_FLAG) > 0,
                    None => return Status::Ok,
                    Some(Err(_)) => return Status::Err,
                }

                Status::Ok
            }

            if let Status::Err = parse_remaining(split, &mut timing_change, &mut kiai) {
                continue;
            }

            // * If beatLength is NaN, speedMultiplier should still be 1
            // * because all comparisons against NaN are false.
            let speed_multiplier = if beat_len < 0.0 {
                (100.0 / -beat_len)
            } else {
                1.0
            };

            if time != pending_diff_points_time {
                if let Some(point) = pending_diff_point.take() {
                    $self.difficulty_points.push_if_not_redundant(point);
                }
            }

            if timing_change {
                let point = TimingPoint::new(time, beat_len.clamp(6.0, 60_000.0));

                $self.timing_points.push(point);
            }

            if !timing_change || pending_diff_point.is_none() {
                pending_diff_point = Some(DifficultyPoint::new(time, beat_len, speed_multiplier));
            }

            let effect_point = EffectPoint::new(time, kiai);
            $self.effect_points.push(effect_point);

            pending_diff_points_time = time;
        }

        if let Some(point) = pending_diff_point {
            $self.difficulty_points.push_if_not_redundant(point);
        }

        $self.timing_points.dedup_by_key(|point| point.time);
        $self.difficulty_points.dedup_by_key(|point| point.time);
        $self.effect_points.dedup_by_key(|point| point.time);

        Ok(empty)
    }};
}

macro_rules! parse_hitobjects_body {
    ($self:ident, $reader:ident, $section:ident) => {{
        let mut unsorted = false;
        let mut prev_time = 0.0;
        let mut empty = true;

        // `point_split` will be of type `Vec<&str>
        // with each element having its lifetime bound to `buf`.
        // To circumvent this, `point_split_raw` will contain
        // the actual `&str` elements transmuted into `(usize, usize)`.
        let mut point_split_raw: Vec<(usize, usize)> = Vec::new();

        // Buffer to re-use for all sliders
        let mut vertices = Vec::new();

        while next_line!($reader)? != 0 {
            if let Some(bytes) = $reader.get_section() {
                *$section = Section::from_bytes(bytes);
                empty = false;
                break;
            }

            let line = $reader.get_line()?;
            let mut split = line.split(',');

            let x = split
                .next()
                .next_field("x pos")?
                .parse()
                .ok()
                .filter(|x| f32::is_in_custom_range(x, MAX_COORDINATE_VALUE as f32))
                .map(|x| x as i32 as f32);

            let y = split
                .next()
                .next_field("y pos")?
                .parse()
                .ok()
                .filter(|x| f32::is_in_custom_range(x, MAX_COORDINATE_VALUE as f32))
                .map(|x| x as i32 as f32);

            let pos = if let (Some(x), Some(y)) = (x, y) {
                Pos2 { x, y }
            } else {
                continue;
            };

            let time_opt = split
                .next()
                .next_field("hitobject time")?
                .trim()
                .parse()
                .ok()
                .filter(f64::is_in_range);

            let time = match time_opt {
                Some(time) => time,
                None => continue,
            };

            if !$self.hit_objects.is_empty() && time < prev_time {
                unsorted = true;
            }

            let kind: u8 = match split.next().next_field("hitobject kind")?.parse() {
                Ok(kind) => kind,
                Err(_) => continue,
            };

            let sound: u8 = match split.next().next_field("sound")?.parse() {
                Ok(sound) => sound,
                Err(_) => continue,
            };

            let kind = if kind & Self::CIRCLE_FLAG > 0 {
                $self.n_circles += 1;

                HitObjectKind::Circle
            } else if kind & Self::SLIDER_FLAG > 0 {
                $self.n_sliders += 1;

                let mut control_points = Vec::new();

                let control_point_iter = split.next().next_field("control points")?.split('|');

                let repeats = match split.next().next_field("repeats")?.parse::<usize>() {
                    // * osu-stable treated the first span of the slider
                    // * as a repeat, but no repeats are happening
                    Ok(repeats @ 0..=9000) => repeats.saturating_sub(1),
                    Ok(_) | Err(_) => continue,
                };

                let mut start_idx = 0;
                let mut end_idx = 0;
                let mut first = true;

                // SAFETY: `Vec<(usize, usize)>` and `Vec<&str>` have the same size and layout.
                let point_split: &mut Vec<&str> =
                    unsafe { std::mem::transmute(&mut point_split_raw) };

                point_split.clear();
                point_split.extend(control_point_iter);

                #[allow(clippy::blocks_in_if_conditions)]
                while {
                    end_idx += 1;

                    end_idx < point_split.len()
                } {
                    // * Keep incrementing end_idx while it's not the start of a new segment
                    // * (indicated by having a type descriptor of length 1).
                    if point_split[end_idx].len() > 1 {
                        continue;
                    }

                    // * Multi-segmented sliders DON'T contain the end point as part of the
                    // * current segment as it's assumed to be the start of the next segment.
                    // * The start of the next segment is the index after the type descriptor.
                    let end_point = point_split.get(end_idx + 1).copied();

                    convert_points(
                        &point_split[start_idx..end_idx],
                        end_point,
                        first,
                        pos,
                        &mut control_points,
                        &mut vertices,
                    )?;

                    start_idx = end_idx;
                    first = false;
                }

                if end_idx > start_idx {
                    convert_points(
                        &point_split[start_idx..end_idx],
                        None,
                        first,
                        pos,
                        &mut control_points,
                        &mut vertices,
                    )?;
                }

                if control_points.is_empty() {
                    HitObjectKind::Circle
                } else {
                    let pixel_len = match split.next().map(str::parse::<f64>) {
                        Some(Ok(len)) if len.is_in_custom_range(MAX_COORDINATE_VALUE as f64) => {
                            (len > 0.0).then_some(len)
                        }
                        Some(_) => continue,
                        None => None,
                    };

                    let edge_sounds_opt = split.next().map(|sounds| {
                        sounds
                            .split('|')
                            .take(repeats + 2)
                            .map(parse_custom_sound)
                            .collect()
                    });

                    let edge_sounds = match edge_sounds_opt {
                        None => Vec::new(),
                        Some(sounds) => sounds,
                    };

                    HitObjectKind::Slider {
                        repeats,
                        pixel_len,
                        control_points,
                        edge_sounds,
                    }
                }
            } else if kind & Self::SPINNER_FLAG > 0 {
                $self.n_spinners += 1;

                let end_time = match split.next().next_field("spinner endtime")?.parse::<f64>() {
                    Ok(end_time) => end_time.max(0.0),
                    Err(_) => continue,
                };

                HitObjectKind::Spinner { end_time }
            } else if kind & Self::HOLD_FLAG > 0 {
                $self.n_sliders += 1;

                let end_time = match split
                    .next()
                    .and_then(|next| next.split(':').next())
                    .map(str::parse::<f64>)
                {
                    Some(Ok(time_)) if time_.is_in_range() => time_.max(time),
                    Some(_) => continue,
                    None => time,
                };

                HitObjectKind::Hold { end_time }
            } else {
                return Err(ParseError::UnknownHitObjectKind);
            };

            $self.hit_objects.push(HitObject {
                pos,
                start_time: time,
                kind,
            });

            $self.sounds.push(sound);

            prev_time = time;
        }

        // BUG: If [General] section comes after [HitObjects] then the mode
        // won't be set yet so mania objects won't be sorted properly
        if $self.mode == GameMode::Mania {
            // First a _stable_ sort by time
            $self
                .hit_objects
                .sort_by(|p1, p2| p1.partial_cmp(p2).unwrap_or(Ordering::Equal));

            // Then the legacy sort for correct position order
            legacy_sort(&mut $self.hit_objects);
        } else if unsorted {
            sort_unstable(&mut $self.hit_objects);
        }

        Ok(empty)
    }};
}

// Required for maps with slider edge sound values above 255 e.g. map /b/80799
fn parse_custom_sound(sound: &str) -> u8 {
    sound
        .bytes()
        .try_fold(0_u8, |sound, byte| match byte {
            b'0'..=b'9' => Some(sound.wrapping_mul(10).wrapping_add((byte & 0xF) as u8)),
            _ => None,
        })
        .unwrap_or(0)
}

macro_rules! parse_body {
    ($input:ident) => {{
        let mut reader = FileReader::new($input);
        next_line!(reader)?;

        if reader.is_initial_empty_line() {
            next_line!(reader)?;
        }

        let mut map = Beatmap {
            version: reader.version()?,
            hit_objects: Vec::with_capacity(256), // TODO: test avg length, same for control points
            sounds: Vec::with_capacity(256),
            ..Default::default()
        };

        let mut section = Section::None;

        loop {
            match section {
                Section::General => section!(map, parse_general, reader, section),
                Section::Difficulty => section!(map, parse_difficulty, reader, section),
                Section::Events => section!(map, parse_events, reader, section),
                Section::TimingPoints => section!(map, parse_timingpoints, reader, section),
                Section::HitObjects => section!(map, parse_hitobjects, reader, section),
                Section::None => {
                    if next_line!(reader)? == 0 {
                        break;
                    }

                    if let Some(bytes) = reader.get_section() {
                        section = Section::from_bytes(bytes);
                    }
                }
            }
        }

        Ok(map)
    }};
}

impl Beatmap {
    const CIRCLE_FLAG: u8 = 1 << 0;
    const SLIDER_FLAG: u8 = 1 << 1;
    // const NEW_COMBO_FLAG: u8 = 1 << 2;
    const SPINNER_FLAG: u8 = 1 << 3;
    // const COMBO_OFFSET_FLAG: u8 = (1 << 4) | (1 << 5) | (1 << 6);
    const HOLD_FLAG: u8 = 1 << 7;
}

mod slider_parsing {
    use crate::ParseError;

    use super::Pos2;

    pub(super) fn convert_points(
        points: &[&str],
        end_point: Option<&str>,
        first: bool,
        offset: Pos2,
        curve_points: &mut Vec<PathControlPoint>,
        vertices: &mut Vec<PathControlPoint>,
    ) -> Result<(), ParseError> {
        let mut path_kind = PathType::from_str(points[0]);

        let read_offset = first as usize;
        let readable_points = points.len() - 1;
        let end_point_len = end_point.is_some() as usize;

        vertices.clear();
        vertices.reserve(read_offset + readable_points + end_point_len);

        // * Fill any non-read points.
        vertices.extend((0..read_offset).map(|_| PathControlPoint::default()));

        // * Parse into control points.
        for &point in points.iter().skip(1) {
            vertices.push(read_point(point, offset)?);
        }

        // * If an endpoint is given, add it to the end.
        if let Some(end_point) = end_point {
            vertices.push(read_point(end_point, offset)?);
        }

        // * Edge-case rules (to match stable).
        if path_kind == PathType::PerfectCurve {
            if let [a, b, c] = &vertices[..] {
                if is_linear(a.pos, b.pos, c.pos) {
                    // * osu-stable special-cased colinear perfect curves to a linear path
                    path_kind = PathType::Linear;
                }
            } else {
                path_kind = PathType::Bezier;
            }
        }

        // * The first control point must have a definite type.
        vertices[0].kind = Some(path_kind);

        // * A path can have multiple implicit segments of the same type if
        // * there are two sequential control points with the same position.
        // * To handle such cases, this code may return multiple path segments
        // * with the final control point in each segment having a non-null type.
        // * For the point string X|1:1|2:2|2:2|3:3, this code returns the segments:
        // * X: { (1,1), (2, 2) }
        // * X: { (3, 3) }
        // * Note: (2, 2) is not returned in the second segments, as it is implicit in the path.
        let mut start_idx = 0;
        let mut end_idx = 0;

        #[allow(clippy::blocks_in_if_conditions)]
        while {
            end_idx += 1;

            end_idx < vertices.len() - end_point_len
        } {
            // * Keep incrementing while an implicit segment doesn't need to be started
            if vertices[end_idx].pos != vertices[end_idx - 1].pos {
                continue;
            }

            // * Legacy Catmull sliders don't support multiple segments,
            // * so adjacent Catmull segments should be treated as a single one.
            // * Importantly, this is not applied to the first control point,
            // * which may duplicate the slider path's position
            // * resulting in a duplicate (0,0) control point in the resultant list.
            if path_kind == PathType::Catmull && end_idx > 1 {
                continue;
            }

            // * The last control point of each segment is not
            // * allowed to start a new implicit segment.
            if end_idx == vertices.len() - end_point_len - 1 {
                continue;
            }

            // * Force a type on the last point, and return
            // * the current control point set as a segment.
            vertices[end_idx - 1].kind = Some(path_kind);
            curve_points.extend(&vertices[start_idx..end_idx]);

            // * Skip the current control point - as it's the same as the one that's just been returned.
            start_idx = end_idx + 1;
        }

        if end_idx > start_idx {
            curve_points.extend(&vertices[start_idx..end_idx]);
        }

        Ok(())
    }

    pub(super) fn read_point(value: &str, start_pos: Pos2) -> Result<PathControlPoint, ParseError> {
        let mut v = value.split(':').map(str::parse);

        match (v.next(), v.next()) {
            (Some(Ok(x)), Some(Ok(y))) => Ok(PathControlPoint::from(Pos2 { x, y } - start_pos)),
            _ => Err(ParseError::InvalidCurvePoints),
        }
    }

    fn is_linear(p0: Pos2, p1: Pos2, p2: Pos2) -> bool {
        ((p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x)).abs() <= f32::EPSILON
    }

    /// Control point for slider curve calculation
    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub struct PathControlPoint {
        /// Control point position.
        pub pos: Pos2,
        /// Path type of the control point.
        /// Only present for the first element of each segment.
        pub kind: Option<PathType>,
    }

    impl From<Pos2> for PathControlPoint {
        #[inline]
        fn from(pos: Pos2) -> Self {
            Self { pos, kind: None }
        }
    }

    /// The type of curve of a slider.
    #[allow(missing_docs)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum PathType {
        Catmull = 0,
        Bezier = 1,
        Linear = 2,
        PerfectCurve = 3,
    }

    impl PathType {
        #[inline]
        fn from_str(s: &str) -> Self {
            match s {
                "L" => Self::Linear,
                "B" => Self::Bezier,
                "P" => Self::PerfectCurve,
                _ => Self::Catmull,
            }
        }
    }
}

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
impl Beatmap {
    /// Parse a beatmap from a `.osu` file.
    ///
    /// As argument you can give anything that implements [`std::io::Read`].
    /// You'll likely want to pass (a reference of) a [`File`](std::fs::File)
    /// or the file's content as a slice of bytes (`&[u8]`).
    pub fn parse<R: Read>(input: R) -> ParseResult<Self> {
        parse_body!(input)
    }

    fn parse_general<R: Read>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_general_body!(self, reader, section)
    }

    fn parse_difficulty<R: Read>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_difficulty_body!(self, reader, section)
    }

    fn parse_events<R: Read>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_events_body!(self, reader, section)
    }

    fn parse_hitobjects<R: Read>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_hitobjects_body!(self, reader, section)
    }

    fn parse_timingpoints<R: Read>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_timingpoints_body!(self, reader, section)
    }
    /// Pass the path to a `.osu` file.
    ///
    /// Useful when you don't want to create the [`File`](std::fs::File) manually.
    /// If you have the file lying around already though (and plan on re-using it),
    /// passing `&file` to [`parse`](Beatmap::parse) should be preferred.
    pub fn from_path<P: AsRef<Path>>(path: P) -> ParseResult<Self> {
        Self::parse(File::open(path)?)
    }
}

#[cfg(any(feature = "async_tokio", feature = "async_std"))]
impl Beatmap {
    /// Parse a beatmap from a `.osu` file.
    ///
    /// As argument you can give anything that implements `tokio::io::AsyncRead`
    /// or `async_std::io::Read`, depending which feature you chose.
    /// You'll likely want to pass a `File`
    /// or the file's content as a slice of bytes (`&[u8]`).
    pub async fn parse<R: AsyncRead + Unpin>(input: R) -> ParseResult<Self> {
        parse_body!(input)
    }

    async fn parse_general<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_general_body!(self, reader, section)
    }

    async fn parse_difficulty<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_difficulty_body!(self, reader, section)
    }

    async fn parse_events<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_events_body!(self, reader, section)
    }

    async fn parse_hitobjects<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_hitobjects_body!(self, reader, section)
    }

    async fn parse_timingpoints<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut FileReader<R>,
        section: &mut Section,
    ) -> ParseResult<bool> {
        parse_timingpoints_body!(self, reader, section)
    }

    /// Pass the path to a `.osu` file.
    ///
    /// Useful when you don't want to create the file manually.
    pub async fn from_path<P: AsRef<Path>>(path: P) -> ParseResult<Self> {
        Self::parse(File::open(path).await?).await
    }
}

#[derive(Copy, Clone, Debug)]
enum Section {
    None,
    General,
    Difficulty,
    TimingPoints,
    HitObjects,
    Events,
}

impl Section {
    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            b"General" => Self::General,
            b"Difficulty" => Self::Difficulty,
            b"TimingPoints" => Self::TimingPoints,
            b"HitObjects" => Self::HitObjects,
            b"Events" => Self::Events,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
    #[test]
    fn parsing_sync() {
        for map_id in map_ids() {
            println!("map_id: {}", map_id);

            let map = match Beatmap::from_path(format!("./maps/{}.osu", map_id)) {
                Ok(map) => map,
                Err(why) => panic!("Error while parsing map: {}", why),
            };

            print_info(map);
            println!("---");
        }
    }

    #[cfg(feature = "async_tokio")]
    #[test]
    fn parsing_async_tokio() {
        use tokio::runtime::Builder;

        Builder::new_current_thread()
            .build()
            .expect("could not start runtime")
            .block_on(async {
                for map_id in map_ids() {
                    println!("map_id: {}", map_id);

                    let map = match Beatmap::from_path(format!("./maps/{}.osu", map_id)).await {
                        Ok(map) => map,
                        Err(why) => panic!("Error while parsing map: {}", why),
                    };

                    print_info(map);
                    println!("---");
                }
            });
    }

    #[cfg(feature = "async_std")]
    #[test]
    fn parsing_async_std() {
        async_std::task::block_on(async {
            for map_id in map_ids() {
                println!("map_id: {}", map_id);

                let map = match Beatmap::from_path(format!("./maps/{}.osu", map_id)).await {
                    Ok(map) => map,
                    Err(why) => panic!("Error while parsing map: {}", why),
                };

                print_info(map);
                println!("---");
            }
        });
    }

    fn map_ids() -> Vec<i32> {
        vec![
            2785319, // osu
            1974394, // mania
            2118524, // catch
            1028484, // taiko
        ]
    }

    fn print_info(map: Beatmap) {
        println!("Mode: {}", map.mode as u8);
        println!("n_circles: {}", map.n_circles);
        println!("n_sliders: {}", map.n_sliders);
        println!("n_spinners: {}", map.n_spinners);
        println!("ar: {}", map.ar);
        println!("od: {}", map.od);
        println!("cs: {}", map.cs);
        println!("hp: {}", map.hp);
        println!("slider_mult: {}", map.slider_mult);
        println!("tick_rate: {}", map.tick_rate);
        println!("hit_objects: {}", map.hit_objects.len());
        println!("stack_leniency: {}", map.stack_leniency);
        println!("timing_points: {}", map.timing_points.len());
        println!("difficulty_points: {}", map.difficulty_points.len());
    }
}
