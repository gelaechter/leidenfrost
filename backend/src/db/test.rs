use rstest::rstest;
use std::{iter, path::PathBuf};
use strum::IntoEnumIterator;

use env_logger::Env;

use crate::{
    api::endpoint_api::{
        AlbumSorting, GetAlbumsParams, GetTracksParams, MusicEndpoint, Pagination, Sort,
        SortOrder::{self, Ascending},
    },
    db::{models::Album, sqlite::EndpointDB},
};

fn enable_logging() {
    let _ = env_logger::Builder::from_env(Env::new().filter("leidenfrost=info"))
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

async fn setup_endpoint() -> EndpointDB {
    let path = PathBuf::from("src/db/test_data.sqlite");
    let db = EndpointDB::open_with_path(path).await;

    db.unwrap()
}

#[tokio::test]
async fn test_get_tracks() {
    enable_logging();
    let endpoint = setup_endpoint().await;

    let tracks = endpoint
        .get_tracks(GetTracksParams {
            pagination: Some(Pagination {
                start_page: 0,
                limit: 100,
            }),
            sorting: None,
        })
        .await
        .unwrap();

    log::info!("{tracks:#?}");
    assert!(tracks.len() == 100)
}