use chrono::NaiveDate;

use crate::backend::{
    api::endpoint_api::ApiContract,
    data::{Album, Artist, Genre, Playlist, RelatedArtist, Track, User},
};

use super::api::JellyfinApi;
use super::data::JFGenre;
use super::data::JFPlaylist;
use super::data::JFSong;
use super::data::JFUser;
use super::data::{JFAlbum, JFAlbumArtist};

/// Essentially a specialized version of the From trait that
/// also passes a self for access to the API
pub trait Normalize<T, R>: Sized + ApiContract {
    #[must_use]
    fn normalize(&self, value: T) -> R;
}

impl Normalize<JFSong, Track> for JellyfinApi {
    fn normalize(&self, value: JFSong) -> Track {
        let JFSong {
            album,
            album_artists,
            album_id,
            album_primary_image_tag,
            artist_items,
            artists,
            backdrop_image_tags,
            channel_id,
            date_created,
            external_urls,
            genre_items,
            genres,
            id,
            image_tags,
            image_blur_hashes,
            index_number,
            is_folder,
            location_type,
            media_sources,
            media_type,
            name,
            parent_index_number,
            path,
            playlist_item_id,
            premiere_date,
            production_year,
            run_time_ticks,
            server_id,
            sort_name,
            r#type,
            user_data,
        } = value;

        Track {
            album_id,
            artists: artist_items
                .into_iter()
                .map(|f| RelatedArtist {
                    id: f.id,
                    image_url: None,
                    name: f.name,
                })
                .collect(),
            bit_rate: media_sources
                .unwrap_or_default()
                .first()
                .map_or(0, |m| m.bitrate / 1000),
            bpm: None,
            channels: None,
            comment: None,
            compilation: None,
            container: media_sources
                .unwrap_or_default()
                .first()
                .map(|m| m.container.clone()),
            disc_number: parent_index_number,
            duration: run_time_ticks.map(|i| i / 10_000_000),
            genres: genre_items
                .unwrap_or_default()
                .into_iter()
                .map(|i| Genre {
                    id: i.id,
                    image_url: None,
                    name: i.name,
                })
                .collect(),
            id,
            image_url: image_tags.primary.map(|_| {
                self.url()
                    .join(&format!("Items/{}/Images/Primary", &id))
                    .expect("should be a valid URL")
            }),
            image_blur_hash: image_blur_hashes.primary.and_then(|h| h.map(|t| t.0)),
            last_played_at: None,
            lyrics: None,
            name,
            file_path: None,
            play_count: user_data.map(|f| f.play_count),
            playlist_id: playlist_item_id,
            release_date: premiere_date
                .map(|s| {
                    // See here for formatter: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
                    NaiveDate::parse_from_str(&s, "%+").expect("Expected valid ISO 8601 DateTime")
                })
                .or(production_year.map(|i| NaiveDate::from_yo_opt(i as i32, 0).unwrap())),
            size: media_sources.and_then(|v| v.first().map(|s| s.size)),
            stream_url: self
                .url()
                .join(&format!("Items/{}/Download", value.id))
                .expect("should be a valid URL"),
            track_number: index_number,
            user_favorite: user_data.map_or(false, |u| u.is_favorite),
            album,
            album_artists: album_artists
                .iter()
                .map(|a| RelatedArtist {
                    id: a.id,
                    image_url: None,
                    name: a.name,
                })
                .collect(),
            artist_name: artist_items.first().map(|a| a.name).unwrap_or_default(),
        }
    }
}

