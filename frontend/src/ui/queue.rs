use crate::ui::{Receiver, app::AppState, components::track_table::TrackRow};
use backend::data_view::TrackView;
use iced::{Element, Task};
use macros::Receiver;

use crate::ui::components::track_table::{
    TrackTable, TrackTableMsg, combined_title_column, duration_column, index_column,
};

#[derive(Receiver)]
#[message(QueueMsg)]
pub struct Queue {
    current_index: usize,
    /// The table displaying the current queue
    table: TrackTable,
}

impl Default for Queue {
    fn default() -> Self {
        let table = TrackTable::new()
            .add_column(index_column())
            .add_column(combined_title_column())
            .add_column(duration_column());

        Self {
            table,
            current_index: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum QueueMsg {
    /// Clears the queue and plays just that track
    Play(TrackView),
    PlayAll(Vec<TrackView>),
    /// Changes playback to a certain index in the queue
    PlayIndex(usize),
    /// Adds some tracks to the queue
    Append(TrackView),
    AppendAll(Vec<TrackView>),
    /// Remove a track from the queue (zero indexed)
    QueueRemove(usize),
    /// Move a track in the queue
    ///
    /// Moves a target track in the queue before the position of another one
    QueueMove {
        target: usize,
        position: usize,
    },
    /// Drives the used track Table
    TableDriver(TrackTableMsg),
    Next,
    Previous,
}

impl Queue {
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, QueueMsg> {
        self.table.view(app_state).map(QueueMsg::TableDriver)
    }

    pub fn update(&mut self, message: QueueMsg, app_state: &mut AppState) -> Task<QueueMsg> {
        // The table driver needs to return a task
        if let QueueMsg::TableDriver(message) = message {
            return self.table.update(message, app_state).map(QueueMsg::TableDriver);
        }

        // Everything else is just side effects
        match message {
            QueueMsg::Play(track_view) => {
                self.table.clear();
                self.current_index = 0;
                let _ = self.update(QueueMsg::Append(track_view), app_state);
                Task::none()
            }
            QueueMsg::PlayAll(track_views) => {
                self.table.clear();
                self.current_index = 0;
                let _ = self.update(QueueMsg::AppendAll(track_views), app_state);
                Task::none()
            }
            QueueMsg::Append(track_view) => {
                self.table.insert(track_view.into());
                Task::none()
            }
            QueueMsg::AppendAll(track_views) => {
                let rows: Vec<TrackRow> = track_views.into_iter().map(TrackRow::from).collect();
                self.table.extend(rows);
                Task::none()
            }
            QueueMsg::QueueRemove(index) => {
                self.table.remove_index(index);
                Task::none()
            }
            QueueMsg::QueueMove { target, position } => {
                if let Some(data) = self.table.remove_index(target) {
                    self.table.insert_at(position, data);
                }
                Task::none()
            }
            QueueMsg::PlayIndex(index) => {
                self.current_index = index;
                Task::none()
            }
            QueueMsg::Next => {
                self.current_index += 1;
                Task::none()
            }
            QueueMsg::Previous => {
                self.current_index -= 1;
                Task::none()
            }
            QueueMsg::TableDriver(_) => {
                panic!("Impossible branch (see above)")
            }
        }
    }
}
