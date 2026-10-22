use iced::{
    Element, Task,
    widget::{self, column},
};

use backend::{
    api::{
        endpoint_api::{EndpointManager, GetTracksParams, Pagination},
        jellyfin::errors::ApiError,
    },
    data_view::TrackView,
};

use crate::ui::{
    Receiver,
    components::{
        style::default_header,
        track_table::{self, TrackRow, TrackTable, TrackTableMsg},
    },
    player::{GenericPlayer, PlayerMsg},
};

/// The tracks route
pub struct Tracks {
    /// The tracks as displayed in the table
    track_table: TrackTable,
    /// An error display
    error: Option<ApiError>,
}

impl Default for Tracks {
    fn default() -> Self {
        let track_table: TrackTable = TrackTable::default()
            .add_column(track_table::index_column())
            .add_column(track_table::combined_title_column())
            .add_column(track_table::album_column())
            .add_column(track_table::duration_column())
            .add_column(track_table::genre_column())
            .on_double_click_row(|track| {
                GenericPlayer::send(PlayerMsg::Play(track.view));
            });

        Self {
            track_table,
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchTracks,
    /// The tracks have been fetched
    TracksFetched(Vec<TrackView>),
    /// The tracks have been fetched
    RowsCreated(Vec<TrackRow>),
    /// A driver for the table
    TableDriver(Box<TrackTableMsg>),
    /// The user requests to play all tracks
    PlayAllTracks,
}

pub type TracksMsg = Message;

impl Tracks {
    pub fn view(&self) -> Element<'_, TracksMsg> {
        let header = default_header("Tracks", Message::PlayAllTracks);

        let tracks = column![
            header,
            self.track_table
                .view()
                .map(|c| Message::TableDriver(Box::new(c)))
        ];

        widget::sensor(tracks)
            .on_show(|_| Message::FetchTracks)
            .key("tracks")
            .into()
    }

    pub fn update(&mut self, message: impl Into<TracksMsg>) -> Task<TracksMsg> {
        match message.into() {
            Message::FetchTracks => {
                // Only fetch first time
                if !self.track_table.is_empty() {
                    return Task::none();
                }

                Task::perform(
                    async {
                        let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                        endpoint
                            .get_tracks(GetTracksParams {
                                pagination: Some(Pagination {
                                    start_page: 0,
                                    limit: 99999,
                                }),
                                sorting: None,
                            })
                            // TODO: error handling
                            .await
                            .unwrap()
                    },
                    Message::TracksFetched,
                )
            }
            Message::TracksFetched(tracks) => {
                log::debug!("Fetched {} tracks", tracks.len());
                // After fetching convert the track_views into rowdata
                let rows = tracks.into_iter().map(TrackRow::from).collect();
                Task::done(Message::RowsCreated(rows))
            }
            Message::RowsCreated(row_data) => {
                log::debug!("Rows created");
                // Insert tracks
                self.track_table.extend(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self
                .track_table
                .update(*m)
                .map(|c| Message::TableDriver(Box::new(c))),
            Message::PlayAllTracks => Task::none(),
        }
    }
}
