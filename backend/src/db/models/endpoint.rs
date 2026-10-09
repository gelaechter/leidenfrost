use sqlx::prelude::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct EndpointEntity {
    /// primary key
    pub id: String,
}