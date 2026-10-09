use sqlx::prelude::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct Playlist {
    pub id: String,
    pub endpoint_id: String,
    pub name: Option<String>,
    pub image_url: Option<super::OrmUrl>,
    pub image_blur_hash: Option<String>,
    pub user_favorite: Option<bool>,
}