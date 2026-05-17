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
    ui::{
        ICMsg, ToCmdMsg, ToOutMsg,
        components::image::{self, Image},
        router::Route,
    },
};

#[derive(Default)]
pub struct Sidebar {
    route: Route,
    playlists: Vec<PlaylistView>,
    images: image::Manager,
}

#[derive(Clone, Debug)]
pub enum Cmd {
    FetchPlaylists,
    PlaylistsFetched(Vec<PlaylistView>),
    ImageDriver(image::Message),
}

#[derive(Clone, Debug)]
pub enum Out {
    UpdateRoute(Route),
}

pub type SidebarMsg = ICMsg<Cmd, Out>;

impl Sidebar {
    pub fn view(&self) -> Element<'_, SidebarMsg> {
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
                self.playlists()
            ]
            .spacing(8),
        )
        .padding(8)
        .into()
    }

    pub fn update(&mut self, message: impl Into<SidebarMsg>) -> Task<SidebarMsg> {
        message.into().cmd(|cmd| {
            let task: Task<SidebarMsg> = match cmd {
                Cmd::FetchPlaylists => {
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
                        |p| Cmd::PlaylistsFetched(p).into(),
                    )
                }
                Cmd::PlaylistsFetched(playlist_views) => {
                    // Load images
                    self.images.insert(playlist_views.iter().filter_map(|view| {
                        let url = view.playlist.image_url.clone();
                        let blurhash = view.playlist.image_blur_hash.clone();
                        url.map(|url| Image::new(url).blurhash_maybe(blurhash))
                    }));

                    self.playlists = playlist_views;
                    Task::none()
                }
                Cmd::ImageDriver(m) => self.images.update(m).map(|m| Cmd::ImageDriver(m).cmd_msg()),
            };
            task
        })
    }

    /// The tab select of the sidebar
    pub fn tab_button<'a>(
        &'a self,
        icon: impl Into<Element<'a, SidebarMsg>>,
        text: impl Into<Element<'a, SidebarMsg>>,
        route: Route,
    ) -> Element<'a, SidebarMsg> {
        widget::button(
            row![
                icon.into(), // Here we manually erase the types
                text.into()
            ]
            .spacing(8),
        )
        .on_press(Out::UpdateRoute(route.clone()).out_msg())
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

    pub fn playlists(&self) -> Element<'_, SidebarMsg> {
        let playlists = self.playlists.iter().map(|view| {
            let image = if let Some(image_url) = view.playlist.image_url.as_ref()
                && let Some(image) = self.images.view(image_url)
            {
                // Show an image if it's ready
                image.map(|m| Cmd::ImageDriver(m).cmd_msg())
            } else {
                // Or show a placeholder
                lucide::disc_album().size(18).into()
            };

            let title = widget::text(view.playlist.name.clone().unwrap_or("No title".to_owned()))
                .wrapping(Wrapping::None)
                .ellipsis(text::Ellipsis::End);

            row![
                widget::container(image).center(64),
                widget::space().width(8),
                widget::container(column![title, widget::space().width(2)])
                    .center_y(64)
                    .clip(true)
            ]
            .into()
        });

        let playlists = widget::column(playlists);
        widget::sensor(playlists)
            .on_show(|_| Cmd::FetchPlaylists.cmd_msg())
            .into()
    }
}
