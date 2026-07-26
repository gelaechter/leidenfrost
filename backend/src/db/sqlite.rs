use std::path::PathBuf;

use async_trait::async_trait;
use sea_orm::{
    ActiveValue, ColumnTrait, Database, DatabaseConnection, DeriveIden, EntityTrait,
    IntoActiveModel, Iterable, ModelTrait, QueryOrder, QuerySelect, QueryTrait,
    sea_query::{Expr, OnConflict},
};
use tokio::{
    runtime::Builder,
    sync::{mpsc, oneshot},
    task::LocalSet,
};

use crate::{
    api::{
        endpoint_api::{
            ArtistAlbums, Capabilities, GetAlbumsParams, GetArtistsParams, GetGenresParams,
            GetPlaylistsParams, GetTracksParams, MusicEndpoint, Pagination, SearchParams,
            SearchResult, SortOrder, TrackSorting,
        },
        jellyfin::errors::ApiError,
    },
    data_view::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView},
    db::models::{artist_albums, endpoint, playlist},
};

use super::models::{album, artist, genre, track};

type Result<T> = std::result::Result<T, ApiError>;

/// Trait that indexes an endpoint's data into the database
/// This allows the endpoints to be used together
#[async_trait]
pub trait IndexableEndpoint: MusicEndpoint {
    /// Provides a unique ID for this API
    /// This is used to matched indexed data in the database to the respective
    /// API
    fn get_id(&self) -> String;

