use sea_orm::prelude::*;

use crate::db::models::OrmUrl;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "artist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub endpoint_id: String,
    #[sea_orm(belongs_to, from = "endpoint_id", to = "id")]
    pub endpoint: BelongsTo<super::endpoint::Entity>,
    pub name: Option<String>,
    #[sea_orm(has_many, via = "artist_albums")]
    pub albums: HasMany<super::album::Entity>,
    pub biography: Option<String>,
    pub image_url: Option<OrmUrl>,
    pub image_blur_hash: Option<String>,
    pub background_image_url: Option<OrmUrl>,
    pub background_image_blurhash: Option<String>,
    pub play_count: Option<i64>,
    pub user_favorite: Option<bool>,
    pub user_rating: Option<i64>,
    /// The tracks the artist has performed
    #[sea_orm(has_many, via = "artist_tracks")]
    pub tracks: HasMany<super::track::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

/// A Link to go directly from album to artists
pub struct ArtistToAlbums;

impl Linked for ArtistToAlbums {
    type FromEntity = super::artist::Entity;
    type ToEntity = super::album::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            super::artist_albums::Relation::Artist.def().rev(), // artist -> artist_albums
            super::artist_albums::Relation::Album.def(),        // artist_albums  -> album
        ]
    }
}

/// A Link to go directly from artist to album via the tracks
pub struct ArtistToAlbumsViaTracks;

impl Linked for ArtistToAlbumsViaTracks {
    type FromEntity = super::artist::Entity;
    type ToEntity = super::album::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            super::artist_tracks::Relation::Artist.def().rev(), // artist -> artist_tracks
            super::artist_tracks::Relation::Track.def(),        // artist_tracks  -> track
            super::track::Relation::Album.def(),                // track -> album
        ]
    }
}
