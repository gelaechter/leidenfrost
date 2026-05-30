use std::{collections::HashSet, result};

use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::backend::{
    api::jellyfin::errors::ApiError,
    data_view::{AlbumView, ArtistView, DiscView, GenreView, PlaylistView, TrackView},
};

/// The different Endpoints that are currently supported
#[derive(Clone, Debug)]
pub enum Endpoint {
    Jellyfin,
    // Navidrome,
    // Spotify,
    // File
    // Subsonic
}

pub trait UserPasswordAuth {
    /// Authenticate a
    async fn auth_user_password(url: Url, username: String, password: String) -> Self;
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

/// Differentiating between these kinds of albums can technically be done
/// through checking if the artist is part of the albums album artists list
/// but doing it this way is nicer IMO
pub struct ArtistAlbums {
    /// Albums an artist has a track on
    pub appears_on: Vec<AlbumView>,
    /// Albums where the artist is credited as album artist
    pub created: Vec<AlbumView>,
}

pub type Result<T> = result::Result<T, ApiError>;

/// What is the API capable of
pub struct Capabilities {
    pub pagination: bool,
    pub image_sizing: bool,
    // The Api supports sorting track views by the following
    pub track_sorting: HashSet<TrackSorting>,
    pub album_sorting: HashSet<AlbumSorting>,
    pub artist_sorting: HashSet<ArtistSorting>,
    pub playlist_sorting: HashSet<PlaylistSorting>,
    pub genre_sorting: HashSet<GenreSorting>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub enum ArtistSorting {
    #[default]
    Name,
    AlbumCount,
    TrackCount,
    Duration,
    Random,
}

#[derive(Debug, Clone, Default)]
pub enum GenreSorting {
    #[default]
    Name,
    AlbumCount,
    TrackCount,
    Duration,
    Random,
}

#[derive(Debug, Clone, Default)]
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
pub trait MusicEndpoint {
    /// Provides the capabilities of this API
    fn capabilities() -> Capabilities;

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
