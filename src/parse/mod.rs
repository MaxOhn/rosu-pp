mod attributes;
mod control_point;
mod error;
mod hitobject;
mod hitsound;
mod pos2;
mod sort;

pub use attributes::BeatmapAttributes;
pub use control_point::{DifficultyPoint, TimingPoint};
pub use error::{ParseError, ParseResult};
pub use hitobject::{HitObject, HitObjectKind};
pub use hitsound::HitSound;
pub use pos2::Pos2;
use sort::sort;

use std::cmp::Ordering;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;

macro_rules! sort {
    ($slice:expr) => {
        $slice.sort_unstable_by(|p1, p2| p1.partial_cmp(&p2).unwrap_or(Ordering::Equal))
    };
}

macro_rules! next_field {
    ($opt:expr) => {
        $opt.ok_or(ParseError::MissingField)?
    };
}

macro_rules! validate_float {
    ($x:expr) => {{
        if $x.is_finite() {
            $x
        } else {
            return Err(ParseError::InvalidFloatingPoint);
        }
    }};
}

macro_rules! line_prepare {
    ($buf:ident) => {{
        let mut line = $buf.trim_end();

        if line.is_empty()
            || line.starts_with("//")
            || line.starts_with(' ')
            || line.starts_with('_')
        {
            $buf.clear();
            continue;
        }

        if let Some(idx) = line.find("//") {
            line = &line[..idx];
        }

        line
    }};
}

macro_rules! section {
    ($map:ident, $func:ident, $reader:ident, $buf:ident, $section:ident) => {
        if $map.$func(&mut $reader, &mut $buf, &mut $section)? {
            break;
        }
    };
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameMode {
    STD = 0,
    TKO = 1,
    CTB = 2,
    MNA = 3,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::STD
    }
}

#[derive(Clone, Default, Debug)]
pub struct Beatmap {
    pub mode: GameMode,
    pub version: u8,

    pub n_circles: u32,
    pub n_sliders: u32,
    pub n_spinners: u32,

    pub ar: f32,
    pub od: f32,
    pub cs: f32,
    pub hp: f32,
    pub sv: f32,
    pub tick_rate: f32,
    pub stack_leniency: f32,

    pub hit_objects: Vec<HitObject>,
    pub timing_points: Vec<TimingPoint>,
    pub difficulty_points: Vec<DifficultyPoint>,
}

pub(crate) const OSU_FILE_HEADER: &str = "osu file format v";

impl Beatmap {
    const CIRCLE_FLAG: u8 = 1 << 0;
    const SLIDER_FLAG: u8 = 1 << 1;
    #[allow(unused)]
    const NEW_COMBO_FLAG: u8 = 1 << 2;
    const SPINNER_FLAG: u8 = 1 << 3;
    #[allow(unused)]
    const COMBO_OFFSET_FLAG: u8 = (1 << 4) | (1 << 5) | (1 << 6);
    const HOLD_FLAG: u8 = 1 << 7;

    pub fn parse<R: Read>(input: R) -> ParseResult<Self> {
        let mut reader = BufReader::new(input);
        let mut buf = String::new();

        reader.read_line(&mut buf)?;

        let mut map = Self::default();

        map.version = match buf.find(OSU_FILE_HEADER) {
            Some(idx) => buf[idx + OSU_FILE_HEADER.len()..].trim_end().parse()?,
            None => return Err(ParseError::IncorrectFileHeader),
        };

        buf.clear();
        map.hit_objects.reserve(256);

        let mut section = Section::None;

        loop {
            match section {
                Section::General => section!(map, parse_general, reader, buf, section),
                Section::Difficulty => section!(map, parse_difficulty, reader, buf, section),
                Section::TimingPoints => section!(map, parse_timingpoints, reader, buf, section),
                Section::HitObjects => section!(map, parse_hitobjects, reader, buf, section),
                Section::None => {
                    if reader.read_line(&mut buf)? == 0 {
                        break;
                    }

                    let line = line_prepare!(buf);

                    if line.starts_with('[') && line.ends_with(']') {
                        section = Section::from_str(&line[1..line.len() - 1]);
                    }

                    buf.clear();
                }
            }
        }

        Ok(map)
    }