    /// Naive implementation which just requests all datatypes
    async fn index_data(&self, db: &DatabaseConnection) {
        log::info!("Indexing endpoint {}", self.get_id());
        // Endpoint ID
        endpoint::Entity::insert(endpoint::ActiveModel {
            id: ActiveValue::Set(self.get_id()),
        })
        .on_conflict_do_nothing() // Allow conflicts if this is already in there
        .exec(db)
        .await
        .unwrap();

        // TODO:
        // There is probably some way to generalize all of this but i really couldn't be
        // bothered. Since you are not me feel free to have a go!

        // Artists
        let mut page = 0;
        while let Ok(artist_views) = self
            .get_artists(GetArtistsParams {
                pagination: self.capabilities().pagination.then_some(Pagination {
                    start_page: page,
                    limit: 100,
                }),
                sorting: None,
            })
            .await
            && !artist_views.is_empty()
        {
            page += 1;
            let iter: Vec<artist::ActiveModel> = artist_views
                .into_iter()
                .map(|view| view.artist.into_active_model())
                .collect();

            for chunk in iter.chunks(1000) {
                log::debug!("Inserting {} artists", chunk.len());
                artist::Entity::insert_many(chunk.to_vec())
                    .on_conflict(
                        OnConflict::column(artist::Column::Id)
                            .update_columns(<artist::Entity as EntityTrait>::Column::iter())
                            .to_owned(),
                    )
                    .exec(db)
                    .await
                    .unwrap();
            }
        }

        // Album artists (AFAIK Jellyfin specific)
        let mut page = 0;
        while self.capabilities().split_artists
            && let Ok(artist_views) = self
                .get_album_artists(GetArtistsParams {
                    pagination: self.capabilities().pagination.then_some(Pagination {
                        start_page: page,
                        limit: 100,
                    }),
                    sorting: None,
                })
                .await
            && !artist_views.is_empty()
        {
            page += 1;
            let iter: Vec<artist::ActiveModel> = artist_views
                .into_iter()
                .map(|view| view.artist.into_active_model())
                .collect();

            for chunk in iter.chunks(1000) {
                log::debug!("Inserting {} album artists", chunk.len());
                artist::Entity::insert_many(chunk.to_vec())
                    .on_conflict(
                        OnConflict::column(artist::Column::Id)
                            .update_columns(<artist::Entity as EntityTrait>::Column::iter())
                            .to_owned(),
                    )
                    .exec(db)
                    .await
                    .unwrap();
            }
        }

        // Albums
        let mut page = 0;
        while let Ok(album_views) = self
            .get_albums(GetAlbumsParams {
                pagination: self.capabilities().pagination.then_some(Pagination {
                    start_page: page,
                    limit: 100,
                }),
                sorting: None,
            })
            .await
            && !album_views.is_empty()
        {
            page += 1;

            let mut albums: Vec<album::ActiveModel> = vec![];
            let mut related_artists: Vec<artist_albums::ActiveModel> = vec![];
            for view in album_views.into_iter() {
                // Create album_artists junctions
                let artists: Vec<artist_albums::ActiveModel> = view
                    .album_artists
                    .into_iter()
                    .map(|artist| {
                        let album_id = view.album.id.clone();
                        artist_albums::ActiveModel {
                            artist_id: ActiveValue::Set(artist.id),
                            album_id: ActiveValue::Set(album_id),
                        }
                    })
                    .collect();
                // As well as the album itself
                let album = view.album.into_active_model();

                albums.push(album);
                related_artists.extend(artists);
            }

            // Insert albums
            for chunk in albums.chunks(1000) {
                log::debug!("Inserting {} albums", chunk.len());
                album::Entity::insert_many(chunk.to_vec())
                    .on_conflict(
                        OnConflict::column(album::Column::Id)
                            .update_columns(<album::Entity as EntityTrait>::Column::iter())
                            .to_owned(),
                    )
                    .exec(db)
                    .await
                    .unwrap();
            }

            // At this point we already inserted all of the artists and the corresponding
            // artist Now we can insert the junction
            for chunk in related_artists.chunks(1000) {
                log::debug!("Inserting {} album artists", chunk.len());
                let insert = artist_albums::Entity::insert_many(chunk.to_vec()).on_conflict(
                    OnConflict::columns([
                        artist_albums::Column::ArtistId,
                        artist_albums::Column::AlbumId,
                    ])
                    .update_columns(<artist_albums::Entity as EntityTrait>::Column::iter())
                    .to_owned(),
                ).on_conflict(OnConflict::constraint("FOREIGN KEY").TODO:);

                log::debug!(
                    "Statement: {}",
                    insert.build(sea_orm::DatabaseBackend::Sqlite)
                );

                insert.exec(db).await.unwrap();
            }
        }

        // Tracks
        let mut page = 0;
        while let Ok(track_views) = self
            .get_tracks(GetTracksParams {
                pagination: self.capabilities().pagination.then_some(Pagination {
                    start_page: page,
                    limit: 100,
                }),
                sorting: None,
            })
            .await
            && !track_views.is_empty()
        {
            page += 1;

            let tracks: Vec<track::ActiveModel> = track_views
                .into_iter()
                .map(|view| view.track.into_active_model())
                .collect();

            for chunk in tracks.chunks(1000) {
                log::debug!("Inserting {} tracks", chunk.len());
                track::Entity::insert_many(chunk.to_vec())
                    .on_conflict(
                        OnConflict::column(track::Column::Id)
                            .update_columns(<track::Entity as EntityTrait>::Column::iter())
                            .to_owned(),
                    )
                    .exec(db)
                    .await
                    .unwrap();
            }
        }

        // Genres
        let mut page = 0;
        while let Ok(genre_views) = self
            .get_genres(GetGenresParams {
                pagination: self.capabilities().pagination.then_some(Pagination {
                    start_page: page,
                    limit: 100,
                }),
                sorting: None,
            })
            .await
            && !genre_views.is_empty()
        {
            page += 1;

            let iter: Vec<genre::ActiveModel> = genre_views
                .into_iter()
                .map(|view| view.genre.into_active_model())
                .collect();

            for chunk in iter.chunks(1000) {
                log::debug!("Inserting {} genres", chunk.len());
                genre::Entity::insert_many(chunk.to_vec())
                    .on_conflict(
                        OnConflict::column(genre::Column::Id)
                            .update_columns(<genre::Entity as EntityTrait>::Column::iter())
                            .to_owned(),
                    )
                    .exec(db)
                    .await
                    .unwrap();
            }
        }

        // Playlist
        let mut page = 0;
        while let Ok(playlist_views) = self
            .get_playlists(GetPlaylistsParams {
                pagination: self.capabilities().pagination.then_some(Pagination {
                    start_page: page,
                    limit: 100,
                }),
                sorting: None,
            })
            .await
            && !playlist_views.is_empty()
        {
            page += 1;

            let iter: Vec<playlist::ActiveModel> = playlist_views
                .into_iter()
                .map(|view| view.playlist.into_active_model())
                .collect();

            for chunk in iter.chunks(1000) {
                log::debug!("Inserting {} playlists", chunk.len());
                playlist::Entity::insert_many(chunk.to_vec())
                    .on_conflict(
                        OnConflict::column(playlist::Column::Id)
                            .update_columns(<playlist::Entity as EntityTrait>::Column::iter())
                            .to_owned(),
                    )
                    .exec(db)
                    .await
                    .unwrap();
            }
        }
    }
}

