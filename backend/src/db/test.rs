use std::path::PathBuf;

use env_logger::Env;

use crate::{
    api::endpoint_api::{GetTracksParams, MusicEndpoint, Pagination},
    db::sqlite::EndpointDB,
};

fn enable_logging() {
    let _ = env_logger::Builder::from_env(Env::new().filter("leidenfrost=info"))
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

#[tokio::test]
async fn test_get_tracks() {
    enable_logging();

    let path = PathBuf::from("src/db/test_data.sqlite");
    let db = EndpointDB::open_with_path(path).await;

    let endpoint = db.unwrap();

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