    fn parse_general<R: Read>(
        &mut self,
        reader: &mut BufReader<R>,
        buf: &mut String,
        section: &mut Section,
    ) -> ParseResult<bool> {
        let mut mode = None;
        let mut stack_leniency = None;
        let mut empty = true;

        while reader.read_line(buf)? != 0 {
            let line = line_prepare!(buf);

            if line.starts_with('[') && line.ends_with(']') {
                *section = Section::from_str(&line[1..line.len() - 1]);
                empty = false;
                buf.clear();
                break;
            }

            let (key, value) = split_colon(&line).ok_or(ParseError::BadLine)?;

            if key == "Mode" {
                mode = match value {
                    "0" => Some(GameMode::STD),
                    "1" => Some(GameMode::TKO),
                    "2" => Some(GameMode::CTB),
                    "3" => Some(GameMode::MNA),
                    _ => return Err(ParseError::InvalidMode),
                };
            } else if key == "StackLeniency" {
                stack_leniency = Some(value.parse()?);
            }

            buf.clear();
        }

        self.mode = next_field!(mode);
        self.stack_leniency = next_field!(stack_leniency);

        Ok(empty)
    }

    fn parse_difficulty<R: Read>(
        &mut self,
        reader: &mut BufReader<R>,
        buf: &mut String,
        section: &mut Section,
    ) -> ParseResult<bool> {
        let mut ar = None;
        let mut od = None;
        let mut cs = None;
        let mut hp = None;
        let mut sv = None;
        let mut tick_rate = None;

        let mut empty = true;

        while reader.read_line(buf)? != 0 {
            let line = line_prepare!(buf);

            if line.starts_with('[') && line.ends_with(']') {
                *section = Section::from_str(&line[1..line.len() - 1]);
                empty = false;
                buf.clear();
                break;
            }

            let (key, value) = split_colon(&line).ok_or(ParseError::BadLine)?;

            match key {
                "ApproachRate" => ar = Some(value.parse()?),
                "OverallDifficulty" => od = Some(value.parse()?),
                "CircleSize" => cs = Some(value.parse()?),
                "HPDrainRate" => hp = Some(value.parse()?),
                "SliderTickRate" => tick_rate = Some(value.parse()?),
                "SliderMultiplier" => sv = Some(value.parse()?),
                _ => {}
            }

            buf.clear();
        }

        self.od = next_field!(od);
        self.cs = next_field!(cs);
        self.hp = next_field!(hp);
        self.ar = ar.unwrap_or(self.od);
        self.sv = next_field!(sv);
        self.tick_rate = next_field!(tick_rate);

        Ok(empty)
    }

    fn parse_timingpoints<R: Read>(
        &mut self,
        reader: &mut BufReader<R>,
        buf: &mut String,
        section: &mut Section,
    ) -> ParseResult<bool> {
        let mut unsorted_timings = false;
        let mut unsorted_difficulties = false;

        let mut prev_diff = 0.0;
        let mut prev_time = 0.0;

        let mut empty = true;

        while reader.read_line(buf)? != 0 {
            let line = line_prepare!(buf);

            if line.starts_with('[') && line.ends_with(']') {
                *section = Section::from_str(&line[1..line.len() - 1]);
                empty = false;
                buf.clear();
                break;
            }

            let mut split = line.split(',');

            let time = next_field!(split.next()).trim().parse::<f32>()?;
            validate_float!(time);

            let beat_len = next_field!(split.next()).trim().parse::<f32>()?;

            if beat_len < 0.0 {
                let point = DifficultyPoint {
                    time,
                    speed_multiplier: -100.0 / beat_len,
                };

                self.difficulty_points.push(point);

                if time < prev_diff {
                    unsorted_difficulties = true;
                } else {
                    prev_diff = time;
                }
            } else {
                let point = TimingPoint {
                    time,
                    bpm: 60_000.0 / beat_len,
                    beat_len,
                };

                self.timing_points.push(point);

                if time < prev_time {
                    unsorted_timings = true;
                } else {
                    prev_time = time;
                }
            }

            buf.clear();
        }

        if unsorted_timings {
            sort!(self.timing_points);
        }

        if unsorted_difficulties {
            sort!(self.difficulty_points);
        }

        Ok(empty)
    }

