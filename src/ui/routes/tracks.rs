use iced::{
    Element, Task,
    widget::{self, column},
};
use url::Url;

use crate::{
    backend::api::{
        endpoint_api::{ApiContract, UserPasswordAuth},
        jellyfin::api::JellyfinApi,
    },
    ui::{
        components::{
            image::{self},
            style::default_header,
            table::{self},
            track_table::{self, TrackCellMsg, TrackRow, TrackTable},
        },
        router::Route,
    },
};

/// The tracks route
pub struct Tracks {
    /// The tracks as displayed in the table
    track_table: TrackTable,
}

impl Default for Tracks {
    fn default() -> Self {
        let track_table = TrackTable::default()
            .add_column(track_table::index_column())
            .add_column(track_table::combined_title_column())
            .add_column(track_table::album_column())
            .add_column(track_table::duration_column())
            .add_column(track_table::genre_column());

        Self { track_table }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchTracks,
    /// The tracks have been fetched
    RowsCreated(Vec<TrackRow>),
    /// A driver for the table
    TableDriver(table::Message<TrackCellMsg>),
    /// The user requests to play all tracks
    PlayAllTracks,
}

#[derive(Debug, Clone)]
pub enum CellMessage {
    /// A driver for images
    ImageDriver(image::IMessage),
    /// The route has been changed (e.g. through clicking an album)
    ChangeRoute(Route),
}

impl Tracks {
    pub fn view(&self) -> Element<'_, Message> {
        let header = default_header("Tracks", Message::PlayAllTracks);

        let tracks = column![header, self.track_table.view().map(Message::TableDriver)];

        widget::sensor(tracks)
            .on_show(|_| Message::FetchTracks)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchTracks => {
                // Only fetch first time
                if !self.track_table.rows().is_empty() {
                    return Task::none();
                }

                Task::future(async {
                    // TODO: Replace with global state
                    let jf = JellyfinApi::auth_user_password(
                        Url::parse("http://***REMOVED***").unwrap(),
                        "***REMOVED***".to_string(),
                        "***REMOVED***".to_string(),
                    )
                    .await;

                    jf.get_tracks().await.unwrap()
                })
                .then(|tracks| {
                    // After fetching convert the track_views into rowdata
                    Task::perform(
                        async {
                            // [`TrackRow::from::<TrackView>()`] is blocking
                            let row_data: Vec<TrackRow> = tokio::task::spawn_blocking(move || {
                                tracks.into_iter().map(TrackRow::from).collect()
                            })
                            .await
                            .unwrap();

                            (0..100).flat_map(|_| row_data.clone()).collect()
                        },
                        Message::RowsCreated,
                    )
                })
            }
            Message::RowsCreated(row_data) => {
                // Insert tracks
                self.track_table.extend(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self.track_table.update(m).map(Message::TableDriver),
            Message::PlayAllTracks => Task::none(),
        }
    }
}
