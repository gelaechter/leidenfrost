use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "playlist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub image_url: Option<super::OrmUrl>,
    #[sea_orm(has_many, via = "playlist_tracks")]
    pub tracks: HasMany<super::track::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}