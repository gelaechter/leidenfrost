use iced::Theme;

use crate::backend::api::endpoint_api::Endpoint;

pub struct Settings {
    pub selected_apis: Vec<Endpoint>,
    pub theme: Theme,
    pub debug_overlay: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_apis: Default::default(),
            theme: Theme::Light,
            debug_overlay: Default::default(),
        }
    }
}
