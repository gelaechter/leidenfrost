use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

pub mod album;
pub mod artist;
pub mod endpoint;
pub mod genre;
pub mod playlist;
pub mod track;

use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteTypeInfo, SqliteValueRef};
use sqlx::{Database, Decode, Encode, Sqlite, Type};

use crate::db::models::track::Lyrics;

/// A [`url::Url`] wrapper that plays nice with SeaORM
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

impl Type<Sqlite> for OrmUrl {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for OrmUrl {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, BoxDynError> {
        <String as Encode<'q, Sqlite>>::encode(self.0.as_str().to_owned(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for OrmUrl {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<Sqlite>>::decode(value)?;
        Ok(OrmUrl(url::Url::parse(&s)?))
    }
}

impl Type<Sqlite> for Lyrics {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for Lyrics {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, BoxDynError> {
        <String as Encode<'q, Sqlite>>::encode(serde_json::to_string(self)?, buf)
    }
}

impl<'r> Decode<'r, Sqlite> for Lyrics {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_str(&s)?)
    }
}
