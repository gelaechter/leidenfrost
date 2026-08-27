use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use async_trait::async_trait;
use sea_orm::{
    ActiveValue, ColumnTrait, Database, DatabaseConnection, DeriveIden, EntityTrait,
    IntoActiveModel, Iterable, ModelTrait, QueryFilter, QueryOrder, QuerySelect, Select,
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
            AlbumSorting, Capabilities, GetAlbumsParams, GetArtistsParams, GetGenresParams,
            GetPlaylistsParams, GetTracksParams, MusicEndpoint, Pagination, SearchParams,
            SearchResult, SortOrder, TrackSorting,
        },
        jellyfin::errors::ApiError,
    },
    data_view::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView},
    db::models::{artist_albums, artist_tracks, endpoint, playlist, playlist_tracks, track_genres},
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
                log::debug!("Inserting {} album-artists", chunk.len());
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
                // 1. Extract artist IDs from the chunk safely
                // (take() returns Some(val) if Set, None if Unset)
                let artist_ids_in_chunk: Vec<String> = chunk
                    .iter()
                    .filter_map(|m| m.artist_id.clone().take())
                    .collect();

                // 2. Find which ones actually exist in the database
                let valid_artist_ids: HashSet<String> = artist::Entity::find()
                    .filter(artist::Column::Id.is_in(artist_ids_in_chunk))
                    .all(db)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|a| a.id)
                    .collect();

                // 3. Filter the chunk to ONLY include rows with valid artists
                let valid_chunk: Vec<artist_albums::ActiveModel> = chunk
                    .iter()
                    .filter(|m| {
                        m.artist_id
                            .clone()
                            .take()
                            .is_some_and(|id| valid_artist_ids.contains(&id))
                    })
                    .cloned() // chunk yields references (&ActiveModel), so we clone to own them
                    .collect();

                log::debug!(
                    "Inserting {} valid album-artist relations (skipping {} due to missing FK)",
                    valid_chunk.len(),
                    chunk.len() - valid_chunk.len()
                );
                let insert =
                    artist_albums::Entity::insert_many(valid_chunk).on_conflict_do_nothing();
                insert.exec_without_returning(db).await.unwrap();
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

            let mut tracks: Vec<track::ActiveModel> = vec![];
            let mut related_genres: Vec<track_genres::ActiveModel> = vec![];
            let mut related_artists: Vec<artist_tracks::ActiveModel> = vec![];
            for view in track_views.into_iter() {
                // Create album_artists junctions
                let genres: Vec<track_genres::ActiveModel> = view
                    .genres
                    .into_iter()
                    .map(|genre| {
                        let track_id = view.track.id.clone();
                        track_genres::ActiveModel {
                            track_id: ActiveValue::Set(track_id),
                            genre_id: ActiveValue::Set(genre.id),
                        }
                    })
                    .collect();

                // Create artist_tracks junctions
                let artists: Vec<artist_tracks::ActiveModel> = view
                    .artists
                    .into_iter()
                    .map(|artist| {
                        let track_id = view.track.id.clone();
                        artist_tracks::ActiveModel {
                            track_id: ActiveValue::Set(track_id),
                            artist_id: ActiveValue::Set(artist.id),
                        }
                    })
                    .collect();

                // As well as the album itself
                let track = view.track.into_active_model();

                tracks.push(track);
                related_genres.extend(genres);
                related_artists.extend(artists);
            }

            // Insert track
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

            // At this point we already inserted all of the genres and the corresponding
            // tracks; Now we can insert the junction
            for chunk in related_genres.chunks(1000) {
                // 1. Extract artist IDs from the chunk safely
                // (take() returns Some(val) if Set, None if Unset)
                let genre_ids_in_chunk: Vec<String> = chunk
                    .iter()
                    .filter_map(|m| m.genre_id.clone().take())
                    .collect();

                // 2. Find which ones actually exist in the database
                let valid_genre_ids: HashSet<String> = genre::Entity::find()
                    .filter(genre::Column::Id.is_in(genre_ids_in_chunk))
                    .all(db)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|a| a.id)
                    .collect();

                // 3. Filter the chunk to ONLY include rows with valid artists
                let valid_chunk: Vec<track_genres::ActiveModel> = chunk
                    .iter()
                    .filter(|m| {
                        m.genre_id
                            .clone()
                            .take()
                            .is_some_and(|id| valid_genre_ids.contains(&id))
                    })
                    .cloned() // chunk yields references (&ActiveModel), so we clone to own them
                    .collect();

                log::debug!(
                    "Inserting {} valid track-genre relations (skipping {} due to missing FK)",
                    valid_chunk.len(),
                    chunk.len() - valid_chunk.len()
                );
                let insert =
                    track_genres::Entity::insert_many(valid_chunk).on_conflict_do_nothing();
                insert.exec_without_returning(db).await.unwrap();
            }

            // At this point we already inserted all of the genres and the corresponding
            // tracks; Now we can insert the junction
            for chunk in related_artists.chunks(1000) {
                // 1. Extract artist IDs from the chunk safely
                // (take() returns Some(val) if Set, None if Unset)
                let artist_ids_in_chunk: Vec<String> = chunk
                    .iter()
                    .filter_map(|m| m.artist_id.clone().take())
                    .collect();

                // 2. Find which ones actually exist in the database
                let valid_artist_ids: HashSet<String> = artist::Entity::find()
                    .filter(artist::Column::Id.is_in(artist_ids_in_chunk))
                    .all(db)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|a| a.id)
                    .collect();

                // 3. Filter the chunk to ONLY include rows with valid artists
                let valid_chunk: Vec<artist_tracks::ActiveModel> = chunk
                    .iter()
                    .filter(|m| {
                        m.artist_id
                            .clone()
                            .take()
                            .is_some_and(|id| valid_artist_ids.contains(&id))
                    })
                    .cloned() // chunk yields references (&ActiveModel), so we clone to own them
                    .collect();

                log::debug!(
                    "Inserting {} valid track-artist relations (skipping {} due to missing FK)",
                    valid_chunk.len(),
                    chunk.len() - valid_chunk.len()
                );
                let insert =
                    artist_tracks::Entity::insert_many(valid_chunk).on_conflict_do_nothing();
                insert.exec_without_returning(db).await.unwrap();
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

            let mut playlists: Vec<playlist::ActiveModel> = vec![];
            let mut related_tracks: Vec<playlist_tracks::ActiveModel> = vec![];
            for view in playlist_views.into_iter() {
                // Create playlist-track junctions by requesting all the playlists tracks
                let mut tracks_page = 0;
                while let Ok(track_views) = self
                    .get_playlist_tracks(
                        view.playlist.id.clone(),
                        GetTracksParams {
                            pagination: self.capabilities().pagination.then_some(Pagination {
                                start_page: tracks_page,
                                limit: 100,
                            }),
                            sorting: None,
                        },
                    )
                    .await
                    && !track_views.is_empty()
                {
                    tracks_page += 1;

                    let tracks: Vec<playlist_tracks::ActiveModel> = track_views
                        .into_iter()
                        .map(|tv| {
                            let playlist_id = view.playlist.id.clone();
                            playlist_tracks::ActiveModel {
                                track_id: ActiveValue::Set(tv.track.id),
                                playlist_id: ActiveValue::Set(playlist_id),
                            }
                        })
                        .collect();

                    related_tracks.extend(tracks);
                }

                // As well as the album itself
                let playlist = view.playlist.into_active_model();

                playlists.push(playlist);
            }

            // Insert playlists
            for chunk in playlists.chunks(1000) {
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

            // At this point we already inserted all of the playlist and the corresponding
            // tracks; Now we can insert the junction
            for chunk in related_tracks.chunks(1000) {
                // 1. Extract artist IDs from the chunk safely
                // (take() returns Some(val) if Set, None if Unset)
                let track_ids_in_chunk: Vec<String> = chunk
                    .iter()
                    .filter_map(|m| m.track_id.clone().take())
                    .collect();

                // 2. Find which ones actually exist in the database
                let valid_track_ids: HashSet<String> = track::Entity::find()
                    .filter(track::COLUMN.id.is_in(track_ids_in_chunk))
                    .all(db)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|a| a.id)
                    .collect();

                // 3. Filter the chunk to ONLY include rows with valid artists
                let valid_chunk: Vec<playlist_tracks::ActiveModel> = chunk
                    .iter()
                    .filter(|m| {
                        m.track_id
                            .clone()
                            .take()
                            .is_some_and(|id| valid_track_ids.contains(&id))
                    })
                    .cloned() // chunk yields references (&ActiveModel), so we clone to own them
                    .collect();

                log::debug!(
                    "Inserting {} valid playlist-track relations (skipping {} due to missing FK)",
                    valid_chunk.len(),
                    chunk.len() - valid_chunk.len()
                );
                let insert =
                    playlist_tracks::Entity::insert_many(valid_chunk).on_conflict_do_nothing();
                insert.exec_without_returning(db).await.unwrap();
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

impl EndpointDB {
    async fn load_genres(&self, ids: &[String]) -> Result<HashMap<String, Vec<genre::Model>>> {
        let rows = album::Entity::find()
            .filter(album::Column::Id.is_in(ids.to_vec()))
            .find_also_linked(album::AlbumToGenres)
            .all(&self.db)
            .await?;

        let mut map: HashMap<String, Vec<genre::Model>> = HashMap::new();
        for (album, genre) in rows {
            if let Some(g) = genre {
                map.entry(album.id).or_default().push(g);
            }
        }
        Ok(map)
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
        let album = album::Entity::find_by_id(album_id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::DataNotFound)?;

        let album_artists = album
            .find_related(artist::Entity)
            .all(&self.db)
            .await?
            .into_iter() // Convert into RelatedArtists
            .map(Into::into)
            .collect();

        let genres = album
            .find_linked(album::AlbumToGenres)
            .all(&self.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        let duration: i64 = album
            .find_related(track::Entity)
            .select_only()
            .column_as(track::COLUMN.duration.sum(), "sum")
            .into_tuple()
            .one(&self.db)
            .await?
            .unwrap();

        Ok(AlbumView {
            album,
            album_artists,
            genres,
            duration: Some(duration),
        })
    }

    /// Fetches all songs from an album
    async fn get_album_tracks(
        &self,
        album_id: String,
        params: GetTracksParams,
    ) -> Result<Vec<TrackView>> {
        let GetTracksParams {
            pagination,
            sorting,
        } = params;

        let album = album::Entity::find_by_id(album_id)
            .one(&self.db)
            .await?
            .ok_or(ApiError::DataNotFound)?;

        let select = album.find_related(track::Entity);

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
            .find_also(track::Entity, genre::Entity); //   track -> genre

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
        let tracks: Vec<(track::Model, Vec<artist::Model>, Vec<genre::Model>)> =
            select.consolidate().all(&self.db).await?;

        // Put into a TrackView
        let tracks = tracks
            .into_iter()
            .map(|(track, artists, genres)| TrackView {
                track,
                artists: artists.into_iter().map(Into::into).collect(),
                album_name: album.title.clone(),
                genres: genres.into_iter().map(Into::into).collect(),
            })
            .collect();

        Ok(tracks)
    }

    /// Fetches all albums
    async fn get_albums(&self, params: GetAlbumsParams) -> Result<Vec<AlbumView>> {
        let GetAlbumsParams {
            pagination,
            sorting,
        } = params;

        let select = album::Entity::find();

        // Pagination
        let paginated_select = if let Some(Pagination { limit, start_page }) = pagination {
            select.limit(limit).offset(start_page)
        } else {
            select
        };

        let select = paginated_select
            .find_also_linked(album::AlbumToGenres)
            .find_also(album::Entity, artist::Entity)
            .group_by(album::Column::Id);

        // Sorting
        let select = if let Some(sorting) = sorting {
            let criteria = match sorting.by {
                AlbumSorting::Name => album::COLUMN.title.0.into_expr(),
                AlbumSorting::AlbumArtist => artist::COLUMN.name.0.into_expr(),
                AlbumSorting::Duration => {
                    // Subquery for duration sort
                    Expr::cust(
                        "SELECT COALESCE(SUM(track.duration), 0) FROM track WHERE track.album_id = album.id",
                    )
                }
                AlbumSorting::Random => Expr::cust("RANDOM()"),
                AlbumSorting::DateAdded => {
                    return Err(ApiError::Unsupported {
                        call: "AlbumSorting::DateAdded".to_owned(),
                        endpoint: "Database".to_owned(),
                    });
                }
                AlbumSorting::DateReleased => track::COLUMN.release_date.0.into_expr(),
                AlbumSorting::TrackCount => {
                    Expr::cust("SELECT COUNT(id) FROM track WHERE track.album_id = album.id")
                }
            };

            match sorting.order {
                SortOrder::Ascending => select.order_by_asc(criteria),
                SortOrder::Descending => select.order_by_desc(criteria),
            }
        } else {
            select
        };

        // Consolidate
        let albums = select.consolidate().all(&self.db).await?;

        // Additionally find the duration for every album
        let duration_map: HashMap<String, i64> = track::Entity::find()
            .select_only()
            .column(track::Column::AlbumId)
            .column_as(track::Column::Duration.sum(), "total_duration")
            .filter(track::Column::AlbumId.is_in(albums.iter().map(|a| &a.0.id)))
            .group_by(track::Column::AlbumId)
            .into_tuple::<(String, i64)>()
            .all(&self.db)
            .await?
            .into_iter()
            .collect();

        // Put into a AlbumView
        let albums = albums
            .into_iter()
            .map(|(album, genres, artists)| AlbumView {
                duration: duration_map.get(&album.id).copied(),
                album_artists: artists.into_iter().map(Into::into).collect(),
                genres: genres.into_iter().map(Into::into).collect(),
                album,
            })
            .collect();

        Ok(albums)
    }

    /// Fetches albums from an artist
    /// THESE ARE EXPLICITLY ALL
    async fn get_artist_albums(
        &self,
        artist_id: String,
        params: GetAlbumsParams,
    ) -> Result<Vec<AlbumView>> {
        let GetAlbumsParams {
            pagination,
            sorting,
        } = params;

        let select: Select<album::Entity> = todo!();

        // Pagination
        let paginated_select = if let Some(Pagination { limit, start_page }) = pagination {
            select.limit(limit).offset(start_page)
        } else {
            select
        };

        let select = paginated_select
            .find_also_linked(album::AlbumToGenres)
            .find_also(album::Entity, artist::Entity)
            .group_by(album::Column::Id);

        // Sorting
        let select = if let Some(sorting) = sorting {
            let criteria = match sorting.by {
                AlbumSorting::Name => album::COLUMN.title.0.into_expr(),
                AlbumSorting::AlbumArtist => artist::COLUMN.name.0.into_expr(),
                AlbumSorting::Duration => {
                    // Subquery for duration sort
                    Expr::cust(
                        "SELECT COALESCE(SUM(track.duration), 0) FROM track WHERE track.album_id = album.id",
                    )
                }
                AlbumSorting::Random => Expr::cust("RANDOM()"),
                AlbumSorting::DateAdded => {
                    return Err(ApiError::Unsupported {
                        call: "AlbumSorting::DateAdded".to_owned(),
                        endpoint: "Database".to_owned(),
                    });
                }
                AlbumSorting::DateReleased => track::COLUMN.release_date.0.into_expr(),
                AlbumSorting::TrackCount => {
                    Expr::cust("SELECT COUNT(id) FROM track WHERE track.album_id = album.id")
                }
            };

            match sorting.order {
                SortOrder::Ascending => select.order_by_asc(criteria),
                SortOrder::Descending => select.order_by_desc(criteria),
            }
        } else {
            select
        };

        // Consolidate
        let albums = select.consolidate().all(&self.db).await?;

        // Additionally find the duration for every album
        let duration_map: HashMap<String, i64> = track::Entity::find()
            .select_only()
            .column(track::Column::AlbumId)
            .column_as(track::Column::Duration.sum(), "total_duration")
            .filter(track::Column::AlbumId.is_in(albums.iter().map(|a| &a.0.id)))
            .group_by(track::Column::AlbumId)
            .into_tuple::<(String, i64)>()
            .all(&self.db)
            .await?
            .into_iter()
            .collect();

        // Put into a AlbumView
        let albums = albums
            .into_iter()
            .map(|(album, genres, artists)| AlbumView {
                duration: duration_map.get(&album.id).copied(),
                album_artists: artists.into_iter().map(Into::into).collect(),
                genres: genres.into_iter().map(Into::into).collect(),
                album,
            })
            .collect();

        Ok(albums)
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
        params: GetTracksParams,
    ) -> Result<Vec<TrackView>> {
        todo!()
    }

    /// Fetches songs containing a search term
    async fn search(&self, search_term: String, params: SearchParams) -> Result<Vec<SearchResult>> {
        todo!()
    }
}
