//! Commonly reused styles / components

use iced::{
    Border, Font,
    Length::{Fill, Shrink},
    Padding, font,
    widget::{self, Container, button, row, text},
};

use crate::ui::components::icons;

/// The header text of a column used in tables
pub fn column_header(content: &str) -> widget::Text<'_> {
    widget::text(content.to_uppercase()).ellipsis(widget::text::Ellipsis::End)
}

/// Slightly transparent text giving the notion of being muted
pub fn muted_text(theme: &iced::Theme) -> widget::text::Style {
    widget::text::Style {
        color: Some(theme.palette().background.base.text.scale_alpha(0.6)),
    }
}

/// A round button usually placed as a header to signify
/// play everything on this page
pub fn header_play_button<'a, T: 'a>() -> widget::Button<'a, T> {
    widget::button(widget::container(icons::play_filled().size(20)).center(Fill))
        .height(42)
        .width(42)
        .style(|theme, status| {
            let mut style = button::primary(theme, status);
            style.border = Border::default().rounded(i32::MAX);
            style
        })
}

/// A round button usually placed as a header to signify
/// play everything on this page
pub fn header_text<'a>(text: impl text::IntoFragment<'a>) -> widget::Text<'a> {
    widget::text(text).size(36).font(Font {
        weight: font::Weight::Bold,
        ..Font::DEFAULT
    })
}

/// The default header for the routes.
/// It consists of a large play button ([`header_play_button()`])
/// and a header text ([`header_text()`])
pub fn default_header<'a, Msg: Clone + 'a>(
    text: impl text::IntoFragment<'a>,
    on_press: Msg,
) -> Container<'a, Msg> {
    let tracks_text = header_text(text);
    let play_button = header_play_button().on_press(on_press);

    widget::container(row![
        play_button,
        widget::space().width(12),
        widget::container(tracks_text).center_y(Fill),
    ])
    .height(Shrink)
    .padding(Padding::new(12.0))
}

/// Provides an em dash String to signify missing data
pub fn em_dash() -> String {
    const EM_DASH: &str = "—";
    EM_DASH.to_string()
}
