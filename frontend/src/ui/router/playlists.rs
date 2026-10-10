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
    api::endpoint_api::{EndpointManager, GetPlaylistsParams, Pagination},
    data_view::PlaylistView,
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
pub struct Playlists {
    /// The tracks as displayed in the table
    playlist_table: Table<PlaylistRow, PlaylistCellMsg>,
}

impl Default for Playlists {
    fn default() -> Self {
        let playlist_table = Table::default()
            .add_column(index_column())
            .add_column(combined_title_column());

        Self { playlist_table }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchPlaylists,
    /// The tracks have been fetched
    RowsCreated(Vec<PlaylistRow>),
    /// A driver for the table
    TableDriver(TableMsg<PlaylistCellMsg>),
    /// The user requests to play all tracks
    PlayAllPlaylists,
}

impl Playlists {
    pub fn view(&self) -> Element<'_, Message> {
        let header = default_header("Playlists", Message::PlayAllPlaylists);

        let albums = column![header, self.playlist_table.view().map(Message::TableDriver)];

        widget::sensor(albums)
            .on_show(|_| Message::FetchPlaylists)
            .key("Playlists")
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchPlaylists => {
                // Only fetch first time
                if !self.playlist_table.rows().is_empty() {
                    return Task::none();
                }

                Task::future(async {
                    let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                    endpoint
                        .get_playlists(GetPlaylistsParams {
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
                    let rows = albums.into_iter().map(PlaylistRow::from).collect();
                    Task::done(Message::RowsCreated(rows))
                })
            }
            Message::RowsCreated(row_data) => {
                // Insert tracks
                self.playlist_table.extend(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self.playlist_table.update(m).map(Message::TableDriver),
            Message::PlayAllPlaylists => Task::none(),
        }
    }
}

/// The data that represents one row in the table
#[derive(Debug, Clone)]
pub struct PlaylistRow {
    pub index: usize,
    pub view: PlaylistView,
    pub image: Option<Image>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Convenience since we mostly want to display [`AlbumView`]s
impl From<PlaylistView> for PlaylistRow {
    fn from(playlist_view: PlaylistView) -> Self {
        // Create an image if available
        let image = playlist_view.playlist.image_url.clone().map(|url| {
            let blurhash = playlist_view.playlist.image_blur_hash.clone();
            Image::new(url)
                .blurhash_maybe(blurhash)
                .debounce_blurhash(Duration::from_millis(10))
                .debounce(Duration::from_millis(500))
                .border_radius(8)
        });

        PlaylistRow {
            view: playlist_view,
            image,
            index: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// A message that drives the cells
#[derive(Debug, Clone)]
pub enum PlaylistCellMsg {
    /// A driver for images
    ImageDriver(image::ImgMsg),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
}

/// A column displaying the index (number)
pub fn index_column() -> Column<PlaylistRow, PlaylistCellMsg> {
    Column::new(
        || column_header("#").into(),
        |row_data: &PlaylistRow| {
            // Simply display the index
            widget::text(row_data.index).style(muted_text).into()
        },
    )
    .align_x(Horizontal::Center)
    .intial_width(Length::Fixed(64.0))
}

/// A column that combines an image, and the name of the Playlist
pub fn combined_title_column() -> Column<PlaylistRow, PlaylistCellMsg> {
    Column::new(
        || column_header("Name").into(),
        |row: &PlaylistRow| {
            let PlaylistRow {
                view: PlaylistView { playlist, .. },
                image,
                ..
            } = row;

            let playlist_name = playlist
                .name
                .clone()
                .unwrap_or("Unnamed Playlist".to_owned());

            // Image
            let image: Element<'_, PlaylistCellMsg> = if let Some(image) = image {
                // Show an image if it's ready
                image.view().map(PlaylistCellMsg::ImageDriver)
            } else {
                // Or show a placeholder
                icons::disc().size(18).into()
            };

            // Playlist name
            let playlist_name = widget::text(playlist_name)
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End);

            // Arrange image, title and Playlists
            row![
                widget::container(image).center(64),
                widget::space().width(8),
                widget::container(playlist_name).center_y(64).clip(true)
            ]
            .clip(true)
            .into()
        },
    )
    .update(|row_data, message| {
        // We need to drive the image
        if let PlaylistCellMsg::ImageDriver(m) = message {
            row_data
                .image
                .as_mut()
                .unwrap()
                .update(m)
                .map(PlaylistCellMsg::ImageDriver)
        } else {
            Task::none()
        }
    })
    .intial_width(Length::FillPortion(2))
}
