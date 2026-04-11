
use crate::backend::db::models::Track;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedArtist {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedAlbum {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedGenre {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackView {
    pub track: Track,
    /// The artists who created this track
    pub artists: Vec<RelatedArtist>,
    /// The name of the album this track belongs to
    pub album_name: Option<String>,
    /// The genres that apply to this track
    pub genres: Vec<RelatedGenre>,
}