impl Normalize<JFAlbum, Album> for JellyfinApi {
    fn normalize(&self, value: JFAlbum) -> Album {
        let JFAlbum {
            album_artists,
            album_primary_image_tag,
            artist_items,
            artists,
            channel_id,
            child_count,
            date_created,
            date_last_media_added,
            external_urls,
            genre_items,
            genres,
            id,
            image_tags,
            image_blur_hashes,
            is_folder,
            location_type,
            name,
            parent_logo_image_tag,
            parent_logo_item_id,
            premiere_date,
            production_year,
            run_time_ticks,
            server_id,
            r#type,
            user_data,
            songs,
        } = value;

        Album {
            album_artists: album_artists
                .unwrap_or_default()
                .into_iter()
                .map(|generic_item| RelatedArtist {
                    id: generic_item.id,
                    image_url: None,
                    name: generic_item.name,
                })
                .collect(),
            artists: artist_items
                .unwrap_or_default()
                .into_iter()
                .map(|f| RelatedArtist {
                    id: f.id.clone(),
                    image_url: None,
                    name: f.name.clone(),
                })
                .collect(),
            backdrop_image_url: None,
            created_at: date_created.clone().unwrap_or_default(),
            duration: Some(run_time_ticks / 10_000_000),
            genres: genre_items
                .unwrap_or_default()
                .into_iter()
                .map(|f| Genre {
                    id: f.id,
                    image_url: None,
                    name: f.name,
                })
                .collect(),
            id,
            image_url: image_tags.primary.map(|_| {
                self.url()
                    .join(&format!("Items/{}/Images/Primary", &value.id))
                    .expect("should be a valid URL")
            }),
            image_blur_hash: image_blur_hashes.primary.and_then(|h| h.map(|t| t.0)),
            is_compilation: None,
            last_palyet_at: None,
            name,
            play_count: user_data.map(|f| f.play_count),
            release_date: premiere_date
                .map(|s| {
                    // See here for formatter: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
                    NaiveDate::parse_from_str(&s, "%+").expect("Expected valid ISO 8601 DateTime")
                })
                .or(production_year.map(|i| NaiveDate::from_yo_opt(i as i32, 0).unwrap())),
            server_id: "".to_string(),
            size: None,
            song_count: child_count,
            songs: songs
                .unwrap_or_default()
                .iter()
                .map(|song| self.normalize(song.clone()))
                .collect(),
            updated_at: date_last_media_added
                .or(value.date_created)
                .unwrap_or_default(),
            user_favorite: value
                .user_data
                .map(|user_data| user_data.is_favorite)
                .unwrap_or_default(),
            user_rating: None,
        }
    }
}

impl Normalize<JFAlbumArtist, Artist> for JellyfinApi {
    fn normalize(&self, value: JFAlbumArtist) -> Artist {
        Artist {
            album_count: None,
            background_image_url: None,
            biography: value.overview,
            duration: value.run_time_ticks.map(|t| t / 10_000_000),
            genres: value
                .genre_items
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|f| Genre {
                    id: f.id.clone(),
                    image_url: None,
                    name: f.name.clone(),
                })
                .collect(),
            id: value.id.clone(),
            image_url: {
                value
                    .image_tags
                    .primary
                    .as_ref()
                    .map(|_| format!("{}Items/{}/Images/Primary", self.url(), &value.id))
            },
            last_played_at: None,
            name: value.name,
            play_count: value.user_data.clone().map(|u| u.play_count),
            server_id: String::new(),
            similar_artists: vec![],
            song_count: None,
            user_favorite: value.user_data.map(|u| u.is_favorite).unwrap_or_default(),
            user_rating: None,
            image_blur_hash: {
                value
                    .image_blur_hashes
                    .primary
                    .as_ref()
                    .and_then(|h| h.values().next().cloned())
            },
        }
    }
}

impl Normalize<JFGenre, Genre> for JellyfinApi {
    fn normalize(&self, value: JFGenre) -> Genre {
        Genre {
            id: value.id.clone(),
            image_url: {
                value
                    .image_tags
                    .primary
                    .as_ref()
                    .map(|_| format!("{}Items/{}/Images/Primary", self.url(), &value.id))
            },
            name: value.name,
        }
    }
}

impl Normalize<JFPlaylist, Playlist> for JellyfinApi {
    fn normalize(&self, value: JFPlaylist) -> Playlist {
        let JFPlaylist {
            name,
            server_id,
            id,
            run_time_ticks,
            is_folder,
            user_data,
            child_count,
            image_tags,
            backdrop_image_tags,
            image_blur_hashes,
        } = value;

        Playlist {
            id,
            image_url: image_tags.primary.map(|_| {
                self.url()
                    .join(&format!("Items/{}/Images/Primary", &value.id))
                    .expect("should be a valid URL")
            }),
            name,
            image_blur_hash: image_blur_hashes
                .primary
                .and_then(|h| h.values().next().cloned()),
        }
    }
}

impl Normalize<JFUser, User> for JellyfinApi {
    fn normalize(&self, value: JFUser) -> User {
        User {
            id: value.id.clone(),
            name: value.name.clone(),
            image_url: value
                .image_tags
                .primary
                .as_ref()
                .map(|_| format!("{}Items/{}/Images/Primary", self.url(), &value.id)),
            has_password: value.has_password.unwrap_or_default(),
        }
    }
}
