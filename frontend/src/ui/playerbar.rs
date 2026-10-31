use iced::{
    Background, Border, Element,
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
    app::AppState,
    components::{
        icons,
        style::{em_dash, muted_text},
        utils::{IntoLink, IntoLinks, format_duration},
    },
    player::{GenericPlayer, PlayerMsg},
    router::Route,
};
use backend::{
    data_view::TrackView,
    player::{PlayerEvent, RepeatMode},
};

#[derive(Default, Receiver)]
#[message(PlayerBarMsg)]
pub struct PlayerBar {
    /// The playback progress in seconds
    displayed_progress: f64,
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
    pub fn update(
        &mut self,
        message: impl Into<PlayerBarMsg>,
        app_state: &mut AppState,
    ) -> Task<PlayerBarMsg> {
        let player = &mut app_state.player;
        match message.into() {
            Cmd::Event(e) => {
                match e {
                    // Only update the current playback position if the user is not seeking
                    PlayerEvent::PlaybackPos(position) if !self.seeking => {
                        self.displayed_progress = position
                    }
                    _ => {}
                }
                Task::none()
            }
            Cmd::VolumeChanged(vol) => {
                player.update(PlayerMsg::Volume(vol));
                Task::none()
            }
            Cmd::Seek(p) => {
                self.seeking = true;
                self.displayed_progress = p;
                Task::none()
            }
            Cmd::FinishSeek(p) => {
                // Seeking is finished so we can reallow player updates
                self.seeking = false;
                // We need to emit the seek so the player can deal with it
                player.update(PlayerMsg::Seek(p));
                Task::none()
            }
            Cmd::Out(notify_player) => {
                match notify_player {
                    Notify::Pause(b) => app_state.player.update(PlayerMsg::SetPaused(b)),
                    Notify::Next => app_state.player.update(PlayerMsg::Next),
                    Notify::Previous => app_state.player.update(PlayerMsg::Previous),
                    Notify::Stop => app_state.player.update(PlayerMsg::Stop),
                    Notify::Shuffle(b) => app_state.player.update(PlayerMsg::SetShuffle(b)),
                    Notify::Repeat(r) => app_state.player.update(PlayerMsg::SetRepeatMode(r)),
                    Notify::PlayRandom => todo!(),
                    Notify::Seek(f) => app_state.player.update(PlayerMsg::Seek(f)),
                    Notify::Volume(u) => app_state.player.update(PlayerMsg::Volume(u)),
                    Notify::ChangeRoute(new_route) => {
                        app_state.route = new_route;
                    }
                };
                Task::none()
            }
        }
    }

