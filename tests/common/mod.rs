mod mode;

use rosu_pp::Beatmap;

pub use self::mode::{Catch, Mania, Mode, Osu, Taiko};

#[macro_export]
#[rustfmt::skip]
macro_rules! test_map {
    ($mode:ident) => {{
        #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
        { common::test_map::<$mode>() }
        #[cfg(any(feature = "async_tokio", feature = "async_std"))]
        { common::test_map::<$mode>().await }
    }};
}

#[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
pub fn test_map<M: Mode>() -> Beatmap {
    let path = format!("./maps/{}.osu", M::TEST_MAP_ID);

    Beatmap::from_path(path).unwrap()
}

#[cfg(any(feature = "async_tokio", feature = "async_std"))]
pub async fn test_map<M: Mode>() -> Beatmap {
    let path = format!("./maps/{}.osu", M::TEST_MAP_ID);

    Beatmap::from_path(path).await.unwrap()
}
