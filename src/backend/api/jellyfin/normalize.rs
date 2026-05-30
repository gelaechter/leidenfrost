use std::str::FromStr;

use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::backend::{
    api::{
        endpoint_api::{
            AlbumSorting, ArtistSorting, GenreSorting, MusicEndpoint, PlaylistSorting, TrackSorting,
        },
        jellyfin::{
            api::JellyfinApi,
            data::{
                BaseItemDto, BaseItemDtoImageBlurHashes, BaseItemImageTags, BaseItemKind,
                ImageBlurHash, ItemSortBy, NameGuidPair, UserItemDataDto,
            },
        },
    },
    data_view::{
        AlbumView, ArtistView, GenreView, PlaylistView, RelatedArtist, RelatedGenre, TrackView,
    },
    db::models::{Album, Artist, Genre, Playlist, Track},
};

pub trait Normalize<T, R>: Sized + MusicEndpoint {
    fn normalize(&self, value: T) -> R;
}

trait WithOrEmpty<T> {
    fn or_empty(self) -> Vec<T>;
}

/// Short form of `unwrap_or_default` for `Option<Vec<T>>`
///  - None -> `Vec::new()`
///  - Some(vec) -> vec
impl<T> WithOrEmpty<T> for Option<Vec<T>> {
    fn or_empty(self) -> Vec<T> {
        self.unwrap_or_default()
    }
}

trait OrEmptySlice<T> {
    fn or_empty(&self) -> &[T];
}

/// Short form of `unwrap_or_default` for `Option<&Vec<T>>`
///  - None -> `Vec::new()`
///  - Some(vec) -> vec
impl<T> OrEmptySlice<T> for Option<&Vec<T>> {
    fn or_empty(&self) -> &[T] {
        self.map_or(&[], Vec::as_slice)
    }
}

impl Normalize<BaseItemDto, TrackView> for JellyfinApi {
    fn normalize(&self, value: BaseItemDto) -> TrackView {
        let BaseItemDto {
            album,
            album_id,
            album_primary_image_tag,
            artist_items,
            genre_items,
            container,
            id,
            image_blur_hashes,
            image_tags,
            index_number,
            media_streams,
            name,
            parent_index_number,
            premiere_date,
            run_time_ticks,
            type_,
            user_data,
            ..
        } = value;

        let UserItemDataDto {
            is_favorite,
            last_played_date,
            play_count,
            ..
        } = user_data.unwrap_or_default();

        // Check if our item is actually an audio track
        // TODO: Should probably be recoverable but since we should deserialize based on
        // BaseItemKind this is fine
        assert!(
            matches!(type_, Some(BaseItemKind::Audio)),
            "Tried to normalize a non BaseItemKind::Audio as a Track"
        );

        let track = Track {
            // Id's are guaranteed
            album_id: album_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            // As are disc numbers since they act as foreign keys
            disc_number: parent_index_number.unwrap_or(1),
            bit_rate: media_streams
                .as_ref()
                .or_empty()
                .iter()
                .find(|m| m.bit_rate.is_some())
                .and_then(|m| m.bit_rate),
            bpm: None,
            channels: media_streams
                .or_empty()
                .iter()
                .find(|m| m.channels.is_some())
                .and_then(|m| m.channels),
            container,
            duration: run_time_ticks.map(|t| t / 10_000_000),
            image_url: {
                let id = match image_tags {
                    // Either the current track has a primary image
                    Some(BaseItemImageTags {
                        primary: Some(_), ..
                    }) => id.clone(), // then we need the track id
                    // Or the album might have an image
                    _ if album_primary_image_tag.is_some() => album_id, // then we use the album id
                    _ => None,
                };

                id.map(|id| {
                    self.url()
                        .join(&format!("/Items/{id}/Images/Primary"))
                        .unwrap()
                        .into()
                })
            },
            image_blur_hash: image_blur_hashes.and_then(|blurhashes| {
                blurhashes
                    .primary
                    .or_empty()
                    .into_iter()
                    .next()
                    .map(|ImageBlurHash { blurhash, .. }| blurhash)
            }),
            last_played_at: last_played_date.and_then(|i| i.parse().ok()),
            // Lyrics are external and need to be fetched from an extra endpoint in JF
            lyrics: None,
            title: name,
            // File pathes are external as well
            file_path: None,
            play_count,
            release_date: premiere_date
                .and_then(|s| NaiveDateTime::from_str(&s).ok().map(|n| n.date())),
            // File size is external as well
            file_size: None,
            //
            stream_url: id.as_ref().map(|id| {
                self.url()
                    .join(&format!("/Items/{id}/Download"))
                    .unwrap()
                    .into()
            }),
            // Initialize after stream_url to allow borrowing
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            track_number: index_number,
            user_favorite: is_favorite,
        };

        TrackView {
            track,
            artists: artist_items
                .or_empty()
                .into_iter()
                .map(|NameGuidPair { id, name }| RelatedArtist { id, name })
                .collect(),
            album_name: album,
            genres: genre_items
                .or_empty()
                .into_iter()
                .map(|NameGuidPair { id, name }| RelatedGenre { id, name })
                .collect(),
        }
    }
}

