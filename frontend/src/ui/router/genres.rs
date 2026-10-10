use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use iced::{
    Element,
    Length::{self},
    Task,
    alignment::Horizontal,
    widget::{
        self, column, row,
        text::{self, Wrapping},
    },
};

use backend::{
    api::endpoint_api::{EndpointManager, GetGenresParams, Pagination},
    data_view::GenreView,
};

use crate::ui::{
    components::{
        icons,
        image::{self, Image},
        style::{column_header, default_header, muted_text},
        table::{Column, Table, TableMsg},
    },
    router::Route,
};

/// The tracks route
pub struct Genres {
    /// The tracks as displayed in the table
    genre_table: Table<GenreRow, GenreCellMsg>,
}

impl Default for Genres {
    fn default() -> Self {
        let genre_table = Table::default()
            .add_column(index_column())
            .add_column(combined_title_column());

        Self { genre_table }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchGenres,
    /// The tracks have been fetched
    RowsCreated(Vec<GenreRow>),
    /// A driver for the table
    TableDriver(TableMsg<GenreCellMsg>),
    /// The user requests to play all tracks
    PlayAllGenres,
}

impl Genres {
    pub fn view(&self) -> Element<'_, Message> {
        let header = default_header("Genres", Message::PlayAllGenres);

        let albums = column![header, self.genre_table.view().map(Message::TableDriver)];

        widget::sensor(albums)
            .on_show(|_| Message::FetchGenres)
            .key("Genres")
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchGenres => {
                // Only fetch first time
                if !self.genre_table.rows().is_empty() {
                    return Task::none();
                }

                Task::future(async {
                    let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                    endpoint
                        .get_genres(GetGenresParams {
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
                    let rows = albums.into_iter().map(GenreRow::from).collect();
                    Task::done(Message::RowsCreated(rows))
                })
            }
            Message::RowsCreated(row_data) => {
                // Insert tracks
                self.genre_table.extend(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self.genre_table.update(m).map(Message::TableDriver),
            Message::PlayAllGenres => Task::none(),
        }
    }
}

/// The data that represents one row in the table
#[derive(Debug, Clone)]
pub struct GenreRow {
    pub index: usize,
    pub view: GenreView,
    pub image: Option<Image>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Convenience since we mostly want to display [`AlbumView`]s
impl From<GenreView> for GenreRow {
    fn from(genre_view: GenreView) -> Self {
        // Create an image if available
        let image = genre_view.genre.image_url.clone().map(|url| {
            let blurhash = genre_view.genre.image_blur_hash.clone();
            Image::new(url)
                .blurhash_maybe(blurhash)
                .debounce_blurhash(Duration::from_millis(10))
                .debounce(Duration::from_millis(500))
                .border_radius(8)
        });

        GenreRow {
            view: genre_view,
            image,
            index: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// A message that drives the cells
#[derive(Debug, Clone)]
pub enum GenreCellMsg {
    /// A driver for images
    ImageDriver(image::ImgMsg),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
}

/// A column displaying the index (number)
pub fn index_column() -> Column<GenreRow, GenreCellMsg> {
    Column::new(
        || column_header("#").into(),
        |row_data: &GenreRow| {
            // Simply display the index
            widget::text(row_data.index).style(muted_text).into()
        },
    )
    .align_x(Horizontal::Center)
    .intial_width(Length::Fixed(64.0))
}

/// A column that combines an image, and the name of the genre
pub fn combined_title_column() -> Column<GenreRow, GenreCellMsg> {
    Column::new(
        || column_header("Name").into(),
        |row: &GenreRow| {
            let GenreRow {
                view: GenreView { genre, .. },
                image,
                ..
            } = row;

            let genre_name = genre.name.clone().unwrap_or("Unnamed Genre".to_owned());

            // Image
            let image: Element<'_, GenreCellMsg> = if let Some(image) = image {
                // Show an image if it's ready
                image.view().map(GenreCellMsg::ImageDriver)
            } else {
                // Or show a placeholder
                icons::disc().size(18).into()
            };

            // Genre name
            let genre_name = widget::text(genre_name)
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End);

            // Arrange image, title and Genres
            row![
                widget::container(image).center(64),
                widget::space().width(8),
                widget::container(genre_name).center_y(64).clip(true)
            ]
            .clip(true)
            .into()
        },
    )
    .update(|row_data, message| {
        // We need to drive the image
        if let GenreCellMsg::ImageDriver(m) = message {
            row_data
                .image
                .as_mut()
                .unwrap()
                .update(m)
                .map(GenreCellMsg::ImageDriver)
        } else {
            Task::none()
        }
    })
    .intial_width(Length::FillPortion(2))
}