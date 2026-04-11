use iced::{
    Color, Element,
    Length::Fill,
    Task,
    widget::{self, container::Style},
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
                .style(|_| Style::default().background(Color::from_rgb(0.9, 0.9, 0.9)))
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
        .style(|_| Style::default().background(Color::from_rgb(0.9, 0.9, 0.9)))
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
