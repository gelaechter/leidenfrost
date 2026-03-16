#[macro_export]
macro_rules! player_button {
    ($content:expr) => {
        iced::widget::button($content)
            .padding(8)
            .style(|_, _| iced::widget::button::Style {
                border: iced::Border {
                    radius: iced::border::Radius::new(2),
                    ..Default::default()
                },
                ..Default::default()
            })
    };
}