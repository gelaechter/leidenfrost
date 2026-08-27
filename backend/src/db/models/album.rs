
use chrono::NaiveDate;
use sea_orm::prelude::*;

use crate::db::models::OrmUrl;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "album")]
pub struct Model<T> {
    #[sea_orm(primary_key)]
    pub id: String,
    pub endpoint_id: String,
    #[sea_orm(belongs_to, from = "endpoint_id", to = "id")]
    pub endpoint: BelongsTo<super::endpoint::Entity>,
    /// The artists who are credited as album artists
    /// This is not necessarily every artist that has
    /// worked on the album (i.e. collaborators / guests)
    #[sea_orm(has_many, via = "artist_albums")]
    pub artists: HasMany<super::artist::Entity>,
    /// The tracks this album consists of
    #[sea_orm(has_many)]
    pub tracks: HasMany<super::track::Entity>,
    /// When this album was released
    pub release_date: Option<NaiveDate>,
    /// The name of this album
    pub title: Option<String>,
    pub image_url: Option<OrmUrl>,
    pub image_blur_hash: Option<String>,
    pub user_favorite: Option<bool>,
}

impl ActiveModelBehavior for ActiveModel {}

/// A Link to go directly from album to artists
pub struct AlbumToArtists;

impl Linked for AlbumToArtists {
    type FromEntity = super::album::Entity;
    type ToEntity = super::artist::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            super::artist_albums::Relation::Album.def().rev(), // album -> artist_albums
            super::artist_albums::Relation::Artist.def(),      // artist_albums  -> artist
        ]
    }
}

/// A Link to go directly from album to genres
pub struct AlbumToGenres;
impl Linked for AlbumToGenres {
    type FromEntity = super::album::Entity;
    type ToEntity = super::genre::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            super::album::Relation::Track.def(),              // album -> track
            super::track_genres::Relation::Track.def().rev(), // track  -> track_genres
            super::track_genres::Relation::Genre.def(),       // track_genres  -> genres
        ]
    }
}
