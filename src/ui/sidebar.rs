use iced::{
    Background, Border, Element,
    Length::Fill,
    widget::{
        self,
        button::{self},
        container, space,
    },
};
use iced::{
    Font, font,
    widget::{column, row, text},
};
use iced_fonts::lucide;

use crate::ui::router::Route;

#[derive(Default)]
pub struct Sidebar {
    route: Route,
}

#[derive(Clone, Debug)]
pub enum Message {
    UpdateRoute(Route),
}

impl Sidebar {
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

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UpdateRoute(route) => self.route = route,
        }
    }
}
