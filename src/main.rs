#![warn(clippy::pedantic)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]

use iced_fonts::LUCIDE_FONT_BYTES;

use crate::ui::app::App;

pub mod backend;
pub mod ui;

fn main() {
    env_logger::init();

    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .font(LUCIDE_FONT_BYTES)
        .run()
        .unwrap();
}
