#![warn(clippy::pedantic)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]
#![feature(vec_into_chunks)]

use crate::ui::app::App;
use env_logger::Env;
use iced::Font;

pub mod backend;
pub mod ui;

pub const LEIDENFROST_ICONS: Font = Font::new("leidenfrost");

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    // TOKIO CONSOLE
    #[cfg(feature = "debug")]
    console_subscriber::init();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(include_bytes!("ui/components/icons/leidenfrost.ttf").as_slice())
        .default_font("leidenfrost".into())
        .run()
        .unwrap();
}
