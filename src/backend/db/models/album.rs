use chrono::NaiveDate;
use sea_orm::prelude::*;

use crate::backend::db::models::OrmUrl;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "album")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    /// The artists who are credited as album artists
    /// This is not necessarily every artist that has
    /// worked on the album (i.e. collaborators / guests)
    #[sea_orm(has_many, via = "artist_albums")]
    pub artists: HasMany<super::artist::Entity>,
    /// The discs this album consists of
    /// they are what contains the tracks
    #[sea_orm(has_many)]
    pub discs: HasMany<super::disc::Entity>,
    /// When this album was released
    pub release_date: Option<NaiveDate>,
    /// The name of this album
    pub title: Option<String>,
    pub image_url: Option<OrmUrl>,
    pub image_blur_hash: Option<String>,
    pub user_rating: Option<i64>,
    pub user_favorite: Option<bool>,
}

impl ActiveModelBehavior for ActiveModel {}
