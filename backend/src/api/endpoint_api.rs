use std::{
    collections::HashSet,
    result,
    sync::{Arc, LazyLock},
};

use async_trait::async_trait;
use reqwest::RequestBuilder;
use sea_orm::DeriveDisplay;
use serde::{Deserialize, Serialize};
use tokio::sync::{OnceCell, RwLock};
use url::Url;

use crate::{
    api::jellyfin::errors::ApiError,
    data_view::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView},
    db::sqlite::{EndpointDB, IndexableEndpoint},
};

/// The different Endpoints that are currently supported
#[derive(Clone, Debug, PartialEq, DeriveDisplay)]
pub enum EndpointType {
    Jellyfin,
    // Navidrome,
    // Spotify,
    // File
    // Subsonic
}

impl EndpointType {
    pub const ALL: [EndpointType; 1] = [EndpointType::Jellyfin];
}

pub trait UserPasswordAuth
where
    Self: std::marker::Sized,
{
    /// Authenticate a
    async fn auth_user_password(url: Url, username: String, password: String) -> Result<Self>;
}

pub trait ImageSize {
    /// Sets the resolution of an API image
    /// This can for example be done through url parameters like
    ///   - width
    ///   - height
    ///   - original
    fn set_image_resolution(request: RequestBuilder, width: u32, height: u32) -> RequestBuilder;
}

pub enum SearchResult {
    Track(TrackView),
    Album(AlbumView),
    Playlist(PlaylistView),
    Genre(GenreView),
    LyricMatch {
        matched_lyric: Option<String>,
        track: TrackView,
    },
}

/// Differentiating between "appears on"- and "created by"-albums can
/// technically be done through checking if the artist is part of the albums
/// album-artists list but doing it this way is nicer IMO
#[derive(Debug, Clone)]
pub struct ArtistAlbums {
    /// Albums an artist has a track on
    pub appears_on: Vec<AlbumView>,
    /// Albums where the artist is credited as album artist
    pub created: Vec<AlbumView>,
}

pub type Result<T> = result::Result<T, ApiError>;

/// What is the API capable of
#[derive(Debug, Clone)]
pub struct Capabilities {
    pub pagination: bool,
    pub image_sizing: bool,
    pub indexable: bool,
    // The Api supports sorting track views by the following
    pub track_sorting: HashSet<TrackSorting>,
    pub album_sorting: HashSet<AlbumSorting>,
    pub artist_sorting: HashSet<ArtistSorting>,
    pub playlist_sorting: HashSet<PlaylistSorting>,
    pub genre_sorting: HashSet<GenreSorting>,
}

// TODO: This will quickly cause issues if not in sync with the sorting enums
// Consider stealing VariantArray from strum
pub static FULL_CAPABILITIES: LazyLock<Capabilities> = LazyLock::new(|| Capabilities {
    pagination: true,
    image_sizing: true,
    indexable: true,
    track_sorting: HashSet::from([
        TrackSorting::Album,
        TrackSorting::AlbumArtist,
        TrackSorting::Title,
        TrackSorting::Artist,
        TrackSorting::Duration,
        TrackSorting::PlayCount,
        TrackSorting::Random,
        TrackSorting::DateAdded,
        TrackSorting::DatePlayed,
        TrackSorting::DateReleased,
    ]),
    album_sorting: HashSet::from([
        AlbumSorting::Name,
        AlbumSorting::AlbumArtist,
        AlbumSorting::TrackCount,
        AlbumSorting::Duration,
        AlbumSorting::DateAdded,
        AlbumSorting::DateReleased,
        AlbumSorting::Random,
    ]),
    artist_sorting: HashSet::from([
        ArtistSorting::Name,
        ArtistSorting::AlbumCount,
        ArtistSorting::TrackCount,
        ArtistSorting::Duration,
        ArtistSorting::Random,
    ]),
    playlist_sorting: HashSet::from([
        PlaylistSorting::Name,
        PlaylistSorting::AlbumCount,
        PlaylistSorting::TrackCount,
        PlaylistSorting::Duration,
        PlaylistSorting::Random,
    ]),
    genre_sorting: HashSet::from([
        GenreSorting::Name,
        GenreSorting::AlbumCount,
        GenreSorting::TrackCount,
        GenreSorting::Duration,
        GenreSorting::Random,
    ]),
});

