use iced::{
    Color, Element,
    Length::Fill,
    widget::{container, container::Style},
};

use crate::ui::routes::tracks::{Tracks, TracksMessage};

#[derive(Default)]
pub struct Router {
    route: Route,
    tracks: Tracks,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Copy)]
pub enum Route {
    #[default]
    Home,
    Favorites,
    Albums,
    Tracks,
    AlbumArtists,
    Artists,
    Genres,
}

pub enum RouterMessage {
    ChangeRoute(Route),
    TracksMessage(TracksMessage),
}

impl Router {
    pub fn view(&self) -> Element<'_, RouterMessage> {
        container(match self.route {
            Route::Home => container("text")
                .style(|_| Style::default().background(Color::from_rgb(0.9, 0.9, 0.9)))
                .width(Fill)
                .height(Fill)
                .into(),
            Route::Favorites => todo!(),
            Route::Albums => todo!(),
            Route::Tracks => self.tracks.view().map(RouterMessage::TracksMessage),
            Route::AlbumArtists => todo!(),
            Route::Artists => todo!(),
            Route::Genres => todo!(),
        })
        .style(|_| Style::default().background(Color::from_rgb(0.9, 0.9, 0.9)))
        .into()
    }

    pub fn update(&mut self, message: RouterMessage) {
        match message {
            RouterMessage::ChangeRoute(route) => self.route = route,
            RouterMessage::TracksMessage(tracks_message) => self.tracks.update(tracks_message),
        }
    }
}
