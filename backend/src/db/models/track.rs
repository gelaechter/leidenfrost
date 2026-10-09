use std::collections::BTreeMap;

use chrono::NaiveDate;
use chrono::{Duration, NaiveDateTime};

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lyrics {
    Plain(Vec<String>),
    Synchronized(BTreeMap<Duration, String>),
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct Track {
    /// An ID that identifies this track uniquely
    /// This ID must be unique across endpoints while being deterministic
    /// The recommended approach is using: endpoint identifier (e.g. URL) +
    /// endpoint local ID
    pub id: String,
    pub endpoint_id: String,
    /// The album this track belongs to
    pub album_id: Option<String>,
    /// The disc this track belongs to
    pub disc_number: i64,
    /// The bitrate of this songs audio file
    pub bit_rate: Option<i64>,
    /// The bpm of this song
    pub bpm: Option<i64>,
    /// The number of audio channels in this audio file
    pub channels: Option<i64>,
    /// The container format of this songs audio file
    pub container: Option<String>,
    /// The duration of the song in seconds
    pub duration: Option<i64>,
    /// The URL of this tracks cover image
    pub image_url: Option<super::OrmUrl>,
    /// The blur hash of this tracks cover
    /// See <https://blurha.sh>/ for more information
    pub image_blur_hash: Option<String>,
    /// When this track was last played
    pub last_played_at: Option<NaiveDateTime>,
    /// The lyrics of this track
    pub lyrics: Option<Lyrics>,
    /// The name of this track
    pub title: Option<String>,
    /// The filepath of this track
    pub file_path: Option<String>,
    /// How often this track has been played
    pub play_count: Option<i64>,
    /// When this track was released
    pub release_date: Option<NaiveDate>,
    /// When this track was added to the endpoint
    /// This is for self hosted endpoints (e.g. JF/Navidrome)
    pub date_added: Option<NaiveDateTime>,
    /// The file size of this track in bytes
    pub file_size: Option<i64>,
    /// The URL pointing to the audio stream of this track
    pub stream_url: Option<super::OrmUrl>,
    /// The number of this track
    pub track_number: Option<i64>,
    /// If this track was favorited by the user
    pub user_favorite: Option<bool>,
}
