use std::{collections::BTreeMap, time::Duration};

use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde::Serialize;
use url::Url;

pub struct Disc {
    number: i64,
    tracks: Vec<Track>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Lyrics {
    Plain(Vec<String>),
    Synchronized(BTreeMap<Duration, String>),
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
    pub bit_rate: i64,
    /// The bpm of this song
    pub bpm: Option<i64>,
    /// The number of audio channles in this audio file
    pub channels: Option<i64>,
    /// A comment for this song
    pub comment: Option<String>,
    /// Is this song a compilation?
    pub compilation: Option<bool>,
    /// The container format of this songs audio file
    pub container: Option<String>,
    pub disc_number: Option<i64>,
    /// The duration of the song in seconds
    pub duration: Option<i64>,
    /// The genres this track belongs to
    pub genres: Vec<Genre>,
    /// The url of this tracks cover image
    pub image_url: Option<Url>,
    /// The blur hash of this tracks cover
    /// See https://blurha.sh/ for more information
    pub image_blur_hash: Option<String>,
    /// When this track was last played
    pub last_played_at: Option<NaiveDateTime>,
    /// The lyrics of this track
    pub lyrics: Option<Lyrics>,
    /// The name of this track
    pub name: String,
    /// The file path of this track
    pub file_path: Option<String>,
    /// How often this track has been played
    pub play_count: Option<i64>,
    /// The id of the playlist if this track belongs to one
    pub playlist_id: Option<String>,
    /// When this track was realeased
    pub release_date: Option<NaiveDate>,
    /// The file size of this track in bytes
    pub size: Option<i64>,
    /// The url pointing to the audio stream of this track
    pub stream_url: Url,
    /// The number of this track
    pub track_number: Option<i64>,
    /// If this track was favorited by the user
    pub user_favorite: bool,
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
    pub duration: Option<i64>,
    pub genres: Vec<Genre>,
    pub image_url: Option<Url>,
    pub image_blur_hash: Option<String>,
    pub is_compilation: Option<bool>,
    pub last_palyet_at: Option<String>,
    pub name: String,
    pub play_count: Option<i64>,
    pub release_date: Option<NaiveDate>,
    pub server_id: String,
    pub size: Option<i64>,
    pub song_count: Option<i64>,
    pub songs: Vec<Track>,
    pub updated_at: String,
    pub user_favorite: bool,
    pub user_rating: Option<i64>,
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
    pub album_count: Option<i64>,
    pub biography: Option<String>,
    pub duration: Option<i64>,
    pub genres: Vec<Genre>,
    pub image_url: Option<String>,
    pub image_blur_hash: Option<String>,
    pub background_image_url: Option<String>,
    pub last_played_at: Option<String>,
    pub name: String,
    pub play_count: Option<i64>,
    pub server_id: String,
    pub similar_artists: Vec<RelatedArtist>,
    pub song_count: Option<i64>,
    pub user_favorite: bool,
    pub user_rating: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Playlist {
    pub id: String,
    pub image_url: Option<Url>,
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
