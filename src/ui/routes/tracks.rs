use std::time::Duration;

use iced::{
    Alignment::Center,
    Border, Color, Element, Font,
    Length::{self, Fill},
    Subscription, Task,
    alignment::Horizontal,
    font,
    widget::{
        self,
        button::{self},
        column, row,
        text::{self, Wrapping},
    },
};
use iced_fonts::lucide;
use url::Url;

use crate::{
    backend::{
        api::{
            endpoint_api::{ApiContract, UserPasswordAuth},
            jellyfin::api::JellyfinApi,
        },
        data_view::{RelatedArtist, RelatedGenre, TrackView},
    },
    ui::{
        components::{
            image::{self, Image},
            table::{self, Column, Table},
            utils::{IntoLink, IntoLinks},
        },
        router::Route,
        util::format_duration,
    },
};

/// The tracks route
pub struct Tracks {
    /// The tracks as displayed in the table
    track_table: Table<RowData, CellMessage>,
}

impl Default for Tracks {
    fn default() -> Self {
        Self {
            track_table: Tracks::track_table(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Initial track fetching
    FetchTracks,
    /// The tracks have been fetched
    RowsCreated(Vec<RowData>),
    /// A driver for the table
    TableDriver(table::Message<CellMessage>),
    PlayAllTracks,
}

#[derive(Debug, Clone)]
struct RowData {
    view: TrackView,
    image: Option<Image>,
}

impl From<TrackView> for RowData {
    /// This conversion is blocking since we decode the blurhashes
    /// beforehand; Treat it as such
    fn from(track_view: TrackView) -> Self {
        // Create an image if available
        let image = track_view.track.image_url.clone().map(|url| {
            let blurhash = track_view.track.image_blur_hash.clone();
            Image::new(url)
                .blurhash_maybe(blurhash)
                .pre_decode_blurhash(64, 64)
                .debounce(Duration::from_millis(500))
        });

        RowData {
            view: track_view,
            image,
        }
    }
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
        let play_button = widget::button(widget::container(lucide::play().size(20)).center(48))
            .height(42)
            .width(42)
            .style(|theme, status| {
                let mut style = button::primary(theme, status);
                style.border = Border::default().rounded(50);
                style
            })
            .on_press(Message::PlayAllTracks);

        let tracks = column![
            widget::space().height(8),
            widget::container(row![
                widget::space().width(12),
                play_button,
                widget::space().width(12),
                widget::container(widget::text("Tracks").size(36).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                }))
                .align_y(Center)
                .height(Fill)
            ])
            .height(48),
            widget::space().height(16),
            self.track_table
                .view()
                .map(Message::TableDriver)
                .explain(Color::BLACK)
        ];

        widget::sensor(tracks)
            .on_show(|_| Message::FetchTracks)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchTracks => {
                // Only fetch first time
                if !self.track_table.data().is_empty() {
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
                            // [`Into::into`]
                            tokio::task::spawn_blocking(move || {
                                tracks.into_iter().map(RowData::from).collect()
                            })
                            .await
                            .unwrap()
                        },
                        Message::RowsCreated,
                    )
                })
            }
            Message::RowsCreated(row_data) => {
                // Insert tracks
                self.track_table.extend_rows(row_data);
                Task::none()
            }
            Message::TableDriver(m) => self.track_table.update(m).map(Message::TableDriver),
            // Only bubbles to the router component
            // Message::ChangeRoute(_) => Task::none(),
            Message::PlayAllTracks => Task::none(),
        }
    }

    /// One row representing the
    fn track_table() -> Table<RowData, CellMessage> {
        // Index column
        let index_col = Column::new(
            || widget::text("#").into(),
            move |row_data: &RowData| widget::text("0").into(),
        )
        .align_x(Horizontal::Center)
        .intial_width(Length::Fixed(64.0));

        // Combined title column
        let title_col = Column::new(|| widget::text("Name").into(), Self::combined_title)
            .update(|row_data, message| {
                // We need to drive the image
                if let CellMessage::ImageDriver(m) = message {
                    row_data
                        .image
                        .as_mut()
                        .unwrap()
                        .update(m)
                        .map(CellMessage::ImageDriver)
                } else {
                    Task::none()
                }
            })
            .intial_width(Length::FillPortion(2));

        // Album column
        let album_col = Column::new(
            || widget::text("Album").into(),
            |row_data: &RowData| {
                let album_name = row_data
                    .view
                    .album_name
                    .clone()
                    .unwrap_or("Unknown album".to_owned());

                album_name
                    .link(
                        Route::Album(row_data.view.track.album_id.clone()),
                        CellMessage::ChangeRoute,
                    )
                    .wrapping(Wrapping::None)
                    .ellipsis(text::Ellipsis::End)
                    .into()
            },
        )
        .intial_width(Length::FillPortion(1));

        // Duration column
        let duration_col = Column::new(
            || widget::text("Duration").into(),
            |row_data: &RowData| {
                widget::text(format_duration(
                    row_data.view.track.duration.unwrap_or_default() as f64,
                ))
                .into()
            },
        )
        .intial_width(Length::Fixed(96.0));

        // Genre column
        let genre_col = Column::new(
            || widget::text("Genres").into(),
            |row_data: &RowData| {
                row_data
                    .view
                    .genres
                    .clone()
                    .into_links(
                        |RelatedGenre { id, name }| {
                            (name.unwrap_or("Unknown genre".to_owned()), Route::Genre(id))
                        },
                        CellMessage::ChangeRoute,
                    )
                    .wrapping(Wrapping::None)
                    .ellipsis(text::Ellipsis::End)
                    .into()
            },
        )
        .intial_width(Length::FillPortion(1));

        Table::default()
            .column(title_col)
            .column(duration_col)
            .column(album_col)
            .column(genre_col)
            .column(index_col)
    }

    pub fn combined_title(row_data: &RowData) -> Element<'_, CellMessage> {
        let RowData {
            view: TrackView { track, artists, .. },
            image,
        } = row_data;

        let track_title = track.title.clone().unwrap_or("No title".to_owned());
        let artists = artists.clone();

        // Image
        let image: Element<'_, CellMessage> = if let Some(image) = image {
            // Show an image if it's ready
            image.view().map(CellMessage::ImageDriver)
        } else {
            // Or show a placeholder
            lucide::disc_album().size(18).into()
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
                        Route::Artist(id.to_string()),
                    )
                },
                CellMessage::ChangeRoute,
            )
            .wrapping(Wrapping::None)
            .ellipsis(text::Ellipsis::End)
            .size(14);

        row![
            widget::container(image).center(64),
            widget::space().width(8),
            widget::container(column![title, widget::space().width(2), artist_text])
                .center_y(64)
                .clip(true)
        ]
        .clip(true)
        .into()
    }
}
