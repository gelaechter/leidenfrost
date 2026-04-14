use iced::{
    Background, Border, Element,
    Length::Fill,
    Task,
    widget::{
        self,
        button::{self},
        container, space,
        text::Wrapping,
    },
};
use iced::{
    Font, font,
    widget::{column, row, text},
};
use iced_fonts::lucide;
use url::Url;

use crate::{
    backend::{
        api::{
            endpoint_api::{ApiContract, UserPasswordAuth},
            jellyfin::api::JellyfinApi,
        },
        data_view::PlaylistView,
    },
    ui::router::Route,
};

#[derive(Default)]
pub struct Sidebar {
    route: Route,
    playlists: Vec<PlaylistView>,
}

#[derive(Clone, Debug)]
pub enum Message {
    FetchPlaylists,
    PlaylistsFetched(Vec<PlaylistView>),
    UpdateRoute(Route),
}

impl Sidebar {
    pub fn view(&self) -> Element<'_, Message> {
        let bold = Font {
            weight: font::Weight::Medium,
            ..Font::DEFAULT
        };

        container(
            column![
                text("My Library").font(bold),
                self.tab_button(lucide::house(), text("Home").font(bold), Route::Home),
                self.tab_button(
                    lucide::heart(),
                    text("Favorites").font(bold),
                    Route::Favorites
                ),
                self.tab_button(lucide::disc_two(), text("Albums").font(bold), Route::Albums),
                self.tab_button(lucide::music(), text("Tracks").font(bold), Route::Tracks),
                self.tab_button(lucide::user(), text("Artists").font(bold), Route::Artists),
                self.tab_button(lucide::tag(), text("Genres").font(bold), Route::Genres),
                space().height(28),
                text("Playlists").font(bold),
            ]
            .spacing(8),
        )
        .padding(8)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateRoute(route) => {
                self.route = route;
                Task::none()
            }
            Message::FetchPlaylists => {
                // Only fetch first time
                if !self.playlists.is_empty() {
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

                        jf.get_playlists().await.unwrap()
                    },
                    Message::PlaylistsFetched,
                )
            }
            Message::PlaylistsFetched(playlist_views) => {
                self.playlists = playlist_views;
                Task::none()
            }
        }
    }

    /// The tab select of the sidebar
    pub fn tab_button<'a>(
        &'a self,
        icon: impl Into<Element<'a, Message>>,
        text: impl Into<Element<'a, Message>>,
        route: Route,
    ) -> Element<'a, Message> {
        widget::button(
            row![
                icon.into(), // Here we manually erase the types
                text.into()
            ]
            .spacing(8),
        )
        .on_press(Message::UpdateRoute(route.clone()))
        .style(move |theme, status| {
            let palette = theme.palette();
            button::Style {
                // Round the button
                border: Border::default().rounded(2),
                // Make gray if hovered over
                background: if matches!(status, button::Status::Hovered) {
                    Some(Background::Color(palette.background.weak.color))
                } else {
                    None
                },
                // Make blue if route is selected
                text_color: if self.route == route {
                    palette.primary.strong.color
                } else {
                    button::Style::default().text_color
                },
                ..Default::default()
            }
        })
        .width(Fill)
        .into()
    }

    pub fn playlists(&self) -> Element<'_, Message> {
        // let playlists = self.playlists.iter().map(|view| {
        //     let image = if let Some(image_id) =
        //         view.playlist.image_url.as_ref().map(|u| Image::uuid(u))
        //         && let Some(image) = self.images.get(&image_id)
        //     {
        //         // Show an image if it's ready
        //         image.view().map(move |m| Message::ImageDriver(image_id, m))
        //     } else {
        //         // Or show a placeholder
        //         lucide::disc_album().size(18).into()
        //     };

        //     let title = widget::text(view.playlist.name.clone().unwrap_or("No title".to_owned()))
        //         .wrapping(Wrapping::None)
        //         .ellipsis(text::Ellipsis::End);

        //     let artist_text = artists
        //         .clone()
        //         .into_links(
        //             |RelatedArtist { id, name }| {
        //                 (
        //                     name.unwrap_or("Unknown artist".to_owned()),
        //                     Route::Artist(id.to_string()),
        //                 )
        //             },
        //             Message::ChangeRoute,
        //         )
        //         .wrapping(Wrapping::None)
        //         .ellipsis(text::Ellipsis::End)
        //         .size(14);

        //     row![
        //         widget::container(image).center(64),
        //         widget::space().width(8),
        //         widget::container(column![title, widget::space().width(2), artist_text])
        //             .center_y(64)
        //             .clip(true)
        //     ]
        //     .into()
        // });

        // widget::column(playlists).into()
        todo!()
    }
}
