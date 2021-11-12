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
use serde::{Deserialize, Serialize};
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
        .expect("failed to create runtiem")
        .block_on(async_main());
}

async fn async_main() {
    let start = Instant::now();
    let map_path = env::var("MAP_PATH").expect("missing `MAP_PATH` environment variable");

    let perf_calc_path_ =
        env::var("PERF_CALC_PATH").expect("missing `PERF_CALC_PATH` environment variable");
    let perf_calc_path = perf_calc_path_.as_str();

    let mut output = StdFile::create("./output_.json").expect("failed to create output file");

    let take = 50;

    let pbr = Mutex::new(ProgressBar::new(take));
    println!("[INFO] Calculating...");

    let result = fs::read_dir(map_path)
        .await
        .map(ReadDirStream::new)
        .expect("failed to open directory at `MAP_PATH`")
        .scan(0, |idx, entry| {
            *idx += 1;

            future::ready(Some((*idx % 75 == 0).then(|| entry)))
        })
        .filter_map(|opt| future::ready(opt))
        .take(take as usize)
        .map(|dir_entry| async {
            let dir_entry = dir_entry?;
            let file_name = dir_entry.file_name();
            let file_path = dir_entry.path();

            let map_id = match file_name
                .to_string_lossy()
                .split('.')
                .next()
                .map(str::parse)
                .transpose()
                .map_err(|_| Error::ParseId)?
            {
                Some(id) => id,
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
        .try_collect::<Vec<Vec<Data>>>()
        .await;

    pbr.lock().unwrap().finish_println(&format!(
        "[INFO] [{:?}] Finished calculating, now flattening...\n",
        start.elapsed(),
    ));

    let mut mode_counts = [0; 4];

    let data: Vec<Data> = match result {
        Ok(data) => data
            .into_iter()
            .map(|d| {
                if let Some(data) = d.get(0) {
                    mode_counts[data.mode as usize] += 1;
                }

                d.into_iter()
            })
            .flatten()
            .collect(),
        Err(err) => return print_err(err.into()),
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
) -> Result<Vec<Data>, Error> {
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

        let data = match serde_json::from_slice(&output.stdout) {
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

        let data = Data::new(mode as u32, map_id, data);
        result.push(data);
    }

    Ok(result)
}

#[derive(Debug)]
enum Error {
    EmptyFileName,
    Io(std::io::Error),
    ParseId,
    ParseMap(rosu_pp::ParseError),
    Serde(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptyFileName => f.write_str("empty file name"),
            Error::Io(_) => f.write_str("io error"),
            Error::ParseId => f.write_str("failed to parse map id"),
            Error::ParseMap(_) => f.write_str("failed to parse map"),
            Error::Serde(_) => f.write_str("failed to deserialize"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmptyFileName => None,
            Self::Io(src) => Some(src),
            Self::ParseId => None,
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

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    mode: u32,
    map_id: u32,
    #[serde(flatten)]
    inner: GenericData,
}

impl Data {
    fn new(mode: u32, map_id: u32, inner: GenericData) -> Self {
        Self {
            mode,
            map_id,
            inner,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct GenericData {
    #[serde(default, alias = "Aim", skip_serializing_if = "Option::is_none")]
    aim: Option<f64>,
    #[serde(default, alias = "Speed", skip_serializing_if = "Option::is_none")]
    speed: Option<f64>,
    #[serde(default, alias = "Accuracy", skip_serializing_if = "Option::is_none")]
    accuracy: Option<f64>,
    #[serde(default, alias = "Flashlight", skip_serializing_if = "Option::is_none")]
    flashlight: Option<f64>,
    #[serde(default, alias = "Strain", skip_serializing_if = "Option::is_none")]
    strain: Option<f64>,
    #[serde(default, alias = "OD", skip_serializing_if = "Option::is_none")]
    od: Option<f64>,
    #[serde(default, alias = "AR", skip_serializing_if = "Option::is_none")]
    ar: Option<f64>,
    #[serde(alias = "Mods")]
    mods: String,
    #[serde(alias = "Stars")]
    stars: f64,
    pp: f64,
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
