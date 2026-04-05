use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "playlist_tracks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub track_id: String,
    #[sea_orm(primary_key)]
    pub playlist_id: String,
    #[sea_orm(belongs_to, from = "track_id", to = "id")]
    pub track: Option<super::track::Entity>,
    #[sea_orm(belongs_to, from = "playlist_id", to = "id")]
    pub playlist: Option<super::playlist::Entity>
}

impl ActiveModelBehavior for ActiveModel {}