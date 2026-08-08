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
    api::endpoint_api::{EndpointManager, GetAlbumsParams, Pagination},
    data_view::{AlbumView, RelatedArtist},
};

use crate::ui::{
    components::{
        icons,
        image::{self, Image},
        style::{column_header, default_header, muted_text},
        table::{Column, Table, TableMsg},
        utils::{IntoLinks, format_duration},
    },
    router::Route,
};

/// The tracks route
pub struct Albums {
    /// The tracks as displayed in the table
    album_table: Table<AlbumRow, AlbumCellMsg>,
}

impl Default for Albums {
    fn default() -> Self {
        let album_table = Table::default()
            .add_column(index_column())
            .add_column(combined_title_column())
            .add_column(duration_column());

        Self { album_table }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchAlbums,
    /// The tracks have been fetched
    RowsCreated(Vec<AlbumRow>),
    /// A driver for the table
    TableDriver(TableMsg<AlbumRow, AlbumCellMsg>),
    /// The user requests to play all tracks
    PlayAllAlbums,
}

impl Albums {
    pub fn view(&self) -> Element<'_, Message> {
        let header = default_header("Albums", Message::PlayAllAlbums);

        let albums = column![header, self.album_table.view().map(Message::TableDriver)];

        widget::sensor(albums)
            .on_show(|_| Message::FetchAlbums)
            .key("albums")
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchAlbums => {
                // Only fetch first time
                if !self.album_table.rows().is_empty() {
                    return Task::none();
                }

                Task::future(async {
                    let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                    endpoint
                        .get_albums(GetAlbumsParams {
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
                    Task::perform(
                        async {
                            // [`AlbumRow::from::<AlbumView>()`] is blocking
                            let row_data: Vec<AlbumRow> = tokio::task::spawn_blocking(move || {
                                albums.into_iter().map(AlbumRow::from).collect()
                            })
                            .await
                            .unwrap();

                            row_data
                        },
                        Message::RowsCreated,
                    )
                })
            }
            Message::RowsCreated(row_data) => {
                // Insert tracks
                self.album_table.extend(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self.album_table.update(m).map(Message::TableDriver),
            Message::PlayAllAlbums => Task::none(),
        }
    }
}

/// The data that represents one row in the table
#[derive(Debug, Clone)]
pub struct AlbumRow {
    pub index: usize,
    pub view: AlbumView,
    pub image: Option<Image>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Convenience since we mostly want to display [`AlbumView`]s
impl From<AlbumView> for AlbumRow {
    /// This conversion is blocking since we decode the blurhashes
    /// beforehand; Treat it as such
    fn from(album_view: AlbumView) -> Self {
        // Create an image if available
        let image = album_view.album.image_url.clone().map(|url| {
            let blurhash = album_view.album.image_blur_hash.clone();
            Image::new(url)
                .blurhash_maybe(blurhash)
                .pre_decode_blurhash(64, 64)
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
                .into()
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
