pub mod endpoint_impl;
pub mod indexable;
pub mod sorting;

use std::{path::PathBuf, time::Duration};

use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

use crate::{
    api::{endpoint_api::SearchResult, jellyfin::errors::ApiError},
    data_view::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView},
};

type Result<T> = std::result::Result<T, ApiError>;

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
        let opts = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            // See <https://www.sqlite.org/wal.html>
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal) // no fsync per commit, still crash-safe
            .foreign_keys(true) // replaces the after_connect PRAGMA
            .busy_timeout(Duration::from_secs(5))
            // TODO: I should do some tests / some more reading on how to
            .pragma("mmap_size", "536870912") // reads served from a RAM mapping
            .pragma("cache_size", "-131072") // 128 MiB page cache
            .pragma("temp_store", "MEMORY");

        let db = SqlitePoolOptions::new().connect_with(opts).await?;

        // the migrations are relative to Cargo.toml
        sqlx::migrate!("src/db/migrations")
            .run(&db)
            .await
            .expect("migration error");

        Ok(EndpointDB { db })
    }
}
