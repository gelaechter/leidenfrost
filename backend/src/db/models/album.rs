use chrono::{NaiveDate, NaiveDateTime};
use sqlx::prelude::FromRow;

use crate::db::models::OrmUrl;

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct Album {
    /// Primary key
    pub id: String,
    pub endpoint_id: String,
    /// When this album was released
    pub release_date: Option<NaiveDate>,
    /// When this track was added to the endpoint
    /// This is for self hosted endpoints (e.g. JF/Navidrome)
    pub date_added: Option<NaiveDateTime>,
    /// The name of this album
    pub title: Option<String>,
    pub image_url: Option<OrmUrl>,
    pub image_blur_hash: Option<String>,
    pub user_favorite: Option<bool>,
}
