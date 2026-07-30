use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "genre")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub endpoint_id: String,
    #[sea_orm(belongs_to, from = "endpoint_id", to = "id")]
    pub endpoint: BelongsTo<super::endpoint::Entity>,
    pub name: Option<String>,
    pub image_url: Option<super::OrmUrl>,
    pub image_blur_hash: Option<String>,
    #[sea_orm(has_many, via = "track_genres")]
    pub tracks: HasMany<super::track::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
