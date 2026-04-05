use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "artist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    #[sea_orm(has_many, via = "artist_albums")]
    pub albums: HasMany<super::album::Entity>,
    pub biography: Option<String>,
    pub image_url: Option<String>,
    pub image_blur_hash: Option<String>,
    pub background_image_url: Option<String>,
    pub play_count: Option<i64>,
    pub user_favorite: Option<bool>,
    pub user_rating: Option<i64>,
    /// The tracks the artist has performed
    #[sea_orm(has_many, via = "artist_tracks")]
    pub tracks: HasMany<super::track::Entity>
}

impl ActiveModelBehavior for ActiveModel {}
