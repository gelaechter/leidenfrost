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

use backend::{
    api::endpoint_api::{EndpointManager, GetPlaylistsParams},
    data_view::PlaylistView,
};
use macros::Receiver;

use crate::ui::{
    Receiver,
    app::AppState,
    components::{
        icons::{self, Icon},
        image::{self, Image},
    },
    router::Route,
};

#[derive(Default, Receiver)]
#[message(SidebarMsg)]
pub struct Sidebar {
    playlists: Vec<SidebarPlaylist>,
}

#[derive(Clone, Debug)]
pub struct SidebarPlaylist {
    view: PlaylistView,
    image: Option<Image>,
}

#[derive(Clone, Debug)]
pub enum Cmd {
    FetchPlaylists,
    ImageDriver((usize, image::ImgMsg)),
    /// Change the route because the user clicked a button
    ChangeRoute(Route),
    PlaylistsFetched(Vec<SidebarPlaylist>),
}

impl From<PlaylistView> for SidebarPlaylist {
    fn from(view: PlaylistView) -> Self {
        let url = view.playlist.image_url.clone();
        let blurhash = view.playlist.image_blur_hash.clone();
        let image = url.map(|url| Image::new(url).blurhash_maybe(blurhash));

        Self { view, image }
    }
}

pub type SidebarMsg = Cmd;

impl Sidebar {
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, SidebarMsg> {
        let bold = Font {
            weight: font::Weight::Medium,
            ..Font::DEFAULT
        };

        container(
            column![
                text("My Library").font(bold),
                self.tab_button(
                    icons::home(),
                    text("Home").font(bold),
                    Route::Home,
                    &app_state.route
                ),
                self.tab_button(
                    icons::search(),
                    text("Search").font(bold),
                    Route::Search,
                    &app_state.route
                ),
                self.tab_button(
                    icons::heart(),
                    text("Favorites").font(bold),
                    Route::Favorites,
                    &app_state.route
                ),
                self.tab_button(
                    icons::disc(),
                    text("Albums").font(bold),
                    Route::Albums,
                    &app_state.route
                ),
                self.tab_button(
                    icons::music(),
                    text("Tracks").font(bold),
                    Route::Tracks,
                    &app_state.route
                ),
                self.tab_button(
                    icons::user(),
                    text("Artists").font(bold),
                    Route::Artists,
                    &app_state.route
                ),
                self.tab_button(
                    icons::tag(),
                    text("Genres").font(bold),
                    Route::Genres,
                    &app_state.route
                ),
                self.tab_button(
                    icons::settings(),
                    text("Settings").font(bold),
                    Route::Settings,
                    &app_state.route
                ),
                space().height(28),
                text("Playlists").font(bold),
                self.playlists()
            ]
            .spacing(8),
        )
        .padding(8)
        .into()
    }

    pub fn update(
        &mut self,
        message: impl Into<SidebarMsg>,
        app_state: &mut AppState,
    ) -> Task<SidebarMsg> {
        match message.into() {
            Cmd::FetchPlaylists => {
                // Only fetch first time
                if !self.playlists.is_empty() {
                    return Task::none();
                }

                Task::perform(
                    async {
                        // TODO: Replace with global state
                        let endpoint = EndpointManager::get_active_endpoint().await.unwrap();

                        let playlists = endpoint
                            .get_playlists(GetPlaylistsParams::default())
                            .await
                            .unwrap();

                        playlists.into_iter().map(SidebarPlaylist::from).collect()
                    },
                    Cmd::PlaylistsFetched,
                )
            }
            Cmd::PlaylistsFetched(playlist_views) => {
                self.playlists = playlist_views;
                Task::none()
            }
            Cmd::ChangeRoute(new_route) => {
                app_state.route = new_route;
                Task::none()
            }
            Cmd::ImageDriver((idx, message)) => {
                if let Some(get_mut) = self.playlists.get_mut(idx)
                    && let Some(ref mut image) = get_mut.image
                {
                    image
                        .update(message)
                        .map(move |m| Cmd::ImageDriver((idx, m)))
                } else {
                    Task::none()
                }
            }
        }
    }

    /// The tab select of the sidebar
    pub fn tab_button<'a>(
        &'a self,
        icon: Icon<'a>,
        text: impl Into<Element<'a, SidebarMsg>>,
        button_route: Route,
        current_route: &'a Route,
    ) -> Element<'a, SidebarMsg> {
        widget::button(row![icon.style(text::default), text.into()].spacing(8))
            .on_press(Cmd::ChangeRoute(button_route.clone()))
            .style(move |theme, status| {
                let palette = theme.palette();
                button::Style {
                    // Round the button
                    border: Border::default().rounded(2),
                    // Make background gray if hovered over
                    background: if matches!(status, button::Status::Hovered) {
                        Some(Background::Color(palette.background.weak.color))
                    } else {
                        None
                    },
                    // Make text blue if route is selected
                    text_color: if *current_route == button_route {
                        palette.primary.strong.color
                    } else {
                        palette.background.base.text
                    },
                    ..Default::default()
                }
            })
            .width(Fill)
            .into()
    }

    pub fn playlists(&self) -> Element<'_, SidebarMsg> {
        let playlists = self.playlists.iter().enumerate().map(|(idx, data)| {
            let image = if let Some(ref image) = data.image {
                image.view().map(move |msg| Cmd::ImageDriver((idx, msg)))
            } else {
                // Or show a placeholder
                icons::disc().size(18).into()
            };

            let title = widget::text(
                data.view
                    .playlist
                    .name
                    .clone()
                    .unwrap_or("No title".to_owned()),
            )
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
            .on_show(|_| Cmd::FetchPlaylists)
            .into()
    }
}
