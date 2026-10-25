//! Contains the logic for the player that actually plays the audio
//! At the moment this uses MPV as the backend as it I deem it highly reliable
//! and feature-complete

use std::sync::LazyLock;

use iced::futures::Stream;
use iced::{Subscription, futures};
use tokio::sync::{
    Mutex,
    mpsc::{self},
};

use backend::{
    data_view::TrackView,
    player::{Player, PlayerEvent, RepeatMode, mpv_player::MpvPlayer},
};

use crate::ui::{
    Receiver, app::AppState, playerbar::{PlayerBar, PlayerBarMsg}, queue::{Queue, QueueMsg},
};

use macros::Receiver;

#[derive(Clone, Debug)]
pub enum Message {
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

pub type PlayerMsg = Message;

static PLAYER_EVENT_CHANNEL: LazyLock<(
    mpsc::UnboundedSender<PlayerEvent>,
    // This feels overkill but we need to own the receiver
    Mutex<mpsc::UnboundedReceiver<PlayerEvent>>,
)> = LazyLock::new(|| {
    // TODO: consider using a normal channel instead and then drop less important
    // events on queue saturation
    //
    // Events like inbetween PlaybackPos for example can be dropped
    // (we care about the most recent)
    let (tx, rx) = mpsc::unbounded_channel();
    (tx, Mutex::new(rx))
});

/// The player is a [`Receiver`] for [`PlayerMsg`] so that we can arbitrarily
/// add items ot the queue
#[derive(Debug, Receiver)]
#[message(PlayerMsg)]
pub struct GenericPlayer {
    player_impl: Option<PlayerImpl>,
}

impl Default for GenericPlayer {
    fn default() -> Self {
        Self {
            player_impl: Some(PlayerImpl::mpv_player()),
        }
    }
}

/// An abstraction over the current player
pub enum PlayerImpl {
    MpvPlayer(MpvPlayer),
}

impl std::fmt::Debug for PlayerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MpvPlayer(_arg0) => f.write_str("MpvPlayer"),
        }
    }
}

impl PlayerImpl {
    /// Unwraps the enum and gets the inner player implementation
    pub fn inner(&self) -> &impl Player {
        match self {
            PlayerImpl::MpvPlayer(mpv_player) => mpv_player,
        }
    }

    /// Constructs a new [`PlayerImpl::MpvPlayer`] variant
    fn mpv_player() -> Self {
        let mpv_player = MpvPlayer::new(|event| {
            let _ = PLAYER_EVENT_CHANNEL.0.send(event).inspect_err(|e| {
                log::error!("The player event channel has been closed: {e:?}");
            });
        })
        .inspect_err(|e| {
            log::error!("Error during player construction {e:?}");
        })
        .unwrap();

        PlayerImpl::MpvPlayer(mpv_player)
    }
}

impl GenericPlayer {
    pub fn update(&mut self, message: impl Into<PlayerMsg>, _app_state: &mut AppState) {
        let Some(player_impl) = self.player_impl.as_ref() else {
            return;
        };
        let player = player_impl.inner();

        // TODO: error handling
        let res = match message.into() {
            // Handle the events emitted from [`Self::subscription`]
            Message::Event(event) => {
                PlayerBar::send(PlayerBarMsg::Event(event));
                Ok(())
            }
            Message::Play(track_view) => {
                player.play(&track_view).unwrap();
                Queue::send(QueueMsg::Play(track_view));
                Ok(())
            }
            Message::SetPaused(paused) => player.pause(paused),
            Message::Seek(position) => player.seek(position),
            Message::Stop => player.stop(),
            Message::PlayAll(track_views) => {
                player.play_all(&track_views).unwrap();
                Queue::send(QueueMsg::PlayAll(track_views));
                Ok(())
            }
            Message::PlayIndex(index) => {
                player.play_index(index).unwrap();
                Queue::send(QueueMsg::PlayIndex(index));
                Ok(())
            }
            Message::Append(track_view) => {
                player.append(&track_view).unwrap();
                Queue::send(QueueMsg::Append(track_view));
                Ok(())
            }
            Message::AppendAll(track_views) => {
                player.append_all(&track_views).unwrap();
                Queue::send(QueueMsg::AppendAll(track_views));
                Ok(())
            }
            Message::QueueRemove(index) => {
                player.queue_remove(index).unwrap();
                Queue::send(QueueMsg::QueueRemove(index));
                Ok(())
            }
            Message::QueueMove { target, position } => {
                player.queue_move(target, position).unwrap();
                Queue::send(QueueMsg::QueueMove { target, position });
                Ok(())
            }
            Message::SetShuffle(b) => player.set_shuffle(b),
            Message::SetRepeatMode(repeat_mode) => player.set_repeat_mode(repeat_mode),
            Message::Next => {
                player.next().unwrap();
                Queue::send(QueueMsg::Next);
                Ok(())
            }
            Message::Previous => {
                player.previous().unwrap();
                Queue::send(QueueMsg::Previous);
                Ok(())
            }
            Message::Volume(percentage) => player.volume(percentage),
        };
    }

    /// Subscribes to the player events
    pub fn subscription() -> Subscription<PlayerMsg> {
        fn subscribe_player_events() -> impl Stream<Item = PlayerEvent> {
            futures::stream::unfold((), |()| async {
                let event = PLAYER_EVENT_CHANNEL.1.lock().await.recv().await?;
                Some((event, ()))
            })
        }

        Subscription::run(subscribe_player_events).map(Message::Event)
    }
}
