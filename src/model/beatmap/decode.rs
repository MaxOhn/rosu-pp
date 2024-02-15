use std::{cmp, slice};

use rosu_map::{
    section::{
        difficulty::{Difficulty, DifficultyState, ParseDifficultyError},
        events::{BreakPeriod, EventType, ParseEventTypeError},
        general::{GameMode, GeneralKey, ParseGameModeError},
        hit_objects::{
            hit_samples::{HitSoundType, ParseHitSoundTypeError},
            HitObjectType, ParseHitObjectTypeError, PathControlPoint, PathType,
        },
        timing_points::{ControlPoint, EffectFlags, ParseEffectFlagsError},
    },
    util::{KeyValue, ParseNumber, ParseNumberError, Pos, StrExt, MAX_PARSE_VALUE},
    DecodeBeatmap, DecodeState,
};

use crate::{
    model::{
        control_point::{
            difficulty_point_at, effect_point_at, DifficultyPoint, EffectPoint, TimingPoint,
        },
        hit_object::{HitObject, HitObjectKind, HoldNote, Slider, Spinner},
    },
    util::{float_ext::FloatExt, sort},
};

use super::{Beatmap, DEFAULT_SLIDER_LENIENCY};

/// The state of a [`Beatmap`] for [`DecodeBeatmap`].
pub struct BeatmapState {
    version: i32,
    stack_leniency: f32,
    mode: GameMode,
    difficulty: DifficultyState,
    breaks: Vec<BreakPeriod>,
    timing_points: Vec<TimingPoint>,
    difficulty_points: Vec<DifficultyPoint>,
    effect_points: Vec<EffectPoint>,
    hit_objects: Vec<HitObject>,
    hit_sounds: Vec<HitSoundType>,

    pending_control_points_time: f64,
    pending_timing_point: Option<TimingPoint>,
    pending_difficulty_point: Option<DifficultyPoint>,
    pending_effect_point: Option<EffectPoint>,

    curve_points: Vec<PathControlPoint>,
    vertices: Vec<PathControlPoint>,
    point_split: Vec<*const str>,
}

impl BeatmapState {
    fn add_pending_point<P: Pending>(&mut self, time: f64, point: P, timing_change: bool) {
        if time.not_eq(self.pending_control_points_time) {
            self.flush_pending_points();
        }

        if timing_change {
            point.push_front(self);
        } else {
            point.push_back(self);
        }

        self.pending_control_points_time = time;
    }

    fn flush_pending_points(&mut self) {
        if let Some(point) = self.pending_timing_point.take() {
            self.add_control_point(point);
        }

        if let Some(point) = self.pending_difficulty_point.take() {
            self.add_control_point(point);
        }

        if let Some(point) = self.pending_effect_point.take() {
            self.add_control_point(point);
        }
    }

    fn add_control_point<P: ControlPoint<Self>>(&mut self, point: P) {
        if !point.check_already_existing(self) {
            point.add(self);
        }
    }

    fn convert_path_str(&mut self, point_str: &str, offset: Pos) -> Result<(), ParseBeatmapError> {
        let f = |this: &mut Self, point_split: &[&str]| {
            let mut start_idx = 0;
            let mut end_idx = 0;
            let mut first = true;

            while {
                end_idx += 1;

                end_idx < point_split.len()
            } {
                let is_letter = point_split[end_idx]
                    .chars()
                    .next()
                    .ok_or(ParseBeatmapError::InvalidHitObjectLine)?
                    .is_ascii_alphabetic();

                if !is_letter {
                    continue;
                }

                let end_point = point_split.get(end_idx + 1).copied();
                this.convert_points(&point_split[start_idx..end_idx], end_point, first, offset)?;

                start_idx = end_idx;
                first = false;
            }

            if end_idx > start_idx {
                this.convert_points(&point_split[start_idx..end_idx], None, first, offset)?;
            }

            Ok(())
        };

        self.point_split(point_str.split('|'), f)
    }

