use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "disc")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub number: i64,
    #[sea_orm(primary_key)]
    pub album_id: String,
    #[sea_orm(belongs_to, from = "album_id", to = "id")]
    pub album: HasOne<super::album::Entity>,
    #[sea_orm(has_many)]
    pub tracks: HasMany<super::track::Entity>
}

impl ActiveModelBehavior for ActiveModel {}
