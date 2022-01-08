use std::{
    env, fmt,
    fs::File as StdFile,
    io,
    path::PathBuf,
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
    time::Instant,
};

use futures::{future, stream::FuturesUnordered, Stream, StreamExt, TryStreamExt};
use pbr::ProgressBar;
use rosu_pp::{Beatmap, GameMode};
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use tokio::{
    fs::{self, DirEntry, File, ReadDir},
    process::Command,
    runtime::Runtime,
};

const OSU_MODS: &[&[&str]] = &[
    &[""],
    &["hd"],
    &["hr"],
    &["dt"],
    &["fl"],
    &["ez"],
    &["ht"],
    &["hd", "fl"],
    &["hr", "dt"],
    &["ez", "dt"],
    &["ht", "ez"],
    &["hd", "hr", "dt"],
];

const TAIKO_MODS: &[&[&str]] = &[
    &[""],
    &["hd"],
    &["hr"],
    &["ht"],
    &["dt"],
    &["hr", "dt"],
    &["ez", "dt"],
];

const CATCH_MODS: &[&[&str]] = &[
    &[""],
    &["hd"],
    &["ht"],
    &["dt"],
    &["ez"],
    &["hd", "dt"],
    &["hr", "dt"],
];

const MANIA_MODS: &[&[&str]] = &[
    &[""],
    &["ht"],
    &["dt"],
    &["ez"],
    &["ez", "ht"],
    &["ez", "nf", "ht"],
];

const OSU: &str = "osu";
const TAIKO: &str = "taiko";
const CATCH: &str = "catch";
const MANIA: &str = "mania";

fn main() {
    dotenv::dotenv().expect("failed to read .env file");

    Runtime::new()
        .expect("failed to create runtime")
        .block_on(async_main());
}

async fn async_main() {
    let start = Instant::now();
    let map_path = env::var("MAP_PATH").expect("missing `MAP_PATH` environment variable");

    let perf_calc_path_ =
        env::var("PERF_CALC_PATH").expect("missing `PERF_CALC_PATH` environment variable");
    let perf_calc_path = perf_calc_path_.as_str();

    let mut output = StdFile::create("./output.json").expect("failed to create output file");

    let take = env::var("MAP_TAKE")
        .expect("missing `MAP_TAKE` environment variable")
        .parse()
        .expect("`MAP_TAKE` must be an integer");

    let mut skip: u32 = env::var("MAP_SKIP")
        .expect("missing `MAP_SKIP` environment variable")
        .parse()
        .expect("`MAP_SKIP` must be an integer");
    skip += 1;

    let pbr = Mutex::new(ProgressBar::new(take));
    println!("[INFO] Calculating...");

    let result = fs::read_dir(map_path)
        .await
        .map(ReadDirStream::new)
        .expect("failed to open directory at `MAP_PATH`")
        .scan(0, |idx, entry| {
            *idx += 1;

            future::ready(Some((*idx % skip == 0).then(|| entry)))
        })
        .filter_map(future::ready)
        .take(take as usize)
        .map(|dir_entry| async {
            let dir_entry = dir_entry?;
            let file_name = dir_entry.file_name();
            let file_name_lossy = file_name.to_string_lossy();
            let file_path = dir_entry.path();

            let map_id_str = file_name_lossy.split('.').next();

            let map_id = match map_id_str.map(str::parse) {
                Some(Ok(id)) => id,
                Some(Err(_)) => return Err(Error::ParseId(map_id_str.unwrap().to_owned())),
                None => return Err(Error::EmptyFileName),
            };

            let file = File::open(&file_path).await?;
            let mode = Beatmap::parse(file).await?.mode;

            let result = handle_map(mode, map_id, file_path, perf_calc_path).await;

            if let Ok(mut progress) = pbr.lock() {
                progress.inc();
            }

            result
        })
        .collect::<FuturesUnordered<_>>()
        .await
        .try_collect::<Vec<Vec<SimulateData>>>()
        .await;

    pbr.lock().unwrap().finish_println(&format!(
        "[INFO] [{:?}] Finished calculating, now flattening...\n",
        start.elapsed(),
    ));

    let mut mode_counts = [0; 4];

    let data: Vec<SimulateData> = match result {
        Ok(data) => data
            .into_iter()
            .map(|d| {
                if let Some(data) = d.get(0) {
                    mode_counts[data.score.mode as usize] += 1;
                }

                d.into_iter()
            })
            .flatten()
            .collect(),
        Err(err) => return print_err(err),
    };

    println!(
        "[INFO] [{:?}] Flattened {} calculations, now writing to file...",
        start.elapsed(),
        data.len()
    );

    match serde_json::to_writer(&mut output, &data) {
        Ok(_) => println!(
            "[INFO] [{:?}] Finished writing data into `output.json`\n\
            Maps:\n  - osu: {}\n  - taiko: {}\n  - catch: {}\n  - mania: {}",
            start.elapsed(),
            mode_counts[0],
            mode_counts[1],
            mode_counts[2],
            mode_counts[3]
        ),
        Err(err) => print_err(err.into()),
    }
}

