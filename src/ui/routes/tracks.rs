use iced::{
    Alignment::Center,
    Element,
    widget::{self, row, table, text},
};

use crate::{backend::data::Track, ui::util::format_duration};
use iced::widget::column;

/// The tracks route
#[derive(Default)]
pub struct Tracks {
    tracks: Vec<Track>,
}

pub enum TracksMessage {}

impl Tracks {
    pub fn view(&self) -> Element<'_, TracksMessage> {
        column!["Tracks", self.track_table()].into()
    }

    pub fn update(&mut self, message: TracksMessage) {
        match message {}
    }

    pub fn track_table(&self) -> Element<'_, TracksMessage> {
        let columns = [
            // Name column
            table::column("Title", |track: &Track| {
                // Combined image and title
                row![
                    // widget::image()
                ]
            }),
            // Duration column
            table::column("Duration", |track: &Track| {
                text(format_duration(track.duration as f64))
            })
            .align_x(Center)
            .align_y(Center),
            // Album column
            table::column("Album", |track: &Track| {
                text(track.album.clone().unwrap_or_default())
            }),
            // Genre
            table::column("Genre", |track: &Track| {
                text(
                    // Join genre names by comma
                    track
                        .genres
                        .iter()
                        .map(|g| g.name.clone())
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            }),
        ];

        table(columns, &self.tracks)
            .padding_x(25)
            .padding_y(5)
            .separator_x(1)
            .separator_y(1)
            .into()
    }
}
