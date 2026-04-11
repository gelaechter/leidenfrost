use std::collections::HashMap;

use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, FillPortion},
    Task,
    widget::{self, column, rich_text, row, table},
};
use iced_fonts::lucide;
use url::Url;
use uuid::Uuid;

use crate::{
    backend::{
        api::{
            endpoint_api::{ApiContract, UserPasswordAuth},
            jellyfin::api::JellyfinApi,
        },
        data_view::{RelatedArtist, RelatedGenre, TrackView},
        db::models::Track,
    },
    ui::{
        components::{
            image::{self, Image},
            utils::IntoLinks,
        },
        router::Route,
        util::format_duration,
    },
};

/// The tracks route
#[derive(Default)]
pub struct Tracks {
    tracks: Vec<TrackView>,
    images: HashMap<Uuid, Image>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchTracks,
    TracksFetched(Vec<TrackView>),
    ImageDriver(Uuid, image::Message),
    ChangeRoute(Route),
}

impl Tracks {
    pub fn view(&self) -> Element<'_, Message> {
        let tracks = column!["Tracks", widget::scrollable(self.track_table())];

        widget::sensor(tracks)
            .on_show(|_| Message::FetchTracks)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchTracks => {
                // Only fetch first time
                if !self.tracks.is_empty() {
                    return Task::none();
                }

                Task::perform(
                    async {
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
                // Create images from tracks
                for v in tracks.iter().filter(|v| v.track.image_url.is_some()) {
                    let url = v.track.image_url.clone().unwrap();
                    let (id, image) = Image::new(url.into(), v.track.image_blur_hash.clone());
                    self.images.insert(id, image);
                }
                // Insert tracks
                self.tracks = tracks;

                Task::none()
            }
            Message::ImageDriver(url, m) => {
                if let Some(image) = self.images.get_mut(&url) {
                    image.update(m).map(move |m| Message::ImageDriver(url, m))
                } else {
                    Task::none()
                }
            }
            // Only bubbles up
            Message::ChangeRoute(_) => Task::none(),
        }
    }

    pub fn combined_title(&self, v: &TrackView) -> Element<'_, Message> {
        let TrackView {
            track: Track {
                image_url, title, ..
            },
            artists,
            ..
        } = v;

        let image = if let Some(image_id) = image_url.as_ref().map(|u| Image::uuid(u))
            && let Some(image) = self.images.get(&image_id)
        {
            image.view().map(move |m| Message::ImageDriver(image_id, m))
        } else {
            // Or show a placeholder
            lucide::disc_album().into()
        };

        let title = widget::text(title.clone().unwrap_or("No title".to_owned()));

        let artist_text = artists
            .clone()
            .into_links(
                |RelatedArtist { id, name }| {
                    (
                        name.unwrap_or("Unknown artist".to_owned()),
                        Route::Artist(id.to_string()),
                    )
                },
                Message::ChangeRoute,
            )
            .size(12);

        row![
            widget::container(image).center(64),
            widget::space().width(8),
            widget::container(column![title, artist_text]).center_y(64)
        ]
        .into()
    }

    pub fn track_table(&self) -> Element<'_, Message> {
        let columns = [
            // Name column
            table::column("Title", |v: &TrackView| {
                // Get the image by id
                self.combined_title(v)
            })
            .width(FillPortion(6))
            .align_y(Center),
            // Duration column
            table::column("Duration", |v: &TrackView| {
                widget::text(format_duration(v.track.duration.unwrap_or_default() as f64))
            })
            .width(FillPortion(1))
            .align_x(Center)
            .align_y(Center),
            // Album column
            table::column("Album", |v: &TrackView| {
                let album_name = v.album_name.clone().unwrap_or("Unknown album".to_owned());

                rich_text![widget::span(album_name).link(Route::Album(v.track.album_id.clone()))]
                    .on_link_click(Message::ChangeRoute)
            })
            .width(FillPortion(2))
            .align_y(Center),
            // Genre
            table::column("Genre", |v: &TrackView| {
                v.genres.clone().into_links(
                    |RelatedGenre { id, name }| {
                        (name.unwrap_or("Unknown genre".to_owned()), Route::Genre(id))
                    },
                    Message::ChangeRoute,
                )
            })
            .width(FillPortion(2))
            .align_y(Center),
        ];

        table(columns, &self.tracks)
            .padding_x(25)
            .padding_y(5)
            .separator_x(1)
            .separator_y(1)
            .width(Fill)
            .into()
    }
}
