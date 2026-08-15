pub mod albums;
pub mod settings;
pub mod tracks;

use iced::{
    Element,
    Length::Fill,
    Task,
    advanced::widget::tree::Tag,
    widget::{self, container::background},
};
use macros::Receiver;

use crate::ui::{
    Receiver,
    components::tagged::tagged,
    router::{
        albums::Albums,
        settings::{Settings, SettingsMsg},
        tracks::Tracks,
    },
    sidebar::{Sidebar, SidebarMsg},
};

#[derive(Default, Receiver)]
#[message(RouterMsg)]
pub struct Router {
    /// public so that the app can access the settings
    pub settings: Settings,
    route: Route,
    tracks: Tracks,
    albums: Albums,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum Route {
    #[default]
    Home,
    Favorites,
    /// A specific album identified by Id
    Album(String),
    Albums,
    Tracks,
    AlbumArtists,
    /// A specific artist identified by Id
    Artist(String),
    Artists,
    /// A specific genre identified by Id
    Genre(String),
    Genres,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Msg {
    ChangeRoute(Route),
    TracksDriver(tracks::TracksMsg),
    AlbumsDriver(albums::Message),
    SettingsDriver(SettingsMsg),
}

pub type RouterMsg = Msg;

impl Router {
    pub fn view(&self) -> Element<'_, RouterMsg> {
        struct Albums;
        struct Home;
        struct Tracks;
        struct Settings;

        /// Stores a view as well as a tag for use with [`tagged`]
        struct RouteView<'a> {
            view: Element<'a, RouterMsg>,
            tag: Tag,
        }

        let route = match self.route {
            Route::Home => RouteView {
                view: widget::container("text").width(Fill).height(Fill).into(),
                tag: Tag::of::<Home>(),
            },
            Route::Favorites => todo!(),
            Route::Albums => RouteView {
                view: self.albums.view().map(Msg::AlbumsDriver),
                tag: Tag::of::<Albums>(),
            },
            Route::Tracks => RouteView {
                view: self.tracks.view().map(Msg::TracksDriver),
                tag: Tag::of::<Tracks>(),
            },
            Route::AlbumArtists => todo!(),
            Route::Artists => todo!(),
            Route::Genres => todo!(),
            Route::Artist(_) => todo!(),
            Route::Genre(_) => todo!(),
            Route::Album(_) => todo!(),
            Route::Settings => RouteView {
                view: self.settings.view().map(Msg::SettingsDriver),
                tag: Tag::of::<Settings>(),
            },
        };

        // Treats the routes as fundamentally different
        // widgets that should not be reconciled between
        let route = tagged(route.view, route.tag);

        widget::container(route)
            .style(|theme| background(theme.palette().background.weakest.color))
            .into()
    }

    pub fn update(&mut self, message: impl Into<RouterMsg>) -> Task<RouterMsg> {
        match message.into() {
            Msg::ChangeRoute(route) => {
                self.route = route.clone();
                // Notify the sidebar of the change so it can show the current route
                Sidebar::send(SidebarMsg::RouteChanged(route));
                Task::none()
            }
            Msg::TracksDriver(message) => self.tracks.update(message).map(Msg::TracksDriver),
            Msg::AlbumsDriver(message) => self.albums.update(message).map(Msg::AlbumsDriver),
            Msg::SettingsDriver(icmsg) => self.settings.update(icmsg).map(Msg::SettingsDriver),
        }
    }
}
