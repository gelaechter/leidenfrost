use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, DatabaseConnection, DeriveIden,
    EntityTrait, IntoActiveModel, JoinType, ModelTrait, QueryOrder, QuerySelect, RelationTrait,
    sea_query::Expr,
};

use crate::backend::{
    api::{
        endpoint_api::{
            ArtistAlbums, Capabilities, GetAlbumsParams, GetArtistsParams, GetGenresParams,
            GetPlaylistsParams, GetTracksParams, MusicEndpoint, Pagination, SearchParams,
            SearchResult, SortOrder, TrackSorting,
        },
        jellyfin::errors::ApiError,
    },
    data_view::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView},
    db::models::{endpoint, playlist},
};

use super::models::{album, artist, genre, track};

type Result<T> = std::result::Result<T, ApiError>;

// TODO: Update path
const DB_PATH: &str = "/home/***REMOVED***/Projects/randale_iced/test.sqlite";

// Read/Write/Create
const MODE: &str = "rwc";

/// Trait that indexes an endpoint's data into the database
/// This allows the endpoints to be used together
trait IndexableEndpoint: MusicEndpoint {
    /// Naive implementation which just requests all datatypes without paging
    async fn index_data(&self, db: &DatabaseConnection) {
        // Endpoint ID
        endpoint::ActiveModel {
            id: ActiveValue::Set(self.get_id()),
        }
        .insert(db)
        .await;

        // Artists
        if let Ok(artist_views) = self
            .get_artists(GetArtistsParams {
                pagination: None,
                sorting: None,
            })
            .await
        {
            let iter = artist_views
                .into_iter()
                .map(|view| view.artist.into_active_model());

            artist::Entity::insert_many(iter).exec(db).await;
        }

        // Albums
        if let Ok(album_views) = self
            .get_albums(GetAlbumsParams {
                pagination: None,
                sorting: None,
            })
            .await
        {
            let iter = album_views
                .into_iter()
                .map(|view| view.album.into_active_model());

            album::Entity::insert_many(iter).exec(db).await;
        }

        // Tracks
        if let Ok(artist_views) = self
            .get_tracks(GetTracksParams {
                pagination: None,
                sorting: None,
            })
            .await
        {
            let iter = artist_views
                .into_iter()
                .map(|view| view.track.into_active_model());

            track::Entity::insert_many(iter).exec(db).await;
        }

        // Genres
        if let Ok(artist_views) = self
            .get_genres(GetGenresParams {
                pagination: None,
                sorting: None,
            })
            .await
        {
            let iter = artist_views
                .into_iter()
                .map(|view| view.genre.into_active_model());

            genre::Entity::insert_many(iter).exec(db).await;
        }

        // Playlist
        if let Ok(artist_views) = self
            .get_playlists(GetPlaylistsParams {
                pagination: None,
                sorting: None,
            })
            .await
        {
            let iter = artist_views
                .into_iter()
                .map(|view| view.playlist.into_active_model());

            playlist::Entity::insert_many(iter).exec(db).await;
        }
    }
}

/// The local database which acts as an index for endpoints
#[derive(Debug, Clone)]
pub struct EndpointDB {
    db: DatabaseConnection,
}

impl EndpointDB {
    pub async fn open() -> Result<Self> {
        let db: DatabaseConnection =
            Database::connect(format!("sqlite://{DB_PATH}?mode={MODE}")).await?;

        // TODO: Naive check if database works
        db.ping().await?;

        // Synchronizes database schema with entity definitions
        //
        // FIXME: Schema discovery is not Send + Sync the best solution here is probably
        //  to split database intialization and usage through the EndpointManager.
        db.get_schema_registry(module_path!().split("::").next().unwrap())
            .sync(&db)
            .await?;

        Ok(EndpointDB { db })
    }
}

#[async_trait]
impl MusicEndpoint for EndpointDB {
    /// Provides the capabilities of this API
    fn capabilities(&self) -> Capabilities {
        todo!()
    }

    /// Provides a unique ID for this API
    /// This is used to matched indexed data in the database to the respective
    /// API
    fn get_id(&self) -> String {
        // Not required since this directly accesses the database
        String::new()
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
            .find_also(track::Entity, album::Entity) //   track -> album
            .join(
                //                                        album -> album_artist
                JoinType::LeftJoin,
                super::models::artist_albums::Relation::Album.def(),
            )
            .join_as(
                //                                        album_artist -> artist (AlbumArtist)
                JoinType::LeftJoin,
                super::models::artist_albums::Relation::Artist.def(),
                AlbumArtist,
            );

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

    /// Fetches all genres
    async fn get_genres(&self, params: GetGenresParams) -> Result<Vec<GenreView>> {
        todo!()
    }

    /// Fetches all playlists
    async fn get_playlists(&self, params: GetPlaylistsParams) -> Result<Vec<PlaylistView>> {
        todo!()
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
