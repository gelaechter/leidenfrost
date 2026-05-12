use iced::{
    Element,
    Length::{Fill, Shrink},
    Padding, Task, Theme,
    widget::{self, row},
    window::{self, Id},
};
use iced_fonts::lucide;

use crate::{backend::api::endpoint_api::Endpoint, ui::components::style::header_text};

pub struct Settings {
    pub selected_apis: Vec<Endpoint>,
    pub zoom_factor: f32,
    pub debug_overlay: bool,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_apis: Default::default(),
            theme: Theme::Light,
            debug_overlay: Default::default(),
            zoom_factor: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    OpenSettings(Id),
    ScaleChanged(String),
}

impl Settings {
    pub fn open(&self) -> Task<Message> {
        let (_, open) = window::open(window::Settings::default());
        open.map(Message::OpenSettings)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let settings_icon = lucide::settings().height(42);

        let header = widget::container(row![
            settings_icon,
            widget::space().width(12),
            widget::container(header_text("Settings")).center_y(Fill),
        ])
        .height(Shrink)
        .padding(Padding::new(12.0));

        header.into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::OpenSettings(id) =>  {},
            Message::ScaleChanged(_) => {},
        }
    }
}
