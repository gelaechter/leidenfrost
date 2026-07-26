use sea_orm::prelude::*;

/// A junction table defining all the tracks that an artist
/// has composed 
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "artist_tracks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub artist_id: String,
    #[sea_orm(primary_key)]
    pub track_id: String,
    #[sea_orm(belongs_to, from = "artist_id", to = "id")]
    pub artist: BelongsTo<super::artist::Entity>,
    #[sea_orm(belongs_to, from = "track_id", to = "id")]
    pub track: BelongsTo<super::track::Entity>
}

impl ActiveModelBehavior for ActiveModel {}