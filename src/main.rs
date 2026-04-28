use iced::{
    Element,
    Length::FillPortion,
    widget::{self, container, row},
};
use iced_fonts::LUCIDE_FONT_BYTES;

use crate::ui::app::App;

pub mod backend;
pub mod ui;

fn main() {
    // TOKIO CONSOLE
    // console_subscriber::init();

    // Component tests (of these only one should be enabled)
    // table_demo();
    // layout_test();

    iced::application(App::default, App::update, App::view)
        .font(LUCIDE_FONT_BYTES)
        .run()
        .unwrap();
}

pub fn layout_test() {
    fn view(_state: &()) -> Element<'_, ()> {
        let items = [
            widget::container("Was")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("für")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("ein")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("hohles")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("produkt")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("das")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("doch")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("ist")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("aber")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("dennoch")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("testen")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
            widget::container("wir")
                .width(FillPortion(1))
                .style(container::bordered_box)
                .into(),
        ];
        widget::container(row(items))
            .style(container::bordered_box)
            .into()
    }

    fn update(_state: &mut (), _message: ()) {}

    iced::application(|| (), update, view)
        .font(LUCIDE_FONT_BYTES)
        .run()
        .unwrap();
}
