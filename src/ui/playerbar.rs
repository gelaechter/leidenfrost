use iced::{
    Border, Element,
    Length::{self, Fill, FillPortion},
    Theme,
    alignment::Horizontal,
    border::Radius,
    widget::{self, Container, column, container::Style, row, text},
};
use iced_fonts::lucide;

use crate::backend::{
    data_view::TrackView,
    mpv_events::{MpvEvent, MpvValue},
};
use crate::ui::{
    components::utils::format_duration,
    mpv::{self, RepeatMode},
};

#[derive(Default)]
pub struct PlayerBar {
    /// The currently playing track
    currently_playing: Option<TrackView>,
    /// If the queue is shuffled or not
    shuffle: bool,
    /// The repeat mode of the player
    repeat_mode: RepeatMode,
    /// For some god-forsaken reason mpv stores the
    /// repeat state as strings ("no" and "inf")
    loop_file: String,
    loop_playlist: String,
    /// The internal volume of the player (not the system)
    volume: u32,
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
    Volume(u32),
}

impl PlayerBar {
    /// Mutates the model whenever a message is dispatched
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Event(MpvEvent::PropertyChange { name, value, .. }) => {
                match (name.as_str(), value) {
                    (mpv::property::SHUFFLE, MpvValue::Bool(s)) => self.shuffle = s,
                    (mpv::property::LOOP_FILE, MpvValue::String(val)) => {
                        self.loop_file = val;
                        self.repeat_mode = self.determine_repeat_mode();
                    }
                    (mpv::property::LOOP_PLAYLIST, MpvValue::String(val)) => {
                        self.loop_playlist = val;
                        self.repeat_mode = self.determine_repeat_mode();
                    }
                    (mpv::property::PAUSE, MpvValue::Bool(p)) => self.paused = p,
                    (mpv::property::TIME_POS, MpvValue::F64(p)) if !self.seeking => {
                        self.progress = p;
                    }
                    (mpv::property::DURATION, MpvValue::F64(d)) => self.duration = d,
                    _ => {}
                }
            }
            Message::Volume(vol) => {
                self.volume = vol;
            }
            Message::Seek(p) => {
                self.seeking = true;
                self.progress = p;
            }
            Message::FinishSeek(_) => {
                // Seeking is finished so we can reallow player updates
                self.seeking = false;
            }
            // Everything else can be passed upwards
            _ => (),
        }
    }

    /// Renders the model after each update
    pub fn view(&self) -> Element<'_, Message> {
        widget::container(row![
            // Left console
            self.left_console()
                .align_left(FillPortion(1))
                .center_y(Fill),
            // Center console
            widget::container(column![
                // Buttons
                self.buttons(),
                // The progress bar
                self.progress_bar(),
            ])
            .center_x(FillPortion(5))
            .center_y(Fill),
            // Right console
            self.right_console()
                .align_right(FillPortion(1))
                .center_y(Fill),
        ])
        .padding(10)
        .center_x(Fill)
        .center_y(86)
        .style(|theme: &Theme| Style::default().background(theme.palette().background.weak.color))
        .into()
    }

    fn progress_bar(&self) -> Container<'_, Message> {
        // Big container outside
        widget::container(
            // Little container inside
            widget::container(
                row![
                    // Current pos label
                    iced::widget::text(format_duration(self.progress)),
                    // Seeking slider
                    iced::widget::slider(0.0..=self.duration, self.progress, Message::Seek)
                        .on_release(Message::FinishSeek(self.progress)),
                    // Total duration
                    iced::widget::text(format_duration(self.duration)),
                ]
                .spacing(8)
                .width(Fill),
            )
            .max_width(800),
        )
        .center(Fill)
    }

    fn buttons(&self) -> Container<'_, Message> {
        const BUTTON_SIZE: u32 = 20;

        let stop_button = player_button(lucide::square().size(BUTTON_SIZE));

        let shuffle_button = player_button(
            lucide::shuffle()
                .style(|t: &Theme| text::Style {
                    color: self.shuffle.then_some(t.palette().primary.strong.color),
                })
                .size(BUTTON_SIZE),
        )
        .on_press(Message::Shuffle(!self.shuffle));

        let skip_back_button =
            player_button(lucide::skip_back().size(BUTTON_SIZE)).on_press(Message::Previous);

        let pause_button = player_button(if self.paused {
            lucide::play().size(BUTTON_SIZE)
        } else {
            lucide::pause().size(BUTTON_SIZE)
        })
        .on_press(Message::Pause(!self.paused));

        let skip_forward_button =
            player_button(lucide::skip_forward().size(BUTTON_SIZE)).on_press(Message::Next);

        let repeat_button = player_button(
            match self.repeat_mode {
                RepeatMode::None => lucide::repeat(),
                RepeatMode::Song => lucide::repeat_one().style(|t: &Theme| text::Style {
                    color: Some(t.palette().primary.strong.color),
                }),
                RepeatMode::Queue => lucide::repeat().style(|t: &Theme| text::Style {
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
        }));

        let play_random_button =
            player_button(lucide::dice_five().size(BUTTON_SIZE)).on_press(Message::PlayRandom);

        // Button container
        widget::container(row![
            stop_button,
            shuffle_button,
            skip_back_button,
            pause_button,
            skip_forward_button,
            repeat_button,
            play_random_button,
        ])
        .width(Length::Fill)
        .align_x(Horizontal::Center)
    }

    fn right_console(&self) -> Container<'_, Message> {
        let mute_button = match self.volume {
            0 => lucide::volume(),
            1..50 => lucide::volume_one(),
            50..=100 => lucide::volume_two(),
            _ => lucide::volume_off(),
        };

        widget::container(row![
            // Mute icon
            mute_button,
            // Volume slider
            iced::widget::slider(0..=100_u32, self.volume, Message::Volume).width(100),
        ])
    }

    fn left_console(&self) -> Container<'_, Message> {
        widget::container(column![widget::text(
            self.currently_playing
                .as_ref()
                .and_then(|t| t.track.title.clone())
                .unwrap_or_default()
        ),])
    }

    fn determine_repeat_mode(&self) -> RepeatMode {
        match (self.loop_file.as_str(), self.loop_playlist.as_str()) {
            ("inf", _) => RepeatMode::Song,
            ("no", "inf") => RepeatMode::Queue,
            ("no", "no") | (_, _) => RepeatMode::None,
        }
    }
}

fn player_button<'a>(content: impl Into<Element<'a, Message>>) -> widget::Button<'a, Message> {
    widget::button(content).style(|_, _| widget::button::Style {
        border: Border {
            radius: Radius::new(2),
            ..Default::default()
        },
        ..Default::default()
    })
}