impl Normalize<BaseItemDto, PlaylistView> for JellyfinApi {
    fn normalize(&self, value: BaseItemDto) -> PlaylistView {
        let BaseItemDto {
            id,
            name,
            image_tags,
            image_blur_hashes,
            run_time_ticks,
            child_count,
            type_,
            user_data,
            ..
        } = value;

        let UserItemDataDto { is_favorite, .. } = user_data.unwrap_or_default();

        // Check if our item is actually an audio track
        // TODO: Should probably be recoverable but since we should deserialize based on
        // BaseItemKind this is fine
        assert!(
            matches!(type_, Some(BaseItemKind::Playlist)),
            "Tried to normalize a non BaseItemKind::Playlist as a Playlist"
        );

        let playlist = Playlist {
            name,
            image_url: {
                let id = match image_tags {
                    // If the playlist has a primary image
                    Some(BaseItemImageTags {
                        primary: Some(_), ..
                    }) => id.clone(), // then we need the playlist id
                    _ => None,
                };

                id.map(|id| {
                    self.url()
                        .join(&format!("/Items/{id}/Images/Primary"))
                        .unwrap()
                        .into()
                })
            },
            image_blur_hash: image_blur_hashes.and_then(|blurhashes| {
                blurhashes
                    .primary
                    .or_empty()
                    .into_iter()
                    .next()
                    .map(|ImageBlurHash { blurhash, .. }| blurhash)
            }),
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_favorite: is_favorite,
        };

        PlaylistView {
            playlist,
            track_count: child_count,
            duration: run_time_ticks.map(|t| t / 10_000_000),
        }
    }
}

impl Normalize<BaseItemDto, AlbumView> for JellyfinApi {
    fn normalize(&self, value: BaseItemDto) -> AlbumView {
        let BaseItemDto {
            album_artists,
            genre_items,
            id,
            image_blur_hashes,
            image_tags,
            name,
            premiere_date,
            run_time_ticks,
            type_,
            user_data,
            ..
        } = value;

        let UserItemDataDto { is_favorite, .. } = user_data.unwrap_or_default();

        // Check if our item is actually an audio track
        // TODO: Should probably be recoverable but since we should deserialize based on
        // BaseItemKind this is fine
        assert!(
            matches!(type_, Some(BaseItemKind::MusicAlbum)),
            "Tried to normalize a non BaseItemKind::MusicAlbum as an Album"
        );

        let album = Album {
            title: name,
            release_date: premiere_date
                .and_then(|s| NaiveDateTime::from_str(&s).ok().map(|n| n.date())),
            image_url: {
                let id = match image_tags {
                    // If the playlist has a primary image
                    Some(BaseItemImageTags {
                        primary: Some(_), ..
                    }) => id.clone(), // then we need the playlist id
                    _ => None,
                };

                id.map(|id| {
                    self.url()
                        .join(&format!("/Items/{id}/Images/Primary"))
                        .unwrap()
                        .into()
                })
            },
            image_blur_hash: image_blur_hashes.and_then(|blurhashes| {
                blurhashes
                    .primary
                    .or_empty()
                    .into_iter()
                    .next()
                    .map(|ImageBlurHash { blurhash, .. }| blurhash)
            }),
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_favorite: is_favorite,
        };

        AlbumView {
            album,
            album_artists: album_artists
                .or_empty()
                .into_iter()
                .map(|NameGuidPair { id, name }| RelatedArtist { id, name })
                .collect(),
            genres: genre_items
                .or_empty()
                .into_iter()
                .map(|NameGuidPair { id, name }| RelatedGenre { id, name })
                .collect(),
            duration: run_time_ticks.map(|t| t / 10_000_000),
        }
    }
}

impl Normalize<BaseItemDto, GenreView> for JellyfinApi {
    fn normalize(&self, value: BaseItemDto) -> GenreView {
        let BaseItemDto {
            id,
            image_blur_hashes,
            image_tags,
            name,
            type_,
            ..
        } = value;

        // Check if our item is actually an audio track
        // TODO: Should probably be recoverable but since we should deserialize based on
        // BaseItemKind this is fine
        assert!(
            matches!(type_, Some(BaseItemKind::MusicGenre)),
            "Tried to normalize a non BaseItemKind::MusicGenre as a Genre"
        );

        let genre = Genre {
            name,
            image_url: {
                let id = match image_tags {
                    // If the playlist has a primary image
                    Some(BaseItemImageTags {
                        primary: Some(_), ..
                    }) => id.clone(), // then we need the playlist id
                    _ => None,
                };

                id.map(|id| {
                    self.url()
                        .join(&format!("/Items/{id}/Images/Primary"))
                        .unwrap()
                        .into()
                })
            },
            image_blur_hash: image_blur_hashes.and_then(|blurhashes| {
                blurhashes
                    .primary
                    .or_empty()
                    .into_iter()
                    .next()
                    .map(|ImageBlurHash { blurhash, .. }| blurhash)
            }),
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        };

        GenreView { genre }
    }
}

