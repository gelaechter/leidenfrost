use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Settings {
    pub endpoint: Endpoint,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Copy)]
pub enum RepeatMode {
    None,
    RepeatSong,
    RepeatQueue,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Track {
    pub id: String,
    /// The name of the album this song belongs to
    pub album: Option<String>,
    /// The artists of the album this song belongs to
    pub album_artists: Vec<RelatedArtist>,
    /// The id of the album this song belongs to
    pub album_id: Option<String>,
    /// The name of the main artist who made this song
    pub artist_name: String,
    /// The artists who made this song
    pub artists: Vec<RelatedArtist>,
    /// The bitrate of this songs audio file
    pub bit_rate: i32,
    /// The bpm of this song
    pub bpm: Option<i32>,
    /// The number of audio channles in this audio file
    pub channels: Option<i32>,
    /// A comment for this song
    pub comment: Option<String>,
    /// Is this song a compilation?
    pub compilation: Option<bool>,
    /// The container format of this songs audio file
    pub container: Option<String>,
    pub created_at: String,
    pub disc_number: i32,
    /// The duration of the song in seconds
    pub duration: i32,
    pub genres: Vec<Genre>,
    pub image_url: Option<String>,
    pub image_blur_hash: Option<String>,
    pub last_played_at: Option<String>,
    pub lyrics: Option<String>,
    pub name: String,
    pub path: Option<String>,
    pub play_count: i32,
    pub playlist_item_id: String,
    pub release_date: Option<String>,
    pub release_year: Option<String>,
    pub server_id: String,
    pub endpoint: Endpoint,
    pub size: i32,
    pub stream_url: String,
    pub track_number: i32,
    pub updated_at: String,
    pub user_favorite: bool,
    pub user_rating: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Endpoint {
    Jellyfin,
    // NAVIDROME,
    // SUBSONIC,
    // FILE,
    // SPOTIFY,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Genre {
    pub id: String,
    pub image_url: Option<String>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Album {
    pub id: String,
    pub album_artists: Vec<RelatedArtist>,
    pub artists: Vec<RelatedArtist>,
    pub backdrop_image_url: Option<String>,
    pub created_at: String,
    pub duration: Option<i32>,
    pub genres: Vec<Genre>,
    pub image_url: Option<String>,
    pub image_blur_hash: Option<String>,
    pub is_compilation: Option<bool>,
    pub last_palyet_at: Option<String>,
    pub name: String,
    pub play_count: Option<i32>,
    pub release_date: Option<String>,
    pub release_year: Option<i32>,
    pub server_id: String,
    pub size: Option<i32>,
    pub song_count: Option<i32>,
    pub songs: Vec<Track>,
    pub updated_at: String,
    pub user_favorite: bool,
    pub user_rating: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct RelatedArtist {
    pub id: String,
    pub image_url: Option<String>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Artist {
    pub id: String,
    pub album_count: Option<i32>,
    pub biography: Option<String>,
    pub duration: Option<i32>,
    pub genres: Vec<Genre>,
    pub image_url: Option<String>,
    pub image_blur_hash: Option<String>,
    pub background_image_url: Option<String>,
    pub last_played_at: Option<String>,
    pub name: String,
    pub play_count: Option<i32>,
    pub server_id: String,
    pub similar_artists: Vec<RelatedArtist>,
    pub song_count: Option<i32>,
    pub user_favorite: bool,
    pub user_rating: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Playlist {
    pub id: String,
    pub image_url: Option<String>,
    pub image_blur_hash: Option<String>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub has_password: bool,
}
