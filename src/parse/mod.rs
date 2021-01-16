mod attributes;
mod control_point;
mod error;
mod hitobject;
mod hitsound;
mod mods;
mod pos2;
mod sort;

pub use attributes::BeatmapAttributes;
pub use control_point::{DifficultyPoint, TimingPoint};
pub use error::{ParseError, ParseResult};
pub use hitobject::{HitObject, HitObjectKind};
pub use hitsound::HitSound;
pub use mods::Mods;
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
    ($opt:expr, $nmbr:ident) => {
        $opt.ok_or(ParseError::MissingField($nmbr))?
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

        // version 4 and lower had an incorrect offset (stable has this set as 24ms off)
        let offset = if map.version < 5 { 24.0 } else { 0.0 };

        buf.clear();
        map.hit_objects.reserve(256);

        let mut mode = None;
        let mut ar = None;
        let mut od = None;
        let mut cs = None;
        let mut hp = None;
        let mut sv = None;
        let mut tick_rate = None;
        let mut stack_leniency = None;

        let mut section = Section::None;
        let mut prev_time = 0.0;
        let mut prev_diff = 0.0;
        let mut unsorted_timings = false;
        let mut unsorted_difficulties = false;
        let mut unsorted_hits = false;

        let mut nmbr = 1;

        while reader.read_line(&mut buf)? != 0 {
            let mut line = buf.trim_end();
            nmbr += 1;

            if line.is_empty()
                || line.starts_with("//")
                || line.starts_with(' ')
                || line.starts_with('_')
            {
                buf.clear();
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                section = Section::from_str(&line[1..line.len() - 1]);
                buf.clear();
                continue;
            }

            if let Some(idx) = line.find("//") {
                line = &line[..idx];
            }

            match section {
                Section::General => {
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
                }
                Section::Difficulty => {
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
                }
                Section::TimingPoints => {
                    let mut split = line.split(',');

                    let time = offset + next_field!(split.next(), nmbr).trim().parse::<f32>()?;
                    validate_float!(time);

                    let beat_len = next_field!(split.next(), nmbr).trim().parse::<f32>()?;

                    if beat_len.is_sign_negative() {
                        let point = DifficultyPoint {
                            time,
                            speed_multiplier: -100.0 / beat_len,
                        };

                        map.difficulty_points.push(point);

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

                        map.timing_points.push(point);

                        if time < prev_time {
                            unsorted_timings = true;
                        } else {
                            prev_time = time;
                        }
                    }
                }

                Section::HitObjects => {
                    let mut split = line.split(',');

                    let pos = Pos2 {
                        x: next_field!(split.next(), nmbr).parse()?,
                        y: next_field!(split.next(), nmbr).parse()?,
                    };

                    let time = offset + next_field!(split.next(), nmbr).trim().parse::<f32>()?;
                    validate_float!(time);

                    if !map.hit_objects.is_empty() && time < prev_time {
                        unsorted_hits = true;
                    }

                    let kind: u8 = next_field!(split.next(), nmbr).parse()?;
                    let sound = split.next().map(str::parse).transpose()?.unwrap_or(0);

                    let kind = if kind & Self::CIRCLE_FLAG > 0 {
                        map.n_circles += 1;

                        HitObjectKind::Circle
                    } else if kind & Self::SLIDER_FLAG > 0 {
                        map.n_sliders += 1;
                        let mut curve_points = Vec::with_capacity(16);
                        curve_points.push(pos);

                        let mut curve_point_iter = next_field!(split.next(), nmbr).split('|');

                        let mut path_type: PathType =
                            next_field!(curve_point_iter.next(), nmbr).parse()?;

                        for pos in curve_point_iter {
                            let mut v = pos.split(':').map(str::parse);

                            match (v.next(), v.next()) {
                                (Some(Ok(x)), Some(Ok(y))) => curve_points.push(Pos2 { x, y }),
                                _ => return Err(ParseError::InvalidCurvePoints),
                            }
                        }

                        if map.version <= 6 && curve_points.len() >= 2 {
                            if path_type == PathType::Linear {
                                path_type = PathType::Bezier;
                            }

                            if curve_points.len() == 2
                                && (pos == curve_points[0] || pos == curve_points[1])
                            {
                                path_type = PathType::Linear;
                            }
                        }

                        if curve_points.is_empty() {
                            HitObjectKind::Circle
                        } else {
                            let repeats = next_field!(split.next(), nmbr).parse::<usize>()?;
                            let len: f32 = next_field!(split.next(), nmbr).parse()?;

                            HitObjectKind::Slider {
                                repeats,
                                pixel_len: len,
                                curve_points,
                                path_type,
                            }
                        }
                    } else if kind & Self::SPINNER_FLAG > 0 {
                        map.n_spinners += 1;
                        let end_time = next_field!(split.next(), nmbr).parse()?;

                        HitObjectKind::Spinner { end_time }
                    } else if kind & Self::HOLD_FLAG > 0 {
                        map.n_sliders += 1;
                        let mut end = time;

                        if let Some(next) = split.next() {
                            end = end.max(next_field!(next.split(':').next(), nmbr).parse()?);
                        }

                        HitObjectKind::Hold { end_time: end }
                    } else {
                        return Err(ParseError::UnknownHitObjectKind);
                    };

                    map.hit_objects.push(HitObject {
                        pos,
                        start_time: time,
                        kind,
                        sound,
                    });

                    prev_time = time;
                }

                Section::None => {}
            }

            buf.clear();
        }

        map.mode = next_field!(mode, nmbr);
        map.ar = next_field!(ar, nmbr);
        map.od = next_field!(od, nmbr);
        map.cs = next_field!(cs, nmbr);
        map.hp = next_field!(hp, nmbr);
        map.sv = next_field!(sv, nmbr);
        map.tick_rate = next_field!(tick_rate, nmbr);
        map.stack_leniency = next_field!(stack_leniency, nmbr);

        if unsorted_timings {
            sort!(map.timing_points);
        }

        if unsorted_difficulties {
            sort!(map.difficulty_points);
        }

        if map.mode == GameMode::MNA {
            sort(&mut map.hit_objects);
        } else if unsorted_hits {
            sort!(map.hit_objects);
        }

        Ok(map)
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
    fn parsing_works() {
        let file = match File::open("E:/Games/osu!/beatmaps/2223745.osu") {
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

        assert_eq!(2 + 2, 4);
    }
}
