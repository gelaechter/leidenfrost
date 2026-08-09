//! Contains the logic for the player that actually plays the audio
//! At the moment this uses MPV as the backend as it I deem it highly reliable
//! and feature-complete

use std::sync::LazyLock;

use iced::futures::Stream;
use iced::{Subscription, Task};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use backend::{
    data_view::TrackView,
    player::{Player, PlayerError, PlayerEvent, mpv_player::MpvPlayer},
};

use crate::ui::{ICMsg, ToErrMsg, ToOutMsg};

#[derive(Clone, Debug)]
pub enum Cmd {
    /// An Event received from the player
    /// this needs to be converted to an [`Out`] for public consumption
    Event(PlayerEvent),
}

#[derive(Clone, Debug)]
pub enum Out {
    Event(PlayerEvent),
    Queue(Box<QueueEvent>),
}

/// An event emitted to the queue to display the data
#[derive(Clone, Debug)]
pub enum QueueEvent {
    /// A track has been appended
    Append(TrackView),
}

pub type PlayerMsg = ICMsg<Cmd, Out, PlayerError>;

static PLAYER_EVENT_CHANNEL: LazyLock<broadcast::Sender<PlayerEvent>> =
    LazyLock::new(|| broadcast::channel(128).0);

pub struct GenericPlayer {
    player_impl: PlayerImpl,
}

pub enum PlayerImpl {
    MpvPlayer(MpvPlayer),
}

impl PlayerImpl {
    /// Unwraps the enum and gets the inner player implementation
    pub fn inner(&self) -> &impl Player {
        match self {
            PlayerImpl::MpvPlayer(mpv_player) => mpv_player,
        }
    }
}

impl Default for GenericPlayer {
    /// By default uses the MPV Player backend
    fn default() -> Self {
        let player = MpvPlayer::new(|event| {
            PLAYER_EVENT_CHANNEL.send(event);
        })
        .expect("Error during player construction");

        let player_impl = PlayerImpl::MpvPlayer(player);
        Self { player_impl }
    }
}

impl GenericPlayer {
    pub fn update(&mut self, message: impl Into<PlayerMsg>) -> Task<PlayerMsg> {
        // Only commands need to be handled
        let icmsg = message.into();
        icmsg.cmd(|c| {
            match c {
                // Handle the events emitted from [`Self::subscription`]
                Cmd::Event(event) => Task::done(Out::Event(event).out_msg()),
            }
        })
    }

    /// Subscribes to the player events
    pub fn subscription() -> Subscription<PlayerMsg> {
        fn subscribe_player_events() -> impl Stream<Item = PlayerEvent> {
            use tokio_stream::StreamExt;

            let stream = PLAYER_EVENT_CHANNEL.subscribe();
            BroadcastStream::new(stream).filter_map(Result::ok)
        }

        Subscription::run(subscribe_player_events).map(|m| Cmd::Event(m).into())
    }
}

impl GenericPlayer {
    fn pause(&self, paused: bool) -> Task<PlayerMsg> {
        if let Err(e) = self.player_impl.inner().pause(paused) {
            return Task::done(e.err_msg())
        }
    }

    fn seek(&self, position: f64) -> backend::player::Result<()> {
        self.player_impl.inner().seek(position)
    }

    fn stop(&self) -> backend::player::Result<()> {
        self.player_impl.inner().stop()
    }

    fn play(&self, track: TrackView) -> backend::player::Result<()> {
        self.player_impl.inner().stop()
    }

    fn play_all(&self, tracks: Vec<TrackView>) -> backend::player::Result<()> {
        self.player_impl.inner().play_all(tracks)
    }

    fn play_index(&self, index: usize) -> backend::player::Result<()> {
        self.player_impl.inner().play_index(index)
    }

    fn append(&self, track: TrackView) -> backend::player::Result<()> {
        self.player_impl.inner().append(track)
    }

    fn append_all(&self, tracks: Vec<TrackView>) -> backend::player::Result<()> {
        self.player_impl.inner().append_all(tracks)
    }

    fn queue_remove(&self, index: usize) -> backend::player::Result<()> {
        self.player_impl.inner().queue_remove(index)
    }

    fn queue_move(&self, target: usize, position: usize) -> backend::player::Result<()> {
        self.player_impl.inner().queue_move(target, position)
    }

    fn set_shuffle(&self, shuffle: bool) -> backend::player::Result<()> {
        self.player_impl.inner().set_shuffle(shuffle)
    }

    fn change_repeat_mode(&self, mode: backend::player::RepeatMode) -> backend::player::Result<()> {
        self.player_impl.inner().change_repeat_mode(mode)
    }

    fn next(&self) -> backend::player::Result<()> {
        self.player_impl.inner().next()
    }

    fn previous(&self) -> backend::player::Result<()> {
        self.player_impl.inner().previous()
    }

    fn volume(&self, percentage: u32) -> backend::player::Result<()> {
        self.player_impl.inner().volume(percentage)
    }
}