fn print_err(err: Error) {
    let mut e: &dyn std::error::Error = &err;
    eprintln!("[ERROR] {}", err);

    while let Some(src) = e.source() {
        eprintln!("[ERROR]  - caused by: {}", src);
        e = src;
    }
}

async fn handle_map(
    mode: GameMode,
    map_id: u32,
    path: PathBuf,
    perf_calc_path: &str,
) -> Result<Vec<SimulateData>, Error> {
    let (mods, mode_str) = match mode {
        GameMode::STD => (OSU_MODS, OSU),
        GameMode::TKO => (TAIKO_MODS, TAIKO),
        GameMode::CTB => (CATCH_MODS, CATCH),
        GameMode::MNA => (MANIA_MODS, MANIA),
    };

    let mut result = Vec::with_capacity(mods.len());

    for mods_ in mods {
        let mut command = Command::new("dotnet");

        command
            .arg(perf_calc_path)
            .arg("simulate")
            .arg(mode_str)
            .arg(&path)
            .arg("--json");

        if !mods_[0].is_empty() {
            for &m in mods_.iter() {
                command.arg("-m").arg(m);
            }
        }

        let output = command.output().await?;

        let mut data: SimulateData = match serde_json::from_slice(&output.stdout) {
            Ok(data) => data,
            Err(_) => {
                let content = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "[ERROR] mods={:?} | mode={:?} | map={}\n{}",
                    mods_, mode, map_id, content
                );

                continue;
            }
        };

        data.score.map_id = map_id;
        result.push(data);
    }

    Ok(result)
}

#[derive(Debug)]
enum Error {
    EmptyFileName,
    Io(std::io::Error),
    ParseId(String),
    ParseMap(rosu_pp::ParseError),
    Serde(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFileName => f.write_str("empty file name"),
            Self::Io(_) => f.write_str("io error"),
            Self::ParseId(s) => write!(f, "failed to parse map id from `{}`", s),
            Self::ParseMap(_) => f.write_str("failed to parse map"),
            Self::Serde(_) => f.write_str("failed to deserialize"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmptyFileName => None,
            Self::Io(src) => Some(src),
            Self::ParseId(_) => None,
            Self::ParseMap(src) => Some(src),
            Self::Serde(src) => Some(src),
        }
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

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SimulateData {
    score: Score,
    #[serde(alias = "performance_attributes")]
    performance: Performance,
    #[serde(alias = "difficulty_attributes")]
    difficulty: Difficulty,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Score {
    #[serde(alias = "ruleset_id")]
    mode: u32,
    #[serde(alias = "beatmap_id")]
    map_id: u32,
    #[serde(alias = "beatmap", skip_serializing)]
    _map: String,
    #[serde(deserialize_with = "deserialize_mods")]
    mods: Vec<String>,
    total_score: u32,
    #[serde(alias = "accuracy")]
    acc: f64,
    combo: u32,
    #[serde(alias = "statistics")]
    stats: Statistics,
}

fn deserialize_mods<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    d.deserialize_seq(ModVisitor)
}

struct ModVisitor;

impl<'de> Visitor<'de> for ModVisitor {
    type Value = Vec<String>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence of mods")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut mods = Vec::with_capacity(seq.size_hint().unwrap_or(0));

        #[derive(Deserialize)]
        struct Mod {
            acronym: String,
        }

        while let Some(elem) = seq.next_element::<Mod>()? {
            mods.push(elem.acronym);
        }

        Ok(mods)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Statistics {
    #[serde(alias = "Perfect", default)]
    perfect: usize,
    #[serde(alias = "Great")]
    great: usize,
    #[serde(alias = "Good", alias = "SmallTickMiss", default)]
    good: usize,
    #[serde(alias = "Ok", alias = "LargeTickHit")]
    ok: usize,
    #[serde(alias = "Meh", alias = "SmallTickHit")]
    meh: usize,
    #[serde(alias = "Miss")]
    miss: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Performance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    aim: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    speed: Option<f64>,
    #[serde(alias = "accuracy", default, skip_serializing_if = "Option::is_none")]
    acc: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flashlight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    effective_miss_count: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scaled_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    difficulty: Option<f64>,
    pp: f64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Difficulty {
    #[serde(alias = "star_rating")]
    stars: f64,
    max_combo: u32,
    #[serde(
        alias = "aim_difficulty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    aim: Option<f64>,
    #[serde(
        alias = "speed_difficulty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    speed: Option<f64>,
    #[serde(
        alias = "flashlight_difficulty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    flashlight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    slider_factor: Option<f64>,
    #[serde(
        alias = "stamina_difficulty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    stamina: Option<f64>,
    #[serde(
        alias = "rhythm_difficulty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    rhythm: Option<f64>,
    #[serde(
        alias = "colour_difficulty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    colour: Option<f64>,
    #[serde(
        alias = "approach_rate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    ar: Option<f64>,
    #[serde(
        alias = "overall_difficulty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    od: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    great_hit_window: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    score_multiplier: Option<f64>,
}

struct ReadDirStream {
    inner: ReadDir,
}

impl ReadDirStream {
    fn new(read_dir: ReadDir) -> Self {
        Self { inner: read_dir }
    }
}

impl Stream for ReadDirStream {
    type Item = io::Result<DirEntry>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_entry(cx).map(Result::transpose)
    }
}
