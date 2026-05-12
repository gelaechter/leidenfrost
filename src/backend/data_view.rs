use crate::backend::db::models::{Album, Disc, Playlist, Track};
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

#[derive(Clone, Debug)]
pub struct AlbumView {
    pub album: Album,
    /// The artists specifically credited as the creators of the album
    pub album_artists: Vec<RelatedArtist>,
    /// The discs/tracks this album consists of
    pub genres: Vec<RelatedGenre>,
    /// The duration of the album in seconds
    pub duration: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct DiscView {
    pub disc: Disc,
    pub track_view: Vec<TrackView>,
}

/// A playlist view only holds metadata about a playlist
/// If you need the playlist tracks use [`ApiContract::`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistView {
    pub playlist: Playlist,
    pub track_count: Option<i64>,
    /// The duration of the playlist in seconds
    pub duration: Option<i64>,
}
