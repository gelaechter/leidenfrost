use iced::{
    Element, Task,
    widget::{self, column},
};

use backend::api::endpoint_api::{EndpointManager, GetAlbumsParams, Pagination};

use crate::ui::{
    app::AppState,
    components::{
        album_table::{AlbumCellMsg, AlbumRow, AlbumTable},
        style::default_header,
        table::TableMsg,
    },
};

/// The tracks route
pub struct Albums {
    /// The tracks as displayed in the table
    album_table: AlbumTable,
}

impl Default for Albums {
    fn default() -> Self {
        let album_table = AlbumTable::default();

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
    TableDriver(TableMsg<AlbumCellMsg>),
    /// The user requests to play all tracks
    PlayAllAlbums,
}

impl Albums {
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, Message> {
        let header = default_header("Albums", Message::PlayAllAlbums);

        let albums = column![
            header,
            self.album_table.view(app_state).map(Message::TableDriver)
        ];

        widget::sensor(albums)
            .on_show(|_| Message::FetchAlbums)
            .key("albums")
            .into()
    }

    pub fn update(&mut self, message: Message, app_state: &mut AppState) -> Task<Message> {
        match message {
            Message::FetchAlbums => {
                // Only fetch first time
                if !self.album_table.is_empty() {
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
                    let rows = albums.into_iter().map(AlbumRow::from).collect();
                    Task::done(Message::RowsCreated(rows))
                })
            }
            Message::RowsCreated(row_data) => {
                // Insert tracks
                self.album_table.extend(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self.album_table.update(m, app_state).map(Message::TableDriver),
            Message::PlayAllAlbums => Task::none(),
        }
    }
}
