use reqwest::RequestBuilder;
use url::Url;

use crate::backend::{
    data_view::{PlaylistView, TrackView},
    db::models::{Album, Artist, Disc, Genre, Playlist, Track},
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

/// An API that supports multiple users
// pub trait MultiUsers {
//     /// Fetches all available users
//     fn get_users(&self, url: Url) -> impl std::future::Future<Output = Vec<User>> + Send;
// }

pub trait Pagination {
    /// Limit the amounts of items retrieved by specifying:
    ///   - `start`: the starting index of the fetch (i.e. skip the first n items)
    ///   - `limit`: the amounts of items to fetch in this request
    fn limit_request(request: RequestBuilder, start: i64, limit: i64) -> RequestBuilder;
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
    Track(Track),
    Album(Album),
    Playlist(Playlist),
    Genre(Genre),
    LyricMatch { matched_lyric: String, track: Track },
}

pub trait ApiContract {
    /// Fetches a specific track
    async fn get_track(&self, song_id: String) -> TrackView;

    /// Fetches all tracks
    async fn get_tracks(&self) -> color_eyre::Result<Vec<TrackView>>;

    /// Fetches all songs from an album
    async fn get_songs_from_album(&self, album_id: String) -> color_eyre::Result<Vec<Disc>>;

    /// Fetches a specific album
    async fn get_album(&self, album_id: String) -> color_eyre::Result<Album>;

    /// Fetches all albums
    async fn get_albums(&self) -> color_eyre::Result<Vec<Album>>;

    /// Fetches albums from an artist
    async fn get_albums_from_artist(&self, artist_id: String) -> color_eyre::Result<Vec<Album>>;

    /// Fetches an artist
    async fn get_artist(&self, artist_id: String) -> color_eyre::Result<Artist>;

    /// Fetches all artists
    async fn get_artists(&self) -> color_eyre::Result<Vec<Artist>>;

    /// Fetches all genres
    async fn get_genres(&self) -> color_eyre::Result<Vec<Genre>>;

    /// Fetches all playlists
    async fn get_playlists(&self) -> color_eyre::Result<Vec<PlaylistView>>;

    // Fetches a playlist
    async fn get_playlist(&self, playlist_id: String) -> color_eyre::Result<PlaylistView>;

    /// Fetches playlist tracks
    async fn get_playlist_tracks(&self, playlist_id: String) -> color_eyre::Result<Vec<TrackView>>;

    /// Fetches songs containing a search term
    async fn search(&self, search_term: String) -> color_eyre::Result<Vec<SearchResult>>;
}
