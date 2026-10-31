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

use crate::ui::{
    app::AppState,
    router::{
        albums::Albums,
        artist::Artist,
        artists::Artists,
        genres::Genres,
        search::SearchPage,
        settings::{Settings, SettingsMsg},
        tracks::Tracks,
    },
    widgets::tagged::tagged,
};

#[derive(Default)]
pub struct Router {
    /// public so that the app can access the settings
    pub settings: Settings,
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
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, RouterMsg> {
        struct Home;

        /// Stores a view as well as a tag for use with [`tagged`]
        struct RouteView<'a> {
            view: Element<'a, RouterMsg>,
            tag: Tag,
        }

        let route = match app_state.route {
            Route::Home => RouteView {
                view: widget::container("text").width(Fill).height(Fill).into(),
                tag: Tag::of::<Home>(),
            },
            Route::Favorites => todo!(),
            Route::Albums => RouteView {
                view: self.albums.view(app_state).map(Msg::AlbumsDriver),
                tag: Tag::of::<Albums>(),
            },
            Route::Tracks => RouteView {
                view: self.tracks.view(app_state).map(Msg::TracksDriver),
                tag: Tag::of::<Tracks>(),
            },
            Route::AlbumArtists => todo!(),
            Route::Artists => RouteView {
                view: self.artists.view(app_state).map(Msg::ArtistsDriver),
                tag: Tag::of::<Artists>(),
            },
            Route::Genres => RouteView {
                view: self.genres.view(app_state).map(Msg::GenresDriver),
                tag: Tag::of::<Genres>(),
            },
            Route::Artist(ref id) => RouteView {
                view: self.artist.view(id, app_state).map(Msg::ArtistDriver),
                tag: Tag::of::<Artist>(),
            },
            Route::Genre(_) => todo!(),
            Route::Album(_) => todo!(),
            Route::Settings => RouteView {
                view: self.settings.view().map(Msg::SettingsDriver),
                tag: Tag::of::<Settings>(),
            },
            Route::Search => RouteView {
                view: self.search_page.view(app_state).map(Msg::SearchDriver),
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

    pub fn update(
        &mut self,
        message: impl Into<RouterMsg>,
        app_state: &mut AppState,
    ) -> Task<RouterMsg> {
        match message.into() {
            Msg::ChangeRoute(route) => {
                app_state.route = route.clone();
                // Notify the sidebar of the change so it can show the current route
                Task::none()
            }
            Msg::TracksDriver(message) => self
                .tracks
                .update(message, app_state)
                .map(Msg::TracksDriver),
            Msg::AlbumsDriver(message) => self
                .albums
                .update(message, app_state)
                .map(Msg::AlbumsDriver),
            Msg::SettingsDriver(message) => self
                .settings
                .update(message, app_state)
                .map(Msg::SettingsDriver),
            Msg::ArtistsDriver(message) => self
                .artists
                .update(message, app_state)
                .map(Msg::ArtistsDriver),
            Msg::GenresDriver(message) => self
                .genres
                .update(message, app_state)
                .map(Msg::GenresDriver),
            Msg::ArtistDriver(message) => self
                .artist
                .update(message, app_state)
                .map(Msg::ArtistDriver),
            Msg::SearchDriver(message) => self
                .search_page
                .update(message, app_state)
                .map(Msg::SearchDriver),
        }
    }
}
