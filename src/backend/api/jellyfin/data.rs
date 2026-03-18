use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFGenericItem {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFExternalUrl {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFImageBlurHashes {
    pub primary: Option<HashMap<String, String>>,
    pub backdrop: Option<HashMap<String, String>>,
    pub logo: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFGenre {
    pub name: String,
    pub server_id: String,
    pub id: String,
    pub channel_id: Option<String>,
    pub r#type: String,
    pub image_tags: JFImageTags,
    pub backdrop_image_tags: Option<Vec<String>>,
    pub image_blur_hashes: JFImageBlurHashes,
    pub location_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFImageTags {
    pub logo: Option<String>,
    pub primary: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFMediaSources {
    pub bitrate: i64,
    pub container: String,
    pub default_audio_stream_index: i64,
    pub e_tag: String,
    pub gen_pts_input: bool,
    pub id: String,
    pub ignore_dts: bool,
    pub ignore_index: bool,
    pub is_infinite_stream: bool,
    pub is_remote: bool,
    pub name: String,
    pub path: String,
    pub protocol: String,
    pub read_at_native_framerate: bool,
    pub requires_closing: bool,
    pub requires_looping: bool,
    pub requires_opening: bool,
    pub run_time_ticks: i64,
    pub size: i64,
    pub supports_direct_play: bool,
    pub supports_direct_stream: bool,
    pub supports_probing: bool,
    pub supports_transcoding: bool,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFUserData {
    pub is_favorite: bool,
    pub key: String,
    pub play_count: i64,
    pub playback_position_ticks: i64,
    pub played: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFUser {
    pub id: String,
    pub name: Option<String>,
    pub image_tags: JFImageTags,
    pub has_password: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFSong {
    pub album: Option<String>,
    pub album_artists: Vec<JFGenericItem>,
    pub album_id: Option<String>,
    pub album_primary_image_tag: Option<String>,
    pub artist_items: Vec<JFGenericItem>,
    pub artists: Vec<String>,
    pub backdrop_image_tags: Option<Vec<String>>,
    pub channel_id: Option<String>,
    pub date_created: Option<String>,
    pub external_urls: Option<Vec<JFExternalUrl>>,
    pub genre_items: Option<Vec<JFGenericItem>>,
    pub genres: Option<Vec<String>>,
    pub id: String,
    pub image_tags: JFImageTags,
    pub image_blur_hashes: JFImageBlurHashes,
    pub index_number: Option<i64>,
    pub is_folder: bool,
    pub location_type: String,
    pub media_sources: Option<Vec<JFMediaSources>>,
    pub media_type: String,
    pub name: String,
    pub parent_index_number: Option<i64>,
    pub path: String,
    pub playlist_item_id: Option<String>,
    pub premiere_date: Option<String>,
    pub production_year: Option<i64>,
    pub run_time_ticks: Option<i64>,
    pub server_id: String,
    pub sort_name: Option<String>,
    pub r#type: String,
    pub user_data: Option<JFUserData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFAlbum {
    pub album_artists: Option<Vec<JFGenericItem>>,
    pub album_primary_image_tag: Option<String>,
    pub artist_items: Option<Vec<JFGenericItem>>,
    pub artists: Vec<String>,
    pub channel_id: Option<String>,
    pub child_count: Option<i64>,
    pub date_created: Option<String>,
    pub date_last_media_added: Option<String>,
    pub external_urls: Option<Vec<JFExternalUrl>>,
    pub genre_items: Option<Vec<JFGenericItem>>,
    pub genres: Option<Vec<String>>,
    pub id: String,
    pub image_tags: JFImageTags,
    pub image_blur_hashes: JFImageBlurHashes,
    pub is_folder: bool,
    pub location_type: String,
    pub name: String,
    pub parent_logo_image_tag: Option<String>,
    pub parent_logo_item_id: Option<String>,
    pub premiere_date: Option<String>,
    pub production_year: Option<i64>,
    pub run_time_ticks: i64,
    pub server_id: String,
    pub r#type: String,
    pub user_data: Option<JFUserData>,
    pub songs: Option<Vec<JFSong>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFAlbumArtist {
    pub backdrop_image_tags: Option<Vec<String>>,
    pub channel_id: Option<String>,
    pub date_created: Option<String>,
    pub external_urls: Option<Vec<JFExternalUrl>>,
    pub genre_items: Option<Vec<JFGenericItem>>,
    pub genres: Option<Vec<String>>,
    pub id: String,
    pub image_tags: JFImageTags,
    pub image_blur_hashes: JFImageBlurHashes,
    pub location_type: String,
    pub name: String,
    pub overview: Option<String>,
    pub run_time_ticks: Option<i64>,
    pub server_id: String,
    pub r#type: String,
    pub user_data: Option<JFUserData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JFPlaylist {
    pub name: String,
    pub server_id: String,
    pub id: String,
    pub run_time_ticks: Option<i64>,
    pub is_folder: Option<bool>,
    pub user_data: Option<JFUserData>,
    pub child_count: Option<i64>,
    pub image_tags: JFImageTags,
    pub backdrop_image_tags: Option<Vec<String>>,
    pub image_blur_hashes: JFImageBlurHashes,
}
