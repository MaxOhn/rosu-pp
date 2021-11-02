use std::{env, fs::File as StdFile};

use futures::{stream::FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt, process::Command, runtime::Runtime};

macro_rules! info {
    ($($args:tt)*) => {
        println!("[INFO] {}", format_args!($($args)*))
    }
}

macro_rules! error {
    ($($args:tt)*) => {
        eprintln!("[ERROR] {}", format_args!($($args)*))
    }
}

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
    let runtime = Runtime::new().expect("failed to create runtiem");

    for mode in [OSU, TAIKO, CATCH, MANIA] {
        runtime.block_on(handle_mode(mode));
    }
}

async fn handle_mode(mode: &'static str) {
    let perf_calc_path_ =
        env::var("PERF_CALC_PATH").expect("missing `PERF_CALC_PATH` environment variable");
    let perf_calc_path = perf_calc_path_.as_str();

    let map_path_ = env::var("MAP_PATH").expect("missing `MAP_PATH` environment variable");
    let map_path = map_path_.as_str();

    let input_filename = format!("./input/{}.csv", mode);

    let mut file = match File::open(&input_filename).await {
        Ok(file) => file,
        Err(err) => {
            return error!(
                "skipping file `{}` because it failed to open: {}",
                input_filename, err
            )
        }
    };

    let mut csv_data = String::new();

    if let Err(err) = file.read_to_string(&mut csv_data).await {
        return error!(
            "skipping file `{}` because it could not be read: {}",
            input_filename, err
        );
    }

    let output_filename = format!("./output/{}.json", mode);

    let mut output = match StdFile::create(&output_filename) {
        Ok(file) => file,
        Err(err) => {
            return error!(
                "skipping file `{}` because its output file `{}` could not be created: {}",
                input_filename, output_filename, err
            )
        }
    };

    let (mods, mode_int) = match mode {
        OSU => (OSU_MODS, 0),
        TAIKO => (TAIKO_MODS, 1),
        CATCH => (CATCH_MODS, 2),
        MANIA => (MANIA_MODS, 3),
        _ => unreachable!(),
    };

    info!(
        "Starting to calculate {} data, each map with {} different mod combinations...",
        mode,
        mods.len()
    );

    let data: Vec<Data> = csv_data
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|id| !id.is_empty())
        .filter_map(|id| match id.parse::<u32>() {
            Ok(id) => Some(id),
            Err(_) => {
                error!("could not parse `{}` as u32", id);

                return None;
            }
        })
        .map(|id| mods.iter().map(move |m| (m, id)))
        .flatten()
        .map(|(mods, map_id)| async move {
            let map_path = format!("{}/{}.osu", map_path, map_id);
            let mut command = Command::new("dotnet");

            command
                .arg(perf_calc_path)
                .arg("simulate")
                .arg(mode)
                .arg(map_path)
                .arg("--json");

            if !mods[0].is_empty() {
                for &m in mods.iter() {
                    command.arg("-m").arg(m);
                }
            }

            let output = match command.output().await {
                Ok(output) => output,
                Err(err) => {
                    error!(
                        "failed to calculate values for map {} on {:?}: {}",
                        map_id, mods, err
                    );

                    return None;
                }
            };

            match serde_json::from_slice(&output.stdout) {
                Ok(data) => Some(Data::new(mode_int, map_id, data)),
                Err(err) => {
                    error!(
                        "failed to deserialize output for map {} on {:?}: {}\n \
                        >stdout: {}\n >stderr: {}",
                        map_id,
                        mods,
                        err,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr),
                    );

                    None
                }
            }
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(|data| async { data })
        .collect::<Vec<Data>>()
        .await;

    info!(
        "Calculated data for {} map-mod pairs, storing in file `{}`...",
        data.len(),
        output_filename
    );

    match serde_json::to_writer(&mut output, &data) {
        Ok(_) => info!("Finished calculating {} data", mode),
        Err(err) => error!(
            "failed to serialize data into file `{}`: {}",
            output_filename, err
        ),
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
    aim: Option<f32>,
    #[serde(default, alias = "Speed", skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
    #[serde(default, alias = "Accuracy", skip_serializing_if = "Option::is_none")]
    accuracy: Option<f32>,
    #[serde(default, alias = "Flashlight", skip_serializing_if = "Option::is_none")]
    flashlight: Option<f32>,
    #[serde(default, alias = "Strain", skip_serializing_if = "Option::is_none")]
    strain: Option<f32>,
    #[serde(default, alias = "OD", skip_serializing_if = "Option::is_none")]
    od: Option<f32>,
    #[serde(default, alias = "AR", skip_serializing_if = "Option::is_none")]
    ar: Option<f32>,
    #[serde(alias = "Mods")]
    mods: String,
    #[serde(alias = "Stars")]
    stars: f32,
    pp: f32,
}
