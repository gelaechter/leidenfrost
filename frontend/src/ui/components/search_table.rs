//! A search table as used in the search view and command palette

use std::time::Duration;

use crate::ui::{
    Receiver,
    components::{
        album_table::{self, AlbumCellMsg, AlbumRow},
        icons,
        image::{self, Image},
        style::{em_dash, muted_text},
        table::{Column, Table, TableMsg},
        track_table::{self, TrackCellMsg, TrackRow},
    },
    player::{GenericPlayer, PlayerMsg},
    router::{
        Route,
        artists::{self, ArtistCellMsg, ArtistRow},
        genres::{self, GenreCellMsg, GenreRow},
        playlists::{self, PlaylistCellMsg, PlaylistRow},
    },
};

use backend::{api::endpoint_api::SearchResult, data_view::TrackView};
use iced::{
    Element,
    Length::{self},
    Task,
    widget::{
        self,
        text::{self, Wrapping},
    },
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
    Playlist(PlaylistCellMsg),
    Genre(GenreCellMsg),
    Lyric(LyricsCellMsg),
}

#[derive(Debug, Clone)]
pub struct LyricsRow {
    pub view: TrackView,
    pub image: Option<Image>,
    pub matched_lyric: Option<String>,
}

/// A message that drives the cells
#[derive(Debug, Clone)]
pub enum LyricsCellMsg {
    /// A driver for images
    ImageDriver(image::ImgMsg),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
}

/// An already configured default search table
///
/// Use [`SearchTable::new`] to get a default one
impl Default for SearchTable {
    fn default() -> Self {
        SearchTable::new()
            .add_column(combined_display())
            .on_double_click_row(|row, state| {
                log::debug!("Double clicked search table row");
                match row {
                    SearchRow::Track(track_row) => {
                        GenericPlayer::send(PlayerMsg::Play(track_row.view));
                    }
                    // Navigate to the artist
                    SearchRow::Artist(artist_row) => {
                        state.route = Route::Artist(artist_row.view.artist.id.clone())
                    }
                    SearchRow::Album(album_row) => todo!(),
                    SearchRow::Playlist(playlist_row) => todo!(),
                    SearchRow::Genre(genre_row) => todo!(),
                    SearchRow::Lyric(lyrics_row) => todo!(),
                }
            })
    }
}

/// A column that combines an image, the title and the artist
pub fn combined_lyrics_column() -> Column<LyricsRow, LyricsCellMsg> {
    Column::new(
        || widget::void().into(),
        |row: &LyricsRow, _app_state| {
            let LyricsRow {
                view: TrackView { track, .. },
                image,
                matched_lyric,
            } = row;

            let track_title = track.title.clone().unwrap_or(em_dash());

            // Image
            let image: Element<'_, LyricsCellMsg> = if let Some(image) = image {
                // Show an image if it's ready
                image.view().map(LyricsCellMsg::ImageDriver)
            } else {
                // Or show a placeholder
                icons::disc().size(18).into()
            };

            // Title
            let title = widget::text(track_title)
                .style(text::base)
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End);

            // Artist list (comma separated)
            let lyric = if let Some(lyric) = matched_lyric {
                widget::text(format!("Matched lyric: {lyric}"))
                    .wrapping(Wrapping::None)
                    .style(muted_text)
                    .ellipsis(text::Ellipsis::End)
                    .size(14)
            } else {
                widget::text("")
            };

            // Arrange image, title and artists
            widget::row![
                widget::container(image).center(64),
                widget::space().width(8),
                widget::container(widget::column![title, widget::space().width(2), lyric])
                    .center_y(64)
                    .clip(true)
            ]
            .clip(true)
            .into()
        },
    )
    .update(|row_data, message, _app_state| {
        // We need to drive the image
        if let LyricsCellMsg::ImageDriver(m) = message {
            row_data
                .image
                .as_mut()
                .unwrap()
                .update(m)
                .map(LyricsCellMsg::ImageDriver)
        } else {
            Task::none()
        }
    })
    .intial_width(Length::FillPortion(2))
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

