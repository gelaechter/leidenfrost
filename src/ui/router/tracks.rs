use std::sync::Arc;

use iced::{
    Element, Task,
    wgpu::hal::Api,
    widget::{self, column},
};

use crate::{
    backend::{
        api::{
            endpoint_api::{GetTracksParams, MusicEndpoint, Pagination},
            jellyfin::errors::ApiError,
        },
        data_view::TrackView,
    },
    ui::{
        ICMsg, ToCmdMsg, ToErrMsg,
        components::{
            style::default_header,
            table::{self},
            track_table::{self, TrackCellMsg, TrackRow, TrackTable, TrackTableMsg},
        },
        router::settings::ENDPOINTS,
    },
};

/// The tracks route
pub struct Tracks {
    /// The tracks as displayed in the table
    track_table: TrackTable,
}

impl Default for Tracks {
    fn default() -> Self {
        let track_table: table::Table<TrackRow, TrackCellMsg> = TrackTable::default()
            .add_column(track_table::index_column())
            .add_column(track_table::combined_title_column())
            .add_column(track_table::album_column())
            .add_column(track_table::duration_column())
            .add_column(track_table::genre_column());

        Self { track_table }
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

pub type TracksMsg = ICMsg<Cmd, Out, ApiError>;

impl Tracks {
    pub fn view(&self) -> Element<'_, TracksMsg> {
        let header = default_header("Tracks", Cmd::PlayAllTracks.cmd_msg());

        let tracks = column![
            header,
            self.track_table
                .view()
                .map(|c| Cmd::TableDriver(Box::new(c)).cmd_msg())
        ];

        widget::sensor(tracks)
            .on_show(|_| Cmd::FetchTracks.cmd_msg())
            .key("tracks")
            .into()
    }

    pub fn update(&mut self, message: impl Into<TracksMsg>) -> Task<TracksMsg> {
        message.into().cmd(|cmd| match cmd {
            Cmd::FetchTracks => {
                // Only fetch first time
                if !self.track_table.rows().is_empty() {
                    return Task::none();
                }

                Task::perform(
                    async {
                        let endpoints = ENDPOINTS.read().await;

                        endpoints
                            .get_tracks(GetTracksParams {
                                pagination: Some(Pagination {
                                    start: 0,
                                    limit: 100,
                                }),
                                sorting: None,
                            })
                            .await
                    },
                    // TODO: There is probably a better way to do this
                    |r| match r {
                        Ok(tracks) => Cmd::TracksFetched(tracks).cmd_msg(),
                        Err(e) => e.err_msg(),
                    },
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
                    },
                    |result| match result {
                        Ok(r) => Cmd::RowsCreated(r).cmd_msg(),
                        Err(e) => ApiError::JoinError.err_msg(),
                    },
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
                .map(|c| Cmd::TableDriver(Box::new(c)).cmd_msg()),
            Cmd::PlayAllTracks => Task::none(),
        })
    }
}