    fn convert_points(
        &mut self,
        points: &[&str],
        end_point: Option<&str>,
        first: bool,
        offset: Pos,
    ) -> Result<(), ParseBeatmapError> {
        fn read_point(value: &str, start_pos: Pos) -> Result<PathControlPoint, ParseBeatmapError> {
            let mut v = value
                .split(':')
                .map(|s| s.parse_with_limits(f64::from(MAX_COORDINATE_VALUE)));

            let (x, y) = v
                .next()
                .zip(v.next())
                .ok_or(ParseBeatmapError::InvalidHitObjectLine)?;

            let pos = Pos::new(x? as i32 as f32, y? as i32 as f32);

            Ok(PathControlPoint::new(pos - start_pos))
        }

        fn is_linear(p0: Pos, p1: Pos, p2: Pos) -> bool {
            ((p1.y - p0.y) * (p2.x - p0.x)).eq((p1.x - p0.x) * (p2.y - p0.y))
        }

        let mut path_type = points
            .first()
            .copied()
            .map(PathType::new_from_str)
            .ok_or(ParseBeatmapError::InvalidHitObjectLine)?;

        let read_offset = usize::from(first);
        let readable_points = points.len() - 1;
        let end_point_len = usize::from(end_point.is_some());

        self.vertices.clear();
        self.vertices
            .reserve(read_offset + readable_points + end_point_len);

        if first {
            self.vertices.push(PathControlPoint::default());
        }

        for &point in points.iter().skip(1) {
            self.vertices.push(read_point(point, offset)?);
        }

        if let Some(end_point) = end_point {
            self.vertices.push(read_point(end_point, offset)?);
        }

        if path_type == PathType::PERFECT_CURVE {
            if let [a, b, c] = self.vertices.as_slice() {
                if is_linear(a.pos, b.pos, c.pos) {
                    path_type = PathType::LINEAR;
                }
            } else {
                path_type = PathType::BEZIER;
            }
        }

        self.vertices[0].path_type = Some(path_type);

        let mut start_idx = 0;
        let mut end_idx = 0;

        while {
            end_idx += 1;

            end_idx < self.vertices.len() - end_point_len
        } {
            if self.vertices[end_idx].pos != self.vertices[end_idx - 1].pos {
                continue;
            }

            if path_type == PathType::CATMULL && end_idx > 1 {
                continue;
            }

            if end_idx == self.vertices.len() - end_point_len - 1 {
                continue;
            }

            self.vertices[end_idx - 1].path_type = Some(path_type);

            self.curve_points.extend(&self.vertices[start_idx..end_idx]);

            start_idx = end_idx + 1;
        }

        if end_idx > start_idx {
            self.curve_points.extend(&self.vertices[start_idx..end_idx]);
        }

        Ok(())
    }

