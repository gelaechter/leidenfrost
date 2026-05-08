use iced::{Element, Task};

use crate::{
    backend::data_view::TrackView,
    ui::components::{
        table,
        track_table::{
            TrackCellMsg, TrackTable, combined_title_column, duration_column, index_column,
        },
    },
};

pub struct Queue {
    table: TrackTable,
}

impl Default for Queue {
    fn default() -> Self {
        let table = TrackTable::default()
            .add_column(index_column())
            .add_column(combined_title_column())
            .add_column(duration_column());

        Self { table }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Clears the queue and plays just that track
    Play(TrackView),
    PlayAll(Vec<TrackView>),
    /// Adds some tracks to the queue
    Append(TrackView),
    AppendAll(Vec<TrackView>),
    /// Remove a track from the queue
    QueueRemove(usize),
    /// Move a track in the queue
    ///
    /// Moves a target track in the queue before the position of another one
    QueueMove {
        target: usize,
        position: usize,
    },
    TableDriver(table::Message<TrackCellMsg>),
}

impl Queue {
    pub fn view(&self) -> Element<'_, Message> {
        self.table.view().map(Message::TableDriver)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        // The table driver needs to return a task
        if let Message::TableDriver(message) = message {
            return self.table.update(message).map(Message::TableDriver);
        };

        // Everything else is just side effects
        match message {
            Message::Play(track_view) => {
                self.table.clear();
                let _ = self.update(Message::Append(track_view));
            }
            Message::PlayAll(track_views) => {
                self.table.clear();
                let _ = self.update(Message::AppendAll(track_views));
            }
            Message::Append(track_view) => {
                self.table.push(track_view.into());
            }
            Message::AppendAll(track_views) => {
                let rows = track_views.into_iter().map(Into::into).collect();
                self.table.extend(rows);
            }
            Message::QueueRemove(index) => {
                self.table.rows_mut().remove(index);
            }
            Message::QueueMove { target, position } => {
                let rows = self.table.rows_mut();
                let row = rows.remove(target);
                rows.insert(position, row);
            }
            _ => {}
        }

        Task::none()
    }
}