    /// Renders the model after each update
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, PlayerBarMsg> {
        let player = &app_state.player;
        widget::container(widget::row([
            // Left console
            self.left_console(player)
                .align_left(FillPortion(1))
                .center_y(Fill)
                .into(),
            // Center console
            widget::container(widget::column([
                // Buttons (these emit messages for the player to deal with)
                self.buttons(player).map(Cmd::Out),
                // The progress bar (it only mutates internal state)
                self.progress_bar(player),
            ]))
            .center_x(FillPortion(1))
            .center_y(Fill)
            .into(),
            // Right console
            self.right_console(player)
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
    fn progress_bar(&self, player: &GenericPlayer) -> Element<'_, Cmd> {
        let track_duration = player
            .current_track()
            .and_then(|index| {
                player
                    .queue()
                    .get_index(index)
                    .and_then(|t| t.view.track.duration)
            })
            .unwrap_or_default();

        // Big container outside
        widget::container(
            // Little container inside
            widget::container(
                widget::row([
                    // Current pos label
                    iced::widget::text(format_duration(self.displayed_progress)).into(),
                    // Seeking slider
                    iced::widget::slider(
                        0.0..=track_duration as f64,
                        self.displayed_progress,
                        Cmd::Seek,
                    )
                    .on_release(Cmd::FinishSeek(self.displayed_progress))
                    .into(),
                    // Total duration
                    iced::widget::text(format_duration(track_duration as f64)).into(),
                ])
                .spacing(8)
                .width(Fill),
            )
            .width(Fill.max(800)),
        )
        .center(Fill)
        .into()
    }

    fn buttons<'a>(&self, player: &'a GenericPlayer) -> Element<'a, Notify> {
        const ICON_SIZE: u32 = 18;

        let stop_button = player_button(icons::square_filled().size(ICON_SIZE));

        let shuffle_button = player_button(
            icons::shuffle()
                .style(move |t: &Theme| text::Style {
                    color: player
                        .is_shuffle()
                        .then_some(t.palette().primary.strong.color),
                })
                .size(ICON_SIZE),
        )
        .on_press(Notify::Shuffle(!player.is_shuffle()));

        let skip_back_button =
            player_button(icons::skip_back_filled().size(ICON_SIZE)).on_press(Notify::Previous);

        let play_pause_button = player_button(widget::center(if player.is_paused() {
            icons::play_filled().style(text::default).size(ICON_SIZE)
        } else {
            icons::pause_filled().style(text::default).size(ICON_SIZE)
        }))
        .style(|theme, _status| {
            let text_color = theme.seed().text;
            button::Style {
                background: Some(Background::Color(text_color)),
                text_color: theme.seed().background,
                border: Border::default().rounded(i32::MAX),
                ..Default::default()
            }
        })
        .width(ICON_SIZE + 20)
        .height(ICON_SIZE + 20)
        .on_press(Notify::Pause(!player.is_paused()));

        let skip_forward_button =
            player_button(icons::skip_forward_filled().size(ICON_SIZE)).on_press(Notify::Next);

        let repeat_button = player_button(
            match player.repeat_mode() {
                RepeatMode::None => icons::repeat(),
                RepeatMode::Song => icons::repeat_one().style(|t: &Theme| text::Style {
                    color: Some(t.palette().primary.strong.color),
                }),
                RepeatMode::Queue => icons::repeat().style(|t: &Theme| text::Style {
                    color: Some(t.palette().primary.strong.color),
                }),
            }
            .size(ICON_SIZE),
        )
        .on_press(Notify::Repeat(match player.repeat_mode() {
            // Cycle repeat mode
            RepeatMode::None => RepeatMode::Queue,
            RepeatMode::Queue => RepeatMode::Song,
            RepeatMode::Song => RepeatMode::None,
        }));

        let play_random_button =
            player_button(icons::dices().size(ICON_SIZE)).on_press(Notify::PlayRandom);

        // Button container
        widget::container(
            row![
                stop_button,
                shuffle_button,
                skip_back_button,
                play_pause_button,
                skip_forward_button,
                repeat_button,
                play_random_button,
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .into()
    }

    fn right_console(&self, player: &GenericPlayer) -> Container<'_, PlayerBarMsg> {
        let mute_button = match player.volume() {
            0 => icons::volume(),
            1..50 => icons::volume_one(),
            50..=100 => icons::volume_two(),
            _ => icons::volume_x(),
        };

        widget::container(row![
            // Mute icon
            mute_button,
            // Volume slider
            iced::widget::slider(0..=100_u32, player.volume(), |v| {
                Cmd::Out(Notify::Volume(v))
            })
            .width(100),
        ])
    }

    fn left_console(&self, player: &GenericPlayer) -> Container<'_, PlayerBarMsg> {
        let currently_playing = player
            .current_track()
            .and_then(|index| player.queue().get_index(index).map(|t| &t.view));

        widget::container(widget::column([
            // Track title
            self.track_title(currently_playing),
            // Artist
            self.artists(currently_playing).map(Cmd::Out),
            // Album Name
            self.album_name(currently_playing).map(Cmd::Out),
        ]))
    }

    /// Displays the track title as part of the left console
    fn track_title(&self, currently_playing: Option<&TrackView>) -> Element<'_, PlayerBarMsg> {
        widget::text(
            currently_playing
                .as_ref()
                .and_then(|t| t.track.title.clone())
                .unwrap_or_default(),
        )
        .into()
    }

    /// Displays the artists with a link to each of them as part of the left
    /// console
    fn artists(&self, currently_playing: Option<&TrackView>) -> Element<'_, Notify> {
        currently_playing
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
    fn album_name(&self, currently_playing: Option<&TrackView>) -> Element<'_, Notify> {
        currently_playing
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
