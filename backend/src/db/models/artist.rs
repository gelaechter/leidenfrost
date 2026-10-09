use chrono::NaiveDateTime;
use sqlx::prelude::FromRow;

use crate::db::models::OrmUrl;

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct Artist {
    /// primary key
    pub id: String,
    pub endpoint_id: String,
    pub name: Option<String>,
    pub biography: Option<String>,
    pub image_url: Option<OrmUrl>,
    pub image_blur_hash: Option<String>,
    pub background_image_url: Option<OrmUrl>,
    pub background_image_blurhash: Option<String>,
    pub play_count: Option<i64>,
    pub user_favorite: Option<bool>,
    pub user_rating: Option<i64>,
    /// When this track was added to the endpoint
    /// This is for self hosted endpoints (e.g. JF/Navidrome)
    pub date_added: Option<NaiveDateTime>,
}
