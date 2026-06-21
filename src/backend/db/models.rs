use std::ops::{Deref, DerefMut};

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

pub mod album;
pub mod artist;
pub mod artist_albums;
pub mod artist_tracks;
pub mod disc;
pub mod genre;
pub mod playlist;
pub mod playlist_tracks;
pub mod track;
pub mod track_genres;
pub mod endpoint;

pub use album::Model as Album;
pub use artist::Model as Artist;
pub use disc::Model as Disc;
pub use genre::Model as Genre;
pub use playlist::Model as Playlist;
pub use track::Model as Track;

/// A [`url::Url`] wrapper that plays nice with SeaORM
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[repr(transparent)]
pub struct OrmUrl(url::Url);

impl Deref for OrmUrl {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OrmUrl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<url::Url> for OrmUrl {
    fn from(value: url::Url) -> Self {
        Self(value)
    }
}

impl From<OrmUrl> for url::Url {
    fn from(value: OrmUrl) -> Self {
        value.0
    }
}

impl<'a> From<&'a OrmUrl> for &'a url::Url {
    fn from(value: &'a OrmUrl) -> Self {
        &value.0
    }
}
