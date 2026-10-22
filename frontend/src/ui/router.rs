pub mod albums;
pub mod artist;
pub mod artists;
pub mod genres;
pub mod playlists;
pub mod search;
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
    command_palette::Message::ChangeRoute,
    router::{
        albums::Albums,
        artist::Artist,
        artists::Artists,
        genres::Genres,
        search::SearchPage,
        settings::{Settings, SettingsMsg},
        tracks::Tracks,
    },
    sidebar::{Sidebar, SidebarMsg},
    widgets::tagged::tagged,
};

#[derive(Default, Receiver)]
#[message(RouterMsg)]
pub struct Router {
    /// public so that the app can access the settings
    pub settings: Settings,
    route: Route,
    tracks: Tracks,
    albums: Albums,
    artists: Artists,
    genres: Genres,
    artist: Artist,
    search_page: SearchPage,
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
    Search,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Msg {
    ChangeRoute(Route),
    TracksDriver(tracks::TracksMsg),
    AlbumsDriver(albums::Message),
    ArtistsDriver(artists::Message),
    GenresDriver(genres::Message),
    SettingsDriver(SettingsMsg),
    ArtistDriver(artist::Message),
    SearchDriver(search::Message),
}

pub type RouterMsg = Msg;

impl Router {
    pub fn view(&self) -> Element<'_, RouterMsg> {
        struct Home;

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
            Route::Artists => RouteView {
                view: self.artists.view().map(Msg::ArtistsDriver),
                tag: Tag::of::<Artists>(),
            },
            Route::Genres => RouteView {
                view: self.genres.view().map(Msg::GenresDriver),
                tag: Tag::of::<Genres>(),
            },
            Route::Artist(ref id) => RouteView {
                view: self.artist.view(id).map(Msg::ArtistDriver),
                tag: Tag::of::<Artist>(),
            },
            Route::Genre(_) => todo!(),
            Route::Album(_) => todo!(),
            Route::Settings => RouteView {
                view: self.settings.view().map(Msg::SettingsDriver),
                tag: Tag::of::<Settings>(),
            },
            Route::Search => RouteView {
                view: self.search_page.view().map(Msg::SearchDriver),
                tag: Tag::of::<SearchPage>(),
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
            Msg::SettingsDriver(message) => self.settings.update(message).map(Msg::SettingsDriver),
            Msg::ArtistsDriver(message) => self.artists.update(message).map(Msg::ArtistsDriver),
            Msg::GenresDriver(message) => self.genres.update(message).map(Msg::GenresDriver),
            Msg::ArtistDriver(message) => self.artist.update(message).map(Msg::ArtistDriver),
            Msg::SearchDriver(message) => self.search_page.update(message).map(Msg::SearchDriver),
        }
    }
}

impl Router {
    pub fn change_route(route: Route) {
        Self::send(Msg::ChangeRoute(route));
    }
}
