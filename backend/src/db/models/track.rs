use std::collections::BTreeMap;

use chrono::NaiveDate;
use chrono::{Duration, NaiveDateTime};

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub enum Lyrics {
    Plain(Vec<String>),
    Synchronized(BTreeMap<Duration, String>),
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "track")]
pub struct Model {
    /// An ID that identifies this track uniquely
    /// This ID must be unique across endpoints while being deterministic
    /// The recommended approach is using: endpoint identifier (e.g. URL) +
    /// endpoint local ID
    #[sea_orm(primary_key)]
    pub id: String,
    pub endpoint_id: String,
    #[sea_orm(belongs_to, from = "endpoint_id", to = "id")]
    pub endpoint: HasOne<super::endpoint::Entity>,
    /// The album this track belongs to
    pub album_id: String,
    /// The disc this track belongs to
    pub disc_number: i64,
    /// The album this track belongs to
    #[sea_orm(belongs_to, from = "album_id", to = "id")]
    pub disc: HasOne<super::album::Entity>,
    /// The artists who made this song
    #[sea_orm(has_many, via = "artist_tracks")]
    pub artists: HasMany<super::artist::Entity>,
    /// The playlists that this track is in
    #[sea_orm(has_many, via = "playlist_tracks")]
    pub playlists: HasMany<super::playlist::Entity>,
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
    /// The genres this track belongs to
    #[sea_orm(has_many, via = "track_genres")]
    pub genres: HasMany<super::genre::Entity>,
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
    /// The file size of this track in bytes
    pub file_size: Option<i64>,
    /// The URL pointing to the audio stream of this track
    pub stream_url: Option<super::OrmUrl>,
    /// The number of this track
    pub track_number: Option<i64>,
    /// If this track was favorited by the user
    pub user_favorite: Option<bool>,
}

impl ActiveModelBehavior for ActiveModel {}