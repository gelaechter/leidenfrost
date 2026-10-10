//! A track table as used in the tracks view and queue

use std::time::Duration;

use crate::ui::{
    components::{
        image::Image,
        style::column_header,
        table::{Column, Table, TableMsg},
        track_table::{self, TrackCellMsg, TrackRow},
    },
    router::{
        albums::{AlbumCellMsg, AlbumRow},
        artists::{self, ArtistCellMsg, ArtistRow},
        genres::{GenreCellMsg, GenreRow},
        playlists::PlaylistRow,
    },
};

use backend::{api::endpoint_api::SearchResult, data_view::TrackView};
use iced::{
    Length::{self},
    Task,
    widget::{self},
};

/// A table showing tracks
pub type SearchTable = Table<SearchRow, SearchCellMsg>;
/// The corresponding message
pub type SearchTableMsg = TableMsg<SearchCellMsg>;

#[derive(Debug, Clone)]
pub enum SearchRow {
    Track(TrackRow),
    Artist(ArtistRow),
    Album(AlbumRow),
    Playlist(PlaylistRow),
    Genre(GenreRow),
    Lyric(LyricsRow),
}

#[derive(Debug, Clone)]
pub enum SearchCellMsg {
    Track(TrackCellMsg),
    Artist(ArtistCellMsg),
    Album(AlbumCellMsg),
    Playlist(GenreCellMsg),
    Genre(GenreCellMsg),
    Lyric(TrackCellMsg),
}

#[derive(Debug, Clone)]
pub struct LyricsRow {
    pub view: TrackView,
    pub image: Option<Image>,
    pub matched_lyric: Option<String>,
}

/// Convenience since we mostly want to display [`TrackView`]s
impl From<SearchResult> for SearchRow {
    fn from(search_result: SearchResult) -> Self {
        match search_result {
            SearchResult::Track(track_view) => SearchRow::Track(TrackRow::from(track_view)),
            SearchResult::Album(album_view) => SearchRow::Album(AlbumRow::from(album_view)),
            SearchResult::Playlist(playlist_view) => {
                SearchRow::Playlist(PlaylistRow::from(playlist_view))
            }
            SearchResult::Genre(genre_view) => SearchRow::Genre(GenreRow::from(genre_view)),
            SearchResult::Artist(artist_view) => SearchRow::Artist(ArtistRow::from(artist_view)),
            SearchResult::LyricMatch {
                matched_lyric,
                track_view,
            } => {
                // Create an image if available
                let image = track_view.track.image_url.clone().map(|url| {
                    let blurhash = track_view.track.image_blur_hash.clone();
                    Image::new(url)
                        .blurhash_maybe(blurhash)
                        .debounce_blurhash(Duration::from_millis(10))
                        .debounce(Duration::from_millis(500))
                        .border_radius(8)
                });

                SearchRow::Lyric(LyricsRow {
                    view: track_view,
                    image,
                    matched_lyric,
                })
            }
        }
    }
}

pub fn search_table() -> SearchTable {
    Table::default()
}

/// A column that combines an image, and the name of the genre
pub fn combined_display() -> Column<SearchRow, SearchCellMsg> {
    Column::new(
        || column_header("Name").into(),
        |row: &SearchRow| match row {
            SearchRow::Track(track_row) => track_table::combined_title_column()
                .view(track_row)
                .map(SearchCellMsg::Track),
            SearchRow::Artist(artist_row) => artists::combined_title_column()
                .view(artist_row)
                .map(SearchCellMsg::Artist),
            _ => widget::text("Kriese").into(),
            SearchRow::Album(album_row) => todo!(),
            SearchRow::Playlist(playlist_row) => todo!(),
            SearchRow::Genre(genre_row) => todo!(),
            SearchRow::Lyric(lyrics_row) => todo!(),
        },
    )
    .update(|row_data, message| {
        // We need to drive the image
        Task::none()
    })
    .intial_width(Length::FillPortion(2))
}