impl Capabilities {
    /// Produces the intersection of two capabilities, thus
    pub fn intersection(self, other: &Capabilities) -> Capabilities {
        Capabilities {
            pagination: self.pagination && other.pagination,
            image_sizing: self.pagination && other.pagination,
            indexable: self.indexable && other.indexable,
            track_sorting: self
                .track_sorting
                .intersection(&other.track_sorting)
                .copied()
                .collect(),
            album_sorting: self
                .album_sorting
                .intersection(&other.album_sorting)
                .copied()
                .collect(),
            artist_sorting: self
                .artist_sorting
                .intersection(&other.artist_sorting)
                .copied()
                .collect(),
            playlist_sorting: self
                .playlist_sorting
                .intersection(&other.playlist_sorting)
                .copied()
                .collect(),
            genre_sorting: self
                .genre_sorting
                .intersection(&other.genre_sorting)
                .copied()
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pagination {
    /// Which page to start with; This is zero indexed
    pub start_page: u64,
    /// How many records one page should contain
    pub limit: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Sort<T>
where
    T: std::fmt::Debug + Clone + Default,
{
    pub order: SortOrder,
    pub by: T,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TrackSorting {
    Album,
    AlbumArtist,
    #[default]
    Title,
    Artist,
    Duration,
    PlayCount,
    Random,
    DateAdded,
    DatePlayed,
    DateReleased,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AlbumSorting {
    #[default]
    Name,
    AlbumArtist,
    TrackCount,
    Duration,
    DateAdded,
    DateReleased,
    Random,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ArtistSorting {
    #[default]
    Name,
    AlbumCount,
    TrackCount,
    Duration,
    Random,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GenreSorting {
    #[default]
    Name,
    AlbumCount,
    TrackCount,
    Duration,
    Random,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PlaylistSorting {
    #[default]
    Name,
    AlbumCount,
    TrackCount,
    Duration,
    Random,
}

#[derive(Debug, Clone, Default)]
pub struct GetTracksParams {
    pub pagination: Option<Pagination>,
    pub sorting: Option<Sort<TrackSorting>>,
}

#[derive(Debug, Clone, Default)]
pub struct GetAlbumsParams {
    pub pagination: Option<Pagination>,
    pub sorting: Option<Sort<AlbumSorting>>,
}

#[derive(Debug, Clone, Default)]
pub struct GetGenresParams {
    pub pagination: Option<Pagination>,
    pub sorting: Option<Sort<GenreSorting>>,
}

#[derive(Debug, Clone, Default)]
pub struct GetArtistsParams {
    pub pagination: Option<Pagination>,
    pub sorting: Option<Sort<ArtistSorting>>,
}

#[derive(Debug, Clone, Default)]
pub struct GetPlaylistsParams {
    pub pagination: Option<Pagination>,
    pub sorting: Option<Sort<PlaylistSorting>>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub pagination: Option<Pagination>,
}

/// Design assumptions about an API:
#[async_trait]
pub trait MusicEndpoint {
    /// Provides the capabilities of this API
    fn capabilities(&self) -> Capabilities;

    /// Fetches a specific track
    async fn get_track(&self, track_id: String) -> Result<TrackView>;

    /// Fetches all tracks
    async fn get_tracks(&self, params: GetTracksParams) -> Result<Vec<TrackView>>;

    /// Fetches a specific album
    async fn get_album(&self, album_id: String) -> Result<AlbumView>;

    /// Fetches all songs from an album
    async fn get_album_tracks(
        &self,
        album_id: String,
        params: GetTracksParams,
    ) -> Result<Vec<TrackView>>;

    /// Fetches all albums
    async fn get_albums(&self, params: GetAlbumsParams) -> Result<Vec<AlbumView>>;

    /// Fetches albums from an artist
    async fn get_artist_albums(
        &self,
        artist_id: String,
        params: GetAlbumsParams,
    ) -> Result<ArtistAlbums>;

    /// Fetches an artist
    async fn get_artist(&self, artist_id: String) -> Result<ArtistView>;

    /// Fetches all artists
    async fn get_artists(&self, params: GetArtistsParams) -> Result<Vec<ArtistView>>;

    /// Fetches all genres
    async fn get_genres(&self, params: GetGenresParams) -> Result<Vec<GenreView>>;

    /// Fetches all playlists
    async fn get_playlists(&self, params: GetPlaylistsParams) -> Result<Vec<PlaylistView>>;

    // Fetches a playlist
    async fn get_playlist(&self, playlist_id: String) -> Result<PlaylistView>;

    /// Fetches playlist tracks
    async fn get_playlist_tracks(
        &self,
        playlist_id: String,
        params: GetPlaylistsParams,
    ) -> Result<Vec<TrackView>>;

    /// Fetches songs containing a search term
    async fn search(&self, search_term: String, params: SearchParams) -> Result<Vec<SearchResult>>;
}

impl std::fmt::Debug for dyn MusicEndpoint + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MusicEndpoint{{{:?}}}", self.capabilities())
    }
}

impl std::fmt::Debug for dyn IndexableEndpoint + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexableEndpoint{{{:?}}}", self.capabilities())
    }
}

/// Is it an online or an offline endpoint
#[derive(Clone, Debug)]
pub enum EndpointKind {
    Online {
        /// The endpoint
        endpoint: Arc<dyn MusicEndpoint + Send + Sync>,
    },
    Indexable {
        /// Is indexing activated
        indexing: bool,
        /// The indexable endpoint
        endpoint: Arc<dyn IndexableEndpoint + Send + Sync>,
    },
}

impl EndpointKind {
    pub fn unwrap(self) -> Arc<dyn MusicEndpoint + Send + Sync> {
        match self {
            EndpointKind::Online { endpoint: e } => e as Arc<dyn MusicEndpoint + Send + Sync>,
            EndpointKind::Indexable {
                endpoint: e,
                indexing: _,
            } => e as Arc<dyn MusicEndpoint + Send + Sync>,
        }
    }
}

// Tokio OnceCell and RwLock since we need to write/access the EndpointManager
// from async contexts anyways
static ENDPOINTS: OnceCell<RwLock<EndpointManager>> = OnceCell::const_new();

/// An endpoint manager that provides access to either one or multiple endpoints
#[derive(Clone)]
pub struct EndpointManager {
    local_db: EndpointDB,
    selected_endpoints: Vec<EndpointKind>,
}

impl EndpointManager {
    /// Constructs a new `EndpointManager`
    async fn new() -> Result<Self> {
        log::debug!("Initializing EndpointManager");
        let endpoint_manager = Self {
            local_db: EndpointDB::open().await?,
            selected_endpoints: Vec::new(),
        };
        log::debug!("Initialized EndpointManager!");
        Ok(endpoint_manager)
    }

    /// Gets or initializes the global EndpointManager instance
    async fn get_manager() -> Result<&'static RwLock<EndpointManager>> {
        ENDPOINTS
            .get_or_try_init(|| async { EndpointManager::new().await.map(RwLock::new) })
            .await
    }

    /// Updates the endpoints currently available to the manager
    pub async fn update_endpoints(endpoints: Vec<EndpointKind>) -> Result<()> {
        log::debug!("update_endpoints: trying to acquire manager lock!");
        let lock = Self::get_manager().await?;
        let mut manager = lock.write().await;
        log::debug!("update_endpoints: got manager lock!");

        manager.selected_endpoints = endpoints;
        Ok(())
    }

    /// Will either return the active endpoint if [`Self::selected_endpoints`]
    /// as a single entry or return the LocalIndex
    pub async fn get_active_endpoint() -> Result<Arc<dyn MusicEndpoint + Send + Sync>> {
        log::debug!("get_active_endpoint: trying to acquire manager lock!");
        let lock = Self::get_manager().await?;
        let manager = lock.read().await;
        log::debug!("get_active_endpoint: got manager lock!");

        // Both are cheap to clone ([`Arc`] and [`DatabaseConnection`] respectively)
        // FIXME: Remove false
        let endpoint = if manager.selected_endpoints.len() == 1 && false {
            manager.selected_endpoints[0].clone().unwrap()
        } else {
            Arc::new(manager.local_db.clone()) as Arc<dyn MusicEndpoint + Send + Sync>
        };

        Ok(endpoint)
    }

    /// Index all the endpoints which are currently set to index
    pub async fn index_all_endpoints() -> Result<()> {
        log::debug!("index_all_endpoints: trying to acquire manager lock!");
        let lock = Self::get_manager().await?;
        let manager = lock.read().await;
        log::debug!("index_all_endpoints: got manager lock!");

        for indexable_endpoint in manager.selected_endpoints.iter().filter_map(|e| match e {
            // Only index indexable endpoints which have indexing activated
            EndpointKind::Indexable { indexing, endpoint } if *indexing => Some(endpoint.clone()),
            _ => None,
        }) {
            indexable_endpoint.index_data(&manager.local_db.db).await;
        }

        Ok(())
    }
}