// Read/Write/Create
const MODE: &str = "rwc";

/// The local database which acts as an index for endpoints
#[derive(Debug, Clone)]
pub struct EndpointDB {
    pub db: DatabaseConnection,
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
        let db: DatabaseConnection =
            Database::connect(format!("sqlite://{path}?mode={MODE}")).await?;

        // TODO: Naive check if database works
        db.ping().await?;

        // Sync the schema
        let tx = Self::start_schema_sync_worker(db.clone());
        let (done_tx, done_rx) = oneshot::channel();
        tx.send(done_tx).unwrap();
        done_rx.await.unwrap()?;

        Ok(EndpointDB { db })
    }

    /// Schema discovery is not Send + Sync so we spawn it in another thread
    fn start_schema_sync_worker(
        db: DatabaseConnection,
    ) -> mpsc::UnboundedSender<oneshot::Sender<Result<()>>> {
        let (tx, mut rx) = mpsc::unbounded_channel::<oneshot::Sender<Result<()>>>();

        std::thread::spawn(move || {
            // SeaORM uses the tokio runtime (i didn't bother to check if we actually need
            // both IO and time)
            let rt = Builder::new_current_thread().enable_all().build().unwrap();

            let local = LocalSet::new();

            local.block_on(&rt, async move {
                while let Some(done) = rx.recv().await {
                    let result = db
                        .get_schema_registry(module_path!().split("::").next().unwrap())
                        .sync(&db)
                        .await
                        .map_err(Into::into);

                    let _ = done.send(result);
                }
            });
        });

        tx
    }
}

#[async_trait]
impl MusicEndpoint for EndpointDB {
    /// Provides the capabilities of this API
    fn capabilities(&self) -> Capabilities {
        todo!()
    }

    /// Fetches a specific track
    async fn get_track(&self, track_id: String) -> Result<TrackView> {
        let track = track::Entity::find_by_id(track_id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::DataNotFound)?;

        let artists = track
            .find_related(artist::Entity)
            .all(&self.db)
            .await?
            .into_iter() // Convert into RelatedArtists
            .map(Into::into)
            .collect();

        let album_name = track
            .find_related(album::Entity)
            .one(&self.db)
            .await?
            .and_then(|album| album.title);

        let genres = track
            .find_related(genre::Entity)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(TrackView {
            track,
            artists,
            album_name,
            genres,
        })
    }

