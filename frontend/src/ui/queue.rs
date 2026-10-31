use crate::ui::app::AppState;
use iced::{Element, Task};

use crate::ui::components::track_table::TrackTableMsg;

/// Queue has no state because all information about the queue is held by the
/// global player
#[derive(Default)]
pub struct Queue {}

#[derive(Debug, Clone)]
pub enum QueueMsg {
    /// Drives the used track Table
    TableDriver(TrackTableMsg),
}

impl Queue {
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, QueueMsg> {
        app_state
            .player
            .queue()
            .view(app_state)
            .map(QueueMsg::TableDriver)
    }

    pub fn update(&mut self, message: QueueMsg, app_state: &mut AppState) -> Task<QueueMsg> {
        match message {
            QueueMsg::TableDriver(message) => {
                // FIXME: as anyone with functioning eyes can see, we copy the entire queue just
                // to mutate it and overwrite it, which is very stupid
                //
                // I did this because if I try mutating the queue in-place then i have to borrow
                // `&mut app_state` twice, once for mutating the queue and once to pass into the
                // [`Table::update`]
                // ```
                // error[E0499]: cannot borrow `app_state.player.queue` as mutable more than once at a time
                //   --> frontend/src/ui/queue.rs:42:17
                //    |
                // 42 | /                 app_state
                // 43 | |                     .player
                // 44 | |                     .queue
                // 45 | |                     .update(message, app_state)
                //    | |______________________------__________---------^ second mutable borrow occurs here
                //    |                        |               |
                //    |                        |               first mutable borrow occurs here
                //    |                        first borrow later used by call
                // ```
                // This will have to be fixed at some point, either by:
                //  1. Creating a RefTable which can display the state of the player queue
                //     without owning the data
                //  2. Moving the queue table out of the player (this again requiresadditional
                //     wiring/messaging)
                //  3. doing some fuckery with RefCell or the like
                //  4. ???
                //
                // I have not yet found a satisfactory solution which is why this piece of code
                // will stay here until I do
                let mut queue = app_state.player.queue.clone();
                let task = queue.update(message, app_state).map(QueueMsg::TableDriver);
                app_state.player.queue = queue;
                task
            }
        }
    }
}
