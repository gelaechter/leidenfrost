use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "track_genres")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub track_id: String,
    #[sea_orm(primary_key)]
    pub genre_id: String,
    #[sea_orm(belongs_to, from = "track_id", to = "id")]
    pub track: BelongsTo<super::track::Entity>,
    #[sea_orm(belongs_to, from = "genre_id", to = "id")]
    pub genre: BelongsTo<super::genre::Entity>
}

impl ActiveModelBehavior for ActiveModel {}