/// A column that shows a row depending on the [`SearchRow`] content
pub fn combined_display() -> Column<SearchRow, SearchCellMsg> {
    Column::new(
        || widget::void().into(),
        // Show the combined title for each entry
        |row: &SearchRow, app_state| match row {
            SearchRow::Track(track_row) => track_table::combined_title_column()
                .view(track_row, app_state)
                .map(SearchCellMsg::Track),
            SearchRow::Artist(artist_row) => artists::combined_title_column()
                .view(artist_row, app_state)
                .map(SearchCellMsg::Artist),
            SearchRow::Album(album_row) => album_table::combined_title_column()
                .view(album_row, app_state)
                .map(SearchCellMsg::Album),
            SearchRow::Playlist(playlist_row) => playlists::combined_title_column()
                .view(playlist_row, app_state)
                .map(SearchCellMsg::Playlist),
            SearchRow::Genre(genre_row) => genres::combined_title_column()
                .view(genre_row, app_state)
                .map(SearchCellMsg::Genre),
            // The lyric entry needs a custom display
            SearchRow::Lyric(lyrics_row) => combined_lyrics_column()
                .view(lyrics_row, app_state)
                .map(SearchCellMsg::Lyric),
        },
    )
    .update(|row_data, message, app_state| {
        use SearchCellMsg as M;
        use SearchRow as R;

        // We need to drive the image
        match (row_data, message) {
            (R::Track(track), M::Track(track_cell_msg)) => match track_cell_msg {
                TrackCellMsg::ImageDriver(img_msg) => track
                    .image
                    .as_mut()
                    .unwrap()
                    .update(img_msg)
                    .map(TrackCellMsg::ImageDriver)
                    .map(SearchCellMsg::Track),
                TrackCellMsg::ChangeRoute(route) => {
                    app_state.route = route;
                    Task::none()
                }
            },
            (R::Artist(artist), M::Artist(artist_cell_msg)) => match artist_cell_msg {
                ArtistCellMsg::ImageDriver(img_msg) => artist
                    .image
                    .as_mut()
                    .unwrap()
                    .update(img_msg)
                    .map(ArtistCellMsg::ImageDriver)
                    .map(SearchCellMsg::Artist),
                ArtistCellMsg::ChangeRoute(route) => {
                    app_state.route = route;
                    Task::none()
                }
            },
            (R::Album(album), M::Album(album_cell_msg)) => match album_cell_msg {
                AlbumCellMsg::ImageDriver(img_msg) => album
                    .image
                    .as_mut()
                    .unwrap()
                    .update(img_msg)
                    .map(AlbumCellMsg::ImageDriver)
                    .map(SearchCellMsg::Album),
                AlbumCellMsg::ChangeRoute(route) => {
                    app_state.route = route;
                    Task::none()
                }
            },
            (R::Playlist(playlist), M::Playlist(playlist_cell_msg)) => match playlist_cell_msg {
                PlaylistCellMsg::ImageDriver(img_msg) => playlist
                    .image
                    .as_mut()
                    .unwrap()
                    .update(img_msg)
                    .map(PlaylistCellMsg::ImageDriver)
                    .map(SearchCellMsg::Playlist),
                PlaylistCellMsg::ChangeRoute(route) => {
                    app_state.route = route;
                    Task::none()
                }
            },
            (R::Genre(genre), M::Genre(genre_cell_msg)) => match genre_cell_msg {
                GenreCellMsg::ImageDriver(img_msg) => genre
                    .image
                    .as_mut()
                    .unwrap()
                    .update(img_msg)
                    .map(GenreCellMsg::ImageDriver)
                    .map(SearchCellMsg::Genre),
                GenreCellMsg::ChangeRoute(route) => {
                    app_state.route = route;
                    Task::none()
                }
            },
            (R::Lyric(lyric), M::Lyric(lyric_cell_msg)) => match lyric_cell_msg {
                LyricsCellMsg::ImageDriver(img_msg) => lyric
                    .image
                    .as_mut()
                    .unwrap()
                    .update(img_msg)
                    .map(LyricsCellMsg::ImageDriver)
                    .map(SearchCellMsg::Lyric),
                LyricsCellMsg::ChangeRoute(route) => {
                    app_state.route = route;
                    Task::none()
                }
            },
            // All other combinations
            (row_data, message) => {
                panic!("Invalid row/message combination: {row_data:?}, {message:?}")
            }
        }
    })
    .intial_width(Length::FillPortion(2))
}
