use iced_fonts::LUCIDE_FONT_BYTES;

use crate::ui::app::App;

pub mod backend;
pub mod ui;

fn main() {
    // TOKIO CONSOLE
    // console_subscriber::init();

    // Component tests (of these only one should be enabled)
    // ui::components::image::test_images_concurrency();

    

    iced::application(App::default, App::update, App::view)
        .executor::<tokio::runtime::Runtime>()
        .font(LUCIDE_FONT_BYTES)
        .run()
        .unwrap();
}
