use iced::{
    Element,
    Length::{self, Fill, FillPortion},
    Task, Theme,
    alignment::Horizontal,
    widget::{
        self, Container, button,
        container::Style,
        row,
        text::{self, Wrapping},
    },
};
use macros::Receiver;

use crate::ui::{
    Receiver,
    components::{
        icons,
        style::{em_dash, muted_text},
        utils::{IntoLink, IntoLinks, format_duration},
    },
    player::{GenericPlayer, PlayerMsg},
    router::{Route, Router, RouterMsg},
};
use backend::{
    data_view::TrackView,
    player::{PlayerEvent, RepeatMode},
};

#[derive(Default, Receiver)]
#[message(PlayerBarMsg)]
pub struct PlayerBar {
    /// The currently playing track
    currently_playing: Option<TrackView>,
    /// If the queue is shuffled or not
    shuffle: bool,
    /// The repeat mode of the player
    repeat_mode: RepeatMode,
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

// Messages meant to update internal state
#[derive(Clone, Debug)]
pub enum Cmd {
    // The player state has updated
    Event(PlayerEvent),
    /// The user starts seeking by moving the slider
    Seek(f64),
    /// The user stops seeking be letting go the slider
    FinishSeek(f64),
    /// The user moves the volume slider
    VolumeChanged(u32),
    /// Messages that signify that another components
    Out(Notify),
}

// Messages that notify other components
#[derive(Clone, Debug)]
pub enum Notify {
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
    /// The user requests a different volume
    Volume(u32),
    /// Request to change the route
    ChangeRoute(Route),
}

impl From<Notify> for PlayerBarMsg {
    fn from(value: Notify) -> Self {
        Self::Out(value)
    }
}

pub type PlayerBarMsg = Cmd;

impl PlayerBar {
    /// Mutates the model whenever a message is dispatched
    pub fn update(&mut self, message: impl Into<PlayerBarMsg>) -> Task<PlayerBarMsg> {
        match message.into() {
            Cmd::Event(e) => {
                match e {
                    PlayerEvent::Shutdown => todo!(),
                    PlayerEvent::Shuffle(shuffle) => self.shuffle = shuffle,
                    PlayerEvent::Repeat(repeat_mode) => self.repeat_mode = repeat_mode,
                    PlayerEvent::Pause(paused) => self.paused = paused,
                    // Only update the current playback position if the user is not seeking
                    PlayerEvent::PlaybackPos(position) if !self.seeking => self.progress = position,
                    PlayerEvent::PlaybackPos(_) => {}
                    PlayerEvent::Duration(duration) => self.duration = duration,
                    PlayerEvent::Volume(volume) => self.volume = volume,
                    PlayerEvent::Error(player_error) => todo!(),
                }
                Task::none()
            }
            Cmd::VolumeChanged(vol) => {
                self.volume = vol;
                // We need to emit the volume so the upper components can update
                Task::done(Cmd::Out(Notify::Volume(vol)))
            }
            Cmd::Seek(p) => {
                self.seeking = true;
                self.progress = p;
                Task::none()
            }
            Cmd::FinishSeek(p) => {
                // Seeking is finished so we can reallow player updates
                self.seeking = false;
                // We need to emit the seek so the player can deal with it
                Task::done(Cmd::Out(Notify::Seek(p)))
            }
            Cmd::Out(notify_player) => {
                match notify_player {
                    Notify::Pause(b) => GenericPlayer::send(PlayerMsg::SetPaused(b)),
                    Notify::Next => GenericPlayer::send(PlayerMsg::Next),
                    Notify::Previous => GenericPlayer::send(PlayerMsg::Previous),
                    Notify::Stop => GenericPlayer::send(PlayerMsg::Stop),
                    Notify::Shuffle(b) => GenericPlayer::send(PlayerMsg::SetShuffle(b)),
                    Notify::Repeat(r) => GenericPlayer::send(PlayerMsg::SetRepeatMode(r)),
                    Notify::PlayRandom => todo!(),
                    Notify::Seek(f) => GenericPlayer::send(PlayerMsg::Seek(f)),
                    Notify::Volume(u) => GenericPlayer::send(PlayerMsg::Volume(u)),
                    Notify::ChangeRoute(r) => Router::send(RouterMsg::ChangeRoute(r)),
                };
                Task::none()
            }
        }
    }

    /// Renders the model after each update
    pub fn view(&self) -> Element<'_, PlayerBarMsg> {
        widget::container(widget::row([
            // Left console
            self.left_console()
                .align_left(FillPortion(1))
                .center_y(Fill)
                .into(),
            // Center console
            widget::container(widget::column([
                // Buttons (these emit messages for the player to deal with)
                self.buttons().map(Cmd::Out),
                // The progress bar (it only mutates internal state)
                self.progress_bar(),
            ]))
            .center_x(FillPortion(1))
            .center_y(Fill)
            .into(),
            // Right console
            self.right_console()
                .align_right(FillPortion(1))
                .center_y(Fill)
                .into(),
        ]))
        .padding(10)
        .center_x(Fill)
        .center_y(86)
        .style(|theme: &Theme| Style::default().background(theme.palette().background.weak.color))
        .into()
    }

