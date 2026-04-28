use iced::{
    Alignment::Center,
    Border, Element, Font,
    Length::Fill,
    Task,
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
    TracksFetched(Vec<TrackView>),
    /// A driver for the table
    TableDriver(table::Message<CellMessage>),
    Play,
}

#[derive(Debug, Clone)]
pub struct RowData {
    view: TrackView,
    image: Option<Image>,
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
            .on_press(Message::Play);

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
            self.track_table.view().map(Message::TableDriver)
        ];

        widget::sensor(tracks)
            .on_show(|_| Message::FetchTracks)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchTracks => {
                // Only fetch first time
                if !self.track_table.is_empty() {
                    return Task::none();
                }

                Task::perform(
                    async {
                        // TODO: Replace with global state
                        let jf = JellyfinApi::auth_user_password(
                            Url::parse("http://***REMOVED***").unwrap(),
                            "***REMOVED***".to_string(),
                            "***REMOVED***".to_string(),
                        )
                        .await;

                        jf.get_tracks().await.unwrap()
                    },
                    Message::TracksFetched,
                )
            }
            Message::TracksFetched(tracks) => {
                let row_data = tracks
                    .into_iter()
                    .map(|track_view| {
                        // Create an image if available
                        let image = track_view.track.image_url.clone().map(|url| {
                            let blurhash = track_view.track.image_blur_hash.clone();
                            Image::new(url).blurhash_maybe(blurhash)
                        });

                        RowData {
                            view: track_view,
                            image,
                        }
                    })
                    .collect();

                // Insert tracks
                self.track_table.extend_rows(row_data);

                Task::none()
            }
            // Message::ImageDriver(m) => self.images.update(m).map(Message::ImageDriver),
            Message::TableDriver(m) => self.track_table.update(m).map(Message::TableDriver),
            // Only bubbles to the router component
            // Message::ChangeRoute(_) => Task::none(),
            Message::Play => todo!(),
        }
    }

    /// One row representing the
    pub fn track_table() -> Table<RowData, CellMessage> {
        // Index column
        let index_col = Column::new(
            || widget::text("#").into(),
            move |row_data: &RowData| widget::text("0").into(),
        );

        let title_col = Column::new(|| widget::text("Name").into(), Self::combined_title)
            .align_x(Horizontal::Center);

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
        );

        // Duration column
        let duration_col = Column::new(
            || widget::text("Duration").into(),
            |row_data: &RowData| {
                widget::text(format_duration(
                    row_data.view.track.duration.unwrap_or_default() as f64,
                ))
                .into()
            },
        );

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
        );

        Table::default()
            .column(index_col)
            .column(title_col)
            .column(duration_col)
            .column(album_col)
            .column(genre_col)
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
        .into()
    }
}
