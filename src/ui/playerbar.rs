use iced::{
    Border, Element, Length,
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{self, container, row, text::Style},
};
use iced_fonts::lucide;

use crate::{
    backend::mpv_events::{MpvEvent, MpvValue},
    ui::{
        components::utils::format_duration,
        player::{self, RepeatMode},
    },
};

#[derive(Default)]
pub struct PlayerBar {
    /// If the queue is shuffled or not
    shuffle: bool,
    /// The repeat mode of the player
    repeat_mode: RepeatMode,
    /// For some god-forsaken reason mpv stores the
    /// repeat state as strings ("no" and "inf")
    loop_file: String,
    loop_playlist: String,
    /// The state of the player
    paused: bool,
    /// The total track duration in seconds
    duration: f64,
    /// The playback progress in seconds
    progress: f64,
    /// We need to block progress updates from the player while seeking
    seeking: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    /// The user requests to pause/unpause the player
    Pause(bool),
    /// The user requests the next track
    Next,
    /// The user requests the previous track
    Previous,
    /// The user stops the player
    Stop,
    /// The user (de)activates shuffle
    Shuffle(bool),
    /// The user sets the repeat mode
    Repeat(RepeatMode),
    /// The user requests random tracks
    PlayRandom,
    /// The user starts seeking by moving the slider
    Seek(f64),
    /// The user stops seeking be letting go the slider
    FinishSeek(f64),
    // The player state has updated
    Event(MpvEvent),
}

impl PlayerBar {
    /// Mutates the model whenever a message is dispatched
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Event(MpvEvent::PropertyChange { name, value, .. }) => {
                match (name.as_str(), value) {
                    (player::property::SHUFFLE, MpvValue::Bool(s)) => self.shuffle = s,
                    (player::property::LOOP_FILE, MpvValue::String(val)) => {
                        self.loop_file = val;
                        self.repeat_mode = self.determine_repeat_mode()
                    }
                    (player::property::LOOP_PLAYLIST, MpvValue::String(val)) => {
                        self.loop_playlist = val;
                        self.repeat_mode = self.determine_repeat_mode()
                    }
                    (player::property::PAUSE, MpvValue::Bool(p)) => self.paused = p,
                    (player::property::TIME_POS, MpvValue::F64(p)) if !self.seeking => {
                        self.progress = p
                    }
                    (player::property::DURATION, MpvValue::F64(d)) => self.duration = d,
                    _ => {}
                }
            }
            Message::Seek(p) => {
                self.seeking = true;
                self.progress = p
            }
            Message::FinishSeek(_) => {
                // Seeking is finished so we can reallow player updates
                self.seeking = false;
            }
            _ => (),
        }
    }

    /// Renders the model after each update
    pub fn view(&self) -> Element<'_, Message> {
        const BUTTON_SIZE: u32 = 20;

        let player_button = |content| {
            widget::button(content)
                .style(|_, _| widget::button::Style {
                    border: Border {
                        radius: Radius::new(2),
                        ..Default::default()
                    },
                    ..Default::default()
                })
        };

        widget::container(widget::column([
            // The player buttons
            widget::container(widget::row([
                player_button(lucide::square().size(BUTTON_SIZE))
                    .on_press(Message::Stop)
                    .into(),
                player_button(
                    lucide::shuffle()
                        .style(|t: &iced::Theme| Style {
                            color: self.shuffle.then_some(t.palette().primary.strong.color),
                        })
                        .size(BUTTON_SIZE),
                )
                .on_press(Message::Shuffle(!self.shuffle))
                .into(),
                player_button(lucide::skip_back().size(BUTTON_SIZE))
                    .on_press(Message::Previous)
                    .into(),
                player_button(if self.paused {
                    lucide::play().size(BUTTON_SIZE)
                } else {
                    lucide::pause().size(BUTTON_SIZE)
                })
                .on_press(Message::Pause(!self.paused))
                .into(),
                player_button(lucide::skip_forward().size(BUTTON_SIZE))
                    .on_press(Message::Next)
                    .into(),
                player_button(
                    match self.repeat_mode {
                        RepeatMode::None => lucide::repeat(),
                        RepeatMode::Song => lucide::repeat_one().style(|t: &iced::Theme| Style {
                            color: Some(t.palette().primary.strong.color),
                        }),
                        RepeatMode::Queue => lucide::repeat().style(|t: &iced::Theme| Style {
                            color: Some(t.palette().primary.strong.color),
                        }),
                    }
                    .size(BUTTON_SIZE),
                )
                .on_press(Message::Repeat(match self.repeat_mode {
                    // Cycle repeat mode
                    RepeatMode::None => RepeatMode::Queue,
                    RepeatMode::Queue => RepeatMode::Song,
                    RepeatMode::Song => RepeatMode::None,
                }))
                .into(),
                player_button(lucide::dice_five().size(BUTTON_SIZE))
                    .on_press(Message::PlayRandom)
                    .into(),
            ]))
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .into(),
            // The progress bar
            container(
                container(
                    row![
                        // Current pos label
                        iced::widget::text(format_duration(self.progress)),
                        // Seeking slider
                        iced::widget::slider(0.0..=self.duration, self.progress, Message::Seek)
                            .on_release(Message::FinishSeek(self.progress)),
                        // Total duration
                        iced::widget::text(format_duration(self.duration)),
                    ]
                    .spacing(8),
                )
                .max_width(800),
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .into(),
        ]))
        .padding(10)
        .width(Length::Fill)
        .height(86)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(container::rounded_box)
        .into()
    }

    fn determine_repeat_mode(&self) -> RepeatMode {
        match (self.loop_file.as_str(), self.loop_playlist.as_str()) {
            ("inf", _) => RepeatMode::Song,
            ("no", "inf") => RepeatMode::Queue,
            ("no", "no") | (_, _) => RepeatMode::None,
        }
    }
}
