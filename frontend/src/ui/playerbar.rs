use std::str::FromStr;

use iced::{
    Element,
    Length::{self, Fill, FillPortion},
    Task, Theme,
    alignment::Horizontal,
    widget::{
        self, Container, button, column,
        container::Style,
        row,
        text::{self, Wrapping},
    },
};
use url::Url;

use crate::ui::{
    ICMsg, Receiver, ToCmdMsg, ToOutMsg, components::{
        icons,
        style::{EM_DASH, muted_text},
        utils::{IntoLink, IntoLinks, format_duration},
    }, player::{self, GenericPlayer}, router::Route,
};
use backend::{
    data_view::TrackView,
    db::models::{OrmUrl, Track},
    player::{PlayerEvent, RepeatMode},
};

#[derive(Default)]
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
    Volume(u32),
}

// Messages meant to be passed upwards
#[derive(Clone, Debug)]
pub enum Out {
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

pub type PlayerBarMsg = ICMsg<Cmd, Out>;

impl PlayerBar {
    /// Mutates the model whenever a message is dispatched
    pub fn update(&mut self, message: impl Into<PlayerBarMsg>) -> Task<PlayerBarMsg> {
        let message = message.into();
        if let PlayerBarMsg::Out(Out::PlayRandom) = message {
            GenericPlayer::send(player::Cmd::Play(TrackView {
                    track: Track {
                        id: String::new(),
                        endpoint_id: String::new(),
                        album_id: String::new(),
                        disc_number: 0,
                        bit_rate: None,
                        bpm: None,
                        channels: None,
                        container: None,
                        duration: None,
                        image_url: None,
                        image_blur_hash: None,
                        last_played_at: None,
                        lyrics: None,
                        title: None,
                        file_path: None,
                        play_count: None,
                        release_date: None,
                        file_size: None,
                        stream_url: Some(Url::from_str("file:///mnt/NAS/Samuel/Music/flac/Flux Pavilion/I Can’t Stop/01 - I Can’t Stop.flac").unwrap().into()),
                        track_number: Some(0),
                        user_favorite: None,
                    },
                    artists: vec![],
                    album_name: None,
                    genres: vec![],
                }));
        }

        // We only need to handle commands
        message.cmd(|c| match c {
            Cmd::Event(e) => {
                match e {
                    PlayerEvent::Shutdown => todo!(),
                    PlayerEvent::Shuffle(shuffle) => self.shuffle = shuffle,
                    PlayerEvent::Repeat(repeat_mode) => self.repeat_mode = repeat_mode,
                    PlayerEvent::Pause(paused) => self.paused = paused,
                    PlayerEvent::PlaybackPos(position) if !self.seeking => self.progress = position,
                    PlayerEvent::PlaybackPos(_) => {}
                    PlayerEvent::Duration(duration) => self.duration = duration,
                    PlayerEvent::Volume(volume) => self.volume = volume,
                    PlayerEvent::Error(player_error) => todo!(),
                }
                Task::none()
            }
            Cmd::Volume(vol) => {
                self.volume = vol;
                // We need to emit the volume so the upper components can update
                Task::done(Out::Volume(vol).out_msg())
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
                Task::done(Out::Seek(p).out_msg())
            }
        })
    }

    /// Renders the model after each update
    pub fn view(&self) -> Element<'_, ICMsg<Cmd, Out>> {
        widget::container(row![
            // Left console
            self.left_console()
                .align_left(FillPortion(1))
                .center_y(Fill),
            // Center console
            widget::container(column![
                // Buttons (these emit messages for the player to deal with)
                self.buttons().map(ToOutMsg::out_msg),
                // The progress bar (it only mutates internal state)
                self.progress_bar().map(ToCmdMsg::cmd_msg),
            ])
            .center_x(FillPortion(1))
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

    fn buttons(&self) -> Element<'_, Out> {
        const BUTTON_SIZE: u32 = 20;

        let stop_button = player_button(icons::square().size(BUTTON_SIZE));

        let shuffle_button = player_button(
            icons::shuffle()
                .style(|t: &Theme| text::Style {
                    color: self.shuffle.then_some(t.palette().primary.strong.color),
                })
                .size(BUTTON_SIZE),
        )
        .on_press(Out::Shuffle(!self.shuffle));

        let skip_back_button =
            player_button(icons::skip_back().size(BUTTON_SIZE)).on_press(Out::Previous);

        let pause_button = player_button(if self.paused {
            icons::play().size(BUTTON_SIZE)
        } else {
            icons::pause().size(BUTTON_SIZE)
        })
        .on_press(Out::Pause(!self.paused));

        let skip_forward_button =
            player_button(icons::skip_forward().size(BUTTON_SIZE)).on_press(Out::Next);

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
        .on_press(Out::Repeat(match self.repeat_mode {
            // Cycle repeat mode
            RepeatMode::None => RepeatMode::Queue,
            RepeatMode::Queue => RepeatMode::Song,
            RepeatMode::Song => RepeatMode::None,
        }));

        let play_random_button =
            player_button(icons::dices().size(BUTTON_SIZE)).on_press(Out::PlayRandom);

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

    fn right_console(&self) -> Container<'_, ICMsg<Cmd, Out>> {
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
            iced::widget::slider(0..=100_u32, self.volume, |v| { Out::Volume(v).out_msg() })
                .width(100),
        ])
    }

    fn left_console(&self) -> Container<'_, ICMsg<Cmd, Out>> {
        widget::container(widget::column([
            // Track title
            self.track_title().map(ToOutMsg::out_msg),
            // Artist
            self.artists().map(ToOutMsg::out_msg),
            // Album Name
            self.album_name().map(ToOutMsg::out_msg),
        ]))
    }

    /// Displays the track title as part of the left console
    fn track_title(&self) -> Element<'_, Out> {
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
    fn artists(&self) -> Element<'_, Out> {
        self.currently_playing
            .as_ref()
            .map(|t: &TrackView| {
                t.artists
                    .as_slice()
                    .into_links(
                        |artist| {
                            (
                                artist.name.clone().unwrap_or(EM_DASH()),
                                Route::Artist(artist.id.clone()),
                            )
                        },
                        Out::ChangeRoute,
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
    fn album_name(&self) -> Element<'_, Out> {
        self.currently_playing
            .as_ref()
            .map(|t| {
                t.album_name
                    .clone()
                    .unwrap_or(EM_DASH())
                    .link(Route::Album(t.track.id.clone()), Out::ChangeRoute)
                    .style(muted_text)
                    .wrapping(Wrapping::None)
                    .ellipsis(text::Ellipsis::End)
            })
            .unwrap_or_default()
            .into()
    }
}

fn player_button<'a>(content: impl Into<Element<'a, Out>>) -> widget::Button<'a, Out> {
    widget::button(content).style(button::text)
}
