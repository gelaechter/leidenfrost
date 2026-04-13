use std::str::FromStr;

use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::backend::{
    api::{
        endpoint_api::ApiContract,
        jellyfin::{
            api::JellyfinApi,
            data::{
                BaseItemDto, BaseItemImageTags, BaseItemKind, ImageBlurHash, NameGuidPair,
                UserItemDataDto,
            },
        },
    },
    data_view::{PlaylistView, RelatedArtist, RelatedGenre, TrackView},
    db::models::{Playlist, Track},
};

pub trait Normalize<T, R>: Sized + ApiContract {
    #[must_use]
    fn normalize(&self, value: T) -> R;
}

/// Short form of `unwrap_or_default` for `Option<Vec<T>>`
///  - None -> Vec::new()
///  - Some(vec) -> vec
trait WithOrEmpty<T> {
    fn or_empty(self) -> Vec<T>;
}

impl<T> WithOrEmpty<T> for Option<Vec<T>> {
    fn or_empty(self) -> Vec<T> {
        self.unwrap_or_default()
    }
}

/// Short form of `unwrap_or_default` for `Option<&Vec<T>>`
///  - None -> Vec::new()
///  - Some(vec) -> vec
trait OrEmptySlice<T> {
    fn or_empty(&self) -> &[T];
}

impl<T> OrEmptySlice<T> for Option<&Vec<T>> {
    fn or_empty(&self) -> &[T] {
        self.map(|v| v.as_slice()).unwrap_or(&[])
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
        // TODO: Should probably be recoverable but since we should deserialize based on BaseItemKind this is fine
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
        // TODO: Should probably be recoverable but since we should deserialize based on BaseItemKind this is fine
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
