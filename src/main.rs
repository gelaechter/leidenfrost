use iced::{Element, Length::FillPortion, widget::{self, container, row}};
use iced_fonts::LUCIDE_FONT_BYTES;
use url::Url;

use crate::{
    backend::{
        api::{
            endpoint_api::{ApiContract, UserPasswordAuth},
            jellyfin::api::JellyfinApi,
        },
        data_view::TrackView,
    },
    ui::{
        app::App,
        components::table::{Column, Table},
    },
};

pub mod backend;
pub mod ui;

fn main() {
    // TOKIO CONSOLE
    // console_subscriber::init();

    // Component tests (of these only one should be enabled)
    table_demo();
    // layout_test();

    iced::application(App::default, App::update, App::view)
        .font(LUCIDE_FONT_BYTES)
        .run()
        .unwrap();
}

pub fn layout_test() {
    fn view(_state: &()) -> Element<'_, ()> {
        let items = [
            widget::container("Was").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("für").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("ein").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("hohles").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("produkt").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("das").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("doch").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("ist").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("aber").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("dennoch").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("testen").width(FillPortion(1)).style(container::bordered_box).into(),
            widget::container("wir").width(FillPortion(1)).style(container::bordered_box).into(),
        ];
        widget::container(row(items)).style(container::bordered_box).into()
    }

    fn update(_state: &mut (), _message: ()) {}

    iced::application(
        || (),
        update,
        view,
    )
    .font(LUCIDE_FONT_BYTES)
    .run()
    .unwrap();
}

pub fn table_demo() {
    let table = || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let jf: JellyfinApi = JellyfinApi::auth_user_password(
                Url::parse("http://***REMOVED***").unwrap(),
                "***REMOVED***".to_string(),
                "***REMOVED***".to_string(),
            )
            .await;

            jf.get_tracks().await.unwrap()
        });

        let name_col = Column::new(
            || widget::text("Name").into(),
            |view: &TrackView| widget::text(view.track.title.clone().unwrap_or_default()).into(),
        );

        let album_col = Column::new(
            || widget::text("Album").into(),
            |view: &TrackView| widget::text(view.album_name.clone().unwrap_or_default()).into(),
        );

        Table::<TrackView, ()>::default()
            .column(name_col)
            .column(album_col)
            .rows(result)
    };

    iced::application(table, Table::update, Table::view)
        .font(LUCIDE_FONT_BYTES)
        .run()
        .unwrap();
}
