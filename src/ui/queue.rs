use iced::Element;
use iced::widget::column;

#[derive(Default)]
pub struct Queue {}

#[derive(Debug, Clone)]
pub enum QueueMessage {}

impl Queue {
    pub fn view(&self) -> Element<'_, QueueMessage> {
        column!["Queue",].into()
    }

    pub fn update(&mut self, message: QueueMessage) {}
}
