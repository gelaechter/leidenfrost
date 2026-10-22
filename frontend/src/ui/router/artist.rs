use backend::{
    api::endpoint_api::{EndpointManager, GetAlbumsParams},
    data_view::ArtistView,
};
use iced::{
    Element, Renderer, Task, Theme,
    widget::{
        self, Text, button,
        text::{self},
    },
};

use crate::ui::{
    components::{
        icons,
        image::{Image, ImgMsg},
    },
    router::albums::AlbumRow,
};

#[derive(Debug, Default)]
pub struct Artist {
    id: Option<String>,
    state: PageState,
}

#[derive(Debug, Default)]
pub enum PageState {
    #[default]
    None,
    Loading,
    Loaded(ArtistData),
}

#[derive(Debug, Clone)]
pub struct ArtistData {
    artist: ArtistView,
    artist_image: Option<Image>,
    albums: Vec<AlbumRow>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Fetches the artist data for a specific artist
    FetchArtist(String),
    ArtistFetched(ArtistData),
    ImageDriver(ImgMsg),
}

const IMAGE_SIZE: u32 = 256;

impl Artist {
    pub fn view<'a>(&'a self, id: &'a str) -> Element<'a, Message> {
        let container = widget::container(match self.state {
            PageState::Loaded(ref artist_data) => self.artist_page(artist_data),
            _ => widget::text("Loading...").into(),
        });

        widget::sensor(container)
            .on_show(move |_| Message::FetchArtist(id.to_owned()))
            .key("artist")
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchArtist(id) => {
                if self.id == Some(id.clone()) {
                    return Task::none();
                }

                Task::perform(
                    async {
                        let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                        let artist = endpoint.get_artist(id.clone()).await.unwrap();

                        let albums = endpoint
                            .get_artist_albums(id, GetAlbumsParams::default())
                            .await
                            .unwrap();

                        let image = artist.artist.image_url.clone().map(|url| {
                            Image::new(url)
                                .blurhash_maybe(artist.artist.image_blur_hash.clone())
                                .height(IMAGE_SIZE)
                                .width(IMAGE_SIZE)
                        });

                        ArtistData {
                            artist,
                            albums: albums.into_iter().map(AlbumRow::from).collect(),
                            artist_image: image,
                        }
                    },
                    Message::ArtistFetched,
                )
            }
            Message::ArtistFetched(artist_data) => {
                self.state = PageState::Loaded(artist_data);
                Task::none()
            }
            Message::ImageDriver(message) => {
                if let PageState::Loaded(ref mut data) = self.state
                    && let Some(image) = &mut data.artist_image
                {
                    image.update(message).map(Message::ImageDriver)
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn artist_page<'a>(&self, artist_data: &'a ArtistData) -> Element<'a, Message> {
        widget::column([self.artist_header(artist_data)])
            .padding(16)
            .into()
    }

    pub fn artist_header<'a>(&self, artist_data: &'a ArtistData) -> Element<'a, Message> {
        // Image
        let image: Element<'_, Message> = if let Some(ref image) = artist_data.artist_image {
            // Show an image if it's ready
            image.view().map(Message::ImageDriver)
        } else {
            // Or show a placeholder
            let icon_size = f64::from(IMAGE_SIZE) * 0.8;
            icons::user().size(icon_size as u32).into()
        };

        let name = widget::text(
            artist_data
                .artist
                .artist
                .name
                .clone()
                .unwrap_or(String::from("Unnamed artist")),
        );

        fn action_button<'a>(
            icon: Text<'a, Theme, Renderer>,
            text: impl text::IntoFragment<'a>,
        ) -> Element<'a, Message> {
            const TEXT_SIZE: u32 = 22;

            widget::button(widget::row([
                icon.size(TEXT_SIZE).into(),
                widget::text(text).size(TEXT_SIZE).into(),
            ]))
            .style(button::primary)
            .into()
        }

        // Play, Next, Last
        let buttons = widget::row([
            action_button(icons::play_filled(), "Play"),
            action_button(icons::play_filled(), "Next"),
            action_button(icons::play_filled(), "Last"),
        ])
        .spacing(10);

        // Displays name, etc. next to the image
        let info_col = widget::column([name.into(), buttons.into()]);

        widget::row([widget::container(image).center(40).into(), info_col.into()]).into()
    }
}
