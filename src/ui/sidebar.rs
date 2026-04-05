use iced::{
    Background, Border, Color, Element,
    Length::Fill,
    widget::{
        self,
        button::{self},
        container,
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
pub enum SidebarMessage {
    UpdateRoute(Route),
}

impl Sidebar {
    /// The tab select of the sidebar
    pub fn tab_button<'a>(
        &'a self,
        icon: impl Into<Element<'a, SidebarMessage>>,
        text: impl Into<Element<'a, SidebarMessage>>,
        route: Route,
    ) -> Element<'a, SidebarMessage> {
        widget::button(
            row![
                icon.into(), // Here we manually erase the types
                text.into()
            ]
            .spacing(8),
        )
        .style(move |theme, status| {
            let palette = theme.extended_palette();
            button::Style {
                // Round the button
                border: Border::default().rounded(2),
                // Make gray if hovered over
                background: if matches!(status, button::Status::Hovered) {
                    Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95)))
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
        .on_press(SidebarMessage::UpdateRoute(route))
        .into()
    }

    pub fn view(&self) -> Element<'_, SidebarMessage> {
        container(
            column![
                text("My Library").font(Font {
                    weight: font::Weight::Medium,
                    ..Font::DEFAULT
                }),
                self.tab_button(lucide::house(), "Home", Route::Home),
                self.tab_button(lucide::heart(), "Favorites", Route::Favorites),
                self.tab_button(lucide::disc_two(), "Albums", Route::Albums),
                self.tab_button(lucide::music(), "Tracks", Route::Tracks),
                self.tab_button(lucide::user(), "Artists", Route::Artists),
                self.tab_button(lucide::tag(), "Genres", Route::Genres),
            ]
            .spacing(8),
        )
        .padding(8)
        .into()
    }

    pub fn update(&mut self, message: SidebarMessage) {
        match message {
            SidebarMessage::UpdateRoute(route) => self.route = route,
        }
    }
}