    /// Fetches all tracks
    async fn get_tracks(&self, params: GetTracksParams) -> Result<Vec<TrackView>> {
        let GetTracksParams {
            pagination,
            sorting,
        } = params;

        let select = track::Entity::find();

        // Pagination
        let select = if let Some(Pagination { limit, start_page }) = pagination {
            select.limit(limit).offset(start_page)
        } else {
            select
        };

        // table name
        #[derive(DeriveIden, Clone, Copy)]
        struct AlbumArtist;

        let select = select
            .find_also(track::Entity, artist::Entity) //  track -> artist
            .find_also(track::Entity, genre::Entity) //   track -> genre
            .find_also(track::Entity, album::Entity); //  track -> album

        // Sorting
        let select = if let Some(sorting) = sorting {
            let criteria = match sorting.by {
                TrackSorting::Album => album::COLUMN.title.0.into_expr(),
                TrackSorting::AlbumArtist => Expr::col((AlbumArtist, artist::Column::Name)),
                TrackSorting::Title => track::COLUMN.title.0.into_expr(),
                TrackSorting::Artist => artist::COLUMN.name.0.into_expr(),
                TrackSorting::Duration => track::COLUMN.duration.0.into_expr(),
                TrackSorting::PlayCount => track::COLUMN.play_count.0.into_expr(),
                TrackSorting::Random => Expr::cust("RANDOM()"),
                TrackSorting::DateAdded => {
                    return Err(ApiError::Unsupported {
                        call: "TrackSorting::DateAdded".to_owned(),
                        endpoint: "Database".to_owned(),
                    });
                }
                TrackSorting::DatePlayed => track::COLUMN.last_played_at.0.into_expr(),
                TrackSorting::DateReleased => track::COLUMN.release_date.0.into_expr(),
            };

            match sorting.order {
                SortOrder::Ascending => select.order_by_asc(criteria),
                SortOrder::Descending => select.order_by_desc(criteria),
            }
        } else {
            select
        };

        // Consolidate
        let tracks: Vec<(
            track::Model,
            Vec<artist::Model>,
            Vec<genre::Model>,
            Vec<album::Model>,
        )> = select.consolidate().all(&self.db).await?;

        // Put into a TrackView
        let tracks = tracks
            .into_iter()
            .map(|(track, artists, genres, albums)| TrackView {
                track,
                artists: artists.into_iter().map(Into::into).collect(),
                album_name: albums.into_iter().next().and_then(|a| a.title),
                genres: genres.into_iter().map(Into::into).collect(),
            })
            .collect();

        Ok(tracks)
    }

    /// Fetches a specific album
    async fn get_album(&self, album_id: String) -> Result<AlbumView> {
        todo!()
    }

    /// Fetches all songs from an album
    async fn get_album_tracks(
        &self,
        album_id: String,
        params: GetTracksParams,
    ) -> Result<Vec<TrackView>> {
        todo!()
    }

    /// Fetches all albums
    async fn get_albums(&self, params: GetAlbumsParams) -> Result<Vec<AlbumView>> {
        todo!()
    }

    /// Fetches albums from an artist
    async fn get_artist_albums(
        &self,
        artist_id: String,
        params: GetAlbumsParams,
    ) -> Result<ArtistAlbums> {
        todo!()
    }

    /// Fetches an artist
    async fn get_artist(&self, artist_id: String) -> Result<ArtistView> {
        todo!()
    }

    /// Fetches all artists
    async fn get_artists(&self, params: GetArtistsParams) -> Result<Vec<ArtistView>> {
        todo!()
    }

    async fn get_album_artists(&self, params: GetArtistsParams) -> Result<Vec<ArtistView>> {
        unimplemented!("The DB should not discerns album artists and normal artists")
    }

    /// Fetches all genres
    async fn get_genres(&self, params: GetGenresParams) -> Result<Vec<GenreView>> {
        todo!()
    }

    /// Fetches all playlists
    async fn get_playlists(&self, params: GetPlaylistsParams) -> Result<Vec<PlaylistView>> {
        // TODO:
        Ok(vec![])
    }

    // Fetches a playlist
    async fn get_playlist(&self, playlist_id: String) -> Result<PlaylistView> {
        todo!()
    }

    /// Fetches playlist tracks
    async fn get_playlist_tracks(
        &self,
        playlist_id: String,
        params: GetPlaylistsParams,
    ) -> Result<Vec<TrackView>> {
        todo!()
    }

    /// Fetches songs containing a search term
    async fn search(&self, search_term: String, params: SearchParams) -> Result<Vec<SearchResult>> {
        todo!()
    }
}
