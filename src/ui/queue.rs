use iced::Element;
use iced::widget::column;

#[derive(Default)]
pub struct Queue {}

#[derive(Debug, Clone)]
pub enum Message {}

impl Queue {
    pub fn view(&self) -> Element<'_, Message> {
        column!["Queue",].into()
    }

    pub fn update(&mut self, message: Message) {}
}
