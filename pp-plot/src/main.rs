use std::{env, error::Error as StdError, fmt, fs::File as StdFile};

use futures::{stream::FuturesUnordered, TryStreamExt};
use plotters::{
    data::fitting_range,
    prelude::{
        Boxplot, ChartBuilder, DrawingAreaErrorKind, IntoDrawingArea, IntoSegmentedCoord,
        Quartiles, SVGBackend, SegmentValue,
    },
    style::WHITE,
};
use rosu_pp::{Beatmap, BeatmapExt, PerformanceAttributes};
use serde::Deserialize;
use tokio::{fs::File, runtime::Runtime};

fn main() {
    dotenv::dotenv().expect("failed to read .env file");

    Runtime::new()
        .expect("failed to create runtime")
        .block_on(async_main());
}

async fn async_main() {
    let map_path_ = env::var("MAP_PATH").expect("missing `MAP_PATH` environment variable");
    let map_path = map_path_.as_str();

    println!("Deserializing data from output.json...");

    let file = StdFile::open("../pp-gen/output.json").expect("failed to open `output.json` file");
    let data: Vec<SimulateData> =
        serde_json::from_reader(file).expect("failed to deserialize data");

    println!(
        "Calculating values for {} map-mod combinations...",
        data.len()
    );

    // Calculate rosu-pp's PerformanceAttributes on all map-mod pairs
    let result = data
        .into_iter()
        .map(|data| async move {
            let path = format!("{}/{}.osu", map_path, data.score.map_id);
            let file = File::open(path).await?;
            let map = Beatmap::parse(file).await?;
            let mods = parse_mods(&data.score.mods);
            let attrs = map.max_pp(mods);

            Ok::<_, Error>((data, attrs, mods))
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await;

    let tuples = match result {
        Ok(attrs) => attrs,
        Err(err) => return print_err(err),
    };

    println!("Evaluating values...");

    // Compare the values from output.json with the PerformanceAttribute values
    let mut evaluators = [
        Evaluator::new("osu"),
        Evaluator::new("taiko"),
        Evaluator::new("catch"),
        Evaluator::new("mania"),
    ];

    for (data, attributes, mods) in tuples {
        evaluators[data.score.mode as usize].process(data, attributes, mods);
    }

    for evaluator in evaluators {
        let mode = evaluator.mode;

        if let Err(err) = evaluator.plot() {
            eprintln!("failed to plot for {}", mode);
            print_err(err);
        }
    }
}

/// Mode specific evaluator containing differences
/// of values from `Data` and `PerformanceAttributes`.
#[derive(Default)]
struct Evaluator {
    mode: &'static str,
    count: usize,

    aim: Option<Vec<f64>>,
    accuracy: Option<Vec<f64>>,
    flashlight: Option<Vec<f64>>,
    speed: Option<Vec<f64>>,
    strain: Option<Vec<f64>>,

    stars: Vec<f64>,
    pp: Vec<f64>,
}

impl Evaluator {
    fn new(mode: &'static str) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// For all mode-specific data points, calculate the
    /// differences of `data`'s value and `attrs`' value
    fn process(&mut self, data: SimulateData, attrs: PerformanceAttributes, mods: u32) {
        self.count += 1;
        self.stars
            .push(difference(data.difficulty.stars, attrs.stars()));
        self.pp.push(difference(data.performance.pp, attrs.pp()));

        match attrs {
            PerformanceAttributes::Catch(_) => {}
            PerformanceAttributes::Mania(attrs) => {
                if let Some(acc) = data.performance.acc {
                    let values = self.accuracy.get_or_insert_with(Vec::new);
                    let entry = difference(acc, attrs.pp_acc);
                    values.push(entry);
                }

                if let Some(strain) = data.performance.difficulty {
                    let values = self.strain.get_or_insert_with(Vec::new);
                    let entry = difference(strain, attrs.pp_strain);
                    values.push(entry);
                }
            }
            PerformanceAttributes::Osu(attrs) => {
                if let Some(acc) = data.performance.acc {
                    let values = self.accuracy.get_or_insert_with(Vec::new);
                    let entry = difference(acc, attrs.pp_acc);
                    values.push(entry);
                }

                if let Some(aim) = data.performance.aim {
                    let values = self.aim.get_or_insert_with(Vec::new);
                    let entry = difference(aim, attrs.pp_aim);
                    values.push(entry);
                }

                if mods & 1024 > 0 {
                    if let Some(flashlight) = data.performance.flashlight {
                        let values = self.flashlight.get_or_insert_with(Vec::new);
                        let entry = difference(flashlight, attrs.pp_flashlight);
                        values.push(entry);
                    }
                }

                if let Some(speed) = data.performance.speed {
                    let values = self.speed.get_or_insert_with(Vec::new);
                    let entry = difference(speed, attrs.pp_speed);
                    values.push(entry);
                }
            }
            PerformanceAttributes::Taiko(attrs) => {
                if let Some(acc) = data.performance.acc {
                    let values = self.accuracy.get_or_insert_with(Vec::new);
                    let entry = difference(acc, attrs.pp_acc);
                    values.push(entry);
                }

                if let Some(strain) = data.performance.difficulty {
                    let values = self.strain.get_or_insert_with(Vec::new);
                    let entry = difference(strain, attrs.pp_strain);
                    values.push(entry);
                }
            }
        }
    }

    /// Plot all gathered differences
    fn plot(self) -> Result<(), Error> {
        let mode = self.mode;
        let output_path = format!("accuracy_{}.svg", mode);
        let dataset = self.to_quartiles();
        let kind_list: Vec<_> = dataset.iter().map(|(kind, _)| *kind).collect();

        let height = kind_list.len() as u32 * 128;
        let root = SVGBackend::new(&output_path, (1024, height)).into_drawing_area();
        root.fill(&WHITE)?;
        let root = root.margin(5, 5, 15, 15);

        let values = dataset
            .iter()
            .map(|(_, quartiles)| quartiles.values())
            .flatten()
            .collect::<Vec<_>>();

        let values_range = fitting_range(values.iter());
        let caption = format!("{} ({} data points)", mode, self.count);

        // Set the chart structure
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .caption(caption, ("sans-serif", 20))
            .build_cartesian_2d(
                0.0..values_range.end + values_range.end * 0.2,
                kind_list[..].into_segmented(),
            )?;

        chart
            .configure_mesh()
            .x_desc("Away from actual value")
            .y_labels(kind_list.len())
            .light_line_style(&WHITE)
            .draw()?;

        // Insert data into the chart
        for (kind, quartile) in dataset.iter() {
            chart.draw_series(std::iter::once(
                Boxplot::new_horizontal(SegmentValue::CenterOf(kind), quartile)
                    .width(20)
                    .whisker_width(0.5),
            ))?;
        }

        root.present()?;

        Ok(())
    }

    fn to_quartiles(&self) -> Vec<(&'static str, Quartiles)> {
        let mut vec = Vec::new();

        println!("---");

        let max = self
            .stars
            .iter()
            .fold(0.0, |m, &n| if n > m { n } else { m });
        let avg = self.stars.iter().copied().sum::<f64>() / self.stars.len() as f64;
        println!("[{}] Stars: average={} | max={}", self.mode, avg, max);
        vec.push(("stars", Quartiles::new(&self.stars)));

        let max = self.pp.iter().fold(0.0, |m, &n| if n > m { n } else { m });
        let avg = self.pp.iter().copied().sum::<f64>() / self.pp.len() as f64;
        println!("[{}] PP: average={} | max={}", self.mode, avg, max);
        vec.push(("pp", Quartiles::new(&self.pp)));

        if let Some(ref acc) = self.accuracy {
            if !acc.is_empty() {
                let max = acc.iter().fold(0.0, |m, &n| if n > m { n } else { m });
                let avg = acc.iter().copied().sum::<f64>() / acc.len() as f64;
                println!("[{}] Accuracy: average={} | max={}", self.mode, avg, max);
            }

            vec.push(("accuracy pp", Quartiles::new(acc)));
        }

        if let Some(ref aim) = self.aim {
            if !aim.is_empty() {
                let max = aim.iter().fold(0.0, |m, &n| if n > m { n } else { m });
                let avg = aim.iter().copied().sum::<f64>() / aim.len() as f64;
                println!("[{}] Aim: average={} | max={}", self.mode, avg, max);
            }

            vec.push(("aim pp", Quartiles::new(aim)));
        }

        if let Some(ref fl) = self.flashlight {
            if !fl.is_empty() {
                let max = fl.iter().fold(0.0, |m, &n| if n > m { n } else { m });
                let avg = fl.iter().copied().sum::<f64>() / fl.len() as f64;
                println!("[{}] Flashlight: average={} | max={}", self.mode, avg, max);
            }

            vec.push(("flashlight pp", Quartiles::new(fl)));
        }

        if let Some(ref speed) = self.speed {
            if !speed.is_empty() {
                let max = speed.iter().fold(0.0, |m, &n| if n > m { n } else { m });
                let avg = speed.iter().copied().sum::<f64>() / speed.len() as f64;
                println!("[{}] Speed: average={} | max={}", self.mode, avg, max);
            }

            vec.push(("speed pp", Quartiles::new(speed)));
        }

        if let Some(ref strain) = self.strain {
            if !strain.is_empty() {
                let max = strain.iter().fold(0.0, |m, &n| if n > m { n } else { m });
                let avg = strain.iter().copied().sum::<f64>() / strain.len() as f64;
                println!("[{}] Strain: average={} | max={}", self.mode, avg, max);
            }

            vec.push(("strain pp", Quartiles::new(strain)));
        }

        vec.reverse();

        vec
    }
}

#[derive(Debug)]
enum Error {
    DrawingArea(String),
    Io(std::io::Error),
    ParseMap(rosu_pp::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DrawingArea(src) => write!(f, "drawing area error: {}", src),
            Self::Io(_) => f.write_str("io error"),
            Self::ParseMap(_) => f.write_str("failed to parse map"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::DrawingArea(_) => None,
            Self::Io(src) => Some(src),
            Self::ParseMap(src) => Some(src),
        }
    }
}

impl<E: StdError + Send + Sync> From<DrawingAreaErrorKind<E>> for Error {
    fn from(e: DrawingAreaErrorKind<E>) -> Self {
        Self::DrawingArea(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rosu_pp::ParseError> for Error {
    fn from(e: rosu_pp::ParseError) -> Self {
        Self::ParseMap(e)
    }
}

fn difference(actual: f64, calculated: f64) -> f64 {
    (actual - calculated).abs()
}

fn parse_mods(mods_list: &[String]) -> u32 {
    let mut mods = 0;

    for m in mods_list {
        match m.as_str() {
            "NF" => mods += 1,
            "EZ" => mods += 2,
            "TD" => mods += 4,
            "HD" => mods += 8,
            "HR" => mods += 16,
            "DT" => mods += 64,
            "RX" => mods += 128,
            "HT" => mods += 256,
            "FL" => mods += 1024,
            _ => panic!("unrecognized mod: {}", m),
        }
    }

    mods
}

fn print_err(err: Error) {
    let mut e: &dyn StdError = &err;
    eprintln!("{}", err);

    while let Some(src) = e.source() {
        eprintln!("  - caused by: {}", src);
        e = src;
    }
}

#[derive(Deserialize)]
struct SimulateData {
    score: Score,
    performance: Performance,
    difficulty: Difficulty,
}

#[derive(Deserialize)]
struct Score {
    mode: u32,
    map_id: u32,
    mods: Vec<String>,
    total_score: u32,
    acc: f64,
    combo: u32,
    stats: Statistics,
}

#[derive(Deserialize)]
struct Statistics {
    #[serde(default)]
    perfect: usize,
    great: usize,
    #[serde(default)]
    good: usize,
    ok: usize,
    meh: usize,
    miss: usize,
}

#[derive(Deserialize)]
struct Performance {
    #[serde(default)]
    aim: Option<f64>,
    #[serde(default)]
    speed: Option<f64>,
    #[serde(default)]
    acc: Option<f64>,
    #[serde(default)]
    flashlight: Option<f64>,
    #[serde(default)]
    effective_miss_count: Option<f64>,
    #[serde(default)]
    scaled_score: Option<f64>,
    #[serde(default)]
    difficulty: Option<f64>,
    pp: f64,
}

#[derive(Deserialize)]
struct Difficulty {
    stars: f64,
    max_combo: u32,
    #[serde(default)]
    aim: Option<f64>,
    #[serde(default)]
    speed: Option<f64>,
    #[serde(default)]
    flashlight: Option<f64>,
    #[serde(default)]
    slider_factor: Option<f64>,
    #[serde(default)]
    stamina: Option<f64>,
    #[serde(default)]
    rhythm: Option<f64>,
    #[serde(default)]
    colour: Option<f64>,
    #[serde(default)]
    ar: Option<f64>,
    #[serde(default)]
    od: Option<f64>,
    #[serde(default)]
    great_hit_window: Option<f64>,
    #[serde(default)]
    score_multiplier: Option<f64>,
}
