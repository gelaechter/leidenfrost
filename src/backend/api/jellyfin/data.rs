use std::collections::HashMap;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

/// Query result container
/// Returned by the ItemsApi
/// <https://typescript-sdk.jellyfin.org/classes/generated-client.ItemsApi.html#getitems>
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemDtoQueryResult {
    pub items: Vec<BaseItemDto>,
    pub start_index: i64,
    pub total_record_count: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemDto {
    pub air_days: Option<Vec<DayOfWeek>>,
    pub airs_after_season_number: Option<i64>,
    pub airs_before_episode_number: Option<i64>,
    pub airs_before_season_number: Option<i64>,
    pub air_time: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub album_artists: Option<Vec<NameGuidPair>>,
    pub album_count: Option<i64>,
    pub album_id: Option<String>,
    pub album_primary_image_tag: Option<String>,
    pub altitude: Option<f64>,
    pub aperture: Option<f64>,
    pub artist_count: Option<i64>,
    pub artist_items: Option<Vec<NameGuidPair>>,
    pub artists: Option<Vec<String>>,
    pub aspect_ratio: Option<String>,
    pub audio: Option<ProgramAudio>,
    pub backdrop_image_tags: Option<Vec<String>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub can_delete: Option<bool>,
    pub can_download: Option<bool>,
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub channel_number: Option<String>,
    pub channel_primary_image_tag: Option<String>,
    pub channel_type: Option<ChannelType>,
    pub chapters: Option<Vec<ChapterInfo>>,
    pub child_count: Option<i64>,
    pub collection_type: Option<CollectionType>,
    pub community_rating: Option<f64>,
    pub completion_percentage: Option<f64>,
    pub container: Option<String>,
    pub critic_rating: Option<f64>,
    pub cumulative_run_time_ticks: Option<i64>,
    pub current_program: Option<Box<BaseItemDto>>,
    pub custom_rating: Option<String>,
    pub date_created: Option<String>,
    pub date_last_media_added: Option<String>,
    pub display_order: Option<String>,
    pub display_preferences_id: Option<String>,
    pub enable_media_source_display: Option<bool>,
    pub end_date: Option<String>,
    pub episode_count: Option<i64>,
    pub episode_title: Option<String>,
    pub etag: Option<String>,
    pub exposure_time: Option<f64>,
    pub external_urls: Option<Vec<ExternalUrl>>,
    pub extra_type: Option<ExtraType>,
    pub focal_length: Option<f64>,
    pub forced_sort_name: Option<String>,
    pub genre_items: Option<Vec<NameGuidPair>>,
    pub genres: Option<Vec<String>>,
    pub has_lyrics: Option<bool>,
    pub has_subtitles: Option<bool>,
    pub height: Option<i64>,
    pub id: Option<String>,
    pub image_blur_hashes: Option<BaseItemDtoImageBlurHashes>,
    pub image_orientation: Option<ImageOrientation>,
    pub image_tags: Option<BaseItemImageTags>,
    pub index_number: Option<i64>,
    pub index_number_end: Option<i64>,
    pub is_folder: Option<bool>,
    pub is_hd: Option<bool>,
    pub is_kids: Option<bool>,
    pub is_live: Option<bool>,
    pub is_movie: Option<bool>,
    pub is_news: Option<bool>,
    pub iso_speed_rating: Option<f64>,
    pub iso_type: Option<IsoType>,
    pub is_place_holder: Option<bool>,
    pub is_premiere: Option<bool>,
    pub is_repeat: Option<bool>,
    pub is_series: Option<bool>,
    pub is_sports: Option<bool>,
    pub latitude: Option<f64>,
    pub local_trailer_count: Option<i64>,
    pub location_type: Option<LocationType>,
    pub lock_data: Option<bool>,
    pub locked_fields: Option<Vec<MetadataField>>,
    pub longitude: Option<f64>,
    pub media_source_count: Option<i64>,
    pub media_sources: Option<Vec<MediaSourceInfo>>,
    pub media_streams: Option<Vec<MediaStream>>,
    pub media_type: Option<MediaType>,
    pub movie_count: Option<i64>,
    pub music_video_count: Option<i64>,
    pub name: Option<String>,
    pub normalization_gain: Option<f64>,
    pub number: Option<String>,
    pub official_rating: Option<String>,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub parent_art_image_tag: Option<String>,
    pub parent_art_item_id: Option<String>,
    pub parent_backdrop_image_tags: Option<Vec<String>>,
    pub parent_backdrop_item_id: Option<String>,
    pub parent_id: Option<String>,
    pub parent_index_number: Option<i64>,
    pub parent_logo_image_tag: Option<String>,
    pub parent_logo_item_id: Option<String>,
    pub parent_primary_image_item_id: Option<String>,
    pub parent_primary_image_tag: Option<String>,
    pub parent_thumb_image_tag: Option<String>,
    pub parent_thumb_item_id: Option<String>,
    pub part_count: Option<i64>,
    pub path: Option<String>,
    pub people: Option<Vec<BaseItemPerson>>,
    pub play_access: Option<PlayAccess>,
    pub playlist_item_id: Option<String>,
    pub preferred_metadata_country_code: Option<String>,
    pub preferred_metadata_language: Option<String>,
    pub premiere_date: Option<String>,
    pub primary_image_aspect_ratio: Option<f64>,
    pub production_locations: Option<Vec<String>>,
    pub production_year: Option<i64>,
    pub program_count: Option<i64>,
    pub program_id: Option<String>,
    pub provider_ids: Option<HashMap<String, Option<String>>>,
    pub recursive_item_count: Option<i64>,
    pub remote_trailers: Option<Vec<MediaUrl>>,
    pub run_time_ticks: Option<i64>,
    pub screenshot_image_tags: Option<Vec<String>>,
    pub season_id: Option<String>,
    pub season_name: Option<String>,
    pub series_count: Option<i64>,
    pub series_id: Option<String>,
    pub series_name: Option<String>,
    pub series_primary_image_tag: Option<String>,
    pub series_studio: Option<String>,
    pub series_thumb_image_tag: Option<String>,
    pub series_timer_id: Option<String>,
    pub server_id: Option<String>,
    pub shutter_speed: Option<f64>,
    pub software: Option<String>,
    pub song_count: Option<i64>,
    pub sort_name: Option<String>,
    pub source_type: Option<String>,
    pub special_feature_count: Option<i64>,
    pub start_date: Option<String>,
    pub status: Option<String>,
    pub studios: Option<Vec<NameGuidPair>>,
    pub taglines: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub timer_id: Option<String>,
    pub trailer_count: Option<i64>,
    pub trickplay: Option<HashMap<String, HashMap<String, TrickplayInfoDto>>>,
    pub type_: Option<BaseItemKind>,
    pub user_data: Option<UserItemDataDto>,
    pub video3d_format: Option<Video3DFormat>,
    pub video_type: Option<VideoType>,
    pub width: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DayOfWeek {
    Friday,
    Monday,
    Saturday,
    Sunday,
    Thursday,
    Tuesday,
    Wednesday,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct NameGuidPair {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProgramAudio {
    Atmos,
    Dolby,
    DolbyDigital,
    Mono,
    Stereo,
    Thx,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChannelType {
    Radio,
    Tv,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ChapterInfo {
    pub image_date_modified: Option<String>,
    pub image_path: Option<String>,
    pub image_tag: Option<String>,
    pub name: Option<String>,
    pub start_position_ticks: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CollectionType {
    Books,
    Boxsets,
    Folders,
    Homevideos,
    Livetv,
    Movies,
    Music,
    Musicvideos,
    Photos,
    Playlists,
    Trailers,
    Tvshows,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExternalUrl {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExtraType {
    BehindTheScenes,
    Clip,
    DeletedScene,
    Featurette,
    Interview,
    Sample,
    Scene,
    Short,
    ThemeSong,
    ThemeVideo,
    Trailer,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemDtoImageBlurHashes {
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub art: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub backdrop: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub banner: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub box_: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub box_rear: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub chapter: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub disc: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub logo: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub menu: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub primary: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub profile: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub screenshot: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub thumb: Option<Vec<ImageBlurHash>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ImageOrientation {
    BottomLeft,
    BottomRight,
    LeftBottom,
    LeftTop,
    RightBottom,
    RightTop,
    TopLeft,
    TopRight,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IsoType {
    BluRay,
    Dvd,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocationType {
    FileSystem,
    Offline,
    Remote,
    Virtual,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MetadataField {
    Cast,
    Genres,
    Name,
    OfficialRating,
    Overview,
    ProductionLocations,
    Runtime,
    Studios,
    Tags,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaSourceInfo {
    pub analyze_duration_ms: Option<i64>,
    pub bitrate: Option<i64>,
    pub buffer_ms: Option<i64>,
    pub container: Option<String>,
    pub default_audio_stream_index: Option<i64>,
    pub default_subtitle_stream_index: Option<i64>,
    pub encoder_path: Option<String>,
    pub encoder_protocol: Option<MediaProtocol>,
    pub etag: Option<String>,
    pub fallback_max_streaming_bitrate: Option<i64>,
    pub formats: Option<Vec<String>>,
    pub gen_pts_input: Option<bool>,
    pub has_segments: Option<bool>,
    pub id: Option<String>,
    pub ignore_dts: Option<bool>,
    pub ignore_index: Option<bool>,
    pub is_infinite_stream: Option<bool>,
    pub iso_type: Option<IsoType>,
    pub is_remote: Option<bool>,
    pub live_stream_id: Option<String>,
    pub media_attachments: Option<Vec<MediaAttachment>>,
    pub media_streams: Option<Vec<MediaStream>>,
    pub name: Option<String>,
    pub open_token: Option<String>,
    pub path: Option<String>,
    pub protocol: Option<MediaProtocol>,
    pub read_at_native_framerate: Option<bool>,
    pub required_http_headers: Option<HashMap<String, Option<String>>>,
    pub requires_closing: Option<bool>,
    pub requires_looping: Option<bool>,
    pub requires_opening: Option<bool>,
    pub run_time_ticks: Option<i64>,
    pub size: Option<i64>,
    pub supports_direct_play: Option<bool>,
    pub supports_direct_stream: Option<bool>,
    pub supports_probing: Option<bool>,
    pub supports_transcoding: Option<bool>,
    pub timestamp: Option<TransportStreamTimestamp>,
    pub transcoding_container: Option<String>,
    pub transcoding_sub_protocol: Option<MediaStreamProtocol>,
    pub transcoding_url: Option<String>,
    pub type_: Option<MediaSourceType>,
    pub use_most_compatible_transcoding_profile: Option<bool>,
    pub video3d_format: Option<Video3DFormat>,
    pub video_type: Option<VideoType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaStream {
    pub aspect_ratio: Option<String>,
    pub audio_spatial_format: Option<AudioSpatialFormat>,
    pub average_frame_rate: Option<f64>,
    pub bit_depth: Option<i64>,
    pub bit_rate: Option<i64>,
    pub bl_present_flag: Option<i64>,
    pub channel_layout: Option<String>,
    pub channels: Option<i64>,
    pub codec: Option<String>,
    pub codec_tag: Option<String>,
    pub codec_time_base: Option<String>,
    pub color_primaries: Option<String>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub comment: Option<String>,
    pub delivery_method: Option<SubtitleDeliveryMethod>,
    pub delivery_url: Option<String>,
    pub display_title: Option<String>,
    pub dv_bl_signal_compatibility_id: Option<i64>,
    pub dv_level: Option<i64>,
    pub dv_profile: Option<i64>,
    pub dv_version_major: Option<i64>,
    pub dv_version_minor: Option<i64>,
    pub el_present_flag: Option<i64>,
    pub hdr10_plus_present_flag: Option<bool>,
    pub height: Option<i64>,
    pub index: Option<i64>,
    pub is_anamorphic: Option<bool>,
    pub is_avc: Option<bool>,
    pub is_default: Option<bool>,
    pub is_external: Option<bool>,
    pub is_external_url: Option<bool>,
    pub is_forced: Option<bool>,
    pub is_hearing_impaired: Option<bool>,
    pub is_interlaced: Option<bool>,
    pub is_text_subtitle_stream: Option<bool>,
    pub language: Option<String>,
    pub level: Option<f64>,
    pub localized_default: Option<String>,
    pub localized_external: Option<String>,
    pub localized_forced: Option<String>,
    pub localized_hearing_impaired: Option<String>,
    pub localized_undefined: Option<String>,
    pub nal_length_size: Option<String>,
    pub packet_length: Option<i64>,
    pub path: Option<String>,
    pub pixel_format: Option<String>,
    pub profile: Option<String>,
    pub real_frame_rate: Option<f64>,
    pub reference_frame_rate: Option<f64>,
    pub ref_frames: Option<i64>,
    pub rotation: Option<i64>,
    pub rpu_present_flag: Option<i64>,
    pub sample_rate: Option<i64>,
    pub score: Option<i64>,
    pub supports_external_stream: Option<bool>,
    pub time_base: Option<String>,
    pub title: Option<String>,
    pub type_: Option<MediaStreamType>,
    pub video_do_vi_title: Option<String>,
    pub video_range: Option<VideoRange>,
    pub video_range_type: Option<VideoRangeType>,
    pub width: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaType {
    Audio,
    Book,
    Photo,
    Unknown,
    Video,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemPerson {
    pub id: Option<String>,
    pub image_blur_hashes: Option<BaseItemPersonImageBlurHashes>,
    pub name: Option<String>,
    pub primary_image_tag: Option<String>,
    pub role: Option<String>,
    pub type_: Option<PersonKind>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayAccess {
    Full,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaUrl {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TrickplayInfoDto {
    pub bandwidth: Option<i64>,
    pub height: Option<i64>,
    pub interval: Option<i64>,
    pub thumbnail_count: Option<i64>,
    pub tile_height: Option<i64>,
    pub tile_width: Option<i64>,
    pub width: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BaseItemKind {
    AggregateFolder,
    Audio,
    AudioBook,
    BasePluginFolder,
    Book,
    BoxSet,
    Channel,
    ChannelFolderItem,
    CollectionFolder,
    Episode,
    Folder,
    Genre,
    LiveTvChannel,
    LiveTvProgram,
    ManualPlaylistsFolder,
    Movie,
    MusicAlbum,
    MusicArtist,
    MusicGenre,
    MusicVideo,
    Person,
    Photo,
    PhotoAlbum,
    Playlist,
    PlaylistsFolder,
    Program,
    Recording,
    Season,
    Series,
    Studio,
    Trailer,
    TvChannel,
    TvProgram,
    UserRootFolder,
    UserView,
    Video,
    Year,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct UserItemDataDto {
    pub is_favorite: Option<bool>,
    pub item_id: Option<String>,
    pub key: Option<String>,
    pub last_played_date: Option<String>,
    pub likes: Option<bool>,
    pub playback_position_ticks: Option<i64>,
    pub play_count: Option<i64>,
    pub played: Option<bool>,
    pub played_percentage: Option<f64>,
    pub rating: Option<f64>,
    pub unplayed_item_count: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Video3DFormat {
    FullSideBySide,
    FullTopAndBottom,
    HalfSideBySide,
    HalfTopAndBottom,
    Mvc,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VideoType {
    BluRay,
    Dvd,
    Iso,
    VideoFile,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaProtocol {
    File,
    Ftp,
    Http,
    Rtmp,
    Rtp,
    Rtsp,
    Udp,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaAttachment {
    pub codec: Option<String>,
    pub codec_tag: Option<String>,
    pub comment: Option<String>,
    pub delivery_url: Option<String>,
    pub file_name: Option<String>,
    pub index: Option<i64>,
    pub mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TransportStreamTimestamp {
    None,
    Valid,
    Zero,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaStreamProtocol {
    Hls,
    Http,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaSourceType {
    Default,
    Grouping,
    Placeholder,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AudioSpatialFormat {
    DolbyAtmos,
    Dtsx,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SubtitleDeliveryMethod {
    Drop,
    Embed,
    Encode,
    External,
    Hls,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MediaStreamType {
    Audio,
    Data,
    EmbeddedImage,
    Lyric,
    Subtitle,
    Video,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VideoRange {
    Hdr,
    Sdr,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VideoRangeType {
    Dovi,
    DoviInvalid,
    DoviWithEl,
    DoviWithElhdr10Plus,
    DoviWithHdr10,
    DoviWithHdr10Plus,
    DoviWithHlg,
    DoviWithSdr,
    Hdr10,
    Hdr10Plus,
    Hlg,
    Sdr,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemPersonImageBlurHashes {
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub art: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub backdrop: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub banner: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub box_: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub box_rear: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub chapter: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub disc: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub logo: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub menu: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub primary: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub profile: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub screenshot: Option<Vec<ImageBlurHash>>,
    #[serde(default, deserialize_with = "blur_entries_from_map")]
    pub thumb: Option<Vec<ImageBlurHash>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum PersonKind {
    Actor,
    AlbumArtist,
    Arranger,
    Artist,
    Author,
    Colorist,
    Composer,
    Conductor,
    CoverArtist,
    Creator,
    Director,
    Editor,
    Engineer,
    GuestStar,
    Illustrator,
    Inker,
    Letterer,
    Lyricist,
    Mixer,
    Penciller,
    Producer,
    Remixer,
    Translator,
    Unknown,
    Writer,
}

// This one doesn't need a pascal case since its not actually provided by Jellyfin
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageBlurHash {
    pub id: String,
    pub blurhash: String,
}

fn blur_entries_from_map<'de, D>(deserializer: D) -> Result<Option<Vec<ImageBlurHash>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the JSON object into a HashMap<String, String>
    // then map it to Vec<ImageBlurHash>.
    let map = Option::<HashMap<String, String>>::deserialize(deserializer)?;
    Ok(map.map(|h| {
        h.into_iter()
            .map(|(id, blurhash)| ImageBlurHash { id, blurhash })
            .collect()
    }))
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemImageTags {
    pub art: Option<String>,
    pub backdrop: Option<String>,
    pub banner: Option<String>,
    pub box_: Option<String>,
    pub box_rear: Option<String>,
    pub chapter: Option<String>,
    pub disc: Option<String>,
    pub logo: Option<String>,
    pub menu: Option<String>,
    pub primary: Option<String>,
    pub profile: Option<String>,
    pub screenshot: Option<String>,
    pub thumb: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum ItemSortBy {
    AiredEpisodeOrder,
    AirTime,
    Album,
    AlbumArtist,
    Artist,
    CommunityRating,
    CriticRating,
    DateCreated,
    DateLastContentAdded,
    DatePlayed,
    Default,
    IndexNumber,
    IsFavoriteOrLiked,
    IsFolder,
    IsPlayed,
    IsUnplayed,
    Name,
    OfficialRating,
    ParentIndexNumber,
    PlayCount,
    PremiereDate,
    ProductionYear,
    Random,
    Runtime,
    SeriesDatePlayed,
    SeriesSortName,
    SortName,
    StartDate,
    Studio,
    VideoBitRate,
}