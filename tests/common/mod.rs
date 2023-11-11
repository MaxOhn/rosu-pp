mod mode;

use rosu_pp::{Beatmap, GameMode};
use std::sync::OnceLock;

pub use self::mode::{Catch, Mania, Mode, Osu, Taiko};

static MAPS: OnceLock<[Beatmap; 4]> = OnceLock::new();

#[macro_export]
#[rustfmt::skip]
macro_rules! test_map {
    ($mode:ident) => {{
        #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
        { common::test_map(rosu_pp::GameMode::$mode) }
        #[cfg(any(feature = "async_tokio", feature = "async_std"))]
        { common::test_map(rosu_pp::GameMode::$mode).await }
    }};
}

#[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
pub fn test_map(mode: GameMode) -> &'static Beatmap {
    let maps = MAPS.get_or_init(|| {
        let osu = Beatmap::from_path(path(GameMode::Osu)).unwrap();
        let taiko = Beatmap::from_path(path(GameMode::Taiko)).unwrap();
        let catch = Beatmap::from_path(path(GameMode::Catch)).unwrap();
        let mania = Beatmap::from_path(path(GameMode::Mania)).unwrap();

        [osu, taiko, catch, mania]
    });

    &maps[mode as usize]
}

#[cfg(feature = "async_tokio")]
pub async fn test_map(mode: GameMode) -> &'static Beatmap {
    let maps = MAPS.get_or_init(|| {
        let fut = async {
            let osu = Beatmap::from_path(path(GameMode::Osu)).await.unwrap();
            let taiko = Beatmap::from_path(path(GameMode::Taiko)).await.unwrap();
            let catch = Beatmap::from_path(path(GameMode::Catch)).await.unwrap();
            let mania = Beatmap::from_path(path(GameMode::Mania)).await.unwrap();

            [osu, taiko, catch, mania]
        };

        match tokio::runtime::Handle::try_current() {
            Ok(h) => std::thread::spawn(move || h.block_on(fut)).join().unwrap(),
            Err(_) => tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(fut),
        }
    });

    &maps[mode as usize]
}

#[cfg(feature = "async_std")]
pub async fn test_map(mode: GameMode) -> &'static Beatmap {
    let maps = MAPS.get_or_init(|| {
        async_std::task::block_on(async {
            let osu = Beatmap::from_path(path(GameMode::Osu)).await.unwrap();
            let taiko = Beatmap::from_path(path(GameMode::Taiko)).await.unwrap();
            let catch = Beatmap::from_path(path(GameMode::Catch)).await.unwrap();
            let mania = Beatmap::from_path(path(GameMode::Mania)).await.unwrap();

            [osu, taiko, catch, mania]
        })
    });

    &maps[mode as usize]
}

fn path(mode: GameMode) -> String {
    let map_id = match mode {
        GameMode::Osu => <Osu as Mode>::TEST_MAP_ID,
        GameMode::Taiko => <Taiko as Mode>::TEST_MAP_ID,
        GameMode::Catch => <Catch as Mode>::TEST_MAP_ID,
        GameMode::Mania => <Mania as Mode>::TEST_MAP_ID,
    };

    format!("./maps/{map_id}.osu")
}
