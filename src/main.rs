use iced_fonts::LUCIDE_FONT_BYTES;

use crate::ui::app::App;

pub mod backend;
pub mod ui;

fn main() {
    iced::application(App::default, App::update, App::view)
        .font(LUCIDE_FONT_BYTES)
        .run()
        .unwrap();
}
