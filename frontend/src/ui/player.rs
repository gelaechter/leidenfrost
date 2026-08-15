//! Contains the logic for the player that actually plays the audio
//! At the moment this uses MPV as the backend as it I deem it highly reliable
//! and feature-complete

use std::sync::LazyLock;

use iced::Subscription;
use iced::futures::Stream;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use backend::{
    data_view::TrackView,
    player::{Player, PlayerEvent, RepeatMode, mpv_player::MpvPlayer},
};

use crate::ui::{
    Receiver,
    playerbar::{PlayerBar, PlayerBarMsg},
    queue::{Queue, QueueMsg},
};

use macros::Receiver;

#[derive(Clone, Debug)]
pub enum Cmd {
    /// An Event received from the player
    Event(PlayerEvent),
    SetPaused(bool),
    Seek(f64),
    Stop,
    Play(TrackView),
    PlayAll(Vec<TrackView>),
    PlayIndex(usize),
    Append(TrackView),
    AppendAll(Vec<TrackView>),
    QueueRemove(usize),
    QueueMove {
        target: usize,
        position: usize,
    },
    SetShuffle(bool),
    SetRepeatMode(RepeatMode),
    Next,
    Previous,
    /// Sets the volume in percent
    Volume(u32),
}

/// An event emitted to the queue to display the data
#[derive(Clone, Debug)]
pub enum QueueEvent {
    /// A track has been appended
    Append(TrackView),
}

pub type PlayerMsg = Cmd;

static PLAYER_EVENT_CHANNEL: LazyLock<broadcast::Sender<PlayerEvent>> =
    LazyLock::new(|| broadcast::channel(128).0);

#[derive(Receiver)]
#[message(PlayerMsg)]
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
            PLAYER_EVENT_CHANNEL.send(event).unwrap();
        })
        .expect("Error during player construction");

        let player_impl = PlayerImpl::MpvPlayer(player);
        Self { player_impl }
    }
}

impl GenericPlayer {
    pub fn update(&mut self, message: impl Into<PlayerMsg>) {
        let player = self.player_impl.inner();

        // TODO: error handling
        let res = match message.into() {
            // Handle the events emitted from [`Self::subscription`]
            Cmd::Event(event) => {
                PlayerBar::send(PlayerBarMsg::Event(event));
                Ok(())
            }
            Cmd::Play(track_view) => {
                player.play(&track_view).unwrap();
                Queue::send(QueueMsg::Play(track_view));
                Ok(())
            }
            Cmd::SetPaused(paused) => player.pause(paused),
            Cmd::Seek(position) => player.seek(position),
            Cmd::Stop => player.stop(),
            Cmd::PlayAll(track_views) => {
                player.play_all(&track_views).unwrap();
                Queue::send(QueueMsg::PlayAll(track_views));
                Ok(())
            }
            Cmd::PlayIndex(index) => {
                player.play_index(index).unwrap();
                Queue::send(QueueMsg::PlayIndex(index));
                Ok(())
            }
            Cmd::Append(track_view) => {
                player.append(&track_view).unwrap();
                Queue::send(QueueMsg::Append(track_view));
                Ok(())
            }
            Cmd::AppendAll(track_views) => {
                player.append_all(&track_views).unwrap();
                Queue::send(QueueMsg::AppendAll(track_views));
                Ok(())
            }
            Cmd::QueueRemove(index) => {
                player.queue_remove(index).unwrap();
                Queue::send(QueueMsg::QueueRemove(index));
                Ok(())
            }
            Cmd::QueueMove { target, position } => {
                player.queue_move(target, position).unwrap();
                Queue::send(QueueMsg::QueueMove { target, position });
                Ok(())
            }
            Cmd::SetShuffle(b) => player.set_shuffle(b),
            Cmd::SetRepeatMode(repeat_mode) => player.set_repeat_mode(repeat_mode),
            Cmd::Next => {
                player.next().unwrap();
                Queue::send(QueueMsg::Next);
                Ok(())
            }
            Cmd::Previous => {
                player.previous().unwrap();
                Queue::send(QueueMsg::Previous);
                Ok(())
            }
            Cmd::Volume(percentage) => player.volume(percentage),
        };
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