impl Normalize<BaseItemDto, ArtistView> for JellyfinApi {
    fn normalize(&self, value: BaseItemDto) -> ArtistView {
        let BaseItemDto {
            backdrop_image_tags,
            id,
            image_blur_hashes,
            image_tags,
            name,
            run_time_ticks,
            type_,
            user_data,
            ..
        } = value;

        let UserItemDataDto {
            is_favorite,
            rating,
            play_count,
            ..
        } = user_data.unwrap_or_default();

        // Check if our item is actually an audio track
        // TODO: Should probably be recoverable but since we should deserialize based on
        // BaseItemKind this is fine
        assert!(
            matches!(type_, Some(BaseItemKind::MusicArtist)),
            "Tried to normalize a non BaseItemKind::MusicArtist as an Artist"
        );

        let BaseItemDtoImageBlurHashes {
            primary, backdrop, ..
        } = image_blur_hashes.unwrap_or_default();

        let artist = Artist {
            name,
            image_url: {
                let id = match image_tags {
                    // If the playlist has a primary image
                    Some(BaseItemImageTags {
                        primary: Some(_), ..
                    }) => id.clone(), // then we need the playlist id
                    _ => None,
                };

                id.map(|id| {
                    self.url()
                        .join(&format!("/Items/{id}/Images/Primary"))
                        .unwrap()
                        .into()
                })
            },
            image_blur_hash: primary
                .or_empty()
                .into_iter()
                .next()
                .map(|ImageBlurHash { blurhash, .. }| blurhash),
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            biography: None,
            background_image_url: backdrop_image_tags.or_empty().first().map(|id| {
                self.url()
                    .join(&format!("Items/{id}/Images/Backdrop"))
                    .unwrap()
                    .into()
            }),
            background_image_blurhash: backdrop
                .or_empty()
                .into_iter()
                .next()
                .map(|ImageBlurHash { blurhash, .. }| blurhash),
            user_favorite: is_favorite,
            user_rating: rating.map(|f| f.round() as i64),
            play_count,
        };

        ArtistView {
            artist,
            duration: run_time_ticks.map(|t| t / 10_000_000),
        }
    }
}

impl From<TrackSorting> for ItemSortBy {
    fn from(value: TrackSorting) -> Self {
        match value {
            TrackSorting::Album => Self::Album,
            TrackSorting::AlbumArtist => Self::AlbumArtist,
            TrackSorting::Title => Self::Name,
            TrackSorting::Artist => Self::Artist,
            TrackSorting::Duration => Self::Runtime,
            TrackSorting::PlayCount => Self::PlayCount,
            TrackSorting::Random => Self::Random,
            TrackSorting::DateAdded => Self::DateLastContentAdded, //TODO: check if this is
            // correct
            TrackSorting::DatePlayed => Self::DatePlayed,
            TrackSorting::DateReleased => Self::PremiereDate,
        }
    }
}

impl From<AlbumSorting> for ItemSortBy {
    fn from(value: AlbumSorting) -> Self {
        match value {
            AlbumSorting::Name => Self::Name,
            AlbumSorting::AlbumArtist => Self::AlbumArtist,
            AlbumSorting::TrackCount => panic!("Unsupported"), //TODO: check if this is correct
            AlbumSorting::Duration => Self::Runtime,
            AlbumSorting::DateAdded => Self::DateLastContentAdded,
            AlbumSorting::DateReleased => Self::PremiereDate,
            AlbumSorting::Random => Self::Random,
        }
    }
}

impl From<ArtistSorting> for ItemSortBy {
    fn from(value: ArtistSorting) -> Self {
        match value {
            ArtistSorting::Name => Self::Name,
            ArtistSorting::AlbumCount => panic!("Unsupported"),
            ArtistSorting::TrackCount => panic!("Unsupported"),
            ArtistSorting::Duration => Self::Runtime,
            ArtistSorting::Random => Self::Random,
        }
    }
}

impl From<GenreSorting> for ItemSortBy {
    fn from(value: GenreSorting) -> Self {
        match value {
            GenreSorting::Name => Self::Name,
            GenreSorting::AlbumCount => panic!("Unsupported"),
            GenreSorting::TrackCount => panic!("Unsupported"),
            GenreSorting::Duration => Self::Runtime,
            GenreSorting::Random => Self::Random,
        }
    }
}

impl From<PlaylistSorting> for ItemSortBy {
    fn from(value: PlaylistSorting) -> Self {
        match value {
            PlaylistSorting::Name => Self::Name,
            PlaylistSorting::AlbumCount => panic!("Unsupported"),
            PlaylistSorting::TrackCount => panic!("Unsupported"),
            PlaylistSorting::Duration => Self::Runtime,
            PlaylistSorting::Random => Self::Random,
        }
    }
}