    fn point_split<'a, I, F, O>(&mut self, point_split: I, f: F) -> O
    where
        I: Iterator<Item = &'a str>,
        F: FnOnce(&mut Self, &[&'a str]) -> O,
    {
        self.point_split.extend(point_split.map(|s| s as *const _));
        let ptr = self.point_split.as_ptr();
        let len = self.point_split.len();

        // SAFETY:
        // - *const str and &str have the same layout.
        // - `self.point_split` is cleared after every use, ensuring that it
        //   does not contain any invalid pointers.
        let point_split = unsafe { slice::from_raw_parts(ptr.cast(), len) };
        let res = f(self, point_split);
        self.point_split.clear();

        res
    }
}

impl DecodeState for BeatmapState {
    fn create(version: i32) -> Self {
        Self {
            version,
            stack_leniency: DEFAULT_SLIDER_LENIENCY,
            mode: GameMode::Osu,
            difficulty: DifficultyState::create(version),
            breaks: Vec::new(),
            timing_points: Vec::with_capacity(1),
            difficulty_points: Vec::new(),
            effect_points: Vec::with_capacity(32),
            hit_objects: Vec::with_capacity(512),
            hit_sounds: Vec::with_capacity(512),
            pending_control_points_time: 0.0,
            pending_timing_point: None,
            pending_difficulty_point: None,
            pending_effect_point: None,
            // mean=13.11 | median=8
            curve_points: Vec::with_capacity(8),
            // mean=16.27 | median=8
            vertices: Vec::with_capacity(8),
            // mean=19.97 | median=8
            point_split: Vec::with_capacity(8),
        }
    }
}

impl From<BeatmapState> for Beatmap {
    fn from(mut state: BeatmapState) -> Self {
        state.flush_pending_points();

        let Difficulty {
            hp_drain_rate,
            circle_size,
            overall_difficulty,
            approach_rate,
            slider_multiplier,
            slider_tick_rate,
        } = state.difficulty.into();

        let mut sorter = sort::TandemSorter::new(
            &state.hit_objects,
            |a, b| a.start_time.total_cmp(&b.start_time),
            true,
        );

        sorter.sort(&mut state.hit_objects);
        sorter.sort(&mut state.hit_sounds);

        if state.mode == GameMode::Mania {
            sort::osu_legacy(&mut state.hit_objects);
        }

        Beatmap {
            version: state.version,
            stack_leniency: state.stack_leniency,
            mode: state.mode,
            ar: approach_rate,
            cs: circle_size,
            hp: hp_drain_rate,
            od: overall_difficulty,
            slider_multiplier,
            slider_tick_rate,
            breaks: state.breaks,
            timing_points: state.timing_points,
            difficulty_points: state.difficulty_points,
            effect_points: state.effect_points,
            hit_objects: state.hit_objects,
            hit_sounds: state.hit_sounds,
        }
    }
}

/// All the ways that parsing a [`Beatmap`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseBeatmapError {
    #[error("failed to parse effect flags")]
    EffectFlags(#[from] ParseEffectFlagsError),
    #[error("failed to parse event type")]
    EventType(#[from] ParseEventTypeError),
    #[error("failed to parse hit object type")]
    HitObjectType(#[from] ParseHitObjectTypeError),
    #[error("failed to parse hit sound type")]
    HitSoundType(#[from] ParseHitSoundTypeError),
    #[error("invalid event line")]
    InvalidEventLine,
    #[error("repeat count is way too high")]
    InvalidRepeatCount,
    #[error("invalid timing point line")]
    InvalidTimingPointLine,
    #[error("invalid hit object line")]
    InvalidHitObjectLine,
    #[error("failed to parse mode")]
    Mode(#[from] ParseGameModeError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("beat length cannot be NaN in a timing control point")]
    TimingControlPointNaN,
    #[error("unknown hit object type")]
    UnknownHitObjectType,
}

impl From<ParseDifficultyError> for ParseBeatmapError {
    fn from(e: ParseDifficultyError) -> Self {
        match e {
            ParseDifficultyError::Number(e) => Self::Number(e),
        }
    }
}

const MAX_COORDINATE_VALUE: i32 = 131_072;

impl DecodeBeatmap for Beatmap {
    type Error = ParseBeatmapError;
    type State = BeatmapState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let Ok(KeyValue { key, value }) = KeyValue::parse(line.trim_comment()) else {
            return Ok(());
        };

        match key {
            GeneralKey::StackLeniency => state.stack_leniency = value.parse_num()?,
            GeneralKey::Mode => state.mode = value.parse()?,
            _ => {}
        }

        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Difficulty::parse_difficulty(&mut state.difficulty, line).map_err(ParseBeatmapError::from)
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let mut split = line.trim_comment().split(',');

        let event_type: EventType = split
            .next()
            .ok_or(ParseBeatmapError::InvalidEventLine)?
            .parse()?;

        if event_type == EventType::Break {
            let Some((start_time, end_time)) = split.next().zip(split.next()) else {
                return Err(ParseBeatmapError::InvalidEventLine);
            };

            let start_time = f64::parse(start_time)?;
            let end_time = start_time.max(f64::parse(end_time)?);

            state.breaks.push(BreakPeriod {
                start_time,
                end_time,
            });
        }

        Ok(())
    }

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let mut split = line.trim_comment().split(',');

        let (time, beat_len) = split
            .next()
            .zip(split.next())
            .ok_or(ParseBeatmapError::InvalidTimingPointLine)?;

        let time = time.parse_num::<f64>()?;

        // Manual `str::parse_num::<f64>` so that NaN does not cause an error
        let beat_len = beat_len
            .trim()
            .parse::<f64>()
            .map_err(ParseNumberError::InvalidFloat)?;

        if beat_len < f64::from(-MAX_PARSE_VALUE) {
            return Err(ParseNumberError::NumberUnderflow.into());
        } else if beat_len > f64::from(MAX_PARSE_VALUE) {
            return Err(ParseNumberError::NumberOverflow.into());
        }

        let speed_multiplier = if beat_len < 0.0 {
            100.0 / -beat_len
        } else {
            1.0
        };

        let _ = split.next(); // timing signature
        let _ = split.next(); // sample set
        let _ = split.next(); // custom sample bank
        let _ = split.next(); // sample volume

        let timing_change = split
            .next()
            .map_or(true, |next| matches!(next.chars().next(), Some('1')));

        let kiai = split
            .next()
            .map(str::parse::<EffectFlags>)
            .transpose()?
            .is_some_and(|flags| flags.has_flag(EffectFlags::KIAI));

        if timing_change {
            if beat_len.is_nan() {
                return Err(ParseBeatmapError::TimingControlPointNaN);
            }

            let timing = TimingPoint::new(time, beat_len);
            state.add_pending_point(time, timing, timing_change);
        }

        let difficulty = DifficultyPoint::new(time, beat_len, speed_multiplier);
        state.add_pending_point(time, difficulty, timing_change);

        let effect = EffectPoint::new(time, kiai);
        state.add_pending_point(time, effect, timing_change);

        state.pending_control_points_time = time;

        Ok(())
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let mut split = line.trim_comment().split(',');

        let (Some(x), Some(y), Some(start_time), Some(kind), Some(sound_type)) = (
            split.next(),
            split.next(),
            split.next(),
            split.next(),
            split.next(),
        ) else {
            return Err(ParseBeatmapError::InvalidHitObjectLine);
        };

        let pos = Pos {
            x: x.parse_with_limits(MAX_COORDINATE_VALUE as f32)? as i32 as f32,
            y: y.parse_with_limits(MAX_COORDINATE_VALUE as f32)? as i32 as f32,
        };

        let start_time = f64::parse(start_time)?;
        let hit_object_type: HitObjectType = kind.parse()?;

        let mut sound: HitSoundType = sound_type.parse()?;

        let mut parse_custom_sound = |bank_info: Option<&str>| {
            let mut split = match bank_info {
                Some(s) if !s.is_empty() => s.split(':'),
                _ => return Ok::<_, ParseNumberError>(()),
            };

            let _ = split.next().map(i32::parse).transpose()?; // normal bank
            let _ = split.next().map(i32::parse).transpose()?; // additional bank
            let _ = split.next().map(i32::parse).transpose()?; // custom sample bank
            let _ = split.next().map(i32::parse).transpose()?; // volume

            // filename
            match split.next() {
                None | Some("") => {}
                Some(_) => sound = HitSoundType::default(),
            }

            Ok(())
        };

        let kind = if hit_object_type.has_flag(HitObjectType::CIRCLE) {
            parse_custom_sound(split.next())?;

            HitObjectKind::Circle
        } else if hit_object_type.has_flag(HitObjectType::SLIDER) {
            let (point_str, repeat_count) = split
                .next()
                .zip(split.next())
                .ok_or(ParseBeatmapError::InvalidHitObjectLine)?;

            let mut len = None;

            let mut repeats = repeat_count.parse_num::<i32>()?;

            if repeats > 9000 {
                return Err(ParseBeatmapError::InvalidRepeatCount);
            }

            repeats = cmp::max(0, repeats - 1);

            if let Some(next) = split.next() {
                let new_len = next
                    .parse_with_limits(f64::from(MAX_COORDINATE_VALUE))?
                    .max(0.0);

                if new_len.not_eq(0.0) {
                    len = Some(new_len);
                }
            }

            let node_sounds = if let Some(sounds) = split.next().map(|sounds| sounds.split('|')) {
                sounds.map(|s| s.parse().unwrap_or_default()).collect()
            } else {
                Box::default()
            };

            let _ = split.next(); // node banks
            parse_custom_sound(split.next())?;

            state.convert_path_str(point_str, pos)?;
            let mut control_points = Vec::with_capacity(state.curve_points.len());
            control_points.append(&mut state.curve_points);

            let slider = Slider {
                expected_dist: len,
                repeats: repeats as usize,
                control_points: control_points.into_boxed_slice(),
                node_sounds,
            };

            HitObjectKind::Slider(slider)
        } else if hit_object_type.has_flag(HitObjectType::SPINNER) {
            let end_time = split
                .next()
                .ok_or(ParseBeatmapError::InvalidHitObjectLine)?
                .parse_num::<f64>()?;

            parse_custom_sound(split.next())?;

            HitObjectKind::Spinner(Spinner { end_time })
        } else if hit_object_type.has_flag(HitObjectType::HOLD) {
            let end_time = if let Some(s) = split.next().filter(|s| !s.is_empty()) {
                let (end_time, bank_info) = s
                    .split_once(':')
                    .ok_or(ParseBeatmapError::InvalidHitObjectLine)?;

                parse_custom_sound(Some(bank_info))?;

                end_time.parse_num::<f64>()?.max(start_time)
            } else {
                start_time
            };

            HitObjectKind::Hold(HoldNote { end_time })
        } else {
            return Err(ParseBeatmapError::UnknownHitObjectType);
        };

        state.hit_objects.push(HitObject {
            pos,
            start_time,
            kind,
        });
        state.hit_sounds.push(sound);

        Ok(())
    }

    fn parse_variables(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_catch_the_beat(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_mania(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}

trait Pending: Sized {
    fn pending(state: &mut BeatmapState) -> &mut Option<Self>;

    fn push_front(self, state: &mut BeatmapState) {
        let pending = Self::pending(state);

        if pending.is_none() {
            *pending = Some(self);
        }
    }

    fn push_back(self, state: &mut BeatmapState) {
        *Self::pending(state) = Some(self);
    }
}

impl Pending for TimingPoint {
    fn pending(state: &mut BeatmapState) -> &mut Option<Self> {
        &mut state.pending_timing_point
    }
}

impl Pending for DifficultyPoint {
    fn pending(state: &mut BeatmapState) -> &mut Option<Self> {
        &mut state.pending_difficulty_point
    }
}

impl Pending for EffectPoint {
    fn pending(state: &mut BeatmapState) -> &mut Option<Self> {
        &mut state.pending_effect_point
    }
}

impl ControlPoint<BeatmapState> for TimingPoint {
    fn check_already_existing(&self, _: &BeatmapState) -> bool {
        false
    }

    fn add(self, state: &mut BeatmapState) {
        match state
            .timing_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
        {
            Err(i) => state.timing_points.insert(i, self),
            Ok(i) => state.timing_points[i] = self,
        }
    }
}

impl ControlPoint<BeatmapState> for DifficultyPoint {
    fn check_already_existing(&self, state: &BeatmapState) -> bool {
        match difficulty_point_at(&state.difficulty_points, self.time) {
            Some(existing) => self.is_redundant(existing),
            None => self.is_redundant(&DifficultyPoint::default()),
        }
    }

    fn add(self, state: &mut BeatmapState) {
        match state
            .difficulty_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
        {
            Err(i) => state.difficulty_points.insert(i, self),
            Ok(i) => state.difficulty_points[i] = self,
        }
    }
}

impl ControlPoint<BeatmapState> for EffectPoint {
    fn check_already_existing(&self, state: &BeatmapState) -> bool {
        match effect_point_at(&state.effect_points, self.time) {
            Some(existing) => self.is_redundant(existing),
            None => self.is_redundant(&EffectPoint::default()),
        }
    }

    fn add(self, state: &mut BeatmapState) {
        match state
            .effect_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
        {
            Err(i) => state.effect_points.insert(i, self),
            Ok(i) => state.effect_points[i] = self,
        }
    }
}
