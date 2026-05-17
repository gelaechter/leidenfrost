pub mod albums;
pub mod tracks;

use iced::{
    Element,
    Length::Fill,
    Task,
    advanced::widget::tree::Tag,
    widget::{self, container::background},
};

use crate::ui::{
    components::tagged::tagged,
    router::{albums::Albums, tracks::Tracks},
    settings::{Settings, SettingsMsg},
};

#[derive(Default)]
pub struct Router {
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
pub enum Message {
    ChangeRoute(Route),
    TracksDriver(tracks::Message),
    AlbumsDriver(albums::Message),
    SettingsDriver(SettingsMsg),
}

impl Router {
    // TODO: Right now we take &Settings everytime we view;
    //  Instead consider having the Router own an Rc<RefCell<Settings>>
    pub fn view<'a>(&'a self, settings: &'a Settings) -> Element<'a, Message> {
        struct Albums;
        struct Home;
        struct Tracks;
        struct Settings;

        /// Stores a view as well as a tag for use with [`tagged`]
        struct RouteView<'a> {
            view: Element<'a, Message>,
            tag: Tag,
        }

        let route = match self.route {
            Route::Home => RouteView {
                view: widget::container("text").width(Fill).height(Fill).into(),
                tag: Tag::of::<Home>(),
            },
            Route::Favorites => todo!(),
            Route::Albums => RouteView {
                view: self.albums.view().map(Message::AlbumsDriver),
                tag: Tag::of::<Albums>(),
            },
            Route::Tracks => RouteView {
                view: self.tracks.view().map(Message::TracksDriver),
                tag: Tag::of::<Tracks>(),
            },
            Route::AlbumArtists => todo!(),
            Route::Artists => todo!(),
            Route::Genres => todo!(),
            Route::Artist(_) => todo!(),
            Route::Genre(_) => todo!(),
            Route::Album(_) => todo!(),
            Route::Settings => RouteView {
                view: settings.view().map(Message::SettingsDriver),
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

    pub fn update(&mut self, settings: &mut Settings, message: Message) -> Task<Message> {
        match message {
            Message::ChangeRoute(route) => {
                self.route = route;
                Task::none()
            }
            Message::TracksDriver(message) => {
                self.tracks.update(message).map(Message::TracksDriver)
            }
            Message::AlbumsDriver(message) => {
                self.albums.update(message).map(Message::AlbumsDriver)
            }
            Message::SettingsDriver(icmsg) => {
                // We directly pass these upwards
                settings.update(icmsg);
                Task::none()
            }
        }
    }
}
