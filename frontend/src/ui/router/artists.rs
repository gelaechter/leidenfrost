use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

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

use backend::{
    api::endpoint_api::{EndpointManager, GetArtistsParams, Pagination},
    data_view::ArtistView,
};

use crate::ui::{
    components::{
        icons,
        image::{self, Image},
        style::{column_header, default_header, muted_text},
        table::{Column, Table, TableMsg},
        utils::format_duration,
    },
    router::Route,
};

/// The tracks route
pub struct Artists {
    /// The tracks as displayed in the table
    artist_table: Table<ArtistRow, ArtistCellMsg>,
}

impl Default for Artists {
    fn default() -> Self {
        let artist_table = Table::default()
            .add_column(index_column())
            .add_column(combined_title_column())
            .add_column(duration_column());

        Self { artist_table }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchArtists,
    /// The tracks have been fetched
    RowsCreated(Vec<ArtistRow>),
    /// A driver for the table
    TableDriver(TableMsg<ArtistCellMsg>),
    /// The user requests to play all tracks
    PlayAllArtists,
}

impl Artists {
    pub fn view(&self) -> Element<'_, Message> {
        let header = default_header("Artists", Message::PlayAllArtists);

        let albums = column![header, self.artist_table.view().map(Message::TableDriver)];

        widget::sensor(albums)
            .on_show(|_| Message::FetchArtists)
            .key("artists")
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchArtists => {
                // Only fetch first time
                if !self.artist_table.rows().is_empty() {
                    return Task::none();
                }

                Task::future(async {
                    let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                    endpoint
                        .get_artists(GetArtistsParams {
                            pagination: Some(Pagination {
                                start_page: 0,
                                limit: 100,
                            }),
                            sorting: None,
                        })
                        .await
                        .unwrap()
                })
                .then(|albums| {
                    // After fetching convert the track_views into rowdata
                    let rows = albums.into_iter().map(ArtistRow::from).collect();
                    Task::done(Message::RowsCreated(rows))
                })
            }
            Message::RowsCreated(row_data) => {
                // Insert tracks
                self.artist_table.extend(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self.artist_table.update(m).map(Message::TableDriver),
            Message::PlayAllArtists => Task::none(),
        }
    }
}

/// The data that represents one row in the table
#[derive(Debug, Clone)]
pub struct ArtistRow {
    pub index: usize,
    pub view: ArtistView,
    pub image: Option<Image>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Convenience since we mostly want to display [`AlbumView`]s
impl From<ArtistView> for ArtistRow {
    fn from(artist_view: ArtistView) -> Self {
        // Create an image if available
        let image = artist_view.artist.image_url.clone().map(|url| {
            let blurhash = artist_view.artist.image_blur_hash.clone();
            Image::new(url)
                .blurhash_maybe(blurhash)
                .debounce_blurhash(Duration::from_millis(10))
                .debounce(Duration::from_millis(500))
                .border_radius(8)
        });

        ArtistRow {
            view: artist_view,
            image,
            index: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// A message that drives the cells
#[derive(Debug, Clone)]
pub enum ArtistCellMsg {
    /// A driver for images
    ImageDriver(image::ImgMsg),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
}

/// A column displaying the index (number)
pub fn index_column() -> Column<ArtistRow, ArtistCellMsg> {
    Column::new(
        || column_header("#").into(),
        |row_data: &ArtistRow| {
            // Simply display the index
            widget::text(row_data.index).style(muted_text).into()
        },
    )
    .align_x(Horizontal::Center)
    .intial_width(Length::Fixed(64.0))
}

/// A column that combines an image, the title and the artist
pub fn combined_title_column() -> Column<ArtistRow, ArtistCellMsg> {
    Column::new(
        || column_header("Name").into(),
        |row: &ArtistRow| {
            let ArtistRow {
                view: ArtistView { artist, .. },
                image,
                ..
            } = row;

            let artist_name = artist.name.clone().unwrap_or("Unnamed Artist".to_owned());

            // Image
            let image: Element<'_, ArtistCellMsg> = if let Some(image) = image {
                // Show an image if it's ready
                image.view().map(ArtistCellMsg::ImageDriver)
            } else {
                // Or show a placeholder
                icons::disc().size(18).into()
            };

            // Artist name
            let artist_name = widget::text(artist_name)
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End);

            // Arrange image, title and artists
            row![
                widget::container(image).center(64),
                widget::space().width(8),
                widget::container(artist_name).center_y(64).clip(true)
            ]
            .clip(true)
            .into()
        },
    )
    .update(|row_data, message| {
        // We need to drive the image
        if let ArtistCellMsg::ImageDriver(m) = message {
            row_data
                .image
                .as_mut()
                .unwrap()
                .update(m)
                .map(ArtistCellMsg::ImageDriver)
        } else {
            Task::none()
        }
    })
    .intial_width(Length::FillPortion(2))
}

/// A column displaying the track duration
pub fn duration_column() -> Column<ArtistRow, ArtistCellMsg> {
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
        |row: &ArtistRow| {
            // Display the duration as a simple time string
            widget::text(format_duration(row.view.duration.unwrap_or_default() as f64))
                .style(muted_text)
                .into()
        },
    )
    .intial_width(Length::Fixed(96.0))
    .align_x(Horizontal::Center)
}
