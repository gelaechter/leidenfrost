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
        table::{self},
        track_table::{self, TrackCellMsg, TrackRow, TrackTable, TrackTableMsg},
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
        let track_table: table::Table<TrackRow, TrackCellMsg> = TrackTable::default()
            .add_column(track_table::index_column())
            .add_column(track_table::combined_title_column())
            .add_column(track_table::album_column())
            .add_column(track_table::duration_column())
            .add_column(track_table::genre_column())
            .on_double_click_row(|track| {
                log::debug!("Playing track by double click");
                GenericPlayer::send(PlayerMsg::Play(track.view));
            });

        Self {
            track_table,
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cmd {
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

#[derive(Debug, Clone)]
pub enum Out {
    /// Play a single track by overriding the queue
    PlayTrack {
        /// The new queue
        tracks: Vec<TrackView>,
        /// The current song in the queue
        index: usize,
    },
}

pub type TracksMsg = Cmd;

impl Tracks {
    pub fn view(&self) -> Element<'_, TracksMsg> {
        let header = default_header("Tracks", Cmd::PlayAllTracks);

        let tracks = column![
            header,
            self.track_table
                .view()
                .map(|c| Cmd::TableDriver(Box::new(c)))
        ];

        widget::sensor(tracks)
            .on_show(|_| Cmd::FetchTracks)
            .key("tracks")
            .into()
    }

    pub fn update(&mut self, message: impl Into<TracksMsg>) -> Task<TracksMsg> {
        match message.into() {
            Cmd::FetchTracks => {
                // Only fetch first time
                if !self.track_table.rows().is_empty() {
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
                    Cmd::TracksFetched,
                )
            }
            Cmd::TracksFetched(tracks) => {
                // After fetching convert the track_views into rowdata
                Task::perform(
                    async {
                        // [`TrackRow::from::<TrackView>()`] is blocking
                        tokio::task::spawn_blocking(move || {
                            tracks.into_iter().map(TrackRow::from).collect()
                        })
                        .await
                        .unwrap()
                    },
                    Cmd::RowsCreated,
                )
            }
            Cmd::RowsCreated(row_data) => {
                // Insert tracks
                self.track_table.extend(row_data);
                Task::none()
            }
            Cmd::TableDriver(m) => self
                .track_table
                .update(*m)
                .map(|c| Cmd::TableDriver(Box::new(c))),
            Cmd::PlayAllTracks => Task::none(),
        }
    }
}
