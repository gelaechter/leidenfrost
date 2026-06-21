//! A track table as used in the tracks view and queue

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crate::ui::components::{icons, table::TableMsg, utils::IntoLinks};
use crate::ui::components::{style::EM_DASH, utils::IntoLink};
use crate::{backend::data_view::RelatedGenre, ui::components::utils::format_duration};
use iced::{
    Element,
    Length::{self, Fill},
    Task,
    alignment::Horizontal,
    widget::{
        self, column, row,
        text::{self, Wrapping},
    },
};

use crate::{
    backend::data_view::{RelatedArtist, TrackView},
    ui::{
        components::{
            image::{self, Image},
            style::{column_header, muted_text},
            table::{Column, Table},
        },
        router::Route,
    },
};

/// A table showing tracks
pub type TrackTable = Table<TrackRow, TrackCellMsg>;
/// The corresponding message
pub type TrackTableMsg = TableMsg<TrackRow, TrackCellMsg>;

/// The data that represents one row in the table
#[derive(Debug, Clone)]
pub struct TrackRow {
    pub index: usize,
    pub view: TrackView,
    pub image: Option<Image>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Convenience since we mostly want to display [`TrackView`]s
impl From<TrackView> for TrackRow {
    /// This conversion is blocking since we decode the blurhashes
    /// beforehand; Treat it as such
    fn from(track_view: TrackView) -> Self {
        // Create an image if available
        let image = track_view.track.image_url.clone().map(|url| {
            let blurhash = track_view.track.image_blur_hash.clone();
            Image::new(url)
                .blurhash_maybe(blurhash)
                .pre_decode_blurhash(64, 64)
                .debounce(Duration::from_millis(500))
                .border_radius(8)
        });

        TrackRow {
            view: track_view,
            image,
            index: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// A message that drives the cells
#[derive(Debug, Clone)]
pub enum TrackCellMsg {
    /// A driver for images
    ImageDriver(image::ImgMsg),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
}

pub fn track_table() -> TrackTable {
    Table::default()
}

/// A column displaying the genres of this track
pub fn genre_column() -> Column<TrackRow, TrackCellMsg> {
    Column::new(
        || column_header("Genres").into(),
        |row: &TrackRow| {
            // Turn the genres into a list of clickable links
            // when clicked it should take you there
            row.view
                .genres
                .clone()
                .into_links(
                    |RelatedGenre { id, name }| {
                        (name.unwrap_or("Unknown genre".to_owned()), Route::Genre(id))
                    },
                    TrackCellMsg::ChangeRoute,
                )
                .style(muted_text)
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End)
                .into()
        },
    )
    .intial_width(Length::FillPortion(1))
}

/// A column displaying the track duration
pub fn duration_column() -> Column<TrackRow, TrackCellMsg> {
    Column::new(
        || {
            // Responsive header switching between a text and an icon
            widget::responsive(|size| {
                widget::container(if size.width >= 80.0 {
                    column_header("Duration")
                } else {
                    icons::clock()
                })
                .center(Fill)
                .into()
            })
            .into()
        },
        |row: &TrackRow| {
            // Display the duration as a simple time string
            widget::text(format_duration(
                row.view.track.duration.unwrap_or_default() as f64
            ))
            .style(muted_text)
            .into()
        },
    )
    .intial_width(Length::Fixed(96.0))
    .align_x(Horizontal::Center)
}

/// A column displaying the album this track belongs to
pub fn album_column() -> Column<TrackRow, TrackCellMsg> {
    Column::new(
        || column_header("Album").into(),
        |row: &TrackRow| {
            // Display the album name as clickable link that takes you there
            let album_name = row
                .view
                .album_name
                .clone()
                .unwrap_or("Unknown album".to_owned());

            album_name
                .link(
                    Route::Album(row.view.track.album_id.clone()),
                    TrackCellMsg::ChangeRoute,
                )
                .style(muted_text)
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End)
                .into()
        },
    )
    .intial_width(Length::FillPortion(1))
}

/// A column that combines an image, the title and the artist
pub fn combined_title_column() -> Column<TrackRow, TrackCellMsg> {
    Column::new(
        || column_header("Name").into(),
        |row: &TrackRow| {
            let TrackRow {
                view: TrackView { track, artists, .. },
                image,
                ..
            } = row;

            let track_title = track.title.clone().unwrap_or(EM_DASH());
            let artists = artists.clone();

            // Image
            let image: Element<'_, TrackCellMsg> = if let Some(image) = image {
                // Show an image if it's ready
                image.view().map(TrackCellMsg::ImageDriver)
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
            let artist_text = artists
                .into_links(
                    |RelatedArtist { id, name }| {
                        (
                            name.unwrap_or("Unknown artist".to_owned()),
                            Route::Artist(id.clone()),
                        )
                    },
                    TrackCellMsg::ChangeRoute,
                )
                .wrapping(Wrapping::None)
                .style(muted_text)
                .ellipsis(text::Ellipsis::End)
                .size(14);

            // Arrange image, title and artists
            row![
                widget::container(image).center(64),
                widget::space().width(8),
                widget::container(column![title, widget::space().width(2), artist_text])
                    .center_y(64)
                    .clip(true)
            ]
            .clip(true)
            .into()
        },
    )
    .update(|row_data, message| {
        // We need to drive the image
        if let TrackCellMsg::ImageDriver(m) = message {
            row_data
                .image
                .as_mut()
                .unwrap()
                .update(m)
                .map(TrackCellMsg::ImageDriver)
        } else {
            Task::none()
        }
    })
    .intial_width(Length::FillPortion(2))
}

/// A column displaying the index (number)
pub fn index_column() -> Column<TrackRow, TrackCellMsg> {
    Column::new(
        || column_header("#").into(),
        |row_data: &TrackRow| {
            // Simply display the index
            widget::text(row_data.index).style(muted_text).into()
        },
    )
    .align_x(Horizontal::Center)
    .intial_width(Length::Fixed(64.0))
}