    /// The progress bar displaying the
    fn progress_bar(&self) -> Element<'_, Cmd> {
        // Big container outside
        widget::container(
            // Little container inside
            widget::container(
                row![
                    // Current pos label
                    iced::widget::text(format_duration(self.progress)),
                    // Seeking slider
                    iced::widget::slider(0.0..=self.duration, self.progress, Cmd::Seek)
                        .on_release(Cmd::FinishSeek(self.progress)),
                    // Total duration
                    iced::widget::text(format_duration(self.duration)),
                ]
                .spacing(8)
                .width(Fill),
            )
            .max_width(800),
        )
        .center(Fill)
        .into()
    }

    fn buttons(&self) -> Element<'_, Notify> {
        const BUTTON_SIZE: u32 = 20;

        let stop_button = player_button(icons::square().size(BUTTON_SIZE));

        let shuffle_button = player_button(
            icons::shuffle()
                .style(|t: &Theme| text::Style {
                    color: self.shuffle.then_some(t.palette().primary.strong.color),
                })
                .size(BUTTON_SIZE),
        )
        .on_press(Notify::Shuffle(!self.shuffle));

        let skip_back_button =
            player_button(icons::skip_back().size(BUTTON_SIZE)).on_press(Notify::Previous);

        let pause_button = player_button(if self.paused {
            icons::play().size(BUTTON_SIZE)
        } else {
            icons::pause().size(BUTTON_SIZE)
        })
        .on_press(Notify::Pause(!self.paused));

        let skip_forward_button =
            player_button(icons::skip_forward().size(BUTTON_SIZE)).on_press(Notify::Next);

        let repeat_button = player_button(
            match self.repeat_mode {
                RepeatMode::None => icons::repeat(),
                RepeatMode::Song => icons::repeat_one().style(|t: &Theme| text::Style {
                    color: Some(t.palette().primary.strong.color),
                }),
                RepeatMode::Queue => icons::repeat().style(|t: &Theme| text::Style {
                    color: Some(t.palette().primary.strong.color),
                }),
            }
            .size(BUTTON_SIZE),
        )
        .on_press(Notify::Repeat(match self.repeat_mode {
            // Cycle repeat mode
            RepeatMode::None => RepeatMode::Queue,
            RepeatMode::Queue => RepeatMode::Song,
            RepeatMode::Song => RepeatMode::None,
        }));

        let play_random_button =
            player_button(icons::dices().size(BUTTON_SIZE)).on_press(Notify::PlayRandom);

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
        .into()
    }

    fn right_console(&self) -> Container<'_, PlayerBarMsg> {
        let mute_button = match self.volume {
            0 => icons::volume(),
            1..50 => icons::volume_one(),
            50..=100 => icons::volume_two(),
            _ => icons::volume_x(),
        };

        widget::container(row![
            // Mute icon
            mute_button,
            // Volume slider
            iced::widget::slider(0..=100_u32, self.volume, |v| {
                Cmd::Out(Notify::Volume(v))
            })
            .width(100),
        ])
    }

    fn left_console(&self) -> Container<'_, PlayerBarMsg> {
        widget::container(widget::column([
            // Track title
            self.track_title(),
            // Artist
            self.artists().map(Cmd::Out),
            // Album Name
            self.album_name().map(Cmd::Out),
        ]))
    }

    /// Displays the track title as part of the left console
    fn track_title(&self) -> Element<'_, PlayerBarMsg> {
        widget::text(
            self.currently_playing
                .as_ref()
                .and_then(|t| t.track.title.clone())
                .unwrap_or_default(),
        )
        .into()
    }

    /// Displays the artists with a link to each of them as part of the left
    /// console
    fn artists(&self) -> Element<'_, Notify> {
        self.currently_playing
            .as_ref()
            .map(|t: &TrackView| {
                t.artists
                    .as_slice()
                    .into_links(
                        |artist| {
                            (
                                artist.name.clone().unwrap_or(em_dash()),
                                Route::Artist(artist.id.clone()),
                            )
                        },
                        Notify::ChangeRoute,
                    )
                    .style(muted_text)
                    .wrapping(Wrapping::None)
                    .ellipsis(text::Ellipsis::End)
            })
            .unwrap_or_default()
            .into()
    }

    /// Displays the album name with a link to the album as part of the left
    /// console
    fn album_name(&self) -> Element<'_, Notify> {
        self.currently_playing
            .as_ref()
            .map(|t| {
                t.album_name
                    .clone()
                    .unwrap_or(em_dash())
                    .link(Route::Album(t.track.id.clone()), Notify::ChangeRoute)
                    .style(muted_text)
                    .wrapping(Wrapping::None)
                    .ellipsis(text::Ellipsis::End)
            })
            .unwrap_or_default()
            .into()
    }
}

fn player_button<'a>(content: impl Into<Element<'a, Notify>>) -> widget::Button<'a, Notify> {
    widget::button(content).style(button::text)
}
