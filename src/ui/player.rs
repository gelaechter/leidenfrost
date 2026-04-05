use iced::{
    Border, Element, Length, Subscription,
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{self, column, container, row},
};
use iced_fonts::lucide;
use libmpv2::Mpv;

use crate::{backend::db::models::track, ui::util::format_duration};

#[derive(Clone, Debug)]
pub enum PlayerMessage {
    /// When the stop button has been pressed
    StopPlayer,
    /// Adds some songs to the queue
    AddToQueue {
        songs: Vec<track::Model>,
    },
    /// Removes some songs from the queue
    RemoveFromQueue {
        songs: Vec<track::Model>,
    },
    /// Changes the repeat mode of the player
    ChangeRepeatMode {
        mode: RepeatMode,
    },
    /// Plays the next song
    Next,
    /// Plays the previous song
    Previous,
    /// Updates the song progress
    UpdateProgress(f64),
    PlayRandom,
    EventOccurred,
}

#[derive(Default, Clone, Debug)]
pub enum RepeatMode {
    #[default]
    /// Do not repeat at all
    None,
    /// Repeat the current song
    Song,
    /// Repeat the queue
    Queue,
}

#[derive(Default)]
pub enum PlayerState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    /// the Mpv instance
    lib_mpv: Mpv,
    /// The queue in an unshuffled state
    /// This way the queue can be unshuffled again
    unshuffled_queue: Vec<track::Model>,
    /// Current queue
    queue: Vec<track::Model>,
    /// Current index in the queue
    current_track: usize,
    /// If the queue is shuffled or not
    shuffle: bool,
    /// The repeat mode of the player
    repeat_mode: RepeatMode,
    /// The state of the player
    state: PlayerState,
    /// The playback progress in milliseconds
    progress: f64,
}

impl Default for Player {
    fn default() -> Self {
        let lib_mpv = Mpv::with_initializer(|init| {
            init.set_option("vid", "no")?;
            Ok(())
        })
        .unwrap();

        Self {
            lib_mpv,
            unshuffled_queue: Default::default(),
            queue: Default::default(),
            current_track: Default::default(),
            shuffle: Default::default(),
            repeat_mode: Default::default(),
            state: Default::default(),
            progress: Default::default(),
        }
    }
}

impl Player {
    /// Mutates the model whenever a message is dispatched
    pub fn update(&mut self, message: PlayerMessage) {
        match message {
            PlayerMessage::AddToQueue { songs } => todo!(),
            PlayerMessage::RemoveFromQueue { songs } => todo!(),
            PlayerMessage::ChangeRepeatMode { mode } => todo!(),
            PlayerMessage::Next => todo!(),
            PlayerMessage::Previous => todo!(),
            PlayerMessage::UpdateProgress(progress) => self.progress = progress,
            PlayerMessage::StopPlayer => {
                self.lib_mpv.command("stop", &[]).unwrap();
            }
            PlayerMessage::PlayRandom => {
                self.lib_mpv
                    .command(
                        "loadfile",
                        &["https://www.youtube.com/watch?v=xe3Wkzc0O3k&list=RDxe3Wkzc0O3k"],
                    )
                    .unwrap();
            }
            PlayerMessage::EventOccurred => todo!(),
        }
    }

    /// Renders the model after each update
    pub fn view(&self) -> Element<'_, PlayerMessage> {
        const BUTTON_SIZE: u32 = 20;

        let time_pos: f64 = self.lib_mpv.get_property("time-pos").unwrap_or_default();
        let duration: f64 = self.lib_mpv.get_property("duration").unwrap_or_default();

        let player_button = |content| {
            widget::button(content)
                .padding(8)
                .style(|_, _| widget::button::Style {
                    border: Border {
                        radius: Radius::new(2),
                        ..Default::default()
                    },
                    ..Default::default()
                })
        };

        container(column![
            // The player buttons
            container(row![
                player_button(lucide::square().size(BUTTON_SIZE))
                    .on_press(PlayerMessage::StopPlayer),
                player_button(lucide::shuffle().size(BUTTON_SIZE)),
                player_button(lucide::skip_back().size(BUTTON_SIZE)),
                player_button(lucide::play().size(BUTTON_SIZE)),
                player_button(lucide::skip_forward().size(BUTTON_SIZE)),
                player_button(lucide::repeat().size(BUTTON_SIZE)),
                player_button(lucide::dice_five().size(BUTTON_SIZE))
                    .on_press(PlayerMessage::PlayRandom)
            ])
            .width(Length::Fill)
            .align_x(Horizontal::Center),
            // The progress bar
            container(
                container(
                    row![
                        // Current pos label
                        iced::widget::text(format_duration(time_pos)),
                        // Seeking slider
                        iced::widget::slider(
                            0.0..=duration,
                            self.progress,
                            PlayerMessage::UpdateProgress
                        ),
                        // Total duration
                        iced::widget::text(format_duration(duration)),
                    ]
                    .spacing(8)
                )
                .max_width(800)
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        ])
        .padding(10)
        .width(Length::Fill)
        .height(86)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(container::rounded_box)
        .into()
    }

    fn subscription(&self) -> Subscription<PlayerMessage> {
        todo!()
    }
}