    fn parse_hitobjects<R: Read>(
        &mut self,
        reader: &mut BufReader<R>,
        buf: &mut String,
        section: &mut Section,
    ) -> ParseResult<bool> {
        let mut unsorted = false;
        let mut prev_time = 0.0;

        let mut empty = true;

        while reader.read_line(buf)? != 0 {
            let line = line_prepare!(buf);

            if line.starts_with('[') && line.ends_with(']') {
                *section = Section::from_str(&line[1..line.len() - 1]);
                empty = false;
                buf.clear();
                break;
            }

            let mut split = line.split(',');

            let pos = Pos2 {
                x: next_field!(split.next()).parse()?,
                y: next_field!(split.next()).parse()?,
            };

            let time = next_field!(split.next()).trim().parse::<f32>()?;
            validate_float!(time);

            if !self.hit_objects.is_empty() && time < prev_time {
                unsorted = true;
            }

            let kind: u8 = next_field!(split.next()).parse()?;
            let sound = split.next().map(str::parse).transpose()?.unwrap_or(0);

            let kind = if kind & Self::CIRCLE_FLAG > 0 {
                self.n_circles += 1;

                HitObjectKind::Circle
            } else if kind & Self::SLIDER_FLAG > 0 {
                self.n_sliders += 1;
                let mut curve_points = Vec::with_capacity(16);
                curve_points.push(pos);

                let mut curve_point_iter = next_field!(split.next()).split('|');

                let mut path_type: PathType = next_field!(curve_point_iter.next()).parse()?;

                for pos in curve_point_iter {
                    let mut v = pos.split(':').map(str::parse);

                    match (v.next(), v.next()) {
                        (Some(Ok(x)), Some(Ok(y))) => curve_points.push(Pos2 { x, y }),
                        _ => return Err(ParseError::InvalidCurvePoints),
                    }
                }

                if self.version <= 6 && curve_points.len() >= 2 {
                    if path_type == PathType::Linear {
                        path_type = PathType::Bezier;
                    }

                    if curve_points.len() == 2 && (pos == curve_points[0] || pos == curve_points[1])
                    {
                        path_type = PathType::Linear;
                    }
                }

                if curve_points.is_empty() {
                    HitObjectKind::Circle
                } else {
                    let repeats = next_field!(split.next()).parse::<usize>()?;
                    let len: f32 = next_field!(split.next()).parse()?;

                    HitObjectKind::Slider {
                        repeats,
                        pixel_len: len,
                        curve_points,
                        path_type,
                    }
                }
            } else if kind & Self::SPINNER_FLAG > 0 {
                self.n_spinners += 1;
                let end_time = next_field!(split.next()).parse()?;

                HitObjectKind::Spinner { end_time }
            } else if kind & Self::HOLD_FLAG > 0 {
                self.n_sliders += 1;
                let mut end = time;

                if let Some(next) = split.next() {
                    end = end.max(next_field!(next.split(':').next()).parse()?);
                }

                HitObjectKind::Hold { end_time: end }
            } else {
                return Err(ParseError::UnknownHitObjectKind);
            };

            self.hit_objects.push(HitObject {
                pos,
                start_time: time,
                kind,
                sound,
            });

            prev_time = time;
            buf.clear();
        }

        // BUG: If [General] section comes after [HitObjects] then the mode
        // won't be set yet so mania objects won't be sorted properly
        if self.mode == GameMode::MNA {
            sort(&mut self.hit_objects);
        } else if unsorted {
            sort!(self.hit_objects);
        }

        Ok(empty)
    }

    #[inline]
    pub fn attributes(&self) -> BeatmapAttributes {
        BeatmapAttributes::new(self.ar, self.od, self.cs, self.hp)
    }
}

#[inline]
fn split_colon(line: &str) -> Option<(&str, &str)> {
    let mut split = line.split(':');

    Some((split.next()?, split.next()?.trim()))
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PathType {
    Catmull = 0,
    Bezier = 1,
    Linear = 2,
    PerfectCurve = 3,
}

impl FromStr for PathType {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(Self::Linear),
            "C" => Ok(Self::Catmull),
            "B" => Ok(Self::Bezier),
            "P" => Ok(Self::PerfectCurve),
            _ => Err(ParseError::InvalidPathType),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Section {
    None,
    General,
    Difficulty,
    TimingPoints,
    HitObjects,
}

impl Section {
    #[inline]
    fn from_str(s: &str) -> Self {
        match s {
            "General" => Self::General,
            "Difficulty" => Self::Difficulty,
            "TimingPoints" => Self::TimingPoints,
            "HitObjects" => Self::HitObjects,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn parsing() {
        let file = match File::open("./maps/61843.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not read file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        println!("Mode: {}", map.mode as u8);
        println!("n_circles: {}", map.n_circles);
        println!("n_sliders: {}", map.n_sliders);
        println!("n_spinners: {}", map.n_spinners);
        println!("ar: {}", map.ar);
        println!("od: {}", map.od);
        println!("cs: {}", map.cs);
        println!("hp: {}", map.hp);
        println!("sv: {}", map.sv);
        println!("tick_rate: {}", map.tick_rate);
        println!("stack_leniency: {}", map.stack_leniency);
        println!("hit_objects: {}", map.hit_objects.len());
        println!("timing_points: {}", map.timing_points.len());
        println!("difficulty_points: {}", map.difficulty_points.len());
    }
}
