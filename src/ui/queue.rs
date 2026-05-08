use iced::Element;
use iced::widget::column;

use crate::ui::components::table::Table;

#[derive(Default)]
pub struct Queue {
    table: Table<RowData, Message>
}

struct RowData {
    
}

#[derive(Debug, Clone)]
pub enum Message {}

impl Queue {
    pub fn view(&self) -> Element<'_, Message> {
        column!["Queue",].into()
    }

    pub fn update(&mut self, message: Message) {}
}
