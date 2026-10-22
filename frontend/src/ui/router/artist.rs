use backend::{
    api::endpoint_api::{EndpointManager, GetAlbumsParams},
    data_view::ArtistView,
};

use iced::{
    Element, Task,
    alignment::Vertical,
    widget::{
        self, Button, button,
        text::{self},
    },
};

use crate::ui::components::{
    album_table::{AlbumRow, AlbumTable, AlbumTableMsg},
    icons::{self, Icon},
    image::{Image, ImgMsg},
    style::header_text,
};

#[derive(Default)]
pub struct Artist {
    id: Option<String>,
    state: PageState,
}

#[derive(Default)]
pub enum PageState {
    #[default]
    None,
    Loading,
    Loaded(Box<ArtistData>),
}

#[derive(Debug, Clone)]
pub struct ArtistData {
    artist: ArtistView,
    artist_image: Option<Image>,
    album_table: AlbumTable,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Fetches the artist data for a specific artist
    FetchArtist(String),
    ArtistFetched(Box<ArtistData>),
    ImageDriver(ImgMsg),
    Play,
    PlayNext,
    PlayLast,
    TableDriver(AlbumTableMsg),
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

                        let rows = albums.into_iter().map(AlbumRow::from);

                        let mut album_table = AlbumTable::default();
                        album_table.extend(rows);

                        ArtistData {
                            artist,
                            album_table,
                            artist_image: image,
                        }
                    },
                    |data| Message::ArtistFetched(Box::new(data)),
                )
            }
            Message::ArtistFetched(artist_data) => {
                self.state = PageState::Loaded(Box::new(*artist_data));
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
            Message::Play => todo!(),
            Message::PlayNext => todo!(),
            Message::PlayLast => todo!(),
            Message::TableDriver(message) => {
                if let PageState::Loaded(ref mut data) = self.state {
                    data.album_table.update(message).map(Message::TableDriver)
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn artist_page<'a>(&self, artist_data: &'a ArtistData) -> Element<'a, Message> {
        widget::column([self.header(artist_data), self.albums(artist_data)])
            .spacing(40)
            .padding(16)
            .into()
    }

    pub fn header<'a>(&self, artist_data: &'a ArtistData) -> Element<'a, Message> {
        // Image
        let image: Element<'_, Message> = if let Some(ref image) = artist_data.artist_image {
            // Show an image if it's ready
            image.view().map(Message::ImageDriver)
        } else {
            // Or show a placeholder
            let icon_size = f64::from(IMAGE_SIZE) * 0.8;
            icons::user().size(icon_size as u32).into()
        };

        let name = artist_data
            .artist
            .artist
            .name
            .clone()
            .unwrap_or(String::from("Unnamed artist"));

        fn action_button<'a>(
            icon: Icon<'a>,
            text: impl text::IntoFragment<'a>,
        ) -> Button<'a, Message> {
            const TEXT_SIZE: u32 = 18;

            widget::button(
                widget::row([
                    icon.style(text::default).size(TEXT_SIZE).into(),
                    widget::text(text).size(TEXT_SIZE).into(),
                ])
                .spacing(8),
            )
            .style(button::primary)
        }

        let play_button = action_button(icons::play_filled(), "Play").on_press(Message::Play);
        let next_button = action_button(icons::play_filled(), "Next").on_press(Message::PlayNext);
        let last_button = action_button(icons::play_filled(), "Last").on_press(Message::PlayLast);

        // Play, Next, Last
        let buttons =
            widget::row([play_button.into(), next_button.into(), last_button.into()]).spacing(10);

        // Displays name, etc. next to the image
        let info_col = widget::column([header_text(name).into(), buttons.into()]).spacing(20);

        widget::row([image, info_col.into()])
            .spacing(12) // Space right of image
            .align_y(Vertical::Bottom)
            .into()
    }

    fn albums<'a>(&self, artist_data: &'a ArtistData) -> Element<'a, Message> {
        widget::column([
            header_text("Released albums").size(22).into(),
            artist_data.album_table.view().map(Message::TableDriver),
        ])
        .spacing(8)
        .into()
    }
}
