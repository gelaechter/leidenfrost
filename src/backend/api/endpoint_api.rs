use std::{
    collections::HashSet,
    result,
    sync::{Arc, LazyLock},
};

use async_trait::async_trait;
use iced::futures::stream;
use reqwest::RequestBuilder;
use sea_orm::{DatabaseConnection, DeriveDisplay};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::backend::{
    api::jellyfin::errors::ApiError,
    data_view::{AlbumView, ArtistView, DiscView, GenreView, PlaylistView, TrackView},
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
    pub fn intersection(self, other: Capabilities) -> Capabilities {
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
    pub start: i64,
    pub limit: i64,
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
pub struct GetPlaylistParams {
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

    /// Provides a unique ID for this API
    /// This is used to matched indexed data in the database to the respective
    /// API
    fn get_id(&self) -> String;

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
    ) -> Result<Vec<DiscView>>;

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
    async fn get_playlists(&self, params: GetPlaylistParams) -> Result<Vec<PlaylistView>>;

    // Fetches a playlist
    async fn get_playlist(&self, playlist_id: String) -> Result<PlaylistView>;

    /// Fetches playlist tracks
    async fn get_playlist_tracks(
        &self,
        playlist_id: String,
        params: GetPlaylistParams,
    ) -> Result<Vec<TrackView>>;

    /// Fetches songs containing a search term
    async fn search(&self, search_term: String, params: SearchParams) -> Result<Vec<SearchResult>>;
}

#[derive(Default, Clone)]
pub struct EndpointManager {
    db: Option<DatabaseConnection>,
    selected_endpoints: Vec<Arc<dyn MusicEndpoint + Send + Sync>>,
}

impl EndpointManager {
    pub fn set_selected_endpoints(&mut self, endpoints: Vec<Arc<dyn MusicEndpoint + Send + Sync>>) {
        self.selected_endpoints = endpoints;
    }
}

#[async_trait]
impl MusicEndpoint for EndpointManager {
    /// Provides the capabilities of this API
    fn capabilities(&self) -> Capabilities {
        let init = (*FULL_CAPABILITIES).clone();
        self.selected_endpoints
            .iter()
            .fold(init, |acc, e| acc.intersection(e.capabilities()))
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
        todo!()
    }

    /// Fetches all tracks
    async fn get_tracks(&self, params: GetTracksParams) -> Result<Vec<TrackView>> {
        todo!()
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
    ) -> Result<Vec<DiscView>> {
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
    async fn get_playlists(&self, params: GetPlaylistParams) -> Result<Vec<PlaylistView>> {
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
        params: GetPlaylistParams,
    ) -> Result<Vec<TrackView>> {
        todo!()
    }

    /// Fetches songs containing a search term
    async fn search(&self, search_term: String, params: SearchParams) -> Result<Vec<SearchResult>> {
        todo!()
    }
}
