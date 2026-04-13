use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{
        self,
        container::background,
    },
};

use crate::ui::routes::tracks::{self, Tracks};

#[derive(Default)]
pub struct Router {
    route: Route,
    tracks: Tracks,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum Route {
    #[default]
    Home,
    Favorites,
    // A specific album identified by Id
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
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeRoute(Route),
    TracksMessage(tracks::Message),
}

impl Router {
    pub fn view(&self) -> Element<'_, Message> {
        widget::container(match self.route {
            Route::Home => widget::container("text")
                .width(Fill)
                .height(Fill)
                .into(),
            Route::Favorites => todo!(),
            Route::Albums => todo!(),
            Route::Tracks => self.tracks.view().map(Message::TracksMessage),
            Route::AlbumArtists => todo!(),
            Route::Artists => todo!(),
            Route::Genres => todo!(),
            Route::Artist(_) => todo!(),
            Route::Genre(_) => todo!(),
            Route::Album(_) => todo!(),
        })
        .style(|theme| background(theme.palette().background.weakest.color))
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeRoute(route) => {
                self.route = route;
                Task::none()
            }
            Message::TracksMessage(tracks_message) => self
                .tracks
                .update(tracks_message)
                .map(Message::TracksMessage),
        }
    }
}
