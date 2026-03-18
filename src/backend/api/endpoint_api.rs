use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::backend::data::{Album, Artist, Disc, Genre, Playlist, Track, User};

pub trait SettingsContract {}

#[derive(Serialize, Deserialize)]
pub enum EndpointAuths {
    JFUserPassword {
        url: Url,
        username: String,
        password: String,
    },
}

pub trait MultiUsers {
    /// Fetches all available users
    async fn get_users(url: Url) -> Vec<User>;
}

pub trait ApiContract {
    /// Fetches a specific track
    async fn get_track(&self, song_id: String) -> Track;

    /// Fetches all tracks
    async fn get_tracks(&self, start: i32, limit: i32) -> color_eyre::Result<Vec<Track>>;

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

    /// Fetches a list of genres
    async fn get_genres(&self) -> color_eyre::Result<Vec<Genre>>;

    /// Fetches a list of genres
    async fn get_playlists(&self) -> color_eyre::Result<Vec<Playlist>>;

    /// Fetches songs containing a search term
    async fn search_songs(&self, search_term: String) -> color_eyre::Result<Vec<Track>>;

    /// Fetches albums containing a search term
    async fn search_albums(&self, search_term: String) -> color_eyre::Result<Vec<Album>>;

    /// Fetches artists containing a search term
    async fn search_artists(&self, search_term: String) -> color_eyre::Result<Vec<Artist>>;
}
