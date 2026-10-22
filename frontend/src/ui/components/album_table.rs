//! A album table as used in the albums view and queue

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crate::ui::{
    components::{
        icons,
        image::{self, Image},
        style::{column_header, muted_text},
        table::{Column, Table, TableMsg},
        utils::{IntoLinks, format_duration},
    },
    router::{Route, Router},
};

use backend::data_view::{AlbumView, RelatedArtist};
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

/// A table showing albums
pub type AlbumTable = Table<AlbumRow, AlbumCellMsg>;
/// The corresponding message
pub type AlbumTableMsg = TableMsg<AlbumCellMsg>;

/// The data that represents one row in the table
#[derive(Debug, Clone)]
pub struct AlbumRow {
    pub index: usize,
    pub view: AlbumView,
    pub image: Option<Image>,
}

impl Default for AlbumTable {
    fn default() -> Self {
        AlbumTable::new()
            .add_column(index_column())
            .add_column(combined_title_column())
            .add_column(duration_column())
            .on_double_click_row(|row| Router::change_route(Route::Album(row.view.album.id)))
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Convenience since we mostly want to display [`AlbumView`]s
impl From<AlbumView> for AlbumRow {
    fn from(album_view: AlbumView) -> Self {
        // Create an image if available
        let image = album_view.album.image_url.clone().map(|url| {
            let blurhash = album_view.album.image_blur_hash.clone();
            Image::new(url)
                .blurhash_maybe(blurhash)
                // .debounce_blurhash(Duration::from_millis(10))
                .debounce(Duration::from_millis(500))
                .border_radius(8)
        });

        AlbumRow {
            view: album_view,
            image,
            index: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// A message that drives the cells
#[derive(Debug, Clone)]
pub enum AlbumCellMsg {
    /// A driver for images
    ImageDriver(image::ImgMsg),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
}

pub fn album_table() -> AlbumTable {
    Table::new()
}

/// A column displaying the index (number)
pub fn index_column() -> Column<AlbumRow, AlbumCellMsg> {
    Column::new(
        || column_header("#").into(),
        |row_data: &AlbumRow| {
            // Simply display the index
            widget::text(row_data.index).style(muted_text).into()
        },
    )
    .align_x(Horizontal::Center)
    .intial_width(Length::Fixed(64.0))
}

/// A column that combines an image, the title and the artist
pub fn combined_title_column() -> Column<AlbumRow, AlbumCellMsg> {
    Column::new(
        || column_header("Name").into(),
        |row: &AlbumRow| {
            let AlbumRow {
                view:
                    AlbumView {
                        album,
                        album_artists: artists,
                        ..
                    },
                image,
                ..
            } = row;

            let track_title = album.title.clone().unwrap_or("No title".to_owned());
            let artists = artists.clone();

            // Image
            let image: Element<'_, AlbumCellMsg> = if let Some(image) = image {
                // Show an image if it's ready
                image.view().map(AlbumCellMsg::ImageDriver)
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
                            Route::Artist(id),
                        )
                    },
                    AlbumCellMsg::ChangeRoute,
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
        if let AlbumCellMsg::ImageDriver(m) = message {
            row_data
                .image
                .as_mut()
                .unwrap()
                .update(m)
                .map(AlbumCellMsg::ImageDriver)
        } else {
            Task::none()
        }
    })
    .intial_width(Length::FillPortion(2))
}

/// A column displaying the track duration
pub fn duration_column() -> Column<AlbumRow, AlbumCellMsg> {
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
            })
            .into()
        },
        |row: &AlbumRow| {
            // Display the duration as a simple time string
            widget::text(format_duration(row.view.duration.unwrap_or_default() as f64))
                .style(muted_text)
                .into()
        },
    )
    .intial_width(Length::Fixed(96.0))
    .align_x(Horizontal::Center)
}
