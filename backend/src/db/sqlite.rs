pub mod endpoint_impl;
pub mod indexable;
pub mod sorting;

use std::path::PathBuf;

use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

use crate::{
    api::{endpoint_api::SearchResult, jellyfin::errors::ApiError},
    data_view::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView},
};

type Result<T> = std::result::Result<T, ApiError>;

// Read/Write/Create
const MODE: &str = "rwc";

/// The local database which acts as an index for endpoints
#[derive(Debug, Clone)]
pub struct EndpointDB {
    pub db: Pool<Sqlite>,
}

impl EndpointDB {
    /// Opens an sqlite db at the default path
    pub async fn open() -> Result<Self> {
        log::debug!("Initializing EndpointDB");
        // Find the data dir
        let dir = directories::ProjectDirs::from("", "", "Leidenfrost")
            .ok_or(ApiError::DbPathInaccessible)?
            .data_dir()
            .to_path_buf();

        std::fs::create_dir_all(&dir).map_err(|_| ApiError::DbPathInaccessible)?;

        let path = dir.join("local_index.sqlite");

        Self::open_with_path(path).await
    }

    /// Opens an sqlite db at the given db
    pub async fn open_with_path(path: PathBuf) -> Result<Self> {
        let path = path
            .into_string()
            .map_err(|_| ApiError::DbPathInaccessible)?;

        // Open db
        let db = SqlitePoolOptions::new()
            .after_connect(|connection, _meta| {
                Box::pin(async move {
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(connection)
                        .await?;

                    Ok(())
                })
            })
            .connect(&format!("sqlite://{path}?mode={MODE}"))
            .await?;

        // the migrations are relative the Cargo.toml
        sqlx::migrate!("src/db/migrations").run(&db).await.unwrap();

        Ok(EndpointDB { db })
    }
}
