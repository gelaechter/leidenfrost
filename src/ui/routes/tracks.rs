use std::collections::HashMap;

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Task,
    widget::{self, row, table, text},
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
        db::models::Track,
    },
    ui::{
        components::image::{self, Image},
        util::format_duration,
    },
};
use iced::widget::column;

/// The tracks route
#[derive(Default)]
pub struct Tracks {
    tracks: Vec<Track>,
    images: HashMap<Uuid, Image>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchTracks,
    TracksFetched(Vec<Track>),
    ImageDriver(Uuid, image::Message),
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
            Message::FetchTracks => Task::perform(
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
            ),
            Message::TracksFetched(tracks) => {
                // Create images from tracks
                for track in tracks.iter().filter(|t| t.image_url.is_some()) {
                    let url = track.image_url.clone().unwrap();
                    let (id, image) = Image::new(url.into(), track.image_blur_hash.clone());
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
        }
    }

    pub fn combined_title(&self, track: &Track) -> Element<'_, Message> {
        let image = if let Some(image_id) = track.image_url.as_ref().map(|u| Image::uuid(u))
            && let Some(image) = self.images.get(&image_id)
        {
            image.view().map(move |m| Message::ImageDriver(image_id, m))
        } else {
            // Or show a placeholder
            widget::container(lucide::disc_album()).into()
        };

        let title = text(track.title.clone().unwrap_or("-".to_owned()));
        let artist = text("TODO:");
        let container = widget::container(column![title, artist]).center_y(64);
        row![image, container].into()
    }

    pub fn track_table(&self) -> Element<'_, Message> {
        let columns = [
            // Name column
            table::column("Title", |track: &Track| {
                // Get the image by id
                self.combined_title(track)
            })
            .align_y(Center),
            // Duration column
            table::column("Duration", |track: &Track| {
                text(format_duration(track.duration.unwrap_or_default() as f64))
            })
            .align_x(Center)
            .align_y(Center),
            // Album column
            table::column("Album", |track: &Track| {
                text(
                    // track.album.clone().unwrap_or_default()
                    "TODO:",
                )
            })
            .align_x(Center)
            .align_y(Center),
            // Genre
            table::column("Genre", |track: &Track| {
                text(
                    // Join genre names by comma
                    // track.
                    //     .genres
                    //     .iter()
                    //     .map(|g| g.name.clone())
                    //     .collect::<Vec<String>>()
                    //     .join(", "),
                    "TODO:",
                )
            })
            .align_x(Center)
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
