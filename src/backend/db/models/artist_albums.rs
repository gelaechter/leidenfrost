use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "artist_albums")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub artist_id: String,
    #[sea_orm(primary_key)]
    pub album_id: String,
    #[sea_orm(belongs_to, from = "artist_id", to = "id")]
    pub artist: Option<super::artist::Entity>,
    #[sea_orm(belongs_to, from = "album_id", to = "id")]
    pub album: Option<super::album::Entity>
}

impl ActiveModelBehavior for ActiveModel {}