use sea_orm::IntoActiveModel;

use crate::backend::db::models::{Album, Artist, Genre, Playlist, Track, artist};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedArtist {
    pub id: String,
    pub name: Option<String>,
}

impl From<Artist> for RelatedArtist {
    fn from(artist: Artist) -> Self {
        RelatedArtist {
            id: artist.id,
            name: artist.name,
        }
    }
}

/// This only needs the name as the item itself usually carries the album id
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

impl From<Genre> for RelatedGenre {
    fn from(genre: Genre) -> Self {
        Self {
            id: genre.id,
            name: genre.name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackView {
    pub track: Track,
    /// The artists who created this track
    pub artists: Vec<RelatedArtist>,
    /// The of the album this track belongs to
    /// [`Track`] already carries the id of the album
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
pub struct ArtistView {
    pub artist: Artist,
    /// The summed duration of this artists tracks
    pub duration: Option<i64>,
}

impl From<ArtistView> for artist::ActiveModel {
    fn from(value: ArtistView) -> Self {
        value.artist.into_active_model()
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenreView {
    pub genre: Genre,
}
