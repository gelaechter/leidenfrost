use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "endpoint")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(has_many)]
    pub artists: HasMany<super::artist::Entity>,
    #[sea_orm(has_many)]
    pub albums: HasMany<super::album::Entity>,
    #[sea_orm(has_many)]
    pub tracks: HasMany<super::track::Entity>,
    #[sea_orm(has_many)]
    pub genre: HasMany<super::genre::Entity>,
    #[sea_orm(has_many)]
    pub playlist: HasMany<super::playlist::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